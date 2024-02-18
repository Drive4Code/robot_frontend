
use crate::interface::Jerry;
use crate::explorer::robot_map_slice_n;
use std::collections::{HashSet, VecDeque};
use charting_tools::charted_coordinate::ChartedCoordinate;
use robotics_lib::interface::Direction;
use robotics_lib::runner::Runnable;
use robotics_lib::world::World;
use robotics_lib::world::{tile::TileType};
use std::cmp::Ordering;
use std::collections::{BinaryHeap};
use robotics_lib::interface::{look_at_sky};
use robotics_lib::utils::calculate_cost_go_with_environment;
use robotics_lib::world::tile::{Content, Tile};
use robotics_lib::interface::teleport;
use robotics_lib::interface::go;


pub(crate) fn go_to_coordinates(
    robot: &mut Jerry,
    map: &Vec<Vec<Option<Tile>>>,
    world: &mut World,
    adjacent: bool,
    destination: (usize, usize),
    n: usize,
) -> Result<Path, String> {
    match get_path_to_coordinates(world, robot,  map,  adjacent, destination, n) {
        Err(e) => Err(e),
        Ok(path) => {
            if !robot.get_energy().has_enough_energy(path.cost) {
                return Err(String::from("Not enough energy!"));
            }

            for action in path.actions.iter() {

                match action {
                    Action::Go(d) => {
                        if let Err(_) = go(robot, world, d.clone()) {
                            return Err(String::from("Error while calling go interface!"));
                        }
                    }
                    Action::Teleport((row, col)) => {
                        if let Err(_) = teleport(robot, world, (*row, *col)) {
                            return Err(String::from(
                                "Error while calling teleport interface!",
                            ));
                        }
                    }
                }
            }

            Ok(path)
        }
    }
}
pub(crate) fn get_path_to_coordinates(
    world: &World,
    robot: &mut Jerry,
    map: &Vec<Vec<Option<Tile>>>,
    adjacent: bool,
    destination: (usize, usize),
    n: usize,
    ) -> Result<Path, String> {
    match robot_map_slice_n(robot, map, n) {
        None => Err(String::from("Map not visible!")),
        Some(map) => {
            let source = (
                robot.get_coordinate().get_row(),
                robot.get_coordinate().get_col(),
            );

            let mut targets = HashSet::new();

            if adjacent {
                targets.extend(get_adjacent_tiles(&map, destination));
            } else {
                targets.insert(destination);
            }
            dijkstra(robot, world, &map, source, targets, None)
        }
    }
}
#[derive(Debug, Clone)]
pub(crate) enum Action {
    Go(Direction),
    Teleport((usize, usize)),
}

#[derive(Debug, Default, Clone)]
pub(crate) struct Path {
    pub(crate) source: (usize, usize),
    pub(crate) destination: (usize, usize),
    pub(crate) actions: VecDeque<Action>,
    pub(crate) cost: usize,
}

impl Path {
    pub(crate) fn new(source: (usize, usize), destination: (usize, usize), cost: usize) -> Path {
        Path {
            source,
            destination,
            actions: VecDeque::new(),
            cost,
        }
    }
}

#[derive(Eq)]
struct State {
    node: (usize, usize),
    distance: usize,
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        other.distance.cmp(&self.distance)
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node && self.distance == other.distance
    }
}

