use cell::{Cellule, State};
use gloo::timers::callback::Interval;
use rand::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::HtmlInputElement;
use yew::html::Scope;
use yew::prelude::InputEvent;
use yew::{classes, html, Component, Context};

mod cell;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "tauri"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

pub struct App {
    cellules: Vec<Cellule>,
    cellules_width: usize,
    cellules_height: usize,
    _interval: Interval,
    active: bool,
    size_cursor: usize,
}

pub enum Msg {
    Tick,
    // Cellule change state
    ToggleCellule(usize),
    AddCellule(usize),
    RemoveCellule(usize),
    // Mouse events
    MouseOver(usize),
    MouseOut(usize),
    // Control events
    Play,
    Reset,
    Pause,
    ChangeSizeCursor(usize),
}

impl App {
    fn relative_idx(&self, i: usize, j: usize) -> Option<usize> {
        if i < self.cellules_height && j < self.cellules_width {
            Some(i * self.cellules_width + j)
        } else {
            None
        }
    }

    fn view_cellule(&self, idx: usize, cellule: &Cellule, link: &Scope<Self>) -> yew::Html {
        let cellule_class = match cellule.state {
            State::Dead | State::MouseOut => "cellule-dead",
            State::Alive => "cellule-alive",
            State::MouseOver => "cellule-mouse-over",
        };
        html! {
            <div
                key={idx}
                class={classes!("simulation-cellule", cellule_class)}
                onmousedown={link.callback(move |e: yew::MouseEvent| match e.button() {
                    0 => Msg::AddCellule(idx),
                    2 => Msg::RemoveCellule(idx),
                    _ => Msg::ToggleCellule(idx),
                })}
                onmouseover={link.callback(move |_| Msg::MouseOver(idx))}
                onmouseout={link.callback(move |_| Msg::MouseOut(idx))}
            >
            </div>
        }
    }

    fn step(&mut self) {
        for i in (0..self.cellules_height).rev() {
            for j in 0..self.cellules_width {
                if let (Some(idx), Some(idx_below)) =
                    (self.relative_idx(i, j), self.relative_idx(i + 1, j))
                {
                    if self.cellules[idx].is_alive() && self.cellules[idx_below].is_dead() {
                        self.cellules[idx].set_dead();
                        self.cellules[idx_below].set_alive();
                    } else if self.cellules[idx].is_alive() && self.cellules[idx_below].is_alive() {
                        self.slip(i, j, idx);
                    }
                }
            }
        }
    }

    fn slip(&mut self, i: usize, j: usize, idx: usize) {
        let mut rng = rand::thread_rng();
        let slips = (-5..=5).collect::<Vec<i32>>();
        let slip: &i32 = slips.choose(&mut rng).unwrap();

        let has_obstacle = {
            if *slip > 0 {
                *slip..0
            } else if *slip < 0 {
                0..*slip
            } else {
                0..1
            }
        }
        .any(|k| {
            self.relative_idx(i, (j as i32 + k) as usize)
                .and_then(|slipped_idx| Some(self.cellules[slipped_idx].is_dead()))
                .unwrap_or(false)
        });

        if let Some(slipped_idx) = self.relative_idx(i + 1, (j as i32 + slip) as usize) {
            if !has_obstacle && self.cellules[slipped_idx].is_dead() {
                self.cellules[idx].set_dead();
                self.cellules[slipped_idx].set_alive();
            }
        }
    }

    fn cicle_cursor(&mut self, idx: usize) -> Vec<usize> {
        let mut market_cells: Vec<usize> = Vec::new();
        for radius in 0..self.size_cursor {
            for rad in 0..360 {
                let x = (radius as f64 * (rad as f64).to_radians().cos()) as i32;
                let y = (radius as f64 * (rad as f64).to_radians().sin()) as i32;
                let i = (idx / self.cellules_width) as i32 + y;
                let j = (idx % self.cellules_width) as i32 + x;
                if let Some(idx) = self.relative_idx(i as usize, j as usize) {
                    if self.cellules[idx].is_dead() {
                        market_cells.push(idx);
                    }
                }
            }
        }
        market_cells
    }
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let callback = ctx.link().callback(|_| Msg::Tick);
        let _interval = Interval::new(10, move || callback.emit(()));

