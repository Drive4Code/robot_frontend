use std::any::Any;
use std::cell::RefCell;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::ops::Add;
use std::rc::Rc;
use bob_lib::tracker::{Goal, GoalTracker};
use charting_tools::charted_coordinate::ChartedCoordinate;
use robotics_lib::interface::{robot_map, Direction};
use robotics_lib::runner::Runnable;
use robotics_lib::utils::LibError;
use robotics_lib::world::coordinates::Coordinate;
use robotics_lib::world::tile::{Tile, Content, TileType};
use robotics_lib::world::World;
use crate::explorer::{explorer_execute, ExplorerData};
use crate::sector_analyzer::{analyzer_execute, SectorData};

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
    ExpectingNiceWeather
}
pub fn execute_mission (jerry: &mut Jerry, world: &mut World){
    let mission = jerry.missions.iter_mut().enumerate().find(|(i, mission)| mission.status != MissionStatus::Completed);
    if mission.is_none() {
        println!("I got nothing to do!");
    }
    if let Some((index, mission)) = mission{
        match mission.name.as_str(){
            | "Explore" => {
                println!("Mission Explorer {:?}", explorer_execute(jerry, world, index));
            }
            | "Sector Analyzer" => {
                mission.status = MissionStatus::Completed;
                let data = mission.additional_data.as_ref().unwrap().downcast_ref::<ActiveRegion>().unwrap();
                let (tl, br) = (data.top_left, data.bottom_right);
                let sector_data = analyzer_execute(world, tl, br);
                println!("{:?}", sector_data);
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
    let mut top_left = (section_index_row * num_rows_per_section, section_index_col * num_cols_per_section);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_tl_and_br_from_spatial_index() { //with sector dimension = 70
        // Test case 1: spatial_index = 1, size = 140
        let spatial_index_1 = 1;
        let size_1 = 140;
        let expected_result_1 = ((0, 70), (69, 139));
        assert_eq!(get_tl_and_br_from_spatial_index(spatial_index_1, size_1), expected_result_1);

        // Test case 2: spatial_index = 1, size = 100
        let spatial_index_2 = 1;
        let size_2 = 100;
        let expected_result_2 = ((0, 70), (69, 99));
        assert_eq!(get_tl_and_br_from_spatial_index(spatial_index_2, size_2), expected_result_2);

        // Test case 3: spatial_index = 0, size = 50
        let spatial_index_3 = 0;
        let size_3 = 50;
        let expected_result_3 = ((0, 0), (49, 49));
        assert_eq!(get_tl_and_br_from_spatial_index(spatial_index_3, size_3), expected_result_3);

        // Test case 4: spatial_index = 8, size = 150
        let spatial_index_4 = 8;
        let size_4 = 150;
        let expected_result_4 = ((140, 140), (149, 149));
        assert_eq!(get_tl_and_br_from_spatial_index(spatial_index_4, size_4), expected_result_4);

        // Test case 5: spatial_index = 2, size = 100
        let spatial_index_5 = 2;
        let size_5 = 100;
        let expected_result_5 = ((70, 0), (99, 69));
        assert_eq!(get_tl_and_br_from_spatial_index(spatial_index_5, size_5), expected_result_5);

        // Test case 6: spatial_index = 7, size = 280
        let spatial_index_6 = 7;
        let size_6 = 280;
        let expected_result_6 = ((70, 210), (139, 279));
        assert_eq!(get_tl_and_br_from_spatial_index(spatial_index_6, size_6), expected_result_6);

    }
    #[test]
    fn test_get_direction(){
        // Test case 1: from = (0, 0), to = (0, 1)
        let from_1 = (0, 0);
        let to_1 = (0, 1);
        let expected_result_1 = Some(Direction::Right);
        assert_eq!(get_direction(from_1, to_1), expected_result_1);

        // Test case 2: from = (0, 1), to = (0, 0)
        let from_2 = (0, 1);
        let to_2 = (0, 0);
        let expected_result_2 = Some(Direction::Left);
        assert_eq!(get_direction(from_2, to_2), expected_result_2);

        // Test case 3: from = (0, 0), to = (1, 0)
        let from_3 = (0, 0);
        let to_3 = (1, 0);
        let expected_result_3 = Some(Direction::Down);
        assert_eq!(get_direction(from_3, to_3), expected_result_3);

        // Test case 4: from = (1, 0), to = (0, 0)
        let from_4 = (1, 0);
        let to_4 = (0, 0);
        let expected_result_4 = Some(Direction::Up);
        assert_eq!(get_direction(from_4, to_4), expected_result_4);
    }
}
