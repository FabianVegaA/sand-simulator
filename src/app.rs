use cell::{Cellule, State};
use gloo::timers::callback::Interval;
use rand::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::HtmlInputElement;
use yew::html::Scope;
use yew::prelude::{Event, InputEvent};
use yew::{classes, html, Component, Context};

use self::cell::Kind;

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
    creation_mode: CreationMode,
    kind_cell: Kind,
    seed: rand::rngs::ThreadRng,
}

#[derive(Clone, PartialEq, Copy)]
pub enum CreationMode {
    Add,
    Remove,
}

pub enum Msg {
    Tick,
    // Cellule change state
    AddCellule(usize),
    RemoveCellule(usize),
    // Mouse events
    MouseOver(usize),
    MouseOut(usize),
    // Control events
    Play,
    Reset,
    Pause,
    // Configuration
    ChangeSizeCursor(usize),
    ChangeCreationMode(CreationMode),
    ChangeKindCell(Kind),
}

impl App {
    fn relative_idx(&self, i: usize, j: usize) -> Option<usize> {
        (i < self.cellules_height && j < self.cellules_width).then(|| i * self.cellules_width + j)
    }

    fn view_cellule(&self, idx: usize, cellule: &Cellule, link: &Scope<Self>) -> yew::Html {
        let cellule_class = match cellule.state {
            State::Alive => "cellule-alive",
            State::MouseOver(true) => "cellule-mouse-over-alive",
            State::MouseOver(false) => "cellule-mouse-over-dead",
            State::Dead | State::MouseOut => "cellule-dead",
        };
        let kind_class = match cellule.kind {
            Some(Kind::Sand) => "sand",
            Some(Kind::Rock) => "rock",
            _ => "air",
        };
        let action = match self.creation_mode {
            CreationMode::Add => Msg::AddCellule,
            CreationMode::Remove => Msg::RemoveCellule,
        };

        let style = if cellule.pressure == 0 {
            "border-radius: 20% 20% 0 0".to_string()
        } else {
            // TODO: use a better formula to calculate brightness
            let brightness = 100.0 - (cellule.pressure as f32) * 1.2;
            format!("filter: brightness({}%)", brightness.max(0.0).min(100.0))
        };

        html! {
            <div
                key={idx}
                class={classes!("simulation-cellule", cellule_class, kind_class)}
                style={style}
                onmousedown={link.callback(move |_| action(idx))}
                onmouseover={link.callback(move |_| Msg::MouseOver(idx))}
                onmouseout={link.callback(move |_| Msg::MouseOut(idx))}
            >
            </div>
        }
    }

    fn step(&mut self) {
        for i in (0..self.cellules_height).rev() {
            for j in 0..self.cellules_width {
                let idx = self.relative_idx(i, j).unwrap();
                let kind = self.cellules[idx].kind;
                match kind {
                    Some(Kind::Sand) => self.step_sand(i, j, idx),
                    _ => continue,
                }
            }
        }
    }

    fn step_sand(&mut self, i: usize, j: usize, idx: usize) {
        let target_pressure = self.pressure(i, j);
        self.cellules[idx].set_pressure(target_pressure);
        if let Some(idx_below) = self.relative_idx(i + 1, j) {
            if self.cellules[idx].is_alive() && self.cellules[idx_below].is_dead() {
                self.move_cell(idx, idx_below);
            } else if self.cellules[idx].is_alive() && self.cellules[idx_below].is_alive() {
                self.move_cell_with_slip(i, j, idx, 2);
            }
        }
    }

    fn move_cell_with_slip(&mut self, i: usize, j: usize, idx: usize, slippage: i32) {
        let slip_coefficient: i32 = *(-slippage..=slippage)
            .collect::<Vec<i32>>()
            .choose(&mut self.seed)
            .unwrap();

        let slip_range: Vec<i32> = if slip_coefficient <= 0 {
            (slip_coefficient..=-1).rev().collect()
        } else {
            (1..=slip_coefficient).collect()
        };

        let slipped_idx = slip_range
            .into_iter()
            .map(|slip| {
                let horizontal_slip = (j.clone() as i32 + slip) as usize;
                self.relative_idx(i, horizontal_slip).and_then(|idx_next| {
                    self.cellules
                        .get_mut(idx_next)
                        .map(Cellule::is_dead)
                        .and_then(|is_dead_next| {
                            self.relative_idx(i + 1, horizontal_slip)
                                .and_then(|idx_below| {
                                    self.cellules
                                        .get_mut(idx_below)
                                        .map(Cellule::is_dead)
                                        .and_then(|is_dead_next_below| {
                                            (is_dead_next && is_dead_next_below)
                                                .then_some(idx_below)
                                        })
                                })
                        })
                })
            })
            .take_while(Option::is_some)
            .last()
            .flatten();

        if let Some(_slipped_idx) = slipped_idx {
            self.move_cell(idx, _slipped_idx as usize);
        }
    }

