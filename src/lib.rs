use yew::prelude::*;
use wasm_bindgen::prelude::*;
// use yew::services::console::ConsoleService;
// use yew::utils::document;

#[function_component(App)]
fn app() -> Html {
    html! {
        <>
        <h1>{ "Hello World" }</h1>
        </>
    }
}

#[wasm_bindgen(start)]
fn run_app() {
    yew::Renderer::<App>::new().render();
}