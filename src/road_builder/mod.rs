use core::panic;
use std::collections::HashSet;

use bessie::bessie::{RpmError, State};
use charting_tools::charted_map::MapKey;
use rand::Rng;
use robotics_lib::interface::{destroy, robot_map, where_am_i, Direction};
use robotics_lib::runner::Runnable;
use robotics_lib::utils::{LibError};
use robotics_lib::world::tile::{Content, Tile, TileType};
use robotics_lib::world::World;
use rust_and_furious_dynamo::dynamo::{Dynamo};
use rust_eze_tomtom::{TomTom};
use crate::explorer::{coordinate_to_direction, is_adjacent};
use crate::resources::{empty_the_backpack, get_content, ResourceCollectorError};
use crate::interface::Jerry;
use crate::sector_analyzer::SectorData;
use crate::utils::{JerryStatus};
use charting_tools::charted_coordinate::ChartedCoordinate;
use charting_tools::charted_paths::{ChartedPaths};
use charting_tools::ChartingTools;
use crate::utils::Mission;
use crate::utils::MissionStatus::Paused;

use crate::utils::MissionStatus::Completed;
use crate::road_builder::RoadBuilderError::RoadNonAccessible;

//plans a road between two nodes
//returns a vector of charted coordinates which is the sequence of tiles to go through
//it also cuts the ends of the path if they are shallow water or deep water

pub fn plan_road(_jerry: &mut Jerry, world: &mut World, node1: (usize, usize), node2: (usize, usize)) -> Vec<ChartedCoordinate> {
    let mut charted_paths  = ChartingTools::tool::<ChartedPaths>()
            .expect("too many tools used!");
    let map = robot_map(world).unwrap();
    charted_paths.init(&map, world);
    let from = ChartedCoordinate::new(node1.0, node1.1);
    let to = ChartedCoordinate::new(node2.0, node2.1);
    let path = charted_paths.shortest_path(from, to);
    if let Some(path) = path{
        return path.1;
    }
    Vec::new()
}
fn shrink_path(path: &mut Vec<ChartedCoordinate>, map: &Vec<Vec<Option<Tile>>>){
    let mut i = 0;
    let to = path.len() / 2;
    while i < to{
        let ends = (path[0], path[path.len() - 1]);
        if let (Some(head),Some(tail)) = (&map[ends.0.0][ends.0.1], &map[ends.1.0][ends.1.1]){

            let head_tile_type = head.tile_type;
            let tail_tile_type = tail.tile_type;
            //stop when we reach a tile that is not water
            if head_tile_type != TileType::ShallowWater && head_tile_type != TileType::DeepWater 
            && tail_tile_type != TileType::ShallowWater && tail_tile_type != TileType::DeepWater{
                break;
            }
            if head.tile_type == TileType::ShallowWater || head.tile_type == TileType::DeepWater{
                path.remove(0);
            }
            if tail.tile_type == TileType::ShallowWater || tail.tile_type == TileType::DeepWater{
                path.remove(path.len() - 1);
            }
        }
        i +=1;
    }
}

