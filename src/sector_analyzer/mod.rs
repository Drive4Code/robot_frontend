use std::collections::{HashMap, HashSet};

use robotics_lib::interface::robot_map;
use robotics_lib::world::tile::TileType::*;
use robotics_lib::world::{World};
use robotics_lib::world::tile::{Content, Tile};

use crate::utils::{get_tl_and_br_from_spatial_index, robot_map_slice};
use crate::utils::{ActiveRegion, Mission};
use crate::utils::MissionStatus::Active;
use crate::morans_i::{morans_i};

use self::dbscan::{Classification, Model};
use self::dbscan::map_into_db_input;



pub fn new_sector_analyzer(spatial_index: usize, world_dim: usize) -> Mission {
    let (tl, br) = get_tl_and_br_from_spatial_index(spatial_index, world_dim);
    Mission {
        name: "Sector Analyzer".to_string(),
        status: Active,
        additional_data: Some(Box::new(ActiveRegion {
            top_left: tl,
            bottom_right: br,
            spatial_index,
        })),
    }
}
/*Goals: to determine if the sector is random or not
    to get the amount of the resources in the sector
    to calculate the available amount of rocks in the sector
    define the nodes (where to put pins)
        resource clusters
        waypoints to other sectors
        markets and banks

*/
pub fn analyzer_execute(world: &mut World, tl: (usize, usize), br: (usize, usize)) -> SectorData{
    let robot_map = robot_map(world).unwrap();
    let sector_map = robot_map_slice(&robot_map,tl, br).unwrap();
    let sector_resources = sector_collectable(&sector_map);
    let mountain_tiles = count_mountain_tiles(&sector_map);
    let is_random = is_content_random(&sector_map);
    let mut zone = find_largest_connected_subset(&sector_map);
    //turn the relative coordinates into absolute
    for coord in zone.iter_mut(){
        coord.0 += tl.0;
        coord.1 += tl.1;
    }
    let mut nodes = vec![get_centroid(&zone)];
    println!("{:?}", get_centroid(&zone));
    if is_random{
        return SectorData {
            resources: sector_resources,
            mountain_tiles,
            is_random: true,
            nodes: nodes,
        };
    }
    let (eps, min_points) = (6.0, 80);
    let model = Model::new(eps, min_points);
    let model_inputs = map_into_db_input(&sector_map);
    let classification = model.run(&model_inputs);
    let mut clusters =  HashMap::new();
    for el in classification.iter(){
        if let Classification::Core((i, j), c) = el{
           if !clusters.contains_key(c){
                clusters.insert(c, Vec::new());
           }
           else{
               clusters.get_mut(c).unwrap().push((*i, *j));
           }
        }
    }
    println!("Clusters len: {}", clusters.len());
    for (c, cores) in clusters.iter(){
        let mut centroid = get_centroid(cores);
        //turn the relative coordinates into absolute
        centroid.0 += tl.0;
        centroid.1 += tl.1;
        println!("Centroid of the cluster is {:?}", centroid);
        nodes.push(centroid);
    }
    SectorData {
        resources: sector_resources,
        mountain_tiles,
        is_random: false,
        nodes,
    }


}
pub fn sector_collectable(sector: &Vec<Vec<Option<Tile>>>) -> HashMap<Content, usize>{
    let mut resources = HashMap::new();
    for row in sector.iter(){
        for tile in row.iter(){
            if let Some(tile) = tile{
                let content = &tile.content;
                if !content.properties().destroy(){
                    continue;
                }  
                let mut count = resources.entry(content.to_default().clone()).or_insert(0);
                match content.get_value(){
                    (Some(amount), None) => {
                        *count += amount;
                    }
                    (_, _) => {}
                }
            }
        }
    }
    resources
}

#[derive(Debug)]
pub struct SectorData{
    pub resources: HashMap<Content, usize>,
    pub mountain_tiles: usize,
    pub is_random: bool,
    pub nodes: Vec<(usize, usize)>,
}

