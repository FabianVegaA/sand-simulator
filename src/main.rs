use gloo_console::log;

mod app;

use app::App;

fn main() {
    log!("Hello, world!");
    yew::Renderer::<App>::new().render();
}