pub(crate) fn dijkstra(
    jerry: &Jerry,
    world: &World,
    map: &Vec<Vec<Option<Tile>>>,
    source: (usize, usize),
    mut targets: HashSet<(usize, usize)>,
    road: Option<Option<&HashSet<ChartedCoordinate>>>
    //None -> normal dijkstra
    //Some(None) -> dijkstra with global road
    //Some(Some(HashSet<ChartedCoordinate>) -> dijkstra with local road
) -> Result<Path, String> {
    if targets.is_empty() && road.is_none() {
        return Err(String::from("Path not found!"));
    }
    if let Some(optional_road) = road {
        match optional_road{
            None => {
                for road_tile in jerry.road_tiles.iter(){
                    targets.insert((road_tile.get_row(), road_tile.get_col()));
                }
            }
            Some(local_road) => {
                for road_tile in local_road.iter(){
                    targets.insert((road_tile.get_row(), road_tile.get_col()));
                }
            }
        }
    }
    let (source_row, source_col) = (source.0, source.1);
    let (mut start_row,mut end_row) = (source_row.saturating_sub(map.len() / 2), 
    source_row.saturating_sub(map.len() / 2) + map.len() - 1);
    let (mut start_col,mut end_col) = (source_col.saturating_sub(map[0].len() / 2), 
    source_col.saturating_sub(map[0].len() / 2) + map[0].len() - 1);

    //case when we use the whole map for the rroad planning
    if map.len() == jerry.world_dim{
        (start_row, end_row) = (0, map.len() - 1);
        (start_col, end_col) = (0, map[0].len() - 1);
    }

    //row - start_row and col - start_col are the coordinates of the map slice
    //row and col are the actual coordinates of the map
    let mut paths = Vec::new();
    let mut teleports = Vec::new();

    for row in start_row..=end_row {
        paths.push(Vec::new());
        for col in start_col..=end_col{
            paths[row - start_row].push(Path::new(source, (row, col), usize::MAX));

            if let Some(tile) = map[row - start_row][col - start_col].as_ref() {
                if tile.tile_type == TileType::Teleport(true) {
                    teleports.push((row, col));
                }
            }
        }
    }

    paths[source_row - start_row][source_col - start_col].cost = 0;

    let mut heap = BinaryHeap::new();
    heap.push(State {
        node: source,
        distance: 0,
    });

    while !heap.is_empty() {
        let (row, col) = heap.peek().unwrap().node;
        let distance = heap.pop().unwrap().distance;
        let len = map[0].len();
        if col - start_col + 1 < len && map[row - start_row][col - start_col + 1].is_some() {
            if let Some(_) = road{
                if let Ok(cost) = calculate_go_cost_for_road(jerry, world, map, (row, col), Direction::Right) {
                    if distance + cost < paths[row - start_row][col - start_col + 1].cost {
                        paths[row - start_row][col - start_col + 1].cost = distance + cost;
                        paths[row - start_row][col - start_col + 1].actions = paths[row - start_row][col - start_col].actions.clone();
                        paths[row - start_row][col - start_col + 1]
                            .actions
                            .push_back(Action::Go(Direction::Right));
                        heap.push(State {
                            node: (row, col + 1),
                            distance: distance + cost,
                        });
                    }
                }
            }
            else {
                if let Ok(cost) = calculate_go_cost(jerry, world, map, (row, col), Direction::Right) {
                    if distance + cost < paths[row - start_row][col - start_col + 1].cost {
                        paths[row - start_row][col - start_col + 1].cost = distance + cost;
                        paths[row - start_row][col - start_col + 1].actions = paths[row - start_row][col - start_col].actions.clone();
                        paths[row - start_row][col - start_col + 1]
                            .actions
                            .push_back(Action::Go(Direction::Right));
                        heap.push(State {
                            node: (row, col + 1),
                            distance: distance + cost,
                        });
                    }
                }
            }
        }
        let len = map.len();
        if row - start_row + 1 < len && map[row - start_row + 1][col - start_col].is_some() {
            if let Some(_) = road{
                if let Ok(cost) = calculate_go_cost_for_road(jerry, world, map, (row, col), Direction::Down) {
                    if distance + cost < paths[row - start_row + 1][col - start_col].cost {
                        paths[row - start_row + 1][col - start_col].cost = distance + cost;
                        paths[row - start_row + 1][col - start_col].actions = paths[row - start_row][col - start_col].actions.clone();
                        paths[row - start_row + 1][col - start_col]
                            .actions
                            .push_back(Action::Go(Direction::Down));
                        heap.push(State {
                            node: (row + 1, col),
                            distance: distance + cost,
                        });
                    }
                }
            }
            else{
                if let Ok(cost) = calculate_go_cost(jerry, world, map, (row, col), Direction::Down) {
                    if distance + cost < paths[row - start_row + 1][col - start_col].cost {
                        paths[row - start_row + 1][col - start_col].cost = distance + cost;
                        paths[row - start_row + 1][col - start_col].actions = paths[row - start_row][col - start_col].actions.clone();
                        paths[row - start_row + 1][col - start_col]
                            .actions
                            .push_back(Action::Go(Direction::Down));
                        heap.push(State {
                            node: (row + 1, col),
                            distance: distance + cost,
                        });
                    }
                }
            }
        }

        if col - start_col > 0 && map[row - start_row][col - start_col - 1].is_some() {
            if let Some(_) = road{
                if let Ok(cost) = calculate_go_cost_for_road(jerry, world, map, (row, col), Direction::Left) {
                    if distance + cost < paths[row - start_row][col - start_col - 1].cost {
                        paths[row - start_row][col - start_col - 1].cost = distance + cost;
                        paths[row - start_row][col - start_col - 1].actions = paths[row - start_row][col - start_col].actions.clone();
                        paths[row - start_row][col - start_col - 1]
                            .actions
                            .push_back(Action::Go(Direction::Left));
                        heap.push(State {
                            node: (row, col - 1),
                            distance: distance + cost,
                        });
                    }
                }
            }
            else{
                if let Ok(cost) = calculate_go_cost(jerry, world, map, (row, col), Direction::Left) {
                    if distance + cost < paths[row - start_row][col - start_col - 1].cost {
                        paths[row - start_row][col - start_col - 1].cost = distance + cost;
                        paths[row - start_row][col - start_col - 1].actions = paths[row - start_row][col - start_col].actions.clone();
                        paths[row - start_row][col - start_col - 1]
                            .actions
                            .push_back(Action::Go(Direction::Left));
                        heap.push(State {
                            node: (row, col - 1),
                            distance: distance + cost,
                        });
                    }
                }
            }
        }

        if row - start_row > 0 && map[row - start_row - 1][col - start_col].is_some() {
            if let Some(_) = road{
                if let Ok(cost) = calculate_go_cost_for_road(jerry, world, map, (row, col), Direction::Up) {
                    if distance + cost < paths[row - start_row - 1][col - start_col].cost {
                        paths[row - start_row - 1][col - start_col].cost = distance + cost;
                        paths[row - start_row - 1][col - start_col].actions = paths[row - start_row][col - start_col].actions.clone();
                        paths[row - start_row - 1][col - start_col]
                            .actions
                            .push_back(Action::Go(Direction::Up));
                        heap.push(State {
                            node: (row - 1, col),
                            distance: distance + cost,
                        });
                    }
                }
            }
            else{
                if let Ok(cost) = calculate_go_cost(jerry, world, map, (row, col), Direction::Up) {
                    if distance + cost < paths[row - start_row - 1][col - start_col].cost {
                        paths[row - start_row - 1][col - start_col].cost = distance + cost;
                        paths[row - start_row - 1][col - start_col].actions = paths[row - start_row][col - start_col].actions.clone();
                        paths[row - start_row - 1][col - start_col]
                            .actions
                            .push_back(Action::Go(Direction::Up));
                        heap.push(State {
                            node: (row - 1, col),
                            distance: distance + cost,
                        });
                    }
                }
            }
        }

        if let Some(tile) = map[row - start_row][col - start_col].as_ref() {
            if tile.tile_type == TileType::Teleport(true) {
                for (teleport_row, teleport_col) in teleports.iter() {
                    if let Ok(cost) =
                        calculate_teleport_cost(jerry, map, (*teleport_row, *teleport_col))
                    {
                        if distance + cost < paths[*teleport_row - start_row][*teleport_col - start_col].cost {
                            paths[*teleport_row - start_row][*teleport_col - start_col].cost = distance + cost;
                            paths[*teleport_row - start_row][*teleport_col - start_col].actions =
                                paths[row - start_row][col - start_col].actions.clone();
                            paths[*teleport_row - start_row][*teleport_col - start_col]
                                .actions
                                .push_back(Action::Teleport((*teleport_row, *teleport_col)));
                            heap.push(State {
                                node: (*teleport_row, *teleport_col),
                                distance: distance + cost,
                            });
                        }
                    }
                }
            }
        }
    }

    let mut ret = Path::default();
    ret.cost = usize::MAX;
    for (target_row, target_col) in targets {
        if target_row < start_row || target_row > end_row || target_col < start_col || target_col > end_col {
            return Err(String::from("Path not found!"));
        }
        if paths[target_row - start_row][target_col - start_col].cost < ret.cost {
            ret = paths[target_row - start_row][target_col - start_col].clone();
        }
    }

    if ret.cost == usize::MAX {
        Err(String::from("Path not found!"))
    } else {
        Ok(ret)
    }
}

