use charting_tools::charted_coordinate::ChartedCoordinate;
use charting_tools::ChartingTools;
use ohcrab_weather::weather_tool::WeatherPredictionTool;
// Project imports
use robotics_lib::energy::Energy;
use robotics_lib::event::events::Event;
use robotics_lib::interface::{look_at_sky, robot_map};

use robotics_lib::runner::backpack::BackPack;
use robotics_lib::runner::{Robot, Runnable, Runner};
use robotics_lib::utils::LibError;
use robotics_lib::world::coordinates::Coordinate;
use robotics_lib::world::environmental_conditions::{EnvironmentalConditions, WeatherType};

use crate::explorer::new_explorer;
use crate::utils::{
    calculate_spatial_index, execute_mission, get_world_dimension, ActiveRegion, Mission,
};
use robotics_lib::world::tile::Content::{
    Bank, Bin, Building, Bush, Coin, Crate, Fire, Fish, Garbage, JollyBlock, Market, Rock,
    Scarecrow, Tree, Water,
};
use robotics_lib::world::tile::TileType::{
    DeepWater, Grass, Hill, Lava, Mountain, Sand, ShallowWater, Snow, Street, Teleport,
};
use robotics_lib::world::tile::{Content, Tile, TileType};
use robotics_lib::world::World;
use rust_and_furious_dynamo::dynamo::Dynamo;
use rust_eze_tomtom::TomTom;
use std::collections::{HashMap, HashSet, VecDeque};
use vent_tool_ascii_crab::Vent;

// Frontend
include!("worldloader.rs");
use bounce::*;
use log::info;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::window;
use yew::prelude::*;
use yew::{function_component, html, Html, Properties};

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
                // <BackP/>
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
    let img_display: String = content_match_day(&props.content);
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
    let cond_state = use_atom::<EnviromentalState>();
    let hour: u8 = cond_state.time[0..2]
        .to_owned()
        .parse::<u8>()
        .expect("Bought the flight to Cali racks & condoms in my suitcase");
    let tile_style: &str;
    let daytime: bool;
    match hour {
        19..=23 | 00..=05 => {
            daytime = false;
        }
        _ => {
            daytime = true;
        }
    }
    match daytime {
        true => match props.tile.tile_type {
            TileType::Wall => tile_style = "background-color: rgb(125, 125, 125);",
            DeepWater => tile_style = "background-color: #2B00FF;",
            ShallowWater => tile_style = "background-color: #00B3FF;",
            Sand => tile_style = "background-color: #FFC400;",
            Grass => tile_style = "background-color: #23B606;",
            Street => tile_style = "background-color: #000000;",
            Hill => tile_style = "background-color: #FFBD4A;",
            Mountain => tile_style = "background-color: #8C8CF9;",
            Snow => tile_style = "background-color: #F5F5F5;",
            Lava => tile_style = "background-color: #F2DA3E;",
            Teleport(_) => tile_style = "background-color: #BC1FEC;",
        },
        false => match props.tile.tile_type {
            TileType::Wall => tile_style = "background-color: rgb(125, 125, 125);",
            DeepWater => tile_style = "background-color: #030C58;",
            ShallowWater => tile_style = "background-color: #074A84;",
            Sand => tile_style = "background-color: #A5931B;",
            Grass => tile_style = "background-color: #0E5411;",
            Street => tile_style = "background-color: #000000;",
            Hill => tile_style = "background-color: #573708;",
            Mountain => tile_style = "background-color: #20314A;",
            Snow => tile_style = "background-color: #C9C9C9;",
            Lava => tile_style = "background-color: #F2DA3E;",
            Teleport(_) => tile_style = "background-color: #56038D;",
        },
    }

    html! {
        <div class={classes!("tile")}>
            <div class={classes!("tile_type")} style={tile_style}/>
            <MapTileContent tile={props.tile.clone()}/>
        </div>

    }
}

#[function_component(MapTileContent)]
pub fn map_tile_content(props: &MapTileProps) -> Html {
    let cond_state = use_atom::<EnviromentalState>();
    let img_display: String;
    let hour: u8 = cond_state.time[0..2]
        .to_owned()
        .parse::<u8>()
        .expect("Bought the flight to Cali racks & condoms in my suitcase");
    match hour {
        19..=23 | 00..=05 => {
            img_display = content_match_day(&props.tile.content);
        }
        _ => {
            img_display = content_match_night(&props.tile.content);
        }
    }

    if img_display == "" {
        html! {<></>}
    } else {
        html! {
            <img  class={classes!("tile_content")} src={img_display}/>
        }
    }
}

