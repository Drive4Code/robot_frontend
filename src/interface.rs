
// Project imports
use std::collections::HashMap;
use rand::Rng;
use strum::IntoEnumIterator;
use robotics_lib::energy::Energy;
use robotics_lib::event::events::{self, Event};
use robotics_lib::interface::{robot_view, Tools};
use robotics_lib::interface::{craft, debug, destroy, go, look_at_sky, teleport, Direction};
use robotics_lib::runner::backpack::{self, BackPack};
use robotics_lib::runner::{Robot, Runnable, Runner};
use robotics_lib::world::coordinates::Coordinate;
use robotics_lib::world::environmental_conditions::EnvironmentalConditions;
use robotics_lib::world::environmental_conditions::WeatherType::{Rainy, Sunny};
use robotics_lib::world::tile::Content::{
    Bank, Bin, Building, Bush, Coin, Crate, Fire, Fish, Garbage, JollyBlock, Market, Rock, Scarecrow, Tree, Water,
};
use robotics_lib::world::tile::TileType::{
    DeepWater, Grass, Hill, Lava, Mountain, Sand, ShallowWater, Snow, Street, Teleport,
};
use robotics_lib::world::tile::{Content, Tile, TileType};
use robotics_lib::world::world_generator::{Generator};
use robotics_lib::world::World;

// Frontend
use yew::prelude::*;
use yew::{function_component, html, Html, Properties};
extern crate yewdux;
use yewdux::prelude::*;
use yew_agent::{oneshot::{use_oneshot_runner, OneshotProvider}};
use bounce::*;
use std::rc::Rc;
use std::cell::RefCell;
use gloo_timers::callback::Timeout;


use log::info;
use wasm_bindgen::JsValue;


// enums to allow updates inside the impl
#[derive(Clone, PartialEq, Store, Atom)]
struct BackpackState {
    size: usize,
    content: HashMap<Content, usize>,
}

impl Default for BackpackState {
    fn default() -> Self {
        Self {
            size: 0,
            content: HashMap::new()
        }
    }
}



#[derive(Clone, PartialEq, Store, Atom)]
struct StartAi(bool);

impl Default for StartAi {
    fn default() -> Self {
        Self(false)
    }
}

#[derive(Clone, PartialEq, Store, Atom)]
struct WorldState {
    size: usize,
    world: Vec<Vec<Option<Tile>>>,
}

impl Default for WorldState {
    fn default() -> Self {
        Self {
            size: 0,
            world: vec![vec![None]], // Temp Vec. The values here are literally useless.
        }
    }
}






#[function_component(Main)]
pub fn main() -> Html{
    let startAi = use_atom::<StartAi>();
    let msg = JsValue::from(format!("Rendered Main"));
    info!("{}", msg.as_string().unwrap());
    html! {
        html! {
            <div id="info">
                { if &startAi.0 == &true {
                    html! {<Ai/>}
                } else {
                    html! {}
                }
                }
                 
                <BackP/>
                <MapView/>
            </div>
            // <div id="grid">
                
            // </div>
            
        }
    }

}

#[function_component(ActivateAi)]
pub fn activateAi() -> Html{
    let startAi = use_atom::<StartAi>();
    startAi.set(StartAi(true));
    html! {
        <></>
    }
}