fn get_coords_row_col(source: (usize, usize), direction: Direction) -> (usize, usize) {
    let (row, col) = source;

    match direction {
        Direction::Up => (row - 1, col),
        Direction::Down => (row + 1, col),
        Direction::Left => (row, col - 1),
        Direction::Right => (row, col + 1),
    }
}

pub(crate) fn get_adjacent_tiles(
    map: &Vec<Vec<Option<Tile>>>,
    tile: (usize, usize),
) -> Vec<(usize, usize)> {
    let mut ret = Vec::new();

    let (row, col) = tile;
    let size = map.len();

    if col + 1 < size && map[row][col + 1].is_some() {
        ret.push((row, col + 1));
    }

    if row + 1 < size && map[row + 1][col].is_some() {
        ret.push((row + 1, col));
    }

    if col > 0 && map[row][col - 1].is_some() {
        ret.push((row, col - 1));
    }

    if row > 0 && map[row - 1][col].is_some() {
        ret.push((row - 1, col));
    }

    ret
}


pub(crate) fn calculate_go_cost(
    _robot: &impl Runnable,
    world: &World,
    map: &Vec<Vec<Option<Tile>>>,
    source: (usize, usize),
    direction: Direction,
) -> Result<usize, String> {
    let (source_row, source_col) = source;
    let start_row = source_row.saturating_sub(map.len() / 2);
    let start_col = source_col.saturating_sub(map[0].len() / 2);
    let (destination_row, destination_col) = get_coords_row_col(source, direction);

    if map[source_row - start_row][source_col - start_col].is_none() {
        return Err(String::from("Source is None!"));
    }
    if map[destination_row - start_row][destination_col - start_col].is_none() {
        return Err(String::from("Destination is None!"));
    }

    let source = map[source_row - start_row][source_col - start_col].clone().unwrap();
    let destination = map[destination_row - start_row][destination_col - start_col].clone().unwrap();
    if destination.tile_type.properties().walk() == false {
        return Err(String::from("Go not allowed!"));
    }

    let mut base_cost = destination.tile_type.properties().cost();
    let mut elevation_cost = 0;

    base_cost =
        calculate_cost_go_with_environment(base_cost, look_at_sky(world), destination.tile_type);

    if destination.elevation > source.elevation {
        elevation_cost = (destination.elevation - source.elevation).pow(2);
    }

    Ok(base_cost + elevation_cost)
}
pub(crate) fn calculate_go_cost_for_road(
    jerry: &Jerry,
    _world: &World,
    map: &Vec<Vec<Option<Tile>>>,
    source: (usize, usize),
    direction: Direction,
) -> Result<usize, String>{
    let (source_row, source_col) = source;
    let mut start_row = source_row.saturating_sub(map.len() / 2);
    let mut start_col = source_col.saturating_sub(map[0].len() / 2);

    if map.len() == jerry.world_dim{
        start_row = 0;
        start_col = 0;
    }
    let (destination_row, destination_col) = get_coords_row_col(source, direction);

    if map[source_row - start_row][source_col - start_col].is_none() {
        return Err(String::from("Source is None!"));
    }
    if map[destination_row - start_row][destination_col - start_col].is_none() {
        return Err(String::from("Destination is None!"));
    }

    let source = map[source_row - start_row][source_col - start_col].clone().unwrap();
    let destination = map[destination_row - start_row][destination_col - start_col].clone().unwrap();
    if destination.tile_type.properties().walk() == false {
        return Err(String::from("Go not allowed!"));
    }

    let base_cost = match destination.tile_type {
        TileType::Street => 0,
        TileType::Hill => 1,
        TileType::Mountain => 0,
        TileType::Sand => 1,
        TileType::ShallowWater => 2,
        TileType::Grass => 1,
        TileType::DeepWater => 3,
        TileType::Lava => 4,
        _ => 1000
    };
    let content_cost = match destination.content.to_default(){
        Content::Crate(_) => 1000,
        Content::None => 0,
        _ => 2,
    };
    let mut elevation_cost = 0;

    if destination.elevation > source.elevation {
        elevation_cost = (destination.elevation - source.elevation).pow(2);
    }
    Ok(base_cost + elevation_cost + content_cost)
}





