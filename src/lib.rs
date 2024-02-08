// Frontend
use yew::prelude::*;
use wasm_bindgen::prelude::*;
use bounce::BounceRoot;
mod ai;
use ai::Ai;

#[function_component(App)]
fn app() -> Html {
    wasm_logger::init(wasm_logger::Config::default());

    html! {
        <>
        <BounceRoot>
            <h1>{ "Robot Pripiat" }</h1>
            <Ai/>
        </BounceRoot>
        
        </>
    }
}

#[wasm_bindgen(start)]
fn run_app() {
    yew::Renderer::<App>::new().render();
}