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
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{window, Element, HtmlInputElement};
use yew::prelude::*;
use yew::{function_component, html, Html, Properties};

// enums to allow updates inside the impl
#[derive(Clone, PartialEq, Atom)]
pub(crate) struct BackpackState {
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
pub(crate) struct WorldState {
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
pub(crate) struct EnviromentalState {
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
pub(crate) struct RobotState {
    coord: (usize, usize),
    // energy: usize,
}

impl Default for RobotState {
    fn default() -> Self {
        Self { coord: (0, 0) }
    }
}

#[derive(Clone, PartialEq, Atom)]
pub(crate) struct EnergyState {
    energy: usize,
    // energy: usize,
}

impl Default for EnergyState {
    fn default() -> Self {
        Self { energy: 0 }
    }
}

#[derive(Clone, PartialEq, Atom)]
pub(crate) struct ExtrasState {
    pub(crate) score: f32,
}

impl Default for ExtrasState {
    fn default() -> Self {
        Self { score: 0.0 }
    }
}

#[derive(Clone, PartialEq, Atom)]
pub(crate) struct StartingSettings {
    start_ai: bool,
    tile_size: f32,
    follow_robot: bool,
    tick_time: u32,
}

impl Default for StartingSettings {
    fn default() -> Self {
        Self {
            start_ai: false,
            tile_size: 40.0,
            follow_robot: true,
            tick_time: 0,
        }
    }
}

#[function_component(Main)]
pub fn main() -> Html {
    let settings = use_atom::<StartingSettings>();
    info!("Rendered Main");

    html! {
        html! {

            { if &settings.start_ai == &true {
                // Normal robot display
                html! {
                    <div id="info">
                        // <BackP/>
                        <EnergyBar/>
                        <EnviromentBar />
                        // <Zoom />
                        <br/>
                        <MapView/>
                        <Menu />
                        // <ScoreDisplay />
                        <TimoAi />
                    </div>
                }
            } else {

                let on_tick_time_input = {
                    let settings = settings.clone();

                    Callback::from(move |e: InputEvent| {
                        let input: HtmlInputElement = e.target_unchecked_into();

                        settings.set(StartingSettings { follow_robot: settings.follow_robot.clone(), start_ai: settings.start_ai.clone(), tile_size: settings.tile_size.clone(), tick_time: input.value().parse::<u32>().expect("Expected u32 as tick time") });
                    })
                };

                // Start the game on button click
                let start_game = {
                    let settings = settings.clone();

                    Callback::from(move |_| {
                        settings.set(StartingSettings { follow_robot: settings.follow_robot.clone(), start_ai: true, tile_size: settings.tile_size.clone(), tick_time: settings.tick_time.clone() });
                    })
                };


                html! {
                    <div id="start">
                        <label for={"ticktime"}>{"Tick Delay (ms)"}</label>
                        <input id={"ticktime"} type={"text"} oninput={on_tick_time_input} value={settings.tick_time.to_string()}/>
                        <button onclick={start_game} >{"Start Game"}</button>
                    </div>
                }
            }

            }



        }
    }
}

#[function_component(Menu)]
fn menu() -> Html {
    let settings = use_atom::<StartingSettings>();

    // Input Callbacks
    let onchange_slider = {
        let settings = settings.clone();

        Callback::from(move |e: yew::prelude::Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let value = input.value().parse::<f32>().unwrap_or(30.0);
            log::info!("value = {}", &value);
            settings.set(StartingSettings {
                start_ai: settings.start_ai.clone(),
                follow_robot: settings.follow_robot.clone(),
                tile_size: value,
                tick_time: settings.tick_time.clone(),
            });
        })
    };

    let follow_bot_fn = {
        let settings = settings.clone();

        Callback::from(move |_| {
            settings.set(StartingSettings {
                follow_robot: !settings.follow_robot.clone(),
                start_ai: true,
                tile_size: settings.tile_size.clone(),
                tick_time: settings.tick_time.clone(),
            });
        })
    };