#[function_component(Ai)]
pub fn ai() -> Html{
    // USESTATES
    let backState = use_atom::<BackpackState>();
    let worldState = use_atom::<WorldState>();

    let msg = JsValue::from(format!("Ai Running"));
    info!("{}", msg.as_string().unwrap());
    let message = use_state(|| "Waiting...".to_string());
    let timeout_handle = use_mut_ref(|| None::<Timeout>);
    {
        let message = message.clone();
    use_effect(move || {
        

    // Setup world
    struct MyRobot(Robot, UseAtomHandle<BackpackState>, UseAtomHandle<WorldState>);
    struct WorldGenerator {
        size: usize,
    }
    impl WorldGenerator {
        fn init(size: usize) -> Self {
            WorldGenerator { size }
        }
    }
    impl Generator for WorldGenerator {
        fn gen(
            &mut self,
        ) -> (
            Vec<Vec<Tile>>,
            (usize, usize),
            EnvironmentalConditions,
            f32,
            Option<HashMap<Content, f32>>,
        ) {
            let mut rng = rand::thread_rng();
            let mut map: Vec<Vec<Tile>> = Vec::new();
            // Initialize the map with default tiles
            for _ in 0..self.size {
                let mut row: Vec<Tile> = Vec::new();
                for _ in 0..self.size {
                    let i_tiletype = rng.gen_range(0..TileType::iter().len());
                    let i_content = rng.gen_range(0..Content::iter().len());
                    let tile_type = match i_tiletype {
                        | 0 => DeepWater,
                        | 1 => ShallowWater,
                        | 2 => Sand,
                        | 3 => Grass,
                        | 4 => Street,
                        | 5 => Hill,
                        | 6 => Mountain,
                        | 7 => Snow,
                        | 8 => Lava,
                        | 9 => Teleport(false),
                        | _ => Grass,
                    };
                    let content = match i_content {
                        | 0 => Rock(0),
                        | 1 => Tree(2),
                        | 2 => Garbage(2),
                        | 3 => Fire,
                        | 4 => Coin(2),
                        | 5 => Bin(2..3),
                        | 6 => Crate(2..3),
                        | 7 => Bank(3..54),
                        | 8 => Water(20),
                        | 10 => Fish(3),
                        | 11 => Market(20),
                        | 12 => Building,
                        | 13 => Bush(2),
                        | 14 => JollyBlock(2),
                        | 15 => Scarecrow,
                        | _ => Content::None,
                    };
                    row.push(Tile {
                        tile_type,
                        content,
                        elevation: 0,
                    });
                }
                map.push(row);
            }
            let environmental_conditions = EnvironmentalConditions::new(&[Sunny, Rainy], 15, 12).unwrap();

            let max_score = rand::random::<f32>();

            (map, (0, 0), environmental_conditions, max_score, None)
        }
    }

    
    impl Runnable for MyRobot {
        fn process_tick(&mut self, world: &mut World) {
            for _ in 0..1 {
                let (tmp, a, b) = debug(self, world);
                let environmental_conditions = look_at_sky(world);
                println!(
                    "Daytime: {:?}, Time:{:?}, Weather: {:?}\n",
                    environmental_conditions.get_time_of_day(),
                    environmental_conditions.get_time_of_day_string(),
                    environmental_conditions.get_weather_condition()
                );
                for elem in tmp.iter() {
                    for tile in elem.iter() {
                        match tile.tile_type {
                            | DeepWater => {
                                print!("DW");
                            }
                            | ShallowWater => {
                                print!("SW");
                            }
                            | Sand => {
                                print!("Sa");
                            }
                            | Grass => {
                                print!("Gr");
                            }
                            | Street => {
                                print!("St");
                            }
                            | Hill => {
                                print!("Hi");
                            }
                            | Mountain => {
                                print!("Mt");
                            }
                            | Snow => {
                                print!("Sn");
                            }
                            | Lava => {
                                print!("La");
                            }
                            | Teleport(_) => {
                                print!("Tl");
                            }
                            | TileType::Wall => {
                                print!("Wl");
                            }
                        }
                        match &tile.content {
                            | Rock(quantity) => print!("->Ro {}", quantity),
                            | Tree(quantity) => print!("->Tr {}", quantity),
                            | Garbage(quantity) => print!("->Gr {}", quantity),
                            | Fire => print!("->Fi -"),
                            | Coin(quantity) => print!("->Co {}", quantity),
                            | Bin(range) => print!("->Bi {}-{}", range.start, range.end),
                            | Crate(range) => print!("->Cr {}-{}", range.start, range.end),
                            | Bank(range) => print!("->Ba {}-{}", range.start, range.end),
                            | Water(quantity) => print!("->Wa {}", quantity),
                            | Content::None => print!("->No -"),
                            | Fish(quantity) => print!("->Fh {}", quantity),
                            | Market(quantity) => print!("->Mk {}", quantity),
                            | Building => print!("->Bui -"),
                            | Bush(quantity) => print!("->Bu {}", quantity),
                            | JollyBlock(quantity) => print!("->Jo {}", quantity),
                            | Scarecrow => print!("->Sc -"),
                        }
                        print!("\t| ");
                    }
                    println!();
                }
                println!("{:?}, {:?}", a, b);
                // match ris {
                //     | Ok(values) => println!("Ok"),
                //     | Err(e) => println!("{:?}", e),
                // }
            }
            println!("HERE {:?}", destroy(self, world, Direction::Down));
            let _ = go(self, world, Direction::Down);
            println!("CRAFT: {:?}", craft(self, Content::Garbage(0)));
            println!("\n\nBACKPACK: {:?}\n\n", self.get_backpack());
            println!("HERE {:?}", teleport(self, world, (1, 1)));
            // Update UI State
            let worldStatus = self.2.clone();
            worldStatus.set(WorldState{world:robotics_lib::interface::robot_map(world).unwrap_or_default(), size: 0 });
        }


        fn handle_event(&mut self, event: Event) {
            println!();
            println!("{:?}", event);
            // Logs the event to the console
            let msg = JsValue::from(format!("{:?}", event));
            info!("{}", msg.as_string().unwrap());
            // Backpack Updates
            let backStatus = self.1.clone();
            match event {
                Event::AddedToBackpack(_,_)|Event::RemovedFromBackpack(_,_)=>{
                let newBack = self.get_backpack();
                let newBackContent = newBack.get_contents();
                let newInside:HashMap<Content, usize> = (newBackContent.iter()).map(|content| (content.0.to_owned(), content.1.to_owned())).collect();
                // HERE Implement the code to update a state inside the ai function component with the value of backpack size and content
                backStatus.set(BackpackState { size: newBack.get_size(), content: newInside });
                },
            Event::Moved(newTile, (coord1, coord2)) => {
                let msg = JsValue::from(format!("Coords: {:?}", self.get_coordinate()));
                info!("{}", msg.as_string().unwrap());
                
            },
            // Event::Ready => todo!(),
            // Event::Terminated => todo!(),
            // Event::TimeChanged(_) => todo!(),
            // Event::DayChanged(_) => todo!(),
            // Event::EnergyRecharged(_) => todo!(),
            // Event::EnergyConsumed(_) => todo!(),
            
            Event::TileContentUpdated(_, _) => {
                let msg = JsValue::from(format!("Updated Coords: {:?}", self.get_coordinate()));
                info!("{}", msg.as_string().unwrap());
            }, 
            _ => println!("Before")
            };
            
            
            println!();
        }

        fn get_energy(&self) -> &Energy {
            &self.0.energy
        }
        fn get_energy_mut(&mut self) -> &mut Energy {
            &mut self.0.energy
        }

        fn get_coordinate(&self) -> &Coordinate {
            &self.0.coordinate
        }
        fn get_coordinate_mut(&mut self) -> &mut Coordinate {
            &mut self.0.coordinate
        }

        fn get_backpack(&self) -> &BackPack {
            &self.0.backpack
        }
        fn get_backpack_mut(&mut self) -> &mut BackPack {
            &mut self.0.backpack
        }

    } 




    
    // RUNNING THE GAME
    let r = MyRobot(Robot::new(), backState.clone(), worldState.clone());
    struct Tool;
    impl Tools for Tool {}
    let mut generator = WorldGenerator::init(4);
    // let mut generator = rip_worldgenerator::MyWorldGen::new();
    let run = Runner::new(Box::new(r), &mut generator);
    //Known bug: 'check_world' inside 'Runner::new()' fails every time
    println!("AO");
    *timeout_handle.borrow_mut() = Some(Timeout::new(1000, move || {
        message.set("Forxaroma".to_string());
        match run {
            | Ok(mut r) => {
                
                    println!("Ehssieh");
                    for _ in 0..1000 {
                        let _ = r.game_tick();
                        let coord = r.get_robot().get_coordinate();
                        let msg = JsValue::from(format!("Coords: {:?}", coord));
                        info!("{}", msg.as_string().unwrap());
                        // robotics_lib::interface::
                    }
               
            }
            | Err(e) => println!("{:?}", e),
        }
    }));
    

    || println!("Forza napoli!")
    });
}
    html! {
        <></>
    }
}



