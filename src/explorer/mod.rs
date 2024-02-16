use std::collections::{HashSet};




use std::rc::Rc;
use std::cell::RefCell;

use charting_tools::charted_coordinate::ChartedCoordinate;
use charting_tools::charted_paths::ChartedPaths;
use charting_tools::ChartingTools;
use robotics_lib::interface::{go, look_at_sky, robot_map, where_am_i, Direction};
use robotics_lib::runner::{Runnable};

use robotics_lib::world::tile::Tile;
use robotics_lib::world::{World};

use rust_eze_tomtom::TomTom;
use crate::biomes::{detect_biome, is_weather_gonna_be_nice_n, is_weather_nice};
use crate::explorer::ExplorerError::{FailedToGo, FrontierNotAccessible};
use crate::interface::Jerry;
use crate::road_builder::generate_road_builders;
use crate::sector_analyzer::{analyzer_execute, new_sector_analyzer};
use crate::utils::{calculate_spatial_index, robot_map_slice, ActiveRegion, JerryStatus, Mission};

use crate::utils::MissionStatus::{Active, Completed};
use rust_and_furious_dynamo::dynamo::Dynamo;
use rand::Rng;

/*
Algorithm:
   Get the frontier tiles (datatype?) - initialize in the beginning
   Find the one with the cheapest cost to go (maybe it makes sense to limit the search with some distance)
   ----How to find the cheapest?----
   If no frontier tiles within 3 tiles around -> iterate over all frontier tiles
   Go to the tile
   Remove the tile from the frontier
   Update the frontier
*/

pub fn new_explorer(jerry: &mut Jerry, world: &mut World, spatial_index: usize) -> Mission{
    let (frontier, frontier_hs) = initialize_frontier(jerry, world);
    Mission{
        name: "Explore".to_string(),
        description: None,
        probability_of_cheating: 0.8,
        goal_tracker: None,
        status: Active,
        additional_data: Some(Box::new(ExplorerData{frontier, frontier_hs, spatial_index})),
    }
}
pub fn explorer_execute(jerry: &mut Jerry, world: &mut World, mission_index: usize) -> Result<(), JerryStatus>{
    //initialize the frontier in the beginning of the simulation
    let mut robot_moved = false;
    let mut counter = 0;
    let mut new_tick = true;
    loop{
        //println!("");
        //if the robot has less than 100 energy on the new tick, use the dynamo tool with the probability of 0.8
        if new_tick && jerry.get_energy().get_energy_level() < 100{
            let mut rng = rand::thread_rng();
            let probability = rng.gen_range(0.0..1.0);
            if probability < 0.8{
                *jerry.get_energy_mut() = Dynamo::update_energy();
            }
        }


        //Debugging
        //print!("Initializing ");
        //let time_initial = std::time::Instant::now();
        

        let map = robot_map(world).unwrap();
        let (robot_view, position) = where_am_i(jerry, world);

        //if the current weather is not nice for a current biome
        //and the weather is gonna become nice in the next n ticks
        //and the robot has low energy
        //wait for the weather to become nice
        //aka throw an error
        let n = 3;
        let env_conditions = look_at_sky(world);
        let current_biome = detect_biome(&robot_view);
        if new_tick && !is_weather_nice(current_biome, env_conditions.get_weather_condition()){
            let tool = &jerry.weather_predictor;
            if is_weather_gonna_be_nice_n(tool, current_biome, n){
                return Err(JerryStatus::ExpectingNiceWeather);
            }
        }

        //Debugging
        //let elapsed_initial = time_initial.elapsed();
        //println!("took {:?} to initialize ", elapsed_initial);
    
        if counter % 10 == 0{

            //Debugging
            //println!("Updating web page ");
            //let time_web = std::time::Instant::now();

            jerry.active_region.top_left = (jerry.get_coordinate().get_row(), jerry.get_coordinate().get_col());
            jerry.active_region.bottom_right = jerry.active_region.top_left;

            //Debugging
            //println!("took {:?} to update web page ", time_web.elapsed());
        }
        counter += 1;
        new_tick = false;
        //update the frontier if the robot has moved
        if robot_moved{
            //Debugging
            //println!("Updating frontier");
            update_frontier(jerry, world, &map, ChartedCoordinate(position.0, position.1), mission_index);
        }
        let mut charted_paths  = ChartingTools::tool::<ChartedPaths>()
            .expect("too many tools used!");
        charted_paths.init(&map, world);
        let mut mission = jerry.missions.get_mut(mission_index);
        let data: &ExplorerData = mission.as_ref().unwrap()
        .additional_data.as_ref()
        .unwrap().downcast_ref().unwrap();
        let spatial_index = data.spatial_index.clone();

        //if the frontier is empty, the robot should stop executing the mission
        //and execute the analyzer
        if data.frontier.is_empty(){
            mission.as_mut().unwrap().status = Completed;

            let new_analyzer = new_sector_analyzer(spatial_index, jerry.world_dim);
            println!("Analyzing sector {}", spatial_index);
            let data = new_analyzer.additional_data.as_ref().unwrap().downcast_ref::<ActiveRegion>().unwrap();
            let (tl, br) = (data.top_left, data.bottom_right);
            let sector_data = analyzer_execute(world, tl, br);
            println!("Sector data: {:?}", sector_data);
            generate_road_builders(jerry, world, sector_data);
            jerry.active_region.top_left = tl;
            jerry.active_region.bottom_right = br;
            let _map = robot_map(world).unwrap();

            return Ok(());
        }

        //Debugging
        //let time_choose = std::time::Instant::now();
        //print!("Choosing tile ");

        let selected_tile = choose_frontier_tile(jerry, charted_paths, mission_index);
        //if the frontier is not accessible, the robot should stop executing the mission
        if selected_tile.is_err(){
            let mut mission = jerry.missions.get_mut(mission_index);
            mission.as_mut().unwrap().status = Completed;

            let new_analyzer = new_sector_analyzer(spatial_index, jerry.world_dim);
            println!("Analyzing sector {}", spatial_index);
            let data = new_analyzer.additional_data.as_ref().unwrap().downcast_ref::<ActiveRegion>().unwrap();
            let (tl, br) = (data.top_left, data.bottom_right);
            let sector_data = analyzer_execute(world, tl, br);
            println!("Sector data: {:?}", sector_data);
            generate_road_builders(jerry, world, sector_data);
            jerry.active_region.top_left = tl;
            jerry.active_region.bottom_right = br;
            
            return Ok(());
        }

        //Debugging
        //println!("took {:?} to choose tile ", time_choose.elapsed());
        //let time_go = std::time::Instant::now();
        //print!("Go to Tile ");


        //try to reach the selected tile or throw an error if the cost to get there is > 1000 or > than the robot has
        let selected_tile = selected_tile.unwrap().0;
        //print!("From {:?} to {:?}", jerry.get_coordinate(), selected_tile)
        
        let selected_tile_cost = selected_tile.1;
        //go to an intermediate tile if the selected tile is too expensive
        if selected_tile_cost > 1000{
            let current_coordinate = (jerry.get_coordinate().get_row(), jerry.get_coordinate().get_col());
            let row_diff = selected_tile.0 as i32 - current_coordinate.0 as i32;
            let col_diff = selected_tile.1 as i32 - current_coordinate.1 as i32;
            let mut intermediate_tile = (0, 0);
            if row_diff > 0{
                intermediate_tile.0 = current_coordinate.0 + row_diff as usize / 2;
            }
            else{
                intermediate_tile.0 = current_coordinate.0 - row_diff.abs() as usize / 2;
            }
            if col_diff > 0{
                intermediate_tile.1 = current_coordinate.1 + col_diff as usize / 2;
            }
            else{
                intermediate_tile.1 = current_coordinate.1 - col_diff.abs() as usize / 2;
            }
            //take the slice of the robot map around the intermediate tile
            for n in 0..map.len(){
                let map_slice = robot_map_slice_n(jerry, &map, n).unwrap();
                'outer: for (i, row) in map_slice.iter().enumerate(){
                    for (j, tile) in row.iter().enumerate(){

                        //if the tile is in the cache and it's walkable, add it to the frontier to reach it on the next iteration
                        if let Some(tile) = tile{
                            if tile.tile_type.properties().walk(){
                                let mut mission = jerry.missions.get_mut(mission_index);
                                let data: &mut ExplorerData = mission.as_mut().unwrap()
                                .additional_data.as_mut().unwrap().downcast_mut().unwrap();
                                data.frontier.push(ChartedCoordinate(i, j));
                                data.frontier_hs.insert(ChartedCoordinate(i, j));
                                break 'outer;
                            }
                        }
                    }
                }
            }
        }
        if selected_tile_cost > jerry.get_energy().get_energy_level(){
            return Err(JerryStatus::LowEnergyWarning);
        }
        if let Err(error) =  go_to_frontier(jerry, &map, world, selected_tile, mission_index){
            
            match error{
                //if the go interface failed (tool error), try to move to a tile next to the current one
                | FailedToGo => {
                     robot_moved = true;
                    continue;
                }
                //if not enough energy, stop executing the mission and wait to recharge
                | ExplorerError::NotEnoughEnergy => {
                    return Err(JerryStatus::LowEnergyWarning);
                }
                | _ => {}
            }
            //println!("Failed to go to the selected tile {:?}, removing it from the frontier", selected_tile);
            return Err(JerryStatus::MissionExecutionError);
        }
        else{

            //Debugging
            //println!("took {:?} to go to tile ", time_go.elapsed());

            robot_moved = true;
            continue;
        }
    }
}
//initialize the frontier when adding the new explorer mission
pub fn initialize_frontier(jerry: &mut Jerry, world: &mut World) -> (Vec<ChartedCoordinate>, HashSet<ChartedCoordinate>){
    let (_, spawn_coordinates) = where_am_i(jerry, world);
    let map = robot_map(world).unwrap();
    let mut frontier: Vec<ChartedCoordinate> = Vec::new();
    let mut frontier_hs = HashSet::new();
    let row = spawn_coordinates.0;
    let col = spawn_coordinates.1;
    let rows = map.len();
    let cols = map[0].len();
    if map[row][col].is_some(){
        if row > 0{
            frontier.push(ChartedCoordinate(row - 1, col));
            frontier_hs.insert(ChartedCoordinate(row - 1, col));
        }
        if row < rows - 1{
            frontier.push(ChartedCoordinate(row + 1, col));
            frontier_hs.insert(ChartedCoordinate(row + 1, col));
        }
        if col > 0{
            frontier.push(ChartedCoordinate(row, col - 1));
            frontier_hs.insert(ChartedCoordinate(row, col - 1));
        }
        if col < cols - 1{
            frontier.push(ChartedCoordinate(row, col + 1));
            frontier_hs.insert(ChartedCoordinate(row, col + 1));
        }
    }
    (frontier, frontier_hs)

}
fn is_frontier(map: &Vec<Vec<Option<Tile>>>, coordinate: (usize, usize)) -> bool{
    let rows = map.len();
    let cols = map[0].len();
    let row = coordinate.0;
    let col = coordinate.1;
    if map[row][col].is_some(){
        if (row > 0 && map[row - 1][col].is_none()) ||
            (row < rows - 1 && map[row + 1][col].is_none()) ||
            (col > 0 && map[row][col - 1].is_none()) ||
            (col < cols - 1 && map[row][col + 1].is_none())
            {
                return true;
            }
    }
    false
}
fn choose_frontier_tile(jerry: &mut Jerry, tool: ChartedPaths, mission_index: usize) -> Result<(ChartedCoordinate, u32), ExplorerError>{

    let mission = jerry.missions.get(mission_index);
    let robot_coord = (jerry.get_coordinate().get_row(), jerry.get_coordinate().get_col());
    let robot_charted_coord = ChartedCoordinate(robot_coord.0, robot_coord.1);
    let data: &ExplorerData = mission.as_ref().unwrap().additional_data.as_ref().unwrap().downcast_ref().unwrap();

    //define the search depth for the frontier
    let search_depth = if data.frontier.len() > 10 { 10 } else { data.frontier.len() };
    let mut frontier_accessible = false;
    let mut start = data.frontier.len();
    let mut candidate_cost = 0;
    //iterate first over the last 10 added frontier tiles then 20 then 30 and so on
    while start > 0{
        let end = start;
        start = start.saturating_sub(search_depth);

        //candidate is the tile with the minimum cost to go to
        //if the cost is u32::MAX, the tile is not accessible
        let candidate = data.frontier[start..end].iter().rev()
        .filter(|coord| data.frontier_hs.contains(coord))
        .min_by_key(|coord|{
            let cost = tool.shortest_path_cost(robot_charted_coord, **coord).unwrap_or(u32::MAX);
            if cost != u32::MAX{
                frontier_accessible = true;
            }
            candidate_cost = cost;
            cost
        });
        //return the accessible tile with the minimum cost
        if frontier_accessible{
            return Ok((ChartedCoordinate(candidate.unwrap().0, candidate.unwrap().1), candidate_cost));
        }
    }
    return Err(FrontierNotAccessible);
}
//REFACTOR THIS!
mod fast_paths;
fn go_to_frontier(jerry: &mut Jerry, map: &Vec<Vec<Option<Tile>>>, 
                world: &mut World, frontier_coordinate: ChartedCoordinate,
                mission_index: usize)
                ->Result<(), ExplorerError>{

    let jerry_coordinate = ChartedCoordinate(jerry.get_coordinate().get_row(), jerry.get_coordinate().get_col());
    //if the destination is adjacent to the robot, move to it
    if is_adjacent(jerry_coordinate, frontier_coordinate){
        let direction = coordinate_to_direction(jerry_coordinate, frontier_coordinate);
        if let Err(error) = go(jerry, world, direction){
            //if not enough energy, return an error without removing the tile from the frontier
            match error{
                | robotics_lib::utils::LibError::NotEnoughEnergy => return Err(ExplorerError::NotEnoughEnergy),
                | _ => {
                    remove_tile_from_frontier(jerry, frontier_coordinate, mission_index);
                    return Err(FailedToGo);
                    }
            }
        }   
        else{
            remove_tile_from_frontier(jerry, frontier_coordinate, mission_index);
            return Ok(());
        }
    }
    //is a frontier tile in the area nxn around the robot?
    //if yes, use fast_paths to go to it without processing the whole map
    //can also try other values for n
    let n = 9;
    if is_within_n(jerry_coordinate, frontier_coordinate, n){
        if let Ok(_) = fast_paths::go_to_coordinates
        (jerry, map, world, false, (frontier_coordinate.0, frontier_coordinate.1), n){
            remove_tile_from_frontier(jerry, frontier_coordinate, mission_index);
            return Ok(());
        }

    }

    //use the tomtom tool if the destination is pretty far from the robot
    //or if the fast_paths failed
    //the errors are handled in the same way as in the function above
    if let Err(_) = TomTom::go_to_coordinates(jerry, world, false, (frontier_coordinate.0, frontier_coordinate.1)){
        if let Err(string) = TomTom::go_to_coordinates(jerry, world, true, (frontier_coordinate.0, frontier_coordinate.1)){
            println!("Error: {:?} failed to go from {:?} to {:?}", string, jerry.get_coordinate(), frontier_coordinate);
            if string.eq("Not enough energy!"){
                return Err(ExplorerError::NotEnoughEnergy);
            }
            else{
                //remove the tile from frontier anyways
                remove_tile_from_frontier(jerry, frontier_coordinate, mission_index);
                return Err(FailedToGo);
            }
        }
        else {
            //if moved successfully, remove the tile from the frontier
            remove_tile_from_frontier(jerry, frontier_coordinate, mission_index);
            return Ok(());
        }
    }
    else {
        //if moved successfully, remove the tile from the frontier
        remove_tile_from_frontier(jerry, frontier_coordinate, mission_index);
        return Ok(())
    }
}
//update the frontier using the 3x3 area around the robot
fn update_frontier(jerry: &mut Jerry, world: &mut World, map: &Vec<Vec<Option<Tile>>>, position: ChartedCoordinate, mission_index: usize){
    let jerry = Rc::new(RefCell::new(jerry));
    let jerry_c = jerry.clone();
    //let data: &mut ExplorerData = mission.additional_data.as_mut().unwrap().downcast_mut().unwrap();
    let x = position.0;
    let y = position.1;
    let len = map.len();
    for i in (x.saturating_sub(1))..=(x + 1).min(len - 1) {
        for j in (y.saturating_sub(1))..=(y + 1).min(len - 1) {
            if (i, j) == (x, y){
                continue;
            }
            let tile_coord = ChartedCoordinate(i, j);
            let spatial_index = calculate_spatial_index(i, j, map.len());

            //check if there's a mission for the spatial index of the tile
            let jerry_immut = jerry.borrow();
            let _data: &ExplorerData = jerry_immut.missions.get(mission_index).unwrap().additional_data.as_ref().unwrap().downcast_ref().unwrap();
            let mission_exists = jerry_immut.missions.iter().any(|mission| {
                if let Some(explorer_data) = mission.additional_data.as_ref().unwrap().downcast_ref::<ExplorerData>(){
                    explorer_data.spatial_index == spatial_index
                }
                else{
                    false
                }
            });
            drop(jerry_immut);

            //if the tile is a frontier tile and it's not in the current mission's spatial index
            //so we need to initialize a new mission for the new spatial index
            if !mission_exists{
                let jerry_mut = jerry.clone();
                let new_mission = new_explorer(&mut jerry_mut.borrow_mut(), world, spatial_index);
                jerry_mut.borrow_mut().missions.push_back(new_mission);
                println!("New mission \"Explore\" for spatial index {}", spatial_index);
            }
            
            let mut jerry_mut = jerry_c.borrow_mut();
            let data: &mut ExplorerData = jerry_mut.missions.get_mut(mission_index).unwrap().additional_data.as_mut().unwrap().downcast_mut().unwrap();
            
            //add a tile to the frontier if it's not already there
            if !data.frontier_hs.contains(&tile_coord) {
                if is_frontier(map, (tile_coord.0, tile_coord.1)) && 
                spatial_index == data.spatial_index{
                    data.frontier.push(tile_coord);
                    data.frontier_hs.insert(tile_coord);
                }
            }
            //remove a tile from the frontier if it's not a frontier tile anymore
            else{
                if !is_frontier(map, (tile_coord.0, tile_coord.1)){
                    let _ = data.frontier_hs.remove(&tile_coord);
                }
            }
        }
    }
}
fn remove_tile_from_frontier(jerry: &mut Jerry, tile: ChartedCoordinate, mission_index: usize){
    let mut mission = jerry.missions.get_mut(mission_index);
    let data: &mut ExplorerData = mission.as_mut().unwrap().additional_data.as_mut().unwrap().downcast_mut().unwrap();
    let index = data.frontier.iter().position(|x| *x == tile).unwrap();
    let _ = data.frontier.remove(index);
    let _ = data.frontier_hs.remove(&tile);
}
pub(crate) fn is_adjacent(a: ChartedCoordinate, b: ChartedCoordinate) -> bool{
    if (a.0 == b.0 && (a.1 == b.1 + 1 || a.1 == b.1.saturating_sub(1))) || (a.1 == b.1 && (a.0 == b.0 + 1 || a.0 == b.0.saturating_sub(1))){
        return true;
    }
    false
}
pub(crate) fn is_within_n(a: ChartedCoordinate, b: ChartedCoordinate, n: usize) -> bool{
    if (a.0 as i32 - b.0 as i32).abs() < n as i32 && (a.1 as i32 - b.1 as i32).abs() < n as i32{
        return true;
    }
    false
}
pub(crate) fn coordinate_to_direction(a: ChartedCoordinate, b: ChartedCoordinate) -> Direction{
    if a.0 == b.0{
        if b.1 == a.1 + 1{
            return Direction::Right;
        }
        else{
            return Direction::Left;
        }
    }
    else{
        if b.0 == a.0 + 1{
            return Direction::Down;
        }
        else{
            return Direction::Up;
        }
    }
}
//returns a slice of the robot map which is nxn area around the robot (or less if the robot is near the edge of the map)
pub(crate) fn robot_map_slice_n
    (jerry: &mut Jerry, robot_map: &Vec<Vec<Option<Tile>>>, n: usize)
     -> Option<Vec<Vec<Option<Tile>>>>{
    let center = (jerry.get_coordinate().get_row(), jerry.get_coordinate().get_col());
    let top_left = (center.0.saturating_sub(n / 2), center.1.saturating_sub(n / 2));
    let bottom_right = (
        if center.0 + n / 2 >= robot_map.len() - 1 {robot_map.len() - 1} else {center.0 +  n / 2}, 
        if center.1  + n / 2 >= robot_map[0].len() - 1 {robot_map[0].len() - 1} else {center.1 +  n / 2}
    );
    robot_map_slice(robot_map, top_left, bottom_right)
}
pub struct ExplorerData{
    pub frontier: Vec<ChartedCoordinate>,
    pub frontier_hs: HashSet<ChartedCoordinate>,
    pub spatial_index: usize,
}
#[derive(Debug)]
pub enum ExplorerError{
    FrontierNotAccessible,
    FailedToGo,
    NotEnoughEnergy,
}