    fn move_cell(&mut self, origin_idx: usize, target_idx: usize) {
        let kind = self.cellules[origin_idx].kind.unwrap().clone();

        self.cellules[origin_idx].set_dead();
        self.cellules[target_idx].set_kind(kind).set_alive()
    }

    fn pressure(&mut self, i: usize, j: usize) -> u8 {
        // TODO: Implement pressure calculation for more directions
        let above = (0..=i)
            .rev()
            .take_while(|k| {
                self.relative_idx(*k, j)
                    .map(|_idx| self.cellules[_idx].is_alive())
                    .unwrap_or(false)
            })
            .count();

        above as u8
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
                    market_cells.push(idx);
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
        let _interval = Interval::new(50, move || callback.emit(()));
        let (cellules_width, cellules_height) = (80, 50);
        let rng = rand::thread_rng();
        let cellules = vec![
            Cellule {
                kind: Some(Kind::Sand),
                state: State::Dead,
                pressure: 0,
            };
            cellules_width * cellules_height
        ];

        Self {
            cellules,
            cellules_width,
            cellules_height,
            _interval,
            seed: rng,
            active: true,
            size_cursor: 8,
            creation_mode: CreationMode::Add,
            kind_cell: Kind::Sand,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::AddCellule(idx) => {
                self.cicle_cursor(idx).iter().for_each(|idx| {
                    self.cellules
                        .get_mut(*idx)
                        .unwrap()
                        .set_kind(self.kind_cell)
                        .set_alive();
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
                self.cicle_cursor(idx)
                    .into_iter()
                    .for_each(|idx| self.cellules.get_mut(idx).unwrap().set_mouse_over());
                true
            }
            Msg::MouseOut(idx) => {
                self.cicle_cursor(idx)
                    .iter()
                    .for_each(|idx| self.cellules.get_mut(*idx).unwrap().set_mouse_out());
                true
            }
            Msg::Pause => {
                self.active = false;
                false
            }
            Msg::Play => {
                self.active = true;
                false
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
                false
            }
            Msg::ChangeCreationMode(mode) => {
                self.creation_mode = mode;
                false
            }
            Msg::ChangeKindCell(kind) => {
                self.kind_cell = kind;
                false
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
            <section class={"simulation-container"}>
                <header class={"simulation-header"}>
                    <h1 class={"simulation-title"}>{ "Sand Simulation" }</h1>
                </header>
                <div class={"simulation-content"}>
                    <section class={"simulation-area"}>
                        <div class={"simulation-grid"}>{for cell_rows}</div>
                    </section>
                    <div class="control-area">
                        <div class="play-control control-buttons">
                            <button class="game-button" onclick={ctx.link().callback(|_| Msg::Play)}>{ "Play" }</button>
                            <button class="game-button" onclick={ctx.link().callback(|_| Msg::Pause)}>{ "Pause" }</button>
                            <button class="game-button" onclick={ctx.link().callback(|_| Msg::Reset)}>{ "Reset" }</button>
                        </div>

                        <label for="size-cursor">{ "Size cursor" }</label>
                        <input
                            type="range"
                            id="size-cursor"
                            class="control-buttons"
                            min="1"
                            max="20"
                            step="1"
                            value={self.size_cursor.to_string()}
                            oninput={handle_change_size_cursor}
                        />

                        <label for="crafte-mode">{ "Creation Mode" }</label>
                        <div class="crafte-mode control-buttons">
                            <button class="crafte-mode-buttom" onclick={ctx.link().callback(|_| Msg::ChangeCreationMode(CreationMode::Add))}>{ "Add" }</button>
                            <button class="crafte-mode-buttom" onclick={ctx.link().callback(|_| Msg::ChangeCreationMode(CreationMode::Remove))}>{ "Remove" }</button>
                        </div>

                        <label for="kind-cell">{ "Kind Cell" }</label>
                        <select class="control-buttons kind-cell" onchange={ctx.link().callback(|e: Event| {
                            let select: HtmlInputElement = e.target().unwrap().unchecked_into();
                            let kind = match select.value().as_str() {
                                "sand" => Kind::Sand,
                                "rock" => Kind::Rock,
                                _ => Kind::Sand
                            };
                            Msg::ChangeKindCell(kind)
                        })}>
                            <option value="sand" selected=true>{ "Sand" }</option>
                            <option value="rock">{ "Rock" }</option>
                        </select>
                    </div>
                </div>
            </section>
        }
    }
}