//the function gets the sector data and adds new missions for the robot
pub fn generate_road_builders(jerry: &mut Jerry, world: &mut World, sector_data: SectorData){
    let nodes = sector_data.nodes;
    let _resources = sector_data.resources;
    let mut missions = 0;
    let map = robot_map(world).unwrap();
     //if there is just one node
     if nodes.len() == 1{
        //we need to connect it to the global road (if it exists)
        //if it does not exist, it becomes a global road
        if jerry.road_tiles.is_empty(){
            jerry.road_tiles.insert(ChartedCoordinate::new(nodes[0].0, nodes[0].1));
        }
        else {
            //else we find a path to the global road
            let target = find_nearest_road_tile(&map, nodes[0], &jerry.road_tiles);
            let path = plan_road(jerry, world, nodes[0], target);
            let status = ConnectionStatus::Global;
            let mission = new_road_builder(&path, status);
            jerry.missions.push_back(mission);
            missions += 1;
        }
        
     }
     //if there are two nodes
     //we build the road between them and then connect the road to the global road
        else if nodes.len() == 2{
            //road between the nodes
            let path = plan_road(jerry, world, nodes[0], nodes[1]);
            let mut to_pave = HashSet::new();
            for tile in &path {
                to_pave.insert(tile.clone());
            }
            let status = ConnectionStatus::NotConnected(0);
            let mission = new_road_builder(&path, status);
            jerry.missions.push_back(mission);
            missions += 1;

            //if there is no global road, the road becomes global
            if jerry.road_tiles.is_empty(){
                for tile in &path{
                    jerry.road_tiles.insert(tile.clone());
                }
            }
            else{
                //connecting the road to the global road if it exists
                let (node1, node2) = nodes_to_connect_2_roads(&to_pave, &jerry.road_tiles);
                let path = plan_road(jerry, world, (node1.0, node1.1), (node2.0, node2.1));
                let status = ConnectionStatus::Global;
                let mission = new_road_builder(&path, status);
                jerry.missions.push_back(mission);
                missions += 1;
            }
        }
        //if there are more than two nodes
        //we connect the two most distant ones
        //connect the other nodes to this road
        //connect the local road network to the global road
        else{
            //road between the two most distant nodes
            let (node1, node2) = get_2_furthest_nodes(&nodes);
            let path = plan_road(jerry, world, (node1.0, node1.1), (node2.0, node2.1));
            let mut to_pave = HashSet::new();
            for tile in &path {
                to_pave.insert(tile.clone());
            }
            let status = ConnectionStatus::NotConnected(0);
            let mission = new_road_builder(&path, status);
            jerry.missions.push_back(mission);
            missions += 1;

            //connecting the other nodes to the road
            for node in nodes.iter(){
                if *node != (node1.0, node1.1) && *node != (node2.0, node2.1){
                    let target = find_nearest_road_tile(&map, *node, &to_pave);
                    let path = plan_road(jerry, world, *node, target);
                    //add the new path tiles to the sector road
                    for tile in &path {
                        to_pave.insert(tile.clone());
                    }
                    let status = ConnectionStatus::Local(0);
                    let mission = new_road_builder(&path, status);
                    jerry.missions.push_back(mission);
                    missions += 1;
                }
            }
            //check if the global road exists
            //if it does, connect the local road network to the global road
            if jerry.road_tiles.is_empty(){
                for tile in &path{
                    jerry.road_tiles.insert(tile.clone());
                }
            }
            else{
                //connecting the local road network to the global road
                let (node1, node2) = nodes_to_connect_2_roads(&to_pave, &jerry.road_tiles);
                let path = plan_road(jerry, world, (node1.0, node1.1), (node2.0, node2.1));
                let status = ConnectionStatus::Global;
                let mission = new_road_builder(&path, status);
                jerry.missions.push_back(mission);
                missions += 1;
            }
        }
    println!("Missions {}", missions);
}

//sector paver struct
//should track the building of the road network in a sector
//calls the road builder
//should save the last paved to be able to come back after collecting resources

