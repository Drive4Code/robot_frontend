use charting_tools::charted_coordinate::ChartedCoordinate;
use charting_tools::ChartingTools;
use ohcrab_weather::weather_tool::WeatherPredictionTool;
// Project imports
use robotics_lib::energy::Energy;
use robotics_lib::event::events::{Event};
use robotics_lib::interface::{robot_map};

use robotics_lib::runner::backpack::{BackPack};
use robotics_lib::runner::{Robot, Runnable, Runner};
use robotics_lib::utils::LibError;
use robotics_lib::world::coordinates::Coordinate;
use robotics_lib::world::environmental_conditions::{EnvironmentalConditions, WeatherType};

use robotics_lib::world::tile::Content::{
    Bank, Bin, Building, Bush, Coin, Crate, Fire, Fish, Garbage, JollyBlock, Market, Rock,
    Scarecrow, Tree, Water,
};
use robotics_lib::world::tile::TileType::{
    DeepWater, Grass, Hill, Lava, Mountain, Sand, ShallowWater, Snow, Street, Teleport,
};
use robotics_lib::world::tile::{Content, Tile, TileType};
use robotics_lib::world::World;
use rust_eze_tomtom::TomTom;
use vent_tool_ascii_crab::Vent;
use std::collections::{HashMap, HashSet, VecDeque};

// Frontend
include!("worldloader.rs");
use std::path::PathBuf;
use yew::prelude::*;
use yew::{function_component, html, Html, Properties};
use bounce::*;
use gloo_timers::callback::Timeout;
use std::cell::RefCell;
use std::rc::Rc;
use log::info;
use wasm_bindgen::JsValue;
use crate::explorer::new_explorer;
use crate::utils::{calculate_spatial_index, execute_mission, get_world_dimension, ActiveRegion, Mission};
use rust_and_furious_dynamo::dynamo::Dynamo;
use web_sys::{window};
use wasm_bindgen_futures::JsFuture;



// enums to allow updates inside the impl
#[derive(Clone, PartialEq, Atom)]
struct BackpackState {
    size: usize,
    content: HashMap<Content, usize>,
}

impl Default for BackpackState {
    fn default() -> Self {
        Self {
            size: 0,
            content: HashMap::new(),
        }
    }
}

#[derive(Clone, PartialEq, Atom)]
struct StartAi(bool);

impl Default for StartAi {
    fn default() -> Self {
        Self(false)
    }
}

#[derive(Clone, PartialEq, Atom)]
struct WorldState {
    enviromentalConditions: EnvironmentalConditions,
    world: Vec<Vec<Option<Tile>>>,
    counter: usize,
}

impl Default for WorldState {
    fn default() -> Self {
        Self {
            enviromentalConditions: EnvironmentalConditions::new(&vec![WeatherType::Sunny], 1, 1).unwrap(),
            world: vec![vec![None]], // Temp Vec. The values here are literally useless.
            counter: 0,
        }
    }
}

#[derive(Clone, PartialEq, Atom)]
struct RobotState {
    coord: (usize, usize),
    // energy: usize,
}

impl Default for RobotState {
    fn default() -> Self {
        Self { coord: (0, 0) }
    }
}

#[function_component(Main)]
pub fn main() -> Html {
    let _startAi = use_atom::<StartAi>();
    let msg = JsValue::from(format!("Rendered Main"));
    info!("{}", msg.as_string().unwrap());
    html! {
        html! {
            <div id="info">
            
                // { if &startAi.0 == &true {
                //     // html! {<Ai/>}
                //     html! {<TimoAi/>}
                //     // html! {<></>}
                // } else {
                //     html! {<></>}
                // }
                // }

                <BackP/>
                <MapView/>
            </div>
            // <div id="grid">

            // </div>

        }
    }
}

#[function_component(ActivateAi)]
pub fn activateAi() -> Html {
    let startAi = use_atom::<StartAi>();
    startAi.set(StartAi(true));
    html! {
        <></>
    }
}

// #[function_component(Ai)]
// pub fn ai() -> Html {
//     // USESTATES
//     let backState = use_atom::<BackpackState>();
//     let world_state = use_atom::<WorldState>();
//     let robotState = use_atom::<RobotState>();