pub fn is_content_random(sector: &Vec<Vec<Option<Tile>>>) -> bool{
    let m = morans_i(sector);
    if m < 0.1{
        return true;
    }
    false
}
pub fn find_largest_connected_subset(map: &Vec<Vec<Option<Tile>>>) -> Vec<(usize, usize)>{
    let mut visited = HashSet::new();
    let mut largest_subset = Vec::new();
    for i in 0..map.len(){
        for j in 0..map[i].len(){
            if visited.contains(&(i, j)){
                continue;
            }
            let mut subset = Vec::new();
            let mut stack = Vec::new();
            stack.push((i, j));
            while let Some((i, j)) = stack.pop(){
                if visited.contains(&(i, j)){
                    continue;
                }
                visited.insert((i, j));
                let tile = &map[i][j];
                if let Some(tile) = tile{
                    match tile.tile_type{
                        Grass | Hill | Street | Mountain => {
                            if tile.content == Content::None || tile.content.properties().destroy(){
                                subset.push((i, j));
                            }
                            for (i, j) in get_neighbours(i, j, map.len(), map[i].len()){
                                stack.push((i, j));
                            }
                        }
                        _ => {}
                    }
                }
            }
            if subset.len() > largest_subset.len(){
                largest_subset = subset;
            }
        }
    }
    largest_subset
}
fn get_neighbours(i: usize, j: usize, rows: usize, cols: usize) -> Vec<(usize, usize)>{
    let mut neighbours = Vec::new();
    if i > 0{
        neighbours.push((i - 1, j));
    }
    if i < rows - 1{
        neighbours.push((i + 1, j));
    }
    if j > 0{
        neighbours.push((i, j - 1));
    }
    if j < cols - 1{
        neighbours.push((i, j + 1));
    }
    neighbours
}
fn count_mountain_tiles(sector: &Vec<Vec<Option<Tile>>>) -> usize{
    let mut count = 0;
    for row in sector.iter(){
        for tile in row.iter(){
            if let Some(tile) = tile{
                if tile.tile_type == Mountain{
                    count += 1;
                }
            }
        }
    }
    count
}
fn get_centroid(cores: &Vec<(usize, usize)>) -> (usize, usize){
    let mut x = 0;
    let mut y = 0;
    for core in cores.iter(){
        x += core.0;
        y += core.1;
    }
    (x/cores.len(), y/cores.len())
}
mod dbscan;


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_sector_collectable() {
        let sector: Vec<Vec<Option<Tile>>> = vec![
            vec![
                Some(Tile{tile_type: Grass, content: Content::Rock(3), elevation: 0}),
                Some(Tile{tile_type: Grass, content: Content::Rock(3), elevation: 0}),
                Some(Tile{tile_type: Grass, content: Content::Rock(3), elevation: 0}),
            ],
            vec![
                Some(Tile{tile_type: Grass, content: Content::Coin(3), elevation: 0}),
                Some(Tile{tile_type: Grass, content: Content::Coin(3), elevation: 0}),
                Some(Tile{tile_type: Grass, content: Content::Coin(3), elevation: 0}),
            ],
            vec![
                Some(Tile{tile_type: Grass, content: Content::Bank(0..10), elevation: 0}),
                Some(Tile{tile_type: Grass, content: Content::Fire, elevation: 0}),
                Some(Tile{tile_type: Grass, content: Content::Crate(0..3), elevation: 0}),
            ],

        ];
        let expected: HashMap<Content, usize> = {
            let mut hm = HashMap::new();
            hm.insert(Content::Rock(0), 3*3);
            hm.insert(Content::Coin(0), 3*3);
            hm.insert(Content::Fire, 1);
            hm
        };
        assert_eq!(sector_collectable(&sector), expected);
    }
    #[test]
    fn test_find_largest_connected_subset(){
        let map: Vec<Vec<Option<Tile>>> = vec![
            vec![
                Some(Tile{tile_type: Grass, content: Content::None, elevation: 0}),
                Some(Tile{tile_type: Sand, content: Content::None, elevation: 0}),
                Some(Tile{tile_type: Sand, content: Content::None, elevation: 0}),
            ],
            vec![
                Some(Tile{tile_type: Hill, content: Content::Crate(0..5), elevation: 0}),
                Some(Tile{tile_type: Hill, content: Content::Tree(2), elevation: 0}),
                Some(Tile{tile_type: Sand, content: Content::None, elevation: 0}),
            ],
            vec![
                Some(Tile{tile_type: Sand, content: Content::None, elevation: 0}),
                Some(Tile{tile_type: Grass, content: Content::None, elevation: 0}),
                Some(Tile{tile_type: Sand, content: Content::None, elevation: 0}),
            ],
        ];
        let subset = find_largest_connected_subset(&map);
        println!("{:?}", subset);
        assert_eq!(subset.len(), 3);
    }
}