    html! {
        <div id="menu">
            <label for={"tilesize"}>{"Tile Size"}</label>
            <input type="range" class="form-range" min="1" max="100" id="tilesize" onchange={onchange_slider.clone()} />
            <button type={"checkbox"} onclick={follow_bot_fn}>{"Toggle Freelook"}</button>
        </div>
    }
}

#[function_component(BackP)]
pub fn backpack() -> Html {
    let back_state = use_atom::<BackpackState>();
    html! {
        <div id={"backpack"}>
            <h2>{"Backpack"}</h2>
            <hr/>
            {"Size: "}{ &back_state.size}
            <br/>
            {"Contents: "}
            { for back_state.content.iter().map(|content| {
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
fn back_item(props: &BackItemProps) -> Html {
    let img_display: &str = content_match_day(&props.content);
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
    let forecast_image = match_forecast(&enviroment_state.forecast, &enviroment_state.time);

    html! {
        <div id="enviroment">
        <h3>{format!("{}", &enviroment_state.time)}</h3>
            <img src={forecast_image} />

        </div>
    }
}

fn match_forecast(conditions: &WeatherType, time: &str) -> &'static str {
    const SUNNY_ICON: &'static str =
        "https://www.pngall.com/wp-content/uploads/2016/07/Sun-PNG-Image-180x180.png";
    const MOON_ICON: &'static str = "img/moon-min.png";
    const RAINY_ICON: &'static str = "https://borregowildflowers.org/img_sys/rain.png";
    const FOGGY_ICON: &'static str = "https://cdn-icons-png.flaticon.com/128/2076/2076827.png";
    const TROPICAL_MONSOON_ICON: &'static str = "https://heat-project.weebly.com/uploads/7/1/4/2/71428073/published/bez-nazxdccwy-1_1.png?1533897845";
    const TRENTINO_SNOW_ICON: &'static str =
        "https://cdn.icon-icons.com/icons2/33/PNG/128/snow_cloud_weather_2787.png";

    let hour: u8 = if time.len() >= 2 {
        time[0..2].parse::<u8>().unwrap_or_default()
    } else {
        12
    };

    match conditions {
        WeatherType::Sunny => match hour {
            19..=23 | 00..=05 => MOON_ICON,
            _ => SUNNY_ICON,
        },
        WeatherType::Rainy => RAINY_ICON,
        WeatherType::Foggy => FOGGY_ICON,
        WeatherType::TropicalMonsoon => TROPICAL_MONSOON_ICON,
        WeatherType::TrentinoSnow => TRENTINO_SNOW_ICON,
    }
}

#[function_component(MapView)]
pub fn map_view() -> Html {
    let world_state = use_atom::<WorldState>();
    let robot_state = use_atom::<RobotState>();
    let settings = use_atom::<StartingSettings>();
    let cond_state = use_atom::<EnviromentalState>();
    
    let world_styles: String;

    let hour: u8 = if cond_state.time.len() >= 2 {
        cond_state.time[0..2].parse::<u8>().unwrap_or_default()
    } else {
        12
    };
    match hour {
        19..=23 | 00..=05 => {
            world_styles = format!(
                "width: {}px; height: {}px; background-color: black;",
                settings.tile_size.clone().to_string(),
                settings.tile_size.clone().to_string()
            );
        }
        _ => {
            world_styles = format!(
                "width: {}px; height: {}px; background-color: var(--background-color);",
                settings.tile_size.clone().to_string(),
                settings.tile_size.clone().to_string()
            );
        }
    }
    
    

    // Use effect for following the robot
    use_effect_with((settings.follow_robot, robot_state.coord), move |_| {
        if settings.follow_robot {
            if let Some(robot_element) = window().and_then(|win| win.document()).and_then(|doc| doc.get_element_by_id("robot")) {
                let robot_html_element = robot_element.dyn_into::<Element>().expect("Failed to cast to HtmlElement");
                robot_html_element.scroll_into_view();
            }
        }
        || ()
    });

    html! {
        <div id={"robot_view"}>
            {for world_state.world.iter().enumerate().map(|(i, row)| {
                html! {
                    <div class={classes!("map_row")}>
                        {for row.iter().enumerate().map(|(j, tile_option)| {
                            html! {
                                <div class={classes!("tile")} style={world_styles.to_owned()}>
                                    {if let Some(tile) = tile_option {
                                        html! {<MapTile tile={tile.to_owned()} />}
                                    } else {
                                        html! {}
                                    }}
                                    {if i == robot_state.coord.0 && j == robot_state.coord.1 {
                                        html! {<img id={"robot"} src={"https://icons.iconarchive.com/icons/google/noto-emoji-smileys/1024/10103-robot-face-icon.png"} />}
                                    } else {
                                        html! {}
                                    }}
                                </div>
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
    let settings = use_atom::<StartingSettings>();
    // let extra_state = use_atom::<ExtrasState>();
    let world_style: &str = &format!(
        "width: {}px; height: {}px;",
        settings.tile_size.clone().to_string(),
        settings.tile_size.clone().to_string()
    )[..];
    let hour: u8 = cond_state.time[0..2]
        .to_owned()
        .parse::<u8>()
        .expect("Expected hour");
    let tile_style: String;
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
            TileType::Wall => {
                tile_style = format!("{} background-color: rgb(125, 125, 125);", world_style)
            }
            DeepWater => tile_style = format!("{} background-color: #2B00FF;", world_style),
            ShallowWater => tile_style = format!("{} background-color: #00B3FF;", world_style),
            Sand => tile_style = format!("{} background-color: #FFC400;", world_style),
            Grass => tile_style = format!("{} background-color: #23B606;", world_style),
            Street => tile_style = format!("{} background-color: #000000;", world_style),
            Hill => tile_style = format!("{} background-color: #FFBD4A;", world_style),
            Mountain => tile_style = format!("{} background-color: #8C8CF9;", world_style),
            Snow => tile_style = format!("{} background-color: #F5F5F5;", world_style),
            Lava => tile_style = format!("{} background-color: #F2DA3E;", world_style),
            Teleport(_) => tile_style = format!("{} background-color: #BC1FEC;", world_style),
        },
        false => match props.tile.tile_type {
            TileType::Wall => {
                tile_style = format!("{} background-color: rgb(125, 125, 125);", world_style)
            }
            DeepWater => tile_style = format!("{} background-color: #030C58;", world_style),
            ShallowWater => tile_style = format!("{} background-color: #074A84;", world_style),
            Sand => tile_style = format!("{} background-color: #A5931B;", world_style),
            Grass => tile_style = format!("{} background-color: #0E5411;", world_style),
            Street => tile_style = format!("{} background-color: #000000;", world_style),
            Hill => tile_style = format!("{} background-color: #573708;", world_style),
            Mountain => tile_style = format!("{} background-color: #20314A;", world_style),
            Snow => tile_style = format!("{} background-color: #C9C9C9;", world_style),
            Lava => tile_style = format!("{} background-color: #F2DA3E;", world_style),
            Teleport(_) => tile_style = format!("{} background-color: #56038D;", world_style),
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
    let img_display: &str;

    let hour: u8 = cond_state.time[0..2]
        .to_owned()
        .parse::<u8>()
        .expect("Expected a number for time");
    img_display = content_match_day(&props.tile.content);

    if img_display == "" {
        html! {<></>}
    } else {
        match hour {
            19..=23 | 00..=05 => {
                html! {
                    <img style={"filter: brightness(50%);"} class={classes!("tile_content")} src={img_display}/>
                }
            }
            _ => {
                html! {
                    <img  class={classes!("tile_content")} src={img_display}/>
                }
            }
        }
    }
}

fn content_match_day(input: &Content) -> &'static str {
    // Define constant image URLs
    const ROCK_IMAGE: &str = "img/rock-min.png";
    const TREE_IMAGE: &str =
        "https://minecraft.wiki/images/thumb/Azalea_Tree.png/250px-Azalea_Tree.png?945ad";
    const GARBAGE_IMAGE: &str = "img/garbage-min.png";
    const FIRE_IMAGE: &str = "img/fire.webp";
    const COIN_IMAGE: &str = "img/coin-min.png";
    const BIN_IMAGE: &str = "img/bin-min.png";
    const CRATE_IMAGE: &str = "https://gamepedia.cursecdn.com/minecraft_gamepedia/b/b3/Chest.png?version=227b3f51ef706a4ce4cf5e91f0e4face";
    const BANK_IMAGE: &str = "https://vignette.wikia.nocookie.net/pixelpeople/images/a/ae/Bank.png/revision/latest?cb=20130904201633";
    const WATER_IMAGE: &str = "img/water-min.png";
    const MARKET_IMAGE: &str = "img/market-min.png";
    const FISH_IMAGE: &str =
        "https://gamepedia.cursecdn.com/minecraft_gamepedia/a/ad/Tropical_Fish_JE2_BE2.png";
    const BUILDING_IMAGE: &str =
        "https://gamepedia.cursecdn.com/minecraft_gamepedia/f/f5/Plains_Cartographer_1.png";
    const BUSH_IMAGE: &str = "img/bush.webp";
    const JOLLY_BLOCK_IMAGE: &str = "img/jolly-min.png";
    const SCARECROW_IMAGE: &str = "img/scarecrow-min.png";
    const EMPTY_IMAGE: &str = "";

    match input {
        Content::Rock(_) => ROCK_IMAGE,
        Content::Tree(_) => TREE_IMAGE,
        Content::Garbage(_) => GARBAGE_IMAGE,
        Content::Fire => FIRE_IMAGE,
        Content::Coin(_) => COIN_IMAGE,
        Content::Bin(_) => BIN_IMAGE,
        Content::Crate(_) => CRATE_IMAGE,
        Content::Bank(_) => BANK_IMAGE,
        Content::Water(_) => WATER_IMAGE,
        Content::Market(_) => MARKET_IMAGE,
        Content::Fish(_) => FISH_IMAGE,
        Content::Building => BUILDING_IMAGE,
        Content::Bush(_) => BUSH_IMAGE,
        Content::JollyBlock(_) => JOLLY_BLOCK_IMAGE,
        Content::Scarecrow => SCARECROW_IMAGE,
        Content::None => EMPTY_IMAGE,
    }
}

#[function_component(ScoreDisplay)]
fn score_display() -> Html {
    let extras = use_atom::<ExtrasState>();

    html! {
        <div id="score">
            <h2>{"Score"}</h2>
            <p>{&extras.score}</p>
        </div>
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
    pub(crate) extras: UseAtomHandle<ExtrasState>,
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
    let back_state = use_atom::<BackpackState>();
    let world_state = use_atom::<WorldState>();
    let robot_state = use_atom::<RobotState>();
    let env_state = use_atom::<EnviromentalState>();
    let energy_state = use_atom::<EnergyState>();
    let extra_state = use_atom::<ExtrasState>();
    let settings = use_atom::<StartingSettings>();

    info!("Ai Running");
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
                let tmp_map = robot_map(&world).unwrap_or_default();
                let tmp_conditions = look_at_sky(&world);
                info!("{:?} Internal Map", tmp_map);
                if tmp_map != self.ws.world {
                    self.ws.set(WorldState {
                        world: tmp_map,
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
                // let tmp_score = get_score(&world);
                // info!("{}", format!("SCORE {}", tmp_score.to_string()));
                // if tmp_score != self.extras.score {
                //     self.extras.set(ExtrasState {score: tmp_score});
                // }
                // info!("CHANGED CONDITIONS");
            }

            fn handle_event(&mut self, event: Event) {
                println!();
                println!("{:?}", event);
                // Logs the event to the console
                // info!("[ EVENT ]{}", format!("{:?}", event));
                // Backpack Updates
                match event {
                    Event::AddedToBackpack(_, _) | Event::RemovedFromBackpack(_, _) => {
                        let new_back = self.get_backpack();
                        let new_back_content = new_back.get_contents();
                        let new_inside: HashMap<Content, usize> = (new_back_content.iter())
                            .map(|content| (content.0.to_owned(), content.1.to_owned()))
                            .collect();
                        // HERE Implement the code to update a state inside the ai function component with the value of backpack size and content
                        info!("[ State Update ] New Backpack State {:?}", new_back_content);
                        // if self.bps.content != new_inside {
                        //     self.bps.set(BackpackState {
                        //         size: new_back.get_size(),
                        //         content: new_inside,
                        //     });
                        //     info!("[ State Update ] New Backpack State");
                        // }
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
                        });
                    }
                    Event::EnergyRecharged(_) | Event::EnergyConsumed(_) => {
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
            bps: back_state.clone(),
            ws: world_state.clone(),
            rs: robot_state.clone(),
            env: env_state.clone(),
            en: energy_state.clone(),
            extras: extra_state.clone(),
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
                let _done = run_game(run, settings.clone()).await;
            });
        }
    }
    html! {
        <></>
    }
}

async fn run_game(
    run: Rc<RefCell<Result<Runner, LibError>>>,
    settings: UseAtomHandle<StartingSettings>,
) -> () {
    let tick_time = settings.tick_time.clone();
    // let mut counter = 0;
    sleep(1000).await;
    for _ in 0..100000 {
        info!("[ RUNNER ] Tick {:?}", tick_time);
        // Get a mutable reference to the Result<Runner>
        let mut runner_result = run.borrow_mut();
        // Handle the Result using map and map_err
        runner_result
            .as_mut()
            .map(|runner| {
                let _ = runner.game_tick();
            })
            .map_err(|e| {
                info!("[ RUNNER ] ERROR WITH RUN: {:?}", e);
            })
            .unwrap_or_else(|_| {
                info!("[ RUNNER ] ERROR WITH RUN. ");
            });
        sleep(tick_time).await;

        // counter = counter +1;
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
