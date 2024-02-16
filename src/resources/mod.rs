use std::cell::RefCell;
use std::collections::HashSet;
use std::f32::consts::E;
use std::rc::Rc;
use crate::interface::Jerry;
use charting_tools::charted_coordinate::ChartedCoordinate;
use robotics_lib::interface::{destroy, put, where_am_i, Direction};
use robotics_lib::runner::Runnable;
use robotics_lib::world::World;
use robotics_lib::world::tile::{Content, Tile, TileType};
use rust_and_furious_dynamo::dynamo;
use rust_eze_tomtom::plain::{self, PlainContent, PlainTileType};
use rust_eze_tomtom::TomTom;
use crate::resources::ResourceCollectorError::*;
//collects a certain resource until the backpack is full
pub fn get_content(jerry: &mut Jerry, world: &mut World, content: Content,
    planned_road: Option<&HashSet<ChartedCoordinate>>, desired_amount: usize) -> Result<usize, ResourceCollectorError> {
   println!("Getting content");
   if desired_amount > jerry.get_backpack().get_size(){
       panic!("You want too fucking much");
   }
   //if there is not enough space in the backpack
   let occupied_space = jerry.get_backpack().get_contents().values().sum::<usize>();
   if desired_amount > jerry.get_backpack().get_size() - occupied_space{
       println!("desired amount is {}, and occupied space is {}", desired_amount, occupied_space);
       let space_needed = desired_amount - occupied_space;
       let _ = empty_the_backpack(jerry, world, planned_road, space_needed)?;
       println!("{:?}", jerry.get_backpack().get_contents().values().sum::<usize>());
   }
   let amount_collected = go_get_content(jerry, world, content)?;
   Ok(amount_collected)
}

//disposes the content till the moment there's enough space in the backpack
pub fn empty_the_backpack(jerry: &mut Jerry, world: &mut World,
   planned_road: Option<&HashSet<ChartedCoordinate>>, space_needed: usize) -> Result<(), ResourceCollectorError> {
       println!("Emptying backpack");
       let mut contents: HashSet<Content> = HashSet::new();
       for (content, amount) in jerry.get_backpack_mut().get_contents().iter()
       {
           if *amount > 0{
               contents.insert(content.clone());
           }
       }
       let mut disposed_content = 0;
       //indicator that we cannot dispose anything
       let mut skipped_all = true;
       while disposed_content < space_needed{
           skipped_all = true;
           for content in contents.iter(){
               if content.to_default() == Content::Rock(0){
                   continue;
               }
               match go_dispose_content(jerry, world, content.to_default().clone(), planned_road){
                   Ok(amount) => {disposed_content += amount; skipped_all = false},
                   Err(NotEnoughEnergy) => return Err(NotEnoughEnergy),
                   Err(NoWayToDispose) | Err(NoContentToDispose) | Err(PathNotFound) => continue,
                   _ => panic!("Unexpected error"),
               }
           }
           if skipped_all{
               break;
           }
       }
       Ok(())
}