//     let msg = JsValue::from(format!("Ai Running"));
//     info!("{}", msg.as_string().unwrap());
//     let message = use_state(|| "Waiting...".to_string());
//     let _timeout_handle = use_mut_ref(|| None::<Timeout>);
//     {
//         let _message = message.clone();
//         use_effect(move || {
//             // Setup world
//             struct MyRobot(
//                 Robot,
//                 UseAtomHandle<BackpackState>,
//                 UseAtomHandle<WorldState>,
//                 UseAtomHandle<RobotState>,
//             );

//             impl Runnable for MyRobot {
//                 fn process_tick(&mut self, world: &mut World) {
//                     for _ in 0..1 {
//                         let (tmp, a, b) = debug(self, world);
//                         let environmental_conditions = look_at_sky(world);
//                         println!(
//                             "Daytime: {:?}, Time:{:?}, Weather: {:?}\n",
//                             environmental_conditions.get_time_of_day(),
//                             environmental_conditions.get_time_of_day_string(),
//                             environmental_conditions.get_weather_condition()
//                         );
//                         for elem in tmp.iter() {
//                             for tile in elem.iter() {
//                                 match tile.tile_type {
//                                     DeepWater => {
//                                         print!("DW");
//                                     }
//                                     ShallowWater => {
//                                         print!("SW");
//                                     }
//                                     Sand => {
//                                         print!("Sa");
//                                     }
//                                     Grass => {
//                                         print!("Gr");
//                                     }
//                                     Street => {
//                                         print!("St");
//                                     }
//                                     Hill => {
//                                         print!("Hi");
//                                     }
//                                     Mountain => {
//                                         print!("Mt");
//                                     }
//                                     Snow => {
//                                         print!("Sn");
//                                     }
//                                     Lava => {
//                                         print!("La");
//                                     }
//                                     Teleport(_) => {
//                                         print!("Tl");
//                                     }
//                                     TileType::Wall => {
//                                         print!("Wl");
//                                     }
//                                 }
//                                 match &tile.content {
//                                     Rock(quantity) => print!("->Ro {}", quantity),
//                                     Tree(quantity) => print!("->Tr {}", quantity),
//                                     Garbage(quantity) => print!("->Gr {}", quantity),
//                                     Fire => print!("->Fi -"),
//                                     Coin(quantity) => print!("->Co {}", quantity),
//                                     Bin(range) => print!("->Bi {}-{}", range.start, range.end),
//                                     Crate(range) => print!("->Cr {}-{}", range.start, range.end),
//                                     Bank(range) => print!("->Ba {}-{}", range.start, range.end),
//                                     Water(quantity) => print!("->Wa {}", quantity),
//                                     Content::None => print!("->No -"),
//                                     Fish(quantity) => print!("->Fh {}", quantity),
//                                     Market(quantity) => print!("->Mk {}", quantity),
//                                     Building => print!("->Bui -"),
//                                     Bush(quantity) => print!("->Bu {}", quantity),
//                                     JollyBlock(quantity) => print!("->Jo {}", quantity),
//                                     Scarecrow => print!("->Sc -"),
//                                 }
//                                 print!("\t| ");
//                             }
//                             println!();
//                         }
//                         println!("{:?}, {:?}", a, b);
//                         // match ris {
//                         //     | Ok(values) => println!("Ok"),
//                         //     | Err(e) => println!("{:?}", e),
//                         // }
//                     }
//                     println!("HERE {:?}", destroy(self, world, Direction::Down));
//                     let _ = go(self, world, Direction::Down);
//                     println!("CRAFT: {:?}", craft(self, Content::Garbage(0)));
//                     println!("\n\nBACKPACK: {:?}\n\n", self.get_backpack());
//                     println!("HERE {:?}", teleport(self, world, (1, 1)));
//                     // Update UI State
//                     let worldStatus = self.2.clone();
//                     // robotics_lib::interface::
//                     let tmpMap = robot_map(&world).unwrap_or_default();
//                     let msg = JsValue::from(format!("TEST {:?}", tmpMap));
//                     info!("{}", msg.as_string().unwrap());
//                     worldStatus.set(WorldState {
//                         world: tmpMap,
//                         enviromentalConditions: worldStatus.enviromentalConditions.clone(),
//                     });
//                 }

