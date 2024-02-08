// Frontend
use yew::prelude::*;
use wasm_bindgen::prelude::*;

mod ai;
use ai::Ai;

#[function_component(App)]
fn app() -> Html {
    wasm_logger::init(wasm_logger::Config::default());

    html! {
        <>
        <h1>{ "Hello World" }</h1>
        <Ai/>
        </>
    }
}

#[wasm_bindgen(start)]
fn run_app() {
    yew::Renderer::<App>::new().render();
}