//goes to a tile with a certain content and collects it (not on the tile directly, but on adjacent tiles)
//returns the amount of collected content
pub fn go_get_content(jerry: &mut Jerry, world: &mut World, content: Content) -> Result<usize, ResourceCollectorError> {
   println!("Going to get content");
   //this is for converting the content to a plain content
   let plain_content = match_to_plain_content(content.clone());
   //check if can reach the resource
   //if not it's a pizdec and we panic (should not happen if i check the amount of rocks on the map) - return a special error which
   //will be handled by the caller
   if let Err(error) = 
       TomTom::go_to_tile(jerry, world, true, None, Some(plain_content)){
           if error == "Path not found!"{
               println!("No path found to the resource {:?}", content);
               return Err(PathNotFound);
           }
           else if error == "Not enough energy!"{
               return Err(NotEnoughEnergy);
           }
           else{
               panic!("Unexpected error: {}", error);
           }
       }
   //else we have arrived to a tile adjacent to a given one
   //we can destroy the resource
   let robot_view = where_am_i(jerry, world).0;
   return get_content_around(jerry, &robot_view, world, content);
   //it's somewhere around
   //possible error -> not enough energy
   //in this case we return an error to a calling function
   //which will decide whether to use the cheat code or not
   //then it gonna call this function again
   
}
//dispose a certain resource until it's all gone from the backpack
//navigates to a closest suitable tile for disposing the resource (again not on the tile directly, but on adjacent tiles)
//and tries to put it there
pub fn go_dispose_content(jerry: &mut Jerry, world: &mut World, content: Content, planned_road: Option<&HashSet<ChartedCoordinate>>) -> Result<usize, ResourceCollectorError> {
   println!("Going to dispose content");
   //get the tile types and contents that can hold the content
   let (tile_types, contents) = ways_to_dispose(content.clone());
   for tile_type in &tile_types{
       let plain_tile_type = match_to_plain_tile_type(tile_type.clone());
       //try to go to a tile of a certain type
       if let Err(error) = TomTom::go_to_tile
       (jerry, world, true, Some(plain_tile_type), Some(PlainContent::None)){
           if error == "Path not found!"{
               println!("No path found to the resource");
               return Err(PathNotFound);
           }
           else if error == "Not enough energy!"{
               return Err(NotEnoughEnergy);
           }
           else{
               panic!("Unexpected error: {}", error);
           }
       }
       //if we have arrived to a tile of a certain type
       //we can dispose the resource
       let robot_view = where_am_i(jerry, world).0;
       let res =  dispose_around(jerry, world, &robot_view, planned_road, content.clone());
       println!("{:?}", res);
       return res;
   }
   for content in &contents{
       let plain_content = match_to_plain_content(content.clone());
       if let Err(error) = TomTom::go_to_tile(jerry, world, true, None, Some(plain_content)){
           if error == "Path not found!"{
               println!("No path found to the resource");
               return Err(PathNotFound);
           }
           else if error == "Not enough energy!"{
               return Err(NotEnoughEnergy);
           }
           else{
               panic!("Unexpected error: {}", error);
           }
       }
       let robot_view = where_am_i(jerry, world).0;
       let res =  dispose_around(jerry, world, &robot_view, planned_road, content.clone());
       println!("{:?}", res);
       return res;
   }
   //if we didn't find a suitable tile to dispose the content
   Err(NoWayToDispose)
}

pub fn get_content_around(jerry: &mut Jerry, view: &Vec<Vec<Option<Tile>>>, world: &mut World, content: Content) -> Result<usize, ResourceCollectorError>{
   println!("Trying to get content around");
   if jerry.get_backpack().get_size() == jerry.get_backpack().get_contents().values().sum::<usize>(){
       return Err(BackPackIsFull);
   }
   let mut total = 0;
   if let Some(tile) = &view[0][1]{
       if content.to_default() == tile.content.to_default(){
           if let Ok(amount) = destroy(jerry, world, Direction::Up){
               total += amount;
           }
           else{
               return Err(NotEnoughEnergy);
           }
       }
   }
   if let Some(tile) = &view[1][0]{
       if content.to_default() == tile.content.to_default(){
           if let Ok(amount) = destroy(jerry, world, Direction::Left){
               total += amount;
           }
           else{
               return Err(NotEnoughEnergy);
           }
       }
   }
   if let Some(tile) = &view[1][2]{
       if content.to_default() == tile.content.to_default(){
           if let Ok(amount) = destroy(jerry, world, Direction::Right){
               total += amount;
           }
           else{
               return Err(NotEnoughEnergy);
           }
       }
   }
   if let Some(tile) = &view[2][1]{
       if content.to_default() == tile.content.to_default(){
           if let Ok(amount) = destroy(jerry, world, Direction::Down){
               total += amount;
           }
           else{
               return Err(NotEnoughEnergy);
           }
       }
   }
   if total == 0{
       return Err(NoContentFound);
   }
   Ok(total)
}

