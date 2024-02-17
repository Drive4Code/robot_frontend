use charting_tools::charted_coordinate::ChartedCoordinate;
use charting_tools::ChartingTools;
use ohcrab_weather::weather_tool::WeatherPredictionTool;
// Project imports
use robotics_lib::energy::Energy;
use robotics_lib::event::events::{Event};
use robotics_lib::interface::{look_at_sky, robot_map};

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
use crate::explorer::new_explorer;
use crate::utils::{calculate_spatial_index, execute_mission, get_world_dimension, ActiveRegion, Mission};
use rust_and_furious_dynamo::dynamo::Dynamo;
use bob_lib::tracker::*;
use rust_eze_tomtom::plain::*;
use robotics_lib::interface::destroy;

// Nick extra imports
use rand::Rng;
use robotics_lib::interface::Direction;
use robotics_lib::interface::go;
use robotics_lib::utils::go_allowed;
use bessie::bessie::*;
use robotics_lib::world::environmental_conditions::WeatherType::*;

// Frontend
include!("worldloader.rs");
use std::path::PathBuf;
use yew::prelude::*;
use yew::{function_component, html, Html, Properties};
use bounce::*;
use std::cell::RefCell;
use std::rc::Rc;
use log::info;
use wasm_bindgen::JsValue;
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
struct WorldState {
    world: Vec<Vec<Option<Tile>>>,
    counter: usize,
}

impl Default for WorldState {
    fn default() -> Self {
        Self {
            world: vec![vec![None]], // Placeholders
            counter: 0,
        }
    }
}

#[derive(Clone, PartialEq, Atom)]
struct EnviromentalState {
    forecast: WeatherType,
    time: String,
}

impl Default for EnviromentalState {
    fn default() -> Self {
        Self {
            forecast: WeatherType::Sunny,
            time: String::new(),
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

#[derive(Clone, PartialEq, Atom)]
struct EnergyState {
    energy: usize,
    // energy: usize,
}

impl Default for EnergyState {
    fn default() -> Self {
        Self { energy: 0 }
    }
}

#[function_component(Main)]
pub fn main() -> Html {
    let msg = JsValue::from(format!("Rendered Main"));
    info!("{}", msg.as_string().unwrap());
    html! {
        html! {
            <div id="info">
                <BackP/>
                <EnergyBar/>
                <EnviromentBar />
                <br/>
                <MapView/>
            </div>

        }
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
    let img_display: String = content_match(&props.content);
    html! {
        <div class={classes!("back_item")}>
            <img  src={img_display}/>
            <h3>{format!("x{}", props.size)}</h3>
        </div>

    }
}

#[function_component(EnergyBar)]
fn energy_display() -> Html {
    let energy_amount = use_atom::<EnergyState>();

    html! {
        <div id="energy">
            <img src={"https://hotemoji.com/images/emoji/2/1gy0ubymkz6p2.png"}/>
            <h3>{&energy_amount.energy}</h3>
        </div>
    }
}

#[function_component(EnviromentBar)]
fn enviroment_display() -> Html {
    let enviroment_state = use_atom::<EnviromentalState>();
    let forecast_image = match_forecast(&enviroment_state.forecast);

    html! {
        <div id="enviroment">
            <img src={forecast_image} />
            <h3>{format!("{}", &enviroment_state.time)}</h3>
        </div>
    }
}

fn match_forecast(conditions: &WeatherType) -> String {
    match conditions {
        WeatherType::Sunny => "https://www.pngall.com/wp-content/uploads/2016/07/Sun-PNG-Image-180x180.png".to_string(),
        WeatherType::Rainy => "https://borregowildflowers.org/img_sys/rain.png".to_string(),
        WeatherType::Foggy => "https://cdn-icons-png.flaticon.com/128/2076/2076827.png".to_string(),
        WeatherType::TropicalMonsoon => "https://heat-project.weebly.com/uploads/7/1/4/2/71428073/published/bez-nazxdccwy-1_1.png?1533897845".to_string(),
        WeatherType::TrentinoSnow => "https://cdn.icon-icons.com/icons2/33/PNG/128/snow_cloud_weather_2787.png".to_string(),
    }
}

#[function_component(MapView)]
pub fn map_view() -> Html {
    let world_state = use_atom::<WorldState>();
    let robotState = use_atom::<RobotState>();

    html! {
        <div id={"robot_view"}>
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
                                None => html! {
                                    // <></>
                                    <div class={classes!("tile")} style={"width: var(--tile-size); height: var(--tile-size); background-color: var(--background-color);"}></div>
                                },
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
    let img_display: String = content_match(&props.tile.content);
    if img_display == "" {
        html! {<></>}
    } else {
        html! {
            <img  class={classes!("tile_content")} src={img_display}/>
        }
    }
    
}

fn content_match(input: &Content) -> String {
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
    pub(crate) env: UseAtomHandle<EnviromentalState>,
    pub(crate) en: UseAtomHandle<EnergyState>,
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
    let env_state = use_atom::<EnviromentalState>();
    let energy_state = use_atom::<EnergyState>();

    let msg = JsValue::from(format!("Ai Running"));
    info!("{}", msg.as_string().unwrap());
    // let runner_ref = use_state_eq(|| None); timeout_jerry
    {
        impl Runnable for Jerry {
            fn process_tick(&mut self, world: &mut World) {
                if self.tick_counter == 0{
                    first_tick(self, world);
                }
                    execute_mission( self, world);
                    info!("{:?} {}", self.robot.energy, self.tick_counter);
                    self.tick_counter += 1;
                    
                    // Update UI State
                    let tmpMap = robot_map(&world).unwrap_or_default();
                    let tmp_conditions = look_at_sky(&world);
                    // info!("{:?} Internal Map",tmpMap);
                    if tmpMap != self.ws.world {
                        self.ws.set(WorldState {
                            world: tmpMap,
                            counter: self.ws.counter.clone() +1
                        });
                        // info!("CHANGED WORLD");
                    }
                    let tmp_time = tmp_conditions.get_time_of_day_string();
                    if self.env.time != tmp_time{
                    self.env.set(EnviromentalState { 
                        forecast: tmp_conditions.get_weather_condition(),
                        time: tmp_time,
                     });
                    }
                    // info!("CHANGED CONDITIONS");

            }

            fn handle_event(&mut self, event: Event) {
                info!("");
                info!("{:?}", event);
                // Logs the event to the console
                let msg = JsValue::from(format!("{:?}", event));
                // info!("[ EVENT ]{}", msg.as_string().unwrap());
                // Backpack Updates
                match event {
                    Event::AddedToBackpack(_, _) | Event::RemovedFromBackpack(_, _) => {
                        let newBack = self.get_backpack();
                        let newBackContent = newBack.get_contents();
                        let newInside: HashMap<Content, usize> = (newBackContent.iter())
                            .map(|content| (content.0.to_owned(), content.1.to_owned()))
                            .collect();
                        // HERE Implement the code to update a state inside the ai function component with the value of backpack size and content
                        if self.bps.content != newInside {
                            self.bps.set(BackpackState {
                                size: newBack.get_size(),
                                content: newInside,
                            });
                            info!("[ State Update ] New Backpack State");
                        }
                        
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
                        // info!("[ State Update ] NEW COORDS: {:?}", tmp_coords);
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
                        
                    // },
                    Event::EnergyRecharged(_) | Event::EnergyConsumed(_) => {
                        // let robotStatus = self.rs.clone();
                        // robotStatus.set(RobotState {coord: robotStatus.coord, energy: self.get_energy().get_energy_level()});
                        let tmp_energy = self.get_energy().get_energy_level();
                        if self.en.energy != tmp_energy {
                            self.en.set(EnergyState {
                                energy: tmp_energy,
                            })
                        }
                    },
                    _ => (),
                };

                info!("");
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
            env: env_state.clone(),
            en: energy_state.clone(),
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
            
            if world_state.counter == 0 {
                info!("STARTING GAME...");
                wasm_bindgen_futures::spawn_local(async move {
                    let _done = run_game(run).await;
                });
            }
    }
    html! {
        <></>
    }
}




































// NICO CODE
#[function_component(NicoAi)]
pub fn nico_ai() -> Html {
    let backState = use_atom::<BackpackState>();
    let world_state = use_atom::<WorldState>();
    let robotState = use_atom::<RobotState>();
    let env_state = use_atom::<EnviromentalState>();
    let energy_state = use_atom::<EnergyState>();

    let msg = JsValue::from(format!("Ai Running"));
    info!("{}", msg.as_string().unwrap());

    pub(crate) struct MyRobot {
        pub(crate) robot: Robot,
        pub(crate) bps: UseAtomHandle<BackpackState>,
        pub(crate) ws: UseAtomHandle<WorldState>,
        pub(crate) rs: UseAtomHandle<RobotState>,
        pub(crate) env: UseAtomHandle<EnviromentalState>,
        pub(crate) en: UseAtomHandle<EnergyState>
    }

    impl Runnable for MyRobot {
        fn process_tick(&mut self, world: &mut World) {
            let mut w = WeatherPredictionTool::new();
            info!("Weather prediction for next tick! The desert will experience a {:?} day! We should get everything done before that!",
                     WeatherPredictionTool::predict(&mut w, 1).unwrap_or(Sunny));
            call(self, world, "rock", 0);
            let tmpMap = robot_map(&world).unwrap_or_default();
            let tmp_conditions = look_at_sky(&world);
            info!("{:?} Internal Map",tmpMap);
            if tmpMap != self.ws.world {
                self.ws.set(WorldState {
                    world: tmpMap,
                    counter: self.ws.counter.clone() + 1
                });
                info!("CHANGED WORLD");
            }
            let tmp_time = tmp_conditions.get_time_of_day_string();
            if self.env.time != tmp_time {
                self.env.set(EnviromentalState {
                    forecast: tmp_conditions.get_weather_condition(),
                    time: tmp_time,
                });
            }
            info!("CHANGED CONDITIONS");
        }

        fn handle_event(&mut self, event: Event) {
            info!("event: {:?}", event.to_string());
            let msg = JsValue::from(format!("{:?}", event));
            // info!("[ EVENT ]{}", msg.as_string().unwrap());
            // Event Updates
            match event {
                Event::AddedToBackpack(_, _) | Event::RemovedFromBackpack(_, _) => {
                    let newBack = self.get_backpack();
                    let newBackContent = newBack.get_contents();
                    let newInside: HashMap<Content, usize> = (newBackContent.iter())
                        .map(|content| (content.0.to_owned(), content.1.to_owned()))
                        .collect();
                    // HERE Implement the code to update a state inside the ai function component with the value of backpack size and content
                    if self.bps.content != newInside {
                        self.bps.set(BackpackState {
                            size: newBack.get_size(),
                            content: newInside,
                        });
                        info!("[ State Update ] New Backpack State");
                    }

                }
                Event::Moved(_, _) => {
                    let tmp_coords = self.get_coordinate();
                    // info!("[ State Update ] NEW COORDS: {:?}", tmp_coords);
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

                // },
                Event::EnergyRecharged(_) | Event::EnergyConsumed(_) => {
                    // let robotStatus = self.rs.clone();
                    // robotStatus.set(RobotState {coord: robotStatus.coord, energy: self.get_energy().get_energy_level()});
                    let tmp_energy = self.get_energy().get_energy_level();
                    if self.en.energy != tmp_energy {
                        self.en.set(EnergyState {
                            energy: tmp_energy,
                        })
                    }
                },
                _ => (),
            };

            info!("");
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

    let robotz = MyRobot {
        robot: Robot::new(),
        bps: backState.clone(),
        ws: world_state.clone(),
        rs: robotState.clone(),
        env: env_state.clone(),
        en: energy_state.clone()
    };
    let mut worldgen = WorldgeneratorUnwrap::init(false, Some(PathBuf::from("nworld.bin")));
    let mut runner = Rc::new(RefCell::new(Runner::new(Box::new(robotz), &mut worldgen)));

    if world_state.counter == 0 {
        info!("STARTING GAME...");
        wasm_bindgen_futures::spawn_local(async move {
            let _done = run_game(runner).await;
        });
    }

    
    fn rock(robot: &mut impl Runnable, world: &mut World) {
        recharge(robot);
            match TomTom::go_to_tile(robot, world, false, None, Some(PlainContent::Rock)) {
                Ok(path) => {
                    recharge(robot);
                    info!("A rock was found!");
                    TomTom::go_to_tile(robot, world, false, None, Some(PlainContent::Rock));
                    go(robot, world, Direction::Down);
                    match bessie::bessie::road_paving_machine(
                        robot,
                        world,
                        Direction::Up,
                        bessie::bessie::State::GetStones) {
                        Ok(b) => info!("{:?}", b),
                        Err(e) => {
                            match e {
                                RpmError::UndefinedError => call(robot, world, "street", 1),
                                _ => info!("{:?}", e)
                            }
                        }
                    }
                },

                _ =>
                    {
                        random_movement(robot, world, 'r');
                    }
            };
        return;
    }

   
    fn street_paving_mode(robot: &mut impl Runnable, world: &mut World, cycle: i32) {
        info!("Let's build some roads");
        recharge(robot);
        while !robot.get_backpack().get_contents().is_empty() {
            match cycle {
                1 => {
                    for n in 1..=10 {
                        if robot.get_coordinate().get_col() == 279 {
                            recover_oob(robot, world, Direction::Right);
                            //street_paving_mode(robot, world, cycle);
                            return;
                        }
                        match destroy(robot, world, Direction::Right) {
                            Err(e) => {
                                recover_oob(robot, world, Direction::Right);
                                //if go_allowed(robot, world, &Direction::Left).is_ok() {
                                    go(robot, world, Direction::Left);
                                // } else {
                                //     street_paving_mode(robot, world, cycle * -1);
                                // }
                                match bessie::bessie::road_paving_machine(
                                    robot,
                                    world,
                                    Direction::Right,
                                    bessie::bessie::State::MakeRoad) {
                                    Err(RpmError::CannotPlaceHere) => {
                                        recover_oob(robot, world, Direction::Right);
                                        //if go_allowed(robot, world, &Direction::Left).is_ok() {
                                            go(robot, world, Direction::Left);
                                        // } else {
                                        //     street_paving_mode(robot, world, cycle * -1);
                                        // }
                                        //street_paving_mode(robot, world, cycle);
                                        return;
                                    },
                                    Ok(p) => info!("{:?}", p),
                                    _ => {
                                        //street_paving_mode(robot, world, -1);
                                        return;
                                    }
                                }
                            },
                            _ => {
                                for (key, value) in robot.get_backpack().get_contents().clone() {
                                    if key == Content::Garbage(0) {
                                        if value == 0usize {
                                            recover_oob(robot, world, Direction::Right);
                                            //if go_allowed(robot, world, &Direction::Left).is_ok() {
                                                go(robot, world, Direction::Left);
                                            // } else {
                                            //     street_paving_mode(robot, world, cycle * -1);
                                            // }
                                            info!("{:?}",
                                                     bessie::bessie::road_paving_machine(
                                                         robot,
                                                         world,
                                                         Direction::Right,
                                                         bessie::bessie::State::MakeRoad
                                                     )
                                            );
                                            return;
                                        } else {
                                            call(robot, world, "garbage", 0);
                                            return;
                                        }
                                    }
                                }
                            }
                        }
                    };
                    if robot.get_coordinate().get_row() == 279 {
                        recover_oob(robot, world, Direction::Down);
                    }
                    match destroy(robot, world, Direction::Down) {
                        Err(e) => {
                            recover_oob(robot, world, Direction::Down);
                            //if go_allowed(robot, world, &Direction::Up).is_ok() {
                                go(robot, world, Direction::Up);
                            // } else {
                            //     street_paving_mode(robot, world, cycle * -1);
                            // }
                            match bessie::bessie::road_paving_machine(
                                robot,
                                world,
                                Direction::Down,
                                bessie::bessie::State::MakeRoad) {
                                Err(RpmError::CannotPlaceHere) => {
                                    recover_oob(robot, world, Direction::Down);
                                    //if go_allowed(robot, world, &Direction::Up).is_ok() {
                                        go(robot, world, Direction::Up);
                                    // } else {
                                    //     street_paving_mode(robot, world, cycle * -1);
                                    // }
                                    //street_paving_mode(robot, world, cycle);
                                    return;
                                },
                                Ok(p) => info!("{:?}", p),
                                _ => {
                                    //street_paving_mode(robot, world, -1);
                                    return;
                                }
                            }
                        },
                        _ => {
                            for (key, value) in robot.get_backpack().get_contents().clone() {
                                if key == Content::Garbage(0) {
                                    if value == 0usize {
                                        recover_oob(robot, world, Direction::Down);
                                        //if go_allowed(robot, world, &Direction::Up).is_ok() {
                                            go(robot, world, Direction::Up);
                                        // } else {
                                        //     street_paving_mode(robot, world, cycle * -1);
                                        // }
                                        info!("{:?}",
                                                 bessie::bessie::road_paving_machine(
                                                     robot,
                                                     world,
                                                     Direction::Down,
                                                     bessie::bessie::State::MakeRoad
                                                 )
                                        );
                                        return;
                                    } else {
                                        call(robot, world, "garbage", 0);
                                        return;
                                    }
                                }
                            }
                        }
                    }
                },
                -1 => {
                    for n in 1..=10 {
                        if robot.get_coordinate().get_col() == 0 {
                            recover_oob(robot, world, Direction::Left);
                            //street_paving_mode(robot, world, cycle);
                            return;
                        }
                        match destroy(robot, world, Direction::Left) {
                            Err(e) => {
                                recover_oob(robot, world, Direction::Left);
                                //if go_allowed(robot, world, &Direction::Right).is_ok() {
                                    go(robot, world, Direction::Right);
                                // } else {
                                //     street_paving_mode(robot, world, cycle * -1);
                                // }
                                match bessie::bessie::road_paving_machine(
                                    robot,
                                    world,
                                    Direction::Left,
                                    bessie::bessie::State::MakeRoad) {
                                    Err(RpmError::CannotPlaceHere) => {
                                        recover_oob(robot, world, Direction::Left);
                                        // if go_allowed(robot, world, &Direction::Right).is_ok() {
                                            go(robot, world, Direction::Right);
                                        // } else {
                                        //     street_paving_mode(robot, world, cycle * -1);
                                        // }
                                        //street_paving_mode(robot, world, cycle);
                                        return;
                                    },
                                    Ok(p) => info!("{:?}", p),
                                    _ => {
                                        //street_paving_mode(robot, world, -1);
                                        return;
                                    }
                                }
                            },
                            _ => {
                                for (key, value) in robot.get_backpack().get_contents().clone() {
                                    if key == Content::Garbage(0) {
                                        if value == 0usize {
                                            recover_oob(robot, world, Direction::Left);
                                            //if go_allowed(robot, world, &Direction::Right).is_ok() {
                                                go(robot, world, Direction::Right);
                                            // } else {
                                            //     street_paving_mode(robot, world, cycle * -1);
                                            // }
                                            info!("{:?}",
                                                     bessie::bessie::road_paving_machine(
                                                         robot,
                                                         world,
                                                         Direction::Left,
                                                         bessie::bessie::State::MakeRoad
                                                     )
                                            );
                                            return;
                                        } else {
                                            call(robot, world, "garbage", 0);
                                            return;
                                        }
                                    }
                                }
                            }
                        }
                    };
                    if robot.get_coordinate().get_row() == 279 {
                        recover_oob(robot, world, Direction::Down);
                    }
                    match destroy(robot, world, Direction::Down) {
                        Err(e) => {
                            recover_oob(robot, world, Direction::Down);
                            //if go_allowed(robot, world, &Direction::Up).is_ok() {
                                go(robot, world, Direction::Up);
                            // } else {
                            //     street_paving_mode(robot, world, cycle * -1);
                            // }
                            match bessie::bessie::road_paving_machine(
                                robot,
                                world,
                                Direction::Down,
                                bessie::bessie::State::MakeRoad) {
                                Err(RpmError::CannotPlaceHere) => {
                                    recover_oob(robot, world, Direction::Down);
                                    //if go_allowed(robot, world, &Direction::Up).is_ok() {
                                        go(robot, world, Direction::Up);
                                    // } else {
                                    //     street_paving_mode(robot, world, cycle * -1);
                                    // }
                                    //street_paving_mode(robot, world, cycle);
                                    return;
                                },
                                Ok(p) => info!("{:?}", p),
                                _ => {
                                    //street_paving_mode(robot, world, -1);
                                    return;
                                }
                            }
                        },
                        _ => {
                            for (key, value) in robot.get_backpack().get_contents().clone() {
                                if key == Content::Garbage(0) {
                                    if value == 0usize {
                                        recover_oob(robot, world, Direction::Down);
                                        //if go_allowed(robot, world, &Direction::Up).is_ok() {
                                            go(robot, world, Direction::Up);
                                        // } else {
                                        //     street_paving_mode(robot, world, cycle * -1);
                                        // }
                                        info!("{:?}",
                                                 bessie::bessie::road_paving_machine(
                                                     robot,
                                                     world,
                                                     Direction::Down,
                                                     bessie::bessie::State::MakeRoad
                                                 )
                                        );
                                        return;
                                    } else {
                                        call(robot, world, "garbage", 0);
                                        return;
                                    }
                                }
                            }
                        }
                    }
                },
                _ => {}
            };
        }
        call(robot, world, "rock", 0);
    }

   
    fn garbage(robot: &mut impl Runnable, world: &mut World) {
        info!("Oh no, That's garbage! Let's clean this place!");
        while robot.get_energy().has_enough_energy(10) {
            recharge(robot);
            match TomTom::go_to_tile(robot, world, false, None, Some(PlainContent::Garbage)) {
                Ok(path) => {
                    info!("Garbage was found!");
                    TomTom::go_to_tile(robot, world, false, None, Some(PlainContent::Garbage));
                    go(robot, world, Direction::Down);
                    destroy(robot, world, Direction::Up);
                    for (key, value) in robot.get_backpack().get_contents().clone() {
                        if key == Content::Garbage(0) {
                            if value == 10 {
                                info!("We found 10 pieces, let's go throw them out");
                                collect_garbage(robot, world);
                                return;
                            }
                        }
                    }
                },

                _ =>
                    {
                        random_movement(robot, world, 'g');
                    }
            }
        }
    }

    
    fn collect_garbage(robot: &mut impl Runnable, world: &mut World) {
        let task = Goal::new("Garbage Man".to_string(), "Dispose of the 10 garbage pieces".to_string(),
                             GoalType::ThrowGarbage, Some(Content::Garbage(0)), 10);
        let mut tracker = GoalTracker::new();
        tracker.add_goal(task);
        info!("{:?}", tracker.get_goals().clone());
        match TomTom::go_to_tile(robot, world, false, None, Some(PlainContent::Bin)) {
            Ok(path) => {
                info!("There's a bin!");
                TomTom::go_to_tile(robot, world, false, None, Some(PlainContent::Bin));
                go(robot, world, Direction::Down);
                match throw_garbage(robot, world, Content::Garbage(0), 10, Direction::Up, &mut tracker) {
                    Ok(..) => {
                        if tracker.get_completed_number() == 1 {
                            info!("{:?}", tracker.get_goals());
                            call(robot, world, "rock", 0);
                            //exit(0)
                        } else {
                            recharge(robot);
                            random_movement(robot, world, 'c');
                        }
                    },
                    Err(..) => {
                        call(robot, world, "garbage", 0);
                    }
                };
            },
            Err(e) => {
                random_movement(robot, world, 'c');
            }
        }
    }

    
    fn call(robot: &mut impl Runnable, world: &mut World, key: &str, streetcycle: i32) {
        recharge(robot);
        match key {
            "rock" => rock(robot, world),
            "street" => street_paving_mode(robot, world, streetcycle),
            "garbage" => garbage(robot, world),
            _ => info!("invalid function call"),
        }
    }
    fn recharge(robot: &mut impl Runnable) {
        *robot.get_energy_mut() = Dynamo::update_energy();
    }
    fn random_movement(robot: &mut impl Runnable, world: &mut World, key: char) {
        let num = rand::thread_rng().gen_range(0..=100);
        match num {
            0..=25 => {
                if go_allowed(robot, world, &Direction::Left).is_ok() {
                    go(robot, world, Direction::Left);
                    match key {
                        'c' => collect_garbage(robot, world),
                        'g' => garbage(robot, world),
                        'r' => rock(robot, world),
                        _ => {}
                    }
                // } else {
                //     random_movement(robot, world, key);
                }
            },
            26..=50 => {
                if go_allowed(robot, world, &Direction::Right).is_ok() {
                    go(robot, world, Direction::Right);
                    match key {
                        'c' => collect_garbage(robot, world),
                        'g' => garbage(robot, world),
                        'r' => rock(robot, world),
                        _ => {}
                    }
                // } else {
                //     random_movement(robot, world, key);
                }
            },
            51..=75 => {
                if go_allowed(robot, world, &Direction::Up).is_ok() {
                    go(robot, world, Direction::Up);
                    match key {
                        'c' => collect_garbage(robot, world),
                        'g' => garbage(robot, world),
                        'r' => rock(robot, world),
                        _ => {}
                    }
                // } else {
                //     random_movement(robot, world, key);
                }
            },
            76..=100 => {
                if go_allowed(robot, world, &Direction::Down).is_ok() {
                    go(robot, world, Direction::Down);
                    match key {
                        'c' => collect_garbage(robot, world),
                        'g' => garbage(robot, world),
                        'r' => rock(robot, world),
                        _ => {}
                    }
                // } else {
                //     random_movement(robot, world, key);
                }
            },
            _ => random_movement(robot, world, key),
        }
    }
    fn recover_oob(robot: &mut impl Runnable, world: &mut World, d: Direction) {
        match d {
            Direction::Up => {
                if robot.get_coordinate().get_row() == 0 {
                    info!("Watch out!");
                    go(robot, world, Direction::Down);
                    go(robot, world, Direction::Down);
                    recharge(robot);
                }
            },
            Direction::Down => {
                if robot.get_coordinate().get_row() == 279 {
                    info!("Watch out!");
                    go(robot, world, Direction::Up);
                    go(robot, world, Direction::Up);
                    recharge(robot);
                }
            },
            Direction::Right => {
                if robot.get_coordinate().get_col() == 279 {
                    info!("Watch out!");
                    go(robot, world, Direction::Left);
                    go(robot, world, Direction::Left);
                    recharge(robot);
                }
            },
            Direction::Left => {
                if robot.get_coordinate().get_col() == 0 {
                    info!("Watch out!");
                    go(robot, world, Direction::Right);
                    go(robot, world, Direction::Right);
                    recharge(robot);
                }
            }
        }
    }
    html! {
        <></>
    }
}




    async fn run_game(run: Rc<RefCell<Result<Runner, LibError>>>) -> () {
        sleep(3000).await;
        for _  in 0..10000 {
            sleep(1000).await;
            info!("[ RUNNER ] Tick");
            // Get a mutable reference to the Result<Runner>
            let mut runner_result = run.borrow_mut();
            // Handle the Result using map and map_err
            runner_result.as_mut().map(|runner| {
                // runner is now a mutable reference to the Runner
                let _ = runner.game_tick();
            }).map_err(|e| {
                info!("[ RUNNER ] ERROR WITH RUN: {:?}", e);
            }).unwrap_or_else(|_| {
                info!("[ RUNNER ] ERROR WITH RUN. ");
            });
        }
    }


    // Custom sleep function to support the web
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