//                 fn handle_event(&mut self, event: Event) {
//                     println!();
//                     println!("{:?}", event);
//                     // Logs the event to the console
//                     let msg = JsValue::from(format!("{:?}", event));
//                     info!("{}", msg.as_string().unwrap());
//                     // Backpack Updates
//                     let backStatus = self.1.clone();
//                     match event {
//                         Event::AddedToBackpack(_, _) | Event::RemovedFromBackpack(_, _) => {
//                             let newBack = self.get_backpack();
//                             let newBackContent = newBack.get_contents();
//                             let newInside: HashMap<Content, usize> = (newBackContent.iter())
//                                 .map(|content| (content.0.to_owned(), content.1.to_owned()))
//                                 .collect();
//                             // HERE Implement the code to update a state inside the ai function component with the value of backpack size and content
//                             backStatus.set(BackpackState {
//                                 size: newBack.get_size(),
//                                 content: newInside,
//                             });
//                         }
//                         Event::Moved(newTile, (coord1, coord2)) => {
//                             self.3.set(RobotState {
//                                 coord: (coord1, coord2),
//                                 energy: self.3.energy.clone()
//                             });
//                         }
//                         // Event::Ready => todo!(),
//                         // Event::Terminated => todo!(),
//                         Event::TimeChanged(newEnviromentalConds) => {
//                             let worldStatus = self.2.clone();
//                             worldStatus.set(WorldState { world: worldStatus.world.clone(), enviromentalConditions: newEnviromentalConds })
//                         },
//                         Event::DayChanged(newEnviromentalConds) => {
//                             let worldStatus = self.2.clone();
//                             worldStatus.set(WorldState { world: worldStatus.world.clone(), enviromentalConditions: newEnviromentalConds })
//                         },
//                         Event::EnergyRecharged(_) => {
//                             let robotStatus = self.3.clone();
//                             robotStatus.set(RobotState {coord: robotStatus.coord, energy: self.get_energy().get_energy_level()});
//                         },
//                         Event::EnergyConsumed(_) => {
//                             let robotStatus = self.3.clone();
//                             robotStatus.set(RobotState {coord: robotStatus.coord, energy: self.get_energy().get_energy_level()});
//                         },
//                         Event::TileContentUpdated(_, _) => {
//                             let msg = JsValue::from(format!(
//                                 "Updated Coords: {:?}",
//                                 self.get_coordinate()
//                             ));
//                             info!("{}", msg.as_string().unwrap());
//                         }
//                         _ => println!("Before"),
//                     };

//                     println!();
//                 }

//                 fn get_energy(&self) -> &Energy {
//                     &self.0.energy
//                 }
//                 fn get_energy_mut(&mut self) -> &mut Energy {
//                     &mut self.0.energy
//                 }

//                 fn get_coordinate(&self) -> &Coordinate {
//                     &self.0.coordinate
//                 }
//                 fn get_coordinate_mut(&mut self) -> &mut Coordinate {
//                     &mut self.0.coordinate
//                 }

//                 fn get_backpack(&self) -> &BackPack {
//                     &self.0.backpack
//                 }
//                 fn get_backpack_mut(&mut self) -> &mut BackPack {
//                     &mut self.0.backpack
//                 }
//             }

//             // RUNNING THE GAME
//             let r = MyRobot(
//                 Robot::new(),
//                 backState.clone(),
//                 world_state.clone(),
//                 robotState.clone(),
//             );
//             struct Tool;
//             impl Tools for Tool {}
//             // let mut generator = WorldGenerator::init(4);
//             let mut generator = WorldgeneratorUnwrap::init(false, Some(PathBuf::from("world.bin")));
//             let run = Runner::new(Box::new(r), &mut generator);
//             //Known bug: 'check_world' inside 'Runner::new()' fails every time
//             println!("AO");
//             use_timeout(move || {
//             match run {
//                 Ok(mut r) => {
//                     for _ in 0..100 {
//                         let _ = &mut r.game_tick();
//                         let tmpCoords = r.get_robot().get_coordinate();
//                         let msg = JsValue::from(format!("Coords: {:?} Robot inside coords: {:?}", tmpCoords, robotState.coord));
                        