/*
   The function should dispose the selected type of content within the robot's view
   verifying if necessary that the tile it throws stuff on is not in the planned road
*/
pub(crate) fn dispose_around(jerry: &mut Jerry, world: &mut World, robot_view: &Vec<Vec<Option<Tile>>>, 
   planned_road: Option<&HashSet<ChartedCoordinate>>, content: Content) -> Result<usize, ResourceCollectorError>{
       println!("Trying to dispose content around");
   
   //total amount of disposed content
   let mut total_disposed = 0;

   //check if the robot has the content to dispose
   if jerry.get_backpack().get_contents().get(&content.to_default()).is_none(){
       return Err(NoContentToDispose);
   }
   //amount of content in a backpack
   let mut amount_to_dispose = *jerry.get_backpack().get_contents().get(&content.to_default()).unwrap();
   if amount_to_dispose == 0{
       return Err(NoContentToDispose);
   }

   //get the tile types and contents that can hold the content
   let (tile_types, contents) = ways_to_dispose(content.clone());
   if let Some(tile) = &robot_view[0][1]{
       
       //check if the tile is not in the planned road
       let actual_tile_coordinate = (jerry.get_coordinate().get_row() - 1, jerry.get_coordinate().get_col());
       let mut on_the_road = false;
       if let Some(planned_road) = planned_road{
           if planned_road.contains(&ChartedCoordinate::from(actual_tile_coordinate)){
               on_the_road = true;
           }
       }
       //2 suitable options:
       //1. the tile is empty and can hold the content
       //2. the tile has a content that can hold the content (crate, bin, bank, market, ...)
       if ((tile_types.contains(&tile.tile_type) && tile.content.to_default() == Content::None) || contents.contains(&tile.content))
           && !on_the_road{
           
           //amount we try to dispose
           let mut tentative = amount_to_dispose;
           while tentative > 0{
               if let Ok(amt) = put(jerry, world, content.clone(), tentative, Direction::Up){
                   total_disposed += amt;
                   tentative -= amt;
               }
               else{
                   //try with a smaller amount
                   tentative -= 1;
               }
           }
           //if we didn't dispose anything, we return an error
           if tentative == 0 && total_disposed == 0{
               return Err(NotEnoughEnergy);
           }
           //if we disposed everything, we return the total amount
           if total_disposed == amount_to_dispose {
               return Ok(total_disposed);
           }
           //if we disposed something, but not everything, we update the amount to dispose and try other tiles
           else if total_disposed > 0 && total_disposed < amount_to_dispose{
               amount_to_dispose -= total_disposed;
           }
           
       }
   }
   if let Some(tile) = &robot_view[1][0]{
       //check if the tile is not in the planned road
       let actual_tile_coordinate = (jerry.get_coordinate().get_row() - 1, jerry.get_coordinate().get_col());
       let mut on_the_road = false;
       if let Some(planned_road) = planned_road{
           if planned_road.contains(&ChartedCoordinate::from(actual_tile_coordinate)){
               on_the_road = true;
           }
       }
       //2 suitable options:
       //1. the tile is empty and can hold the content
       //2. the tile has a content that can hold the content (crate, bin, bank, market, ...)
       if ((tile_types.contains(&tile.tile_type) && tile.content.to_default() == Content::None) || contents.contains(&tile.content))
           && !on_the_road{
           
           //amount we try to dispose
           let mut tentative = amount_to_dispose;
           while tentative > 0{
               if let Ok(amt) = put(jerry, world, content.clone(), tentative, Direction::Left){
                   total_disposed += amt;
                   tentative -= amt;
               }
               else{
                   //try with a smaller amount
                   tentative -= 1;
               }
           }
           //if we didn't dispose anything, we return an error
           if tentative == 0 && total_disposed == 0{
               return Err(NotEnoughEnergy);
           }
           //if we disposed everything, we return the total amount
           if total_disposed == amount_to_dispose {
               return Ok(total_disposed);
           }
           //if we disposed something, but not everything, we update the amount to dispose and try other tiles
           else if total_disposed > 0 && total_disposed < amount_to_dispose{
               amount_to_dispose -= total_disposed;
           }
           
       }
   }
   if let Some(tile) = &robot_view[1][2]{
       //check if the tile is not in the planned road
       let actual_tile_coordinate = (jerry.get_coordinate().get_row() - 1, jerry.get_coordinate().get_col());
       let mut on_the_road = false;
       if let Some(planned_road) = planned_road{
           if planned_road.contains(&ChartedCoordinate::from(actual_tile_coordinate)){
               on_the_road = true;
           }
       }
       //2 suitable options:
       //1. the tile is empty and can hold the content
       //2. the tile has a content that can hold the content (crate, bin, bank, market, ...)
       if ((tile_types.contains(&tile.tile_type) && tile.content.to_default() == Content::None) || contents.contains(&tile.content))
           && !on_the_road{
           
           //amount we try to dispose
           let mut tentative = amount_to_dispose;
           while tentative > 0{
               if let Ok(amt) = put(jerry, world, content.clone(), tentative, Direction::Right){
                   total_disposed += amt;
                   tentative -= amt;
               }
               else{
                   //try with a smaller amount
                   tentative -= 1;
               }
           }
           //if we didn't dispose anything, we return an error
           if tentative == 0 && total_disposed == 0{
               return Err(NotEnoughEnergy);
           }
           //if we disposed everything, we return the total amount
           if total_disposed == amount_to_dispose {
               return Ok(total_disposed);
           }
           //if we disposed something, but not everything, we update the amount to dispose and try other tiles
           else if total_disposed > 0 && total_disposed < amount_to_dispose{
               amount_to_dispose -= total_disposed;
           }
           
       }
   }
   if let Some(tile) = &robot_view[2][1]{
       //check if the tile is not in the planned road
       let actual_tile_coordinate = (jerry.get_coordinate().get_row() - 1, jerry.get_coordinate().get_col());
       let mut on_the_road = false;
       if let Some(planned_road) = planned_road{
           if planned_road.contains(&ChartedCoordinate::from(actual_tile_coordinate)){
               on_the_road = true;
           }
       }
       //2 suitable options:
       //1. the tile is empty and can hold the content
       //2. the tile has a content that can hold the content (crate, bin, bank, market, ...)
       if ((tile_types.contains(&tile.tile_type) && tile.content.to_default() == Content::None) || contents.contains(&tile.content))
           && !on_the_road{
           
           //amount we try to dispose
           let mut tentative = amount_to_dispose;
           while tentative > 0{
               if let Ok(amt) = put(jerry, world, content.clone(), tentative, Direction::Down){
                   total_disposed += amt;
                   tentative -= amt;
               }
               else{
                   //try with a smaller amount
                   tentative -= 1;
               }
           }
           //if we didn't dispose anything, we return an error
           if tentative == 0 && total_disposed == 0{
               return Err(NotEnoughEnergy);
           }
           //if we disposed everything, we return the total amount
           if total_disposed == amount_to_dispose {
               return Ok(total_disposed);
           }
           //if we disposed something, but not everything, we update the amount to dispose and try other tiles
           else if total_disposed > 0 && total_disposed < amount_to_dispose{
               amount_to_dispose -= total_disposed;
           }
           
       }
   }
   if total_disposed == 0{
       return Err(NoWayToDispose);
   }
   Ok(total_disposed)
}
/*
   Should work like:
   Content = Tree. Way to dispose - empty suitablle tile or a crate
*/
fn ways_to_dispose(content: Content) -> (HashSet<TileType>, HashSet<Content>){
   let mut tile_types = HashSet::new();
   let mut contents = HashSet::new();
   let possible_tile_types = vec![
       TileType::Grass,
       TileType::Sand,
       TileType::Snow,
       TileType::ShallowWater,
       TileType::DeepWater,
       TileType::Street,
       TileType::Hill,
       TileType::Mountain,
       TileType::Lava,
       TileType::Teleport(false),
       TileType::Teleport(true),
       TileType::Wall,
   ];
   let possible_contents = vec![
       Content::Rock(0),
       Content::Tree(0),
       Content::Bush(0),
       Content::Garbage(0),
       Content::JollyBlock(0),
       Content::Coin(0),
       Content::Fire,
       Content::Water(0),
       Content::Fish(0),
       Content::None,
       Content::Market(0),
       Content::Bank(0..0),
       Content::Crate(0..0),
       Content::Bin(0..0),
       Content::Building,
       Content::Scarecrow,
   ];
   for tile_type in possible_tile_types{
       if tile_type.properties().can_hold(&content.to_default()){
           tile_types.insert(tile_type);
       }
   }
   for content_type in possible_contents{
       if let Some(acceptable) = content_type.properties().disposable(){
           if *acceptable == content.to_default(){
               contents.insert(content_type);
           }
       }
   }
   (tile_types, contents)
}
fn match_to_plain_content(content: Content) -> PlainContent{
   match content{
       Content::Rock(_) => PlainContent::Rock,
       Content::Tree(_) => PlainContent::Tree,
       Content::Bush(_) => PlainContent::Bush,
       Content::Coin(_) => PlainContent::Coin,
       Content::Fish(_) => PlainContent::Fish,
       Content::Water(_) => PlainContent::Water,
       Content::JollyBlock(_) => PlainContent::JollyBlock,
       Content::Fire => PlainContent::Fire,
       Content::Garbage(_) => PlainContent::Garbage,
       _ => panic!("This content is not collectable")
   }
}
fn match_to_plain_tile_type(tile_type: TileType) -> PlainTileType{
   match tile_type{
       TileType::Grass => PlainTileType::Grass,
       TileType::Sand => PlainTileType::Sand,
       TileType::Snow => PlainTileType::Snow,
       TileType::ShallowWater => PlainTileType::ShallowWater,
       TileType::DeepWater => PlainTileType::DeepWater,
       TileType::Street => PlainTileType::Street,
       TileType::Hill => PlainTileType::Hill,
       TileType::Mountain => PlainTileType::Mountain,
       TileType::Lava => PlainTileType::Lava,
       TileType::Teleport(_) => PlainTileType::Teleport,
       TileType::Wall => PlainTileType::Wall,
   }
}
#[derive(Debug, Copy, Clone)]
pub enum ResourceCollectorError{
   BackPackIsFull,
   NotEnoughEnergy,
   PathNotFound,
   NoContentFound,
   NoWayToDispose,
   NoContentToDispose,
}