pub(crate) fn calculate_teleport_cost(
    robot: &impl Runnable,
    map: &Vec<Vec<Option<Tile>>>,
    destination: (usize, usize),
) -> Result<usize, String> {
    let (source_row, source_col) = (
        robot.get_coordinate().get_row(),
        robot.get_coordinate().get_col(),
    );
    let start_row = source_row.saturating_sub(map.len() / 2);
    let start_col = source_col.saturating_sub(map[0].len() / 2);

    let (destination_row, destination_col) = (destination.0, destination.1);

    let size = map.len();

    if source_row >= size || source_col >= size {
        return Err(String::from("Source out of bounds!"));
    }

    if destination_row >= size || destination_col >= size {
        return Err(String::from("Destination out of bounds!"));
    }

    match &map[source_row - start_row][source_col - start_col] {
        None => {
            return Err(String::from("Source is None!"));
        }
        Some(tile) => {
            if tile.tile_type != TileType::Teleport(true) {
                return Err(String::from("Source is not a teleport!"));
            }
        }
    }

    match &map[destination_row - start_row][destination_col - start_col] {
        None => {
            return Err(String::from("Destination is None!"));
        }
        Some(tile) => {
            if tile.tile_type != TileType::Teleport(true) {
                return Err(String::from("Destination is not a teleport!"));
            }
        }
    }

    Ok(30)
}
pub fn path_to_coordinates(path: &Path) -> Vec<ChartedCoordinate>{
    let mut ret = Vec::new();
    let mut row = path.source.0;
    let mut col = path.source.1;
    for action in path.actions.iter(){
        match action{
            Action::Go(Direction::Up) => {
                row -= 1;
            }
            Action::Go(Direction::Down) => {
                row += 1;
            }
            Action::Go(Direction::Left) => {
                col -= 1;
            }
            Action::Go(Direction::Right) => {
                col += 1;
            }
            Action::Teleport((r, c)) => {
                row = *r;
                col = *c;
            }
        }
        ret.push(ChartedCoordinate::new(row, col));
    }
    ret
}