pub fn new_road_builder(path: &Vec<ChartedCoordinate>, connect_to: ConnectionStatus) -> Mission{
    let mut to_pave = HashSet::new();
    for tile in path {
        to_pave.insert(tile.clone());
    }

    Mission {
        name: "Road Builder".to_string(),
        description: None,
        probability_of_cheating: 0.7,
        goal_tracker: None,
        status: Paused,
        additional_data: Some(Box::new(RoadBuilderData{to_pave: to_pave, paved: HashSet::new(), connect: connect_to})),
    }
}
//executes the road builder mission
//builds a road between two nodes by placing rocks
//ideally, path should not contain crates
//if it does, the function will skip a tile
//if it contains a teleport, the function will teleport to that tile
pub fn road_builder_execute(jerry: &mut Jerry, world: &mut World, mission_index: usize) -> Result<(), JerryStatus> {
    let new_tick = true;
    
    /*
        Following algorithm:
        Figure out the closest road tile to pave
        Go to that tile
        Try to pave it
        If not enough energy, return the error and with the prob of cheating update it
        If not enough material, try to collect it, come back and pave again
     */
    
    loop{
        //if the robot has less than 100 energy on the new tick, use the dynamo tool with the probability of 0.8
        if new_tick && jerry.get_energy().get_energy_level() < 100{
            let mut rng = rand::thread_rng();
            let probability = rng.gen_range(0.0..1.0);
            if probability < 0.7{
                *jerry.get_energy_mut() = Dynamo::update_energy();
            }
        }

        //initializing necessary tools and data

        let map = robot_map(world).unwrap();
        let (_robot_view, _position) = where_am_i(jerry, world);
        let mut charted_paths  = ChartingTools::tool::<ChartedPaths>()
            .expect("too many tools used!");
        charted_paths.init(&map, world);
        let mission = jerry.missions.get_mut(mission_index).unwrap();
        let road_builder_data = mission.additional_data.as_mut().unwrap().downcast_mut::<RoadBuilderData>().unwrap();
        
        //debugging
        println!("Paving {:?}", mission_index);
        //

        //completion condition
        if road_builder_data.to_pave.is_empty(){
            println!("I built the road!");
            mission.status = Completed;
            return Ok(());
        }
        let selected_tile = choose_tile_to_pave(jerry, charted_paths, mission_index);

        //if the selected tile to pave is not accessible, panic, because this should not happen
        if selected_tile.is_err(){
            panic!("Selected tile to pave is not accessible");
        }

        //try to reach the tiles adjacent to the selected tile or throw an error if it is too expensive
        let selected_tile = selected_tile.unwrap().0;
        let selected_tile_cost = selected_tile.1;
        //panic if the selected tile is too expensive <- REFACTOR THIS
        if selected_tile_cost > 1000{
            panic!("Too Expensive bro")
        }
        if selected_tile_cost > jerry.get_energy().get_energy_level(){
            return Err(JerryStatus::LowEnergyWarning);
        }

        if let Err(error) =  go_and_pave(jerry, &map, world, selected_tile, mission_index){

            let mission = jerry.missions.get_mut(mission_index).unwrap();
            let road_builder_data = mission.additional_data.as_mut().unwrap().downcast_mut::<RoadBuilderData>().unwrap();
            match error{
                //if not enough energy, return the error and with the prob of cheating update it or wait
                | RoadBuilderError::NotEnoughEnergy => {
                    return Err(JerryStatus::LowEnergyWarning);
                }
                //if cannot get the material, stop executing the mission
                | RoadBuilderError::CannotGetMaterial => {
                    println!("Cannot get the material");
                    mission.status = Completed;
                    return Ok(());
                }
                //if cannot pave the tile, skip it, remove it from the to_pave set and continue
                | RoadBuilderError::CannotPaveTile => {
                    road_builder_data.to_pave.remove(&selected_tile);
                    continue;
                }
                //other errors should not be propagated
                | _ => {panic!("{:?}", error)}
            }
        }
        //successfully paved the tile and go_and_pave has deleted it from the to_pave set
        else{
            continue;
        }
    }
}
fn go_and_pave(jerry: &mut Jerry, map: &Vec<Vec<Option<Tile>>>, world: &mut World, tile: ChartedCoordinate, mission_index: usize) -> Result<(), RoadBuilderError>{
    
    let jerry_coordinate = ChartedCoordinate(jerry.get_coordinate().get_row(), jerry.get_coordinate().get_col());
    //if the destination is adjacent to the robot, try to pave it
    if is_adjacent(jerry_coordinate, tile){
        let direction = coordinate_to_direction(jerry_coordinate, tile);
        //if failed to pave, return the error
        if let Err(error) = bessie_controller(jerry, map, world, direction, mission_index){
            return Err(error);
        }
        //else modify the mission data and return Ok
        else{
            let mission = jerry.missions.get_mut(mission_index).unwrap();
            let road_builder_data = mission.additional_data.as_mut().unwrap().downcast_mut::<RoadBuilderData>().unwrap();
            road_builder_data.to_pave.remove(&tile);
            road_builder_data.paved.insert(tile);
            return Ok(());
        }
    }
    //else try to reach the tile adjacent to the target and try bessie there
    else{
        if let Err(error) = TomTom::go_to_coordinates(jerry, world, true, (tile.0, tile.1)){
            if error.eq("Not enough energy!"){
                return Err(RoadBuilderError::NotEnoughEnergy);
            }
            else{
                //Should not fail to go to the tile
                panic!("{:?}", error);
            }
        }
        else {
            //if moved successfully, do the bessie
            let jerry_coordinate = ChartedCoordinate(jerry.get_coordinate().get_row(), jerry.get_coordinate().get_col());
            let direction = coordinate_to_direction(jerry_coordinate, tile);
            if let Err(error) = bessie_controller(jerry, map, world, direction, mission_index){
                return Err(error);
            }
            //else modify the mission data and return Ok
            else{
                let mission = jerry.missions.get_mut(mission_index).unwrap();
                let road_builder_data = mission.additional_data.as_mut().unwrap().downcast_mut::<RoadBuilderData>().unwrap();
                road_builder_data.to_pave.remove(&tile);
                road_builder_data.paved.insert(tile);
                return Ok(());
            } 
        }
    }
}
fn bessie_controller(jerry: &mut Jerry, map: &Vec<Vec<Option<Tile>>>, world: &mut World, direction: Direction, mission_index: usize) -> Result<(), RoadBuilderError>{
    let vent_tool1 = jerry.vent.clone();
    let vent_tool2 = jerry.vent.clone();
    if let Err(error) = bessie::bessie::road_paving_machine(jerry, world, direction.clone(), State::MakeRoad){
        //if not enough energy, return an error without removing the tile from the frontier
        match error{
            //Normally, should not happen but we'll skip the tile
            | RpmError::CannotPlaceHere => {return Err(RoadBuilderError::CannotPaveTile);}
            //try to destroy the content and pave again, if the tile does contain a crate, skip it and do not pave returning a specific error
            | RpmError::MustDestroyContentFirst => {
                let tile_coord = direction_to_coordinate((jerry.get_coordinate().get_row(), jerry.get_coordinate().get_col()), direction.clone());
                //force to unwrap because should be known
                let tile = map[tile_coord.0][tile_coord.1].as_ref().unwrap();
                if Content::Crate(0..0) == tile.content.to_default(){
                    return Err(RoadBuilderError::CannotPaveTile);
                }
                if let Err(error) = destroy(jerry, world, direction.clone()){
                    match error{
                        | LibError::NotEnoughEnergy => return Err(RoadBuilderError::NotEnoughEnergy),
                        //if not enough space in the backpack to destroy the content
                        | LibError::NotEnoughSpace(space_needed) => {
                            vent_tool1.borrow_mut().create_waypoint(jerry, 1000);
                            let mission = jerry.missions.get_mut(mission_index).unwrap();
                            let road_builder_data = mission.additional_data.as_mut().unwrap().downcast_mut::<RoadBuilderData>().unwrap();
                            let planned_road = &road_builder_data.to_pave.clone();
                            if let Err(error) = empty_the_backpack(jerry, world, Some(planned_road), space_needed){
                                match error{
                                    //if not enough energy, return the error
                                    | ResourceCollectorError::NotEnoughEnergy => return Err(RoadBuilderError::NotEnoughEnergy),
                                    | _ => {panic!("unexpected {:?}", error)}
                                }
                            }
                            //if disposed the content, return to the waypoint and try to pave again
                            else{
                                if let Err(error) = vent_tool1.borrow_mut().vent_waypoint(jerry, world, 1000){
                                    match error{
                                        //pretty much the only error that can happen
                                        | vent_tool_ascii_crab::VentError::NotEnoughEnergy => return Err(RoadBuilderError::NotEnoughEnergy),
                                        //otherwise, idk
                                        | _ => panic!("{:?}", error),
                                    }
                                }
                                drop(vent_tool1);
                                //if the robot has returned, try to pave again
                                return bessie_controller(jerry, map, world, direction, mission_index);

                            }
                        }
                        | _ => {panic!("{:?}", error)}
                    }
                }
                //if the content is successfully destroyed, try to pave again
                else{
                    return bessie_controller(jerry, map, world, direction, mission_index);
                }
            }
            //No material in the backpack or not enough material to pave
            | RpmError::NoRockHere | RpmError::NotEnoughMaterial=> {
                //need to search for rocks
                //remember the current position to return back after the search using the vent tool waypoint
                let _current_position = (jerry.get_coordinate().get_row(), jerry.get_coordinate().get_col());
                vent_tool2.borrow_mut().create_waypoint(jerry, 1000);
                let mission = jerry.missions.get_mut(mission_index).unwrap();
                let road_builder_data = mission.additional_data.as_mut().unwrap().downcast_mut::<RoadBuilderData>().unwrap();
                let planned_road = &road_builder_data.to_pave.clone();
                if let Err(error) = get_content(jerry, world, Content::Rock(0), Some(&planned_road), 20){
                    match error{
                        //if not enough energy, just propagate the error
                        | ResourceCollectorError::NotEnoughEnergy => return Err(RoadBuilderError::NotEnoughEnergy),
                        //Normally, should not happen
                        | ResourceCollectorError::BackPackIsFull => panic!("Backpack is full"),
                        //if the path to the content is not found
                        |ResourceCollectorError::PathNotFound => return Err(RoadBuilderError::CannotGetMaterial),
                        _ => panic!("Unexpected {:?}", error),
                    }
                }
                //if found the content, return to the waypoint and try to pave again
                else{
                    if let Err(error) = vent_tool2.borrow_mut().vent_waypoint(jerry, world, 1000){
                        match error{
                            //pretty much the only error that can happen
                            | vent_tool_ascii_crab::VentError::NotEnoughEnergy => return Err(RoadBuilderError::NotEnoughEnergy),
                            //otherwise, idk
                            | _ => panic!("{:?}", error),
                        }
                    }
                    //if the robot has returned, try to pave again
                    drop(vent_tool2);
                    return bessie_controller(jerry, map, world, direction, mission_index);
            }

            }
            | RpmError::NotEnoughEnergy => return Err(RoadBuilderError::NotEnoughEnergy),
            _ => {panic!("{:?}", error)}
        }
    }
    Ok(())   
}
//choose the tile to pave with the cheapest cost of going to
fn choose_tile_to_pave(jerry: &mut Jerry, tool: ChartedPaths, mission_index: usize) -> Result<(ChartedCoordinate, u32), RoadBuilderError>{

    let mission = jerry.missions.get(mission_index);
    let robot_coord = (jerry.get_coordinate().get_row(), jerry.get_coordinate().get_col());
    let robot_charted_coord = ChartedCoordinate(robot_coord.0, robot_coord.1);
    let data: &RoadBuilderData = mission.as_ref().unwrap().additional_data.as_ref().unwrap().downcast_ref().unwrap();

    let search_depth = if data.to_pave.len() > 10 { 10 } else { data.to_pave.len() };
    let mut road_accessible = false;
    let mut start = data.to_pave.len();
    let mut candidate_cost = 0;
    //iterate first over the last 10 added frontier tiles then 20 then 30 and so on
    while start > 0{
        let _end = start;
        start = start.saturating_sub(search_depth);

        //candidate is the tile with the minimum cost to go to
        //if the cost is u32::MAX, the tile is not accessible
        let candidate = data.to_pave.iter()
        .min_by_key(|coord|{
            let cost = tool.shortest_path_cost(robot_charted_coord, **coord).unwrap_or(u32::MAX);
            if cost != u32::MAX{
                road_accessible = true;
            }
            candidate_cost = cost;
            cost
        });
        //return the accessible tile with the minimum cost
        if road_accessible{
            return Ok((ChartedCoordinate(candidate.unwrap().0, candidate.unwrap().1), candidate_cost));
        }
    }
    return Err(RoadNonAccessible);
}
pub fn get_road_required_resources(path: &Vec<ChartedCoordinate>, map: &Vec<Vec<Option<Tile>>>) -> (usize, usize, usize){
    let mut energy = 0;
    let mut rocks = 0;
    let mut backpack_space = 0;
    for coord in path.iter(){
        if let Some(tile) =  &map[coord.0][coord.1]{
            match tile.tile_type{
                | TileType::Lava => {
                    rocks += 3;
                    energy += 9;
                }
                | TileType::DeepWater => {
                    rocks += 3;
                    energy += 6;
                }
                | TileType::ShallowWater => {
                    rocks += 2;
                    energy += 2;
                }
                | TileType::Teleport(_) | TileType::Street=> {}
                | _ => {
                    rocks += 1;
                    energy += 1;
                }
            }
            match tile.content{
                | Content::None => {}
                | Content::Bush(amount) | Content::Tree(amount) | Content::Fish(amount) | Content::Water(amount) => {
                    energy += tile.content.properties().cost() * amount;
                    backpack_space += amount;
                }
                | Content::Fire => {
                    energy += tile.content.properties().cost() * tile.content.properties().max();
                    backpack_space += tile.content.properties().max();
                }
                | _ => {}
            }
        }
    }
    (rocks, energy, backpack_space)
}