//                         info!("{}", msg.as_string().unwrap());
//                         // robotics_lib::interface::
//                     }
//                 }
//                 Err(e) => println!("{:?}", e),
//             }
//         }, 100);
//             || println!("Done!")
//         });
//     }
//     html! {
//         <></>
//     }
// }

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
    let img_display: String = contentMatch(&props.content);
    html! {
        <div class={classes!("back_item")}>
            <img  src={img_display}/>
            <h3>{format!("x{}", props.size)}</h3>
        </div>

    }
}

#[derive(PartialEq, Atom, Clone)]
struct CustomTimer {
    time: usize,
}

impl Default for CustomTimer {
    fn default() -> Self {
        Self {
            time: 0,
        }
    }
}

#[function_component(MapView)]
pub fn map_view() -> Html {
    let world_state = use_atom::<WorldState>();
    let robotState = use_atom::<RobotState>();
    let version = use_atom::<CustomTimer>();

    // Ref to store the timeout task
    // let timeout_task = use_mut_ref(|| None);

    // Update the world state periodically
    // {
    //     let world_state = world_state.clone();
    //     let version = version.clone();
    //     use_effect(move || {
    //         // Set up the timeout to periodically update the world state
    //         info!("RERENDERRRRR n.{}  WORLD: {:?}  ", version.time.clone(), world_state.world.clone());
    //         let handle = Timeout::new(100, move || {
    //                 // Update the world state here
    //                 // For example, you might fetch new data from the server
    //                 // or run some logic to update the state
    //                 // world_state.set(WorldState {world: world_state.world.clone(), enviromentalConditions: world_state.enviromentalConditions.clone()}); // Set the new world state
    //                 version.set(CustomTimer {time: version.time.clone() +1,});
    //             },
    //         );

    //         // Store the timeout task in the ref
    //         *timeout_task.borrow_mut() = Some(handle);

    //         // Cleanup function to cancel the timeout when the component unmounts
    //         move || {
    //             if let Some(task) = timeout_task.borrow_mut().take() {
    //                 task.cancel();
    //             }
    //         }
    //     });
    // }

    html! {
        <div id={"robot_view"}>
            <h2>{"Map"}</h2>
            <br/>
            {for world_state.world.clone().iter().enumerate().map(|(i, row)| {
                html! {
                    < div class={classes!("map_row")}>
                        { for row.iter().enumerate().map(|(j, tile_option)| {
                            match tile_option {
                                Some(tile) => html! {
                                    <div class={"tile"}>
                                    <MapTile tile={tile.clone()}/>
                                    {if i == robotState.coord.0.clone() && j == robotState.coord.1.clone() {
                                       html! {<img id={"robot"} src={"https://icons.iconarchive.com/icons/google/noto-emoji-smileys/1024/10103-robot-face-icon.png"} />}
                                    } else {
                                        html! {}
                                    }}
                                    </div>
                                    },
                                None => html! {<></>},
                            }
                        })}
                    </div>
                }
            })}
            <div>
                {format!("Conditions: {:?}", &world_state.enviromentalConditions)}
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct MapTileProps {
    tile: Tile,
}

#[function_component(MapTile)]
pub fn map_tile(props: &MapTileProps) -> Html {
    let img_display: &str;
    match props.tile.tile_type {
        TileType::Wall=>img_display="https://preview.redd.it/vf95xj8n6k251.jpg?auto=webp&s=8354e27ac03c946cf2c8a39bad68456a5e685bd0",
        DeepWater=>img_display="https://static.planetminecraft.com/files/image/minecraft/texture-pack/2020/887/13327895-pack_l.jpg",
        ShallowWater => img_display="https://i.pinimg.com/736x/7f/39/c5/7f39c55042808b3064364db03df40ac0.jpg",
        Sand =>img_display="https://www.fractalcamo.com/uploads/5/9/0/2/5902948/s189772745713394276_p3861_i148_w750.jpeg",
        Grass => img_display="https://i.pinimg.com/originals/cf/5e/27/cf5e272e452b9c7caa8fa0523eeeba9f.png",
        Street => img_display="https://lh3.bunny.novaskin.me/nOVtS_Zjk_vKOf48sw_x9Z2Cn8zIHYhs3TXEifYtbyriEWjS1D4i9W4bl5WmSdn9_SJp3Qy9Y41azSu-L8OQ2Q",
        Hill => img_display="https://i.redd.it/oxk07labr9b71.jpg",
        Mountain => img_display="https://www.filterforge.com/filters/11635-v8.jpg",
        Snow => img_display="https://dm0qx8t0i9gc9.cloudfront.net/watermarks/image/rDtN98Qoishumwih/white-snow-minecraft-pattern_SB_PM.jpg",
        Lava => img_display="https://www.fractalcamo.com/uploads/5/9/0/2/5902948/s189772745713394276_p8111_i149_w1500.jpeg",
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
    let img_display: String = contentMatch(&props.tile.content);
    if img_display == "" {
        html! {<></>}
    } else {
        html! {
            <img  class={classes!("tile_content")} src={img_display}/>
        }
    }
    
}

fn contentMatch(input: &Content) -> String {
    match input {
        Rock(_) =>return  "https://media.forgecdn.net/avatars/84/877/636198378292789888.png".to_string(),
        Tree(_) =>return  "https://minecraft.wiki/images/thumb/Azalea_Tree.png/250px-Azalea_Tree.png?945ad".to_string(),
        Garbage(_) => return "https://freepngimg.com/thumb/minecraft/70728-block-shelter-mine-terraria-minecraft:-pocket-edition.png".to_string(),
        Fire => return "https://gamepedia.cursecdn.com/minecraft_gamepedia/archive/3/30/20200127071142!Fire.png?version=2b5a474706c157ed26f2758972649977".to_string(),
        Coin(_) => return "https://webstockreview.net/images/coin-clipart-fandom-7.png".to_string(),
        Bin(_) => return "https://cdn.modrinth.com/data/Y9vogxIg/icon.png".to_string(),
        Crate(_) => return "https://gamepedia.cursecdn.com/minecraft_gamepedia/b/b3/Chest.png?version=227b3f51ef706a4ce4cf5e91f0e4face".to_string(),
        Bank(_) =>return  "https://vignette.wikia.nocookie.net/pixelpeople/images/a/ae/Bank.png/revision/latest?cb=20130904201633".to_string(),
        Water(_) => return "https://lh3.googleusercontent.com/MA3xe8ff0oksJ6Z_vBrg2scDLlX_uAXQxSnHfi5Ivc2MBPMWluYYrPGXHcSFWEtTQ8dTX-SQm4GAf-CJZKFkhA=s400".to_string(),
        Market(_) => return "https://gamepedia.cursecdn.com/minecraft_de_gamepedia/3/3c/Dorf.png".to_string(),
        Fish(_) =>return  "https://gamepedia.cursecdn.com/minecraft_gamepedia/a/ad/Tropical_Fish_JE2_BE2.png".to_string(),
        Building => return "https://gamepedia.cursecdn.com/minecraft_gamepedia/f/f5/Plains_Cartographer_1.png".to_string(),
        Bush(_) => return "https://gamepedia.cursecdn.com/minecraft_gamepedia/5/54/Berry_Bush_%28The_Aether%29.png?version=bb068bff721dfc749d68f5b87345dd56".to_string(),
        JollyBlock(_) => return "https://www.tynker.com/minecraft/editor/block/diamond_block/5cc07b98cebfbd1c2154195a/?image=true".to_string(),
        Scarecrow => return "https://lh3.googleusercontent.com/Wa9r8of1_KTeOtj5wEfDgRxUM2cq3MqrCVdUYkQy8D2hCtNZnuAFdJ1fF8D6lgpQRkRgLkkN8H1Yjnsr-oDclQ=s400".to_string(),
        Content::None => return "".to_string(),        
}
}





// TIMO CODE
pub(crate) struct Jerry{
    pub(crate) robot: Robot,
    pub(crate) bps: UseAtomHandle<BackpackState>,
    pub(crate) ws: UseAtomHandle<WorldState>,
    pub(crate) rs: UseAtomHandle<RobotState>,
    pub(crate) tick_counter: usize,
    pub(crate) world_dim: usize,
    pub(crate) active_region: ActiveRegion,
    pub(crate) road_tiles: HashSet<ChartedCoordinate>,
    pub(crate) vent: Rc<RefCell<Vent>>,
    pub(crate) dynamo: Dynamo,
    pub(crate) weather_predictor: WeatherPredictionTool,
    pub(crate) tom_tom: TomTom,
    pub(crate) charting_tools: ChartingTools,
    pub(crate) missions: VecDeque<Mission>,
}

#[function_component(TimoAi)]
pub fn timo_ai() -> Html {
    // USESTATES
    let backState = use_atom::<BackpackState>();
    let world_state = use_atom::<WorldState>();
    let robotState = use_atom::<RobotState>();

    let msg = JsValue::from(format!("Ai Running"));
    info!("{}", msg.as_string().unwrap());
    // let runner_ref = use_state_eq(|| None); timeout_jerry
    let timeout_handle = use_mut_ref(|| None::<Timeout>);
    {
        impl Runnable for Jerry {
            fn process_tick(&mut self, world: &mut World) {
                if self.tick_counter == 0{
                    first_tick(self, world);
                }
                    execute_mission( self, world);
                    println!("{:?} {}", self.robot.energy, self.tick_counter);
                    self.tick_counter += 1;
                    
                    // Update UI State
                    let tmpMap = robot_map(&world).unwrap_or_default();
                    info!("{:?} It's been a long way...",tmpMap);
                    let status = self.ws.clone();
                    if tmpMap != self.ws.world {
                        status.set(WorldState {
                            world: tmpMap,
                            enviromentalConditions: status.enviromentalConditions.clone(),
                            counter: status.counter.clone() +1
                        });
                        info!("CHANGED STATE");
                    }
            }

            fn handle_event(&mut self, event: Event) {
                println!();
                println!("{:?}", event);
                // Logs the event to the console
                let msg = JsValue::from(format!("{:?}", event));
                info!("[ EVENT ]{}", msg.as_string().unwrap());
                // Backpack Updates
                match event {
                    Event::AddedToBackpack(_, _) | Event::RemovedFromBackpack(_, _) => {
                        let newBack = self.get_backpack();
                        let newBackContent = newBack.get_contents();
                        let newInside: HashMap<Content, usize> = (newBackContent.iter())
                            .map(|content| (content.0.to_owned(), content.1.to_owned()))
                            .collect();
                        // HERE Implement the code to update a state inside the ai function component with the value of backpack size and content
                        self.bps.set(BackpackState {
                            size: newBack.get_size(),
                            content: newInside,
                        });
                    }
                    Event::Moved(_, position) => {
                        if position.0 >= self.active_region.bottom_right.0 {
                            self.active_region.bottom_right.0 = if position.0 == self.world_dim - 1 { self.world_dim - 1 } else { position.0 + 1 };
                        }
                        if position.1 >= self.active_region.bottom_right.1 {
                            self.active_region.bottom_right.1 = if position.1 == self.world_dim - 1 { self.world_dim - 1 } else { position.1 + 1 };
                        }
                        if position.0 <= self.active_region.top_left.0 {
                            self.active_region.top_left.0 = if position.0 == 0 { 0 } else { position.0 - 1 };
                        }
                        if position.1 <= self.active_region.top_left.1 {
                            self.active_region.top_left.1 = if position.1 == 0 { 0 } else { position.1 - 1 };
                        }
                        let tmp_coords = self.get_coordinate();
                        info!("NEW COOOOOOORDS{:?}", tmp_coords);
                        self.rs.set(RobotState {
                            coord: (tmp_coords.get_row(), tmp_coords.get_col()),
                            // energy: self.rs.energy.clone()
                        });
                    },
                    // Event::Ready => todo!(),
                    // Event::Terminated => todo!(),
                    // Event::TimeChanged(newEnviromentalConds) => {
                    //     let worldStatus = self.ws.clone();
                    //     worldStatus.set(WorldState { world: worldStatus.world.clone(), enviromentalConditions: newEnviromentalConds })
                    // },
                    // Event::DayChanged(newEnviromentalConds) => {
                    //     let worldStatus = self.ws.clone();
                    //     worldStatus.set(WorldState { world: worldStatus.world.clone(), enviromentalConditions: newEnviromentalConds })
                    // },
                    Event::EnergyRecharged(_) => {
                        // let robotStatus = self.rs.clone();
                        // robotStatus.set(RobotState {coord: robotStatus.coord, energy: self.get_energy().get_energy_level()});
                    },
                    Event::EnergyConsumed(_) => {
                        // let robotStatus = self.rs.clone();
                        // robotStatus.set(RobotState {coord: robotStatus.coord, energy: self.get_energy().get_energy_level()});
                    },
                    Event::TileContentUpdated(_, _) => {
                        let msg = JsValue::from(format!(
                            "Updated Coords: {:?}",
                            self.get_coordinate()
                        ));
                        info!("{}", msg.as_string().unwrap());
                    }
                    _ => println!("Before"),
                };

                println!();
            }

            fn get_energy(&self) -> &Energy {
                &self.robot.energy
            }
            fn get_energy_mut(&mut self) -> &mut Energy {
                &mut self.robot.energy
            }

            fn get_coordinate(&self) -> &Coordinate {
                &self.robot.coordinate
            }
            fn get_coordinate_mut(&mut self) -> &mut Coordinate {
                &mut self.robot.coordinate
            }

            fn get_backpack(&self) -> &BackPack {
                &self.robot.backpack
            }
            fn get_backpack_mut(&mut self) -> &mut BackPack {
                &mut self.robot.backpack
            }
        }
        fn first_tick(jerry: &mut Jerry, world: &mut World){
            let size = get_world_dimension(world);
            jerry.world_dim = size;
            jerry.active_region.spatial_index = calculate_spatial_index(jerry.get_coordinate().get_row(), jerry.get_coordinate().get_col(), size);
            let explorer = new_explorer(jerry, world, jerry.active_region.spatial_index);
            jerry.missions.push_back(explorer);
        }
        // RUNNING THE GAME
        let r = Jerry{
            robot: Robot::new(),
            bps: backState.clone(),
            ws: world_state.clone(),
            rs: robotState.clone(),
            tick_counter: 0,
            world_dim: 0,
            active_region: ActiveRegion{
                top_left: (279,279), 
                bottom_right: (0,0), 
                spatial_index: 0
            },
            vent: Rc::new(RefCell::new(Vent::new())),
            road_tiles: HashSet::new(),
            dynamo: Dynamo{},
            weather_predictor: WeatherPredictionTool::new(),
            tom_tom: TomTom{},
            charting_tools: ChartingTools,
            missions: VecDeque::new(),
            };

            let mut generator = WorldgeneratorUnwrap::init(false, Some(PathBuf::from("world.bin")));
            let run = Rc::new(RefCell::new(Runner::new(Box::new(r), &mut generator)));
            

        use_effect(move || {
            // Setup world
            let _timeout_handle = timeout_handle.clone();
            let _runner_result = run.clone();
            
            wasm_bindgen_futures::spawn_local(async move {
                let _done = run_game(run).await;
            });
            //Known bug: 'check_world' inside 'Runner::new()' fails every time
            info!("RERUNNING TICK");

            // *timeout_handle.borrow_mut() = Some(Timeout::new(1000, move || {
            //     for _  in 0..10000 {
            //         // Get a mutable reference to the Result<Runner>
                    
            //         // Handle the Result using map and map_err
                    
            //     }
                
            // }));

            || {};
        });
    }
    html! {
        <></>
    }
}




    async fn run_game(run: Rc<RefCell<Result<Runner, LibError>>>) -> () {
        // let _ = Delay::new(Duration::from_secs(1)).await;
        
        for _  in 0..10000 {
            sleep(5000).await;
            info!("Cook buddy!");
            // Get a mutable reference to the Result<Runner>
            let mut runner_result = run.borrow_mut();
            // Handle the Result using map and map_err
            runner_result.as_mut().map(|runner| {
                // runner is now a mutable reference to the Runner
                let _ = runner.game_tick();
                let _robot_coordinate = runner.get_robot().get_coordinate();
                // info!("LOST SOULSSSS   {:?} {:?}", world_state.world.clone(), robot_coordinate);
            }).map_err(|e| {
                info!("[ SEVERE ] ERROR WITH RUN: {:?}", e);
            }).unwrap_or_else(|_| {
                // Handle the case where the Result is an Err
                // This could be a no-op or some error handling code
            });
        }
    }


    async fn sleep(duration: u32) {
        let promise = js_sys::Promise::new(&mut |resolve, _| {
            window()
                .unwrap()
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    &resolve,
                    duration as i32,
                )
                .unwrap();
        });
    
        let _ = JsFuture::from(promise).await;
    }