        let (cellules_width, cellules_height) = (100, 50);
        let cellules = vec![Cellule { state: State::Dead }; cellules_width * cellules_height];
        Self {
            cellules,
            cellules_width,
            cellules_height,
            _interval,
            active: true,
            size_cursor: 8,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToggleCellule(idx) => {
                self.cicle_cursor(idx).iter().for_each(|idx| {
                    self.cellules.get_mut(*idx).unwrap().swap();
                });
                true
            }
            Msg::AddCellule(idx) => {
                self.cicle_cursor(idx).iter().for_each(|idx| {
                    self.cellules.get_mut(*idx).unwrap().set_alive();
                });
                true
            }
            Msg::RemoveCellule(idx) => {
                self.cicle_cursor(idx).iter().for_each(|idx| {
                    self.cellules.get_mut(*idx).unwrap().set_dead();
                });
                true
            }
            Msg::Tick => {
                if self.active {
                    self.step();
                    return true;
                }
                false
            }
            Msg::MouseOver(idx) => {
                self.cicle_cursor(idx).into_iter().for_each(|idx| {
                    self.cellules.get_mut(idx).unwrap().set_mouse_over();
                });

                true
            }
            Msg::MouseOut(idx) => {
                self.cicle_cursor(idx).iter().for_each(|idx| {
                    self.cellules.get_mut(*idx).unwrap().set_mouse_out();
                });
                true
            }
            Msg::Pause => {
                self.active = false;
                true
            }
            Msg::Play => {
                self.active = true;
                true
            }
            Msg::Reset => {
                self.cellules
                    .iter_mut()
                    .for_each(|cellule| cellule.set_dead());
                self.active = true;
                true
            }
            Msg::ChangeSizeCursor(size) => {
                self.size_cursor = size;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> yew::Html {
        let handle_change_size_cursor = ctx.link().callback(|e: InputEvent| {
            let input: HtmlInputElement = e.target().unwrap().unchecked_into();
            let value = input.value().parse().unwrap_or(8);
            Msg::ChangeSizeCursor(value)
        });
        let cell_rows = self
            .cellules
            .chunks(self.cellules_width)
            .enumerate()
            .map(|(key, row)| {
                let idx_offset = key * self.cellules_width;
                let cells = row.iter().enumerate().map(|(idx, cellule)| {
                    let idx = idx + idx_offset;
                    self.view_cellule(idx, cellule, ctx.link())
                });
                html! {
                    <div key={key} class={"simulation-row"}>{for cells}</div>
                }
            });

        html! {
            <>
                <section class={"simulation-container"}>
                    <header class={"simulation-header"}>
                        <h1 class={"simulation-title"}>{ "Sand Simulation" }</h1>
                    </header>
                    <div class={"simulation-content"}>
                        <section class={"simulation-area"}>
                            <div class={"simulation-grid"}>{for cell_rows}</div>
                        </section>
                        <div class="game-buttons">
                            <button class="game-button" onclick={ctx.link().callback(|_| Msg::Play)}>{ "Play" }</button>
                            <button class="game-button" onclick={ctx.link().callback(|_| Msg::Pause)}>{ "Pause" }</button>
                            <button class="game-button" onclick={ctx.link().callback(|_| Msg::Reset)}>{ "Reset" }</button>
                            <input
                                type="range"
                                min="1"
                                max="20"
                                step="1"
                                value={self.size_cursor.to_string()}
                                oninput={handle_change_size_cursor}
                            />
                        </div>
                    </div>
                </section>
            </>
        }
    }
}