#[function_component(BackP)]
pub fn backpack() -> Html {
    let backState = use_atom::<BackpackState>();
    html! {
        <div id={"backpack"}>
            <h2>{"Backpack"}</h2>
            <hr/>
            {"Size: "}{ &backState.size}
            <br/>
            {"Contents: "} //{ format!("{:?}", &backState.content)}
            { for backState.content.iter().map(|content| {
                match content.1 {
                    0 => html! {<></>},
                    _ => html! {
                        <BackItem content={content.0.clone()} size={content.1.clone()}/>
                    }
                }
                
            })}
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct BackItemProps {
    content: Content,
    size: usize,
}


#[function_component(BackItem)]
fn backItem(props: &BackItemProps) -> Html {
    let img_display:&str;
    match props.content {
        Rock(_) => img_display = "https://www.freeiconspng.com/uploads/rock-stone-png-picture-0.png",
        Tree(_) => img_display = "https://pluspng.com/img-png/tree-png-tree-png-1903-2304-1903.png",
        Garbage(_) => img_display = "https://www.pngmart.com/files/7/Garbage-PNG-Clipart.png",
        Fire => img_display = "https://i1.wp.com/sreditingzone.com/wp-content/uploads/2018/07/fire-png-6.png?resize=859,1024&ssl=1",
        Coin(_) => img_display = "https://webstockreview.net/images/coin-clipart-fandom-7.png",
        Bin(_) => img_display = "https://purepng.com/public/uploads/large/purepng.com-trash-cantrash-cansteelplasticdustbinrecyclebin-1421526645886ziucf.png",
        Crate(_) => img_display = "https://www.textures4photoshop.com/tex/thumbs/wooden-box-PNG-free-thumb19.png",
        Bank(_) => img_display = "https://pngimg.com/uploads/bank/bank_PNG24.png",
        Water(_) => img_display = "https://pngimg.com/uploads/water/water_PNG3290.png",
        Market(_) => img_display = "https://static.vecteezy.com/system/resources/previews/013/822/272/original/3d-illustration-store-market-png.png",
        Fish(_) => img_display = "https://www.freeiconspng.com/uploads/fish-png-4.png",
        Building => img_display = "https://www.pngmart.com/files/21/3D-Building-PNG-Pic.png",
        Bush(_) => img_display = "https://pluspng.com/img-png/bush-png-bush-png-image-1024.png",
        JollyBlock(_) => img_display = "https://www.tynker.com/minecraft/editor/block/diamond_block/5cc07b98cebfbd1c2154195a/?image=true",
        Scarecrow => img_display = "https://clipground.com/images/scarecrow-png.png",
        Content::None => img_display = "https://www.freeiconspng.com/uploads/no-image-icon-11.PNG",        
}
    html! {
        <div class={classes!("back_item")}>
            <img  src={img_display}/>
            <h3>{format!("x{}", props.size)}</h3>
        </div>
        
    }
}


#[function_component(MapView)]
pub fn map_view() -> Html {
    let worldState = use_atom::<WorldState>();
    html! {
        <div id={"robot_view"}>
            <h2>{"Map"}</h2>
            {for worldState.clone().world.iter().map(|row| {
                html! {
                    < div class={classes!("map_row")}>
                        { for row.iter().map(|tile_option| {
                            match tile_option {
                                Some(tile) => html! {<MapTile tile={tile.clone()} />},
                                None => html! {<></>},
                            }
                        })}
                    </div>
                }
            })}
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct MapTileProps {
    tile: Tile,
}

#[function_component(MapTile)]
pub fn map_tile(props: &MapTileProps) -> Html {
    let img_display:&str;
    match props.tile.tile_type {
        TileType::Wall=>img_display="https://static.wikia.nocookie.net/terraria_gamepedia/images/3/3e/Wood_Wall_%28placed%29.png/revision/latest?cb=20160525222715&format=original",
        DeepWater=>img_display="https://static.wikia.nocookie.net/terraria_gamepedia/images/9/9d/Water.png/revision/latest?cb=20200809004326&format=original",
        ShallowWater => img_display="https://static.wikia.nocookie.net/terraria_gamepedia/images/9/9d/Water.png/revision/latest?cb=20200809004326&format=original",
        Sand => img_display="https://gamepedia.cursecdn.com/terraria_gamepedia/7/77/Sand_Block_(placed).png?version=bb7b9cb0559589bdbb9d828cff6f216a",
        Grass => img_display="https://media.istockphoto.com/photos/background-comprised-of-small-green-leaves-picture-id182794428?k=6&m=182794428&s=612x612&w=0&h=FnVDRO-yH29d6GfytobxfRL2aUfMnWwCXPt3PDOH0pE=",
        Street => img_display="https://static.wikia.nocookie.net/terraria_gamepedia/images/8/8e/Stone_Block_(placed).png/revision/latest?cb=20160626075258",
        Hill => img_display="https://pixelartmaker-data-78746291193.nyc3.digitaloceanspaces.com/image/4ea15419789cb52.png",
        Mountain => img_display="https://gamepedia.cursecdn.com/terraria_gamepedia/5/5c/Ash_Block_(placed).png?version=c199d2114093e0b4fc64f638cb4345ce",
        Snow => img_display="https://static.wikia.nocookie.net/terraria_gamepedia/images/d/d4/Snow_Block_%28placed%29.png/revision/latest?cb=20160624153650&format=original",
        Lava => img_display="https://static.wikia.nocookie.net/terraria_gamepedia/images/2/27/Lava.png/revision/latest?cb=20200809004452&format=original",
        Teleport(_) => img_display="https://gamepedia.cursecdn.com/minecraft_es_gamepedia/e/e4/NetherPortal.gif?version=ee833d337bb150e012426cb883b337a7", 
}
    html! {
        <div class={classes!("tile")}>
            <img class={classes!("tile_type")} src={img_display}/>
            <MapTileContent tile={props.tile.clone()}/>
        </div>
        
    }
}

#[function_component(MapTileContent)]
pub fn map_tile_content(props: &MapTileProps) -> Html {
    let img_display:&str;
    match props.tile.content {
        Rock(_) => img_display = "https://www.freeiconspng.com/uploads/rock-stone-png-picture-0.png",
        Tree(_) => img_display = "https://pluspng.com/img-png/tree-png-tree-png-1903-2304-1903.png",
        Garbage(_) => img_display = "https://www.pngmart.com/files/7/Garbage-PNG-Clipart.png",
        Fire => img_display = "https://i1.wp.com/sreditingzone.com/wp-content/uploads/2018/07/fire-png-6.png?resize=859,1024&ssl=1",
        Coin(_) => img_display = "https://webstockreview.net/images/coin-clipart-fandom-7.png",
        Bin(_) => img_display = "https://purepng.com/public/uploads/large/purepng.com-trash-cantrash-cansteelplasticdustbinrecyclebin-1421526645886ziucf.png",
        Crate(_) => img_display = "https://www.textures4photoshop.com/tex/thumbs/wooden-box-PNG-free-thumb19.png",
        Bank(_) => img_display = "https://pngimg.com/uploads/bank/bank_PNG24.png",
        Water(_) => img_display = "https://pngimg.com/uploads/water/water_PNG3290.png",
        Market(_) => img_display = "https://static.vecteezy.com/system/resources/previews/013/822/272/original/3d-illustration-store-market-png.png",
        Fish(_) => img_display = "https://www.freeiconspng.com/uploads/fish-png-4.png",
        Building => img_display = "https://www.pngmart.com/files/21/3D-Building-PNG-Pic.png",
        Bush(_) => img_display = "https://pluspng.com/img-png/bush-png-bush-png-image-1024.png",
        JollyBlock(_) => img_display = "https://www.tynker.com/minecraft/editor/block/diamond_block/5cc07b98cebfbd1c2154195a/?image=true",
        Scarecrow => img_display = "https://clipground.com/images/scarecrow-png.png",
        Content::None => img_display = "https://www.freeiconspng.com/uploads/no-image-icon-11.PNG",        
}
        

        // _ => img_display = "ERR",
    
    html! {
        <img class={classes!("tile_content")} src={img_display}/>
    }
}