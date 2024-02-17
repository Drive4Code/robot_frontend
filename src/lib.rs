// Frontend
use yew::prelude::*;
use wasm_bindgen::prelude::*;
use bounce::BounceRoot;


pub mod interface;
pub mod biomes;
pub mod explorer;
pub mod morans_i;
pub mod utils;
pub mod sector_analyzer;
pub mod road_builder;
pub mod resources;
pub mod fast_paths;

use interface::{Main, TimoAi};

#[function_component(App)]
fn app() -> Html {
    wasm_logger::init(wasm_logger::Config::default());

    html! {
        <>
        <BounceRoot>
            // <h1>{ "Robot" }</h1>
            <Main/>
            // This next line chooses which AI to start
            <TimoAi />
        </BounceRoot>
        
        </>
    }
}

#[wasm_bindgen(start)]
fn run_app() {
    // RunGame::registrar().register();
    yew::Renderer::<App>::new().render();
}