// Frontend
use yew::prelude::*;
use wasm_bindgen::prelude::*;
use bounce::BounceRoot;
mod interface;
use interface::{Main, ActivateAi};

#[function_component(App)]
fn app() -> Html {
    wasm_logger::init(wasm_logger::Config::default());

    html! {
        <>
        <BounceRoot>
            <h1>{ "Robot Pripiat" }</h1>
            <Main/>
            <ActivateAi/>
        </BounceRoot>
        
        </>
    }
}

#[wasm_bindgen(start)]
fn run_app() {
    yew::Renderer::<App>::new().render();
}