fn content_match_day(input: &Content) -> String {
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

fn content_match_night(input: &Content) -> String {
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
pub(crate) struct Jerry {
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
                if self.tick_counter == 0 {
                    first_tick(self, world);
                }
                execute_mission(self, world);
                println!("{:?} {}", self.robot.energy, self.tick_counter);
                self.tick_counter += 1;

                // Update UI State
                let tmpMap = robot_map(&world).unwrap_or_default();
                let tmp_conditions = look_at_sky(&world);
                info!("{:?} Internal Map", tmpMap);
                if tmpMap != self.ws.world {
                    self.ws.set(WorldState {
                        world: tmpMap,
                        counter: self.ws.counter.clone() + 1,
                    });
                    // info!("CHANGED WORLD");
                }
                let tmp_time = tmp_conditions.get_time_of_day_string();
                if self.env.time != tmp_time {
                    self.env.set(EnviromentalState {
                        forecast: tmp_conditions.get_weather_condition(),
                        time: tmp_time,
                    });
                }
                // info!("CHANGED CONDITIONS");
            }

            fn handle_event(&mut self, event: Event) {
                println!();
                println!("{:?}", event);
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
                            self.active_region.bottom_right.0 = if position.0 == self.world_dim - 1
                            {
                                self.world_dim - 1
                            } else {
                                position.0 + 1
                            };
                        }
                        if position.1 >= self.active_region.bottom_right.1 {
                            self.active_region.bottom_right.1 = if position.1 == self.world_dim - 1
                            {
                                self.world_dim - 1
                            } else {
                                position.1 + 1
                            };
                        }
                        if position.0 <= self.active_region.top_left.0 {
                            self.active_region.top_left.0 =
                                if position.0 == 0 { 0 } else { position.0 - 1 };
                        }
                        if position.1 <= self.active_region.top_left.1 {
                            self.active_region.top_left.1 =
                                if position.1 == 0 { 0 } else { position.1 - 1 };
                        }
                        let tmp_coords = self.get_coordinate();
                        // info!("[ State Update ] NEW COORDS: {:?}", tmp_coords);
                        self.rs.set(RobotState {
                            coord: (tmp_coords.get_row(), tmp_coords.get_col()),
                            // energy: self.rs.energy.clone()
                        });
                    }
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
                            self.en.set(EnergyState { energy: tmp_energy })
                        }
                    }
                    _ => (),
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
        fn first_tick(jerry: &mut Jerry, world: &mut World) {
            let size = get_world_dimension(world);
            jerry.world_dim = size;
            jerry.active_region.spatial_index = calculate_spatial_index(
                jerry.get_coordinate().get_row(),
                jerry.get_coordinate().get_col(),
                size,
            );
            let explorer = new_explorer(jerry, world, jerry.active_region.spatial_index);
            jerry.missions.push_back(explorer);
        }
        // RUNNING THE GAME
        let r = Jerry {
            robot: Robot::new(),
            bps: backState.clone(),
            ws: world_state.clone(),
            rs: robotState.clone(),
            env: env_state.clone(),
            en: energy_state.clone(),
            tick_counter: 0,
            world_dim: 0,
            active_region: ActiveRegion {
                top_left: (279, 279),
                bottom_right: (0, 0),
                spatial_index: 0,
            },
            vent: Rc::new(RefCell::new(Vent::new())),
            road_tiles: HashSet::new(),
            dynamo: Dynamo {},
            weather_predictor: WeatherPredictionTool::new(),
            tom_tom: TomTom {},
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

//NICO CODE
pub(crate) struct MyRobot {
        pub(crate) robot: Robot,
        pub(crate) bps: UseAtomHandle<BackpackState>,
        pub(crate) ws: UseAtomHandle<WorldState>,
        pub(crate) rs: UseAtomHandle<RobotState>,
        pub(crate) env: UseAtomHandle<EnviromentalState>,
        pub(crate) en: UseAtomHandle<EnergyState>
    }


async fn run_game(run: Rc<RefCell<Result<Runner, LibError>>>) -> () {
    sleep(3000).await;
    for _ in 0..10000 {
        sleep(1).await;
        info!("[ RUNNER ] Tick");
        // Get a mutable reference to the Result<Runner>
        let mut runner_result = run.borrow_mut();
        // Handle the Result using map and map_err
        runner_result
            .as_mut()
            .map(|runner| {
                // runner is now a mutable reference to the Runner
                let _ = runner.game_tick();
                let _robot_coordinate = runner.get_robot().get_coordinate();
            })
            .map_err(|e| {
                info!("[ RUNNER ] ERROR WITH RUN: {:?}", e);
            })
            .unwrap_or_else(|_| {
                info!("[ RUNNER ] ERROR WITH RUN. ");
            });
    }
}

// Custom sleep function to support the web
async fn sleep(duration: u32) {
    let promise = js_sys::Promise::new(&mut |resolve, _| {
        window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, duration as i32)
            .unwrap();
    });


    let _ = JsFuture::from(promise).await;
}

