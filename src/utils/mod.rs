use std::any::Any;


use std::hash::{Hash, Hasher};


use bob_lib::tracker::{GoalTracker};

use robotics_lib::interface::{Direction};

use robotics_lib::utils::LibError;

use robotics_lib::world::tile::{Tile};
use robotics_lib::world::World;
use crate::explorer::{explorer_execute};
use crate::interface::Jerry;
use crate::road_builder::road_builder_execute;


//use crate::road_builder::{build_road, road_builder_execute};

pub const SECTOR_DIMENSION: usize = 70;
pub struct Mission {
    pub name: String,
    pub description: Option<String>,
    pub goal_tracker: Option<GoalTracker>,
    pub status: MissionStatus,
    pub probability_of_cheating: f64,
    pub additional_data: Option<Box<dyn Any>>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MissionStatus{
    New,
    Active,
    Paused,
    Completed,
}
#[derive(Debug)]
pub struct ActiveRegion{
    pub(crate) top_left: (usize, usize),
    pub(crate) bottom_right: (usize, usize),
    /*
        restricts the robot's activity to a certain region of the map
        for example if sector dimension is 70:
        index of the grid sector -> 70x70 square
        if map is less than 70x70, then the index is 0 always
        if the map is 140x140, then the
        index = 0 <-> (0, 0) to (69, 69)
        index = 1 <-> (0, 70) to (69, 139)
        index = 2 <-> (70, 0) to (139, 69)
        index = 3 <-> (70, 70) to (139, 139)
        
        if the grid is 80x80:
         then the index = 1 <-> (0, 70) to (69, 79)
         index = 2 <-> (70, 0) to (79, 69)
         index = 3 <-> (70, 70) to (79, 79)
     */
    pub(crate) spatial_index: usize,
    
}

#[derive(Debug)]
pub enum JerryStatus{
    MissionExecutionError,
    LowEnergyWarning,
    Common(LibError),
    ExpectingNiceWeather,
}
pub(crate) fn execute_mission (jerry: &mut Jerry, world: &mut World){


    //after every 2 completed explorers, need to build the roads
    //so if the # of completed explorers is even, set other active explorers to pause and continue with the road builders
    let mut completed_explorers = 0;
    let mut active_explorers = 0;
    let mut waiting_explorers = 0;
    let mut completed_road_builders = 0;
    let mut waiting_road_builders = 0;
    let mut active_road_builders = 0;
    for mission in jerry.missions.iter(){
        match mission.name.as_str(){
            | "Explore" => {
                match mission.status{
                    MissionStatus::Completed => completed_explorers += 1,
                    MissionStatus::Paused => waiting_explorers += 1,
                    MissionStatus::Active => active_explorers += 1,
                    _ => {}
                    
                }
            }
            | "Road Builder" => {
                match  mission.status{
                    MissionStatus::Completed => completed_road_builders += 1,
                    MissionStatus::Paused => waiting_road_builders += 1,
                    MissionStatus::Active => active_road_builders += 1,
                    _ => {}
                    
                }
            }
            | _ => {}
        }
    }
    println!("Act Exp: {} Wait Exp: {} Comp Exp: {} Act RB: {} Wait RB: {} Comp RB {}", 
    active_explorers, waiting_explorers, completed_explorers, active_road_builders, waiting_road_builders, completed_road_builders);
    //after every 2 completed explorers, set road builders to active and pause the active explorers
    if completed_explorers > 0 && completed_explorers % 2 == 0 && waiting_road_builders > 0{
        for mission in jerry.missions.iter_mut(){
            if mission.name == "Explore" && mission.status == MissionStatus::Active{
                mission.status = MissionStatus::Paused;
            }
            if mission.name == "Road Builder" && mission.status == MissionStatus::Paused{
                mission.status = MissionStatus::Active;
            }
        }
    }
    //after every 2 completed builders set explorers to active and pause the active builders
    if waiting_explorers > 0 && completed_road_builders > 0 && (completed_road_builders % 2 == 0 || active_road_builders == 0){
        for mission in jerry.missions.iter_mut(){
            if mission.name == "Explore" && mission.status == MissionStatus::Paused{
                mission.status = MissionStatus::Active;
            }
            if mission.name == "Road Builder" && mission.status == MissionStatus::Active{
                mission.status = MissionStatus::Paused;
            }
        }
    }
    let mission = jerry.missions.iter_mut().enumerate().find(|(_i, mission)| mission.status == MissionStatus::Active);
    if mission.is_none() {
        println!("I got nothing to do!");
    }
    if let Some((index, mission)) = mission{
        match mission.name.as_str(){
            | "Explore" => {
                println!("Mission Explorer {:?}", explorer_execute(jerry, world, index));
            }
            | "Road Builder" => {
                if mission.status == MissionStatus::New{
                    mission.status = MissionStatus::Active;
                }
                jerry.active_region.top_left = (jerry.world_dim - 1, jerry.world_dim - 1);
                jerry.active_region.bottom_right = (0, 0);
                println!("Mission Road Builder {:?}", road_builder_execute(jerry, world, index));
            }

            | _ => {}
        }
    }
}
pub fn get_world_dimension(world: &mut World) -> usize{
    println!("{:?}", world.get_discoverable());
    ((world.get_discoverable() as f64 / 3.0 - 1.0) * 10.0).sqrt() as usize
}
pub fn calculate_spatial_index(row: usize, col: usize, size: usize) -> usize {
    let num_cols_per_section = SECTOR_DIMENSION;
    let num_rows_per_section = SECTOR_DIMENSION;
    let num_sections_cols = (size as f64 / num_cols_per_section as f64).ceil() as usize;
    let section_index_col = col / num_cols_per_section;
    let section_index_row = row / num_rows_per_section;
    let section_index = section_index_row * num_sections_cols + section_index_col;
    section_index
}
pub fn get_tl_and_br_from_spatial_index(spatial_index: usize, size: usize) -> ((usize, usize), (usize, usize)){

    if size < SECTOR_DIMENSION {
        return ((0, 0), (size - 1, size - 1));
    }
    let num_cols_per_section = SECTOR_DIMENSION;
    let num_rows_per_section = SECTOR_DIMENSION;
    let num_sections_cols = (size as f64 / num_cols_per_section as f64).ceil() as usize;
    let section_index_col = spatial_index % num_sections_cols;
    let section_index_row = spatial_index / num_sections_cols;

    //first calculate the default values
    let top_left = (section_index_row * num_rows_per_section, section_index_col * num_cols_per_section);
    let mut bottom_right = (top_left.0 + num_rows_per_section - 1, top_left.1 + num_cols_per_section - 1);
    println!("{:?}", (top_left, bottom_right));
    if size % SECTOR_DIMENSION == 0 {
        return (top_left, bottom_right);
    }
    //if it's the last column, no problems with the top left, but need to adjust the bottom right
    if section_index_col == num_sections_cols - 1 {
        bottom_right = (bottom_right.0, size - 1);
    }
    //the same for the last row, but need to adjust in a different way
    if section_index_row == num_sections_cols - 1 {
        bottom_right = (size - 1, bottom_right.1);
    }
    (top_left, bottom_right)
}
//returns a slice of the robot map with the top left and bottom right corners defined by the coordinates
pub fn robot_map_slice
    (robot_map: &Vec<Vec<Option<Tile>>>,top_left: (usize, usize), bottom_right: (usize, usize))
     -> Option<Vec<Vec<Option<Tile>>>>{
    let mut map = Vec::new();
    for i in top_left.0..=bottom_right.0{
        let mut row = Vec::new();
        for j in top_left.1..=bottom_right.1{
            row.push(robot_map[i][j].clone());
        }
        map.push(row);
    }
    Some(map)
}
pub fn get_direction(from: (usize, usize), to: (usize, usize)) -> Option<Direction>{
    let (from_i, from_j) = from;
    let (to_i, to_j) = to;
    if from_i == to_i{
        if from_j < to_j{
            return Some(Direction::Right);
        }
        else{
            return Some(Direction::Left);
        }
    }
    if from_j == to_j{
        if from_i < to_i{
            return Some(Direction::Down);
        }
        else{
            return Some(Direction::Up);
        }
    }
    None
}