pub fn find_nearest_road_tile(map: &Vec<Vec<Option<Tile>>>, coord: (usize, usize), road: &HashSet<ChartedCoordinate>) -> (usize, usize){
    let mut radius: i32 = 1;
    loop{
        if radius as usize > map.len() && radius as usize > map[0].len(){
            panic!("No road found");
        }
        for i in -radius..=radius{
            for j in -radius..=radius{
                if coord.0 as i32 + i < 0 || 
                    coord.1 as i32 + j < 0 || 
                    coord.0 as i32 + i >= map.len() as i32 || 
                    coord.1 as i32 + j >= map[0].len() as i32{
                    continue;
                }
                else{
                    let new_coord = (coord.0 as i32 + i, coord.1 as i32 + j);
                    if road.contains(&ChartedCoordinate::new(new_coord.0 as usize, new_coord.1 as usize)){
                        return (new_coord.0 as usize, new_coord.1 as usize);
                    }
                }
            }
        }
        radius += 1;
    }
}
fn get_2_furthest_nodes(nodes: &Vec<(usize, usize)>) -> (ChartedCoordinate, ChartedCoordinate){
    let mut max_distance = 0;
    let mut node1 = ChartedCoordinate::new(0, 0);
    let mut node2 = ChartedCoordinate::new(0, 0);
    for n1 in nodes.iter(){
        for n2 in nodes.iter(){
            let distance = (n1.0 as i32 - n2.0 as i32).abs() + (n1.1 as i32 - n2.1 as i32).abs();
            if distance > max_distance{
                max_distance = distance;
                node1 = ChartedCoordinate::new(n1.0, n1.1);
                node2 = ChartedCoordinate::new(n2.0, n2.1);
            }
        }
    }
    (node1, node2)
}
fn nodes_to_connect_2_roads(road1: &HashSet<ChartedCoordinate>, road2: &HashSet<ChartedCoordinate>) -> (ChartedCoordinate, ChartedCoordinate) {
    let mut min_distance = 0;
    let mut node1 = ChartedCoordinate::new(0, 0);
    let mut node2 = ChartedCoordinate::new(0, 0);
    for n1 in road1.iter(){
        for n2 in road2.iter(){
            let distance = (n1.0 as i32 - n2.0 as i32).abs() + (n1.1 as i32 - n2.1 as i32).abs();
            if distance < min_distance || min_distance == 0{
                min_distance = distance;
                node1 = *n1;
                node2 = ChartedCoordinate::new(n2.0, n2.1);
            }
        }
    }
    (node1, node2)
}
fn direction_to_coordinate(coord: (usize, usize), direction: Direction) -> (usize, usize){
    match direction{
        | Direction::Up => (coord.0 - 1, coord.1),
        | Direction::Down => (coord.0 + 1, coord.1),
        | Direction::Left => (coord.0, coord.1 - 1),
        | Direction::Right => (coord.0, coord.1 + 1),
    }
}
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum RoadBuilderError{
    NotEnoughEnergy,
    NotEnoughMaterial,
    CannotGetMaterial,
    RoadNonAccessible,
    CannotPaveTile,
}
pub struct RoadBuilderData{
    to_pave: HashSet<ChartedCoordinate>,
    paved: HashSet<ChartedCoordinate>,
    connect: ConnectionStatus,
}
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum ConnectionStatus{
    NotConnected(usize), //usize is a spatial index of the main road of the sector
    Global,  //if we connect the node to the global road
    Local(usize), //usize is a spatial index of the main toad of the sector to which we connect the road
}