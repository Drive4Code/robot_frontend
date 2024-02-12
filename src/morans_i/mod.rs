use robotics_lib::world::tile::Tile;
use robotics_lib::world::tile::{Content, TileType};
use rand::Rng;

/*
    Moran's I is a measure of spatial autocorrelation
    The formula for Moran's I is:
    I = N/(W*denominator_sum)*enumerator_sum
    Where:
    N = the number of spatial units
    W = the sum of the weights of the spatial units
    enumerator_sum = the sum of the products of the differences between the values of the spatial units 
                    and the mean value of the spatial units
    denominator_sum = the sum of the squared differences between the values of the spatial units 
                    and the mean value of the spatial units

    Values for content types (only the ones that the robot can destroy) are as follows:
    None (and those that cannot be destroyed) -> random value between 1 and 9
    Rock -> 1
    Tree -> 2
    Water -> 3
    Coin -> 4
    Fire -> 5
    Garbage -> 6
    Fish -> 7
    Bush -> 8
    JollyBlock -> 9
 */
pub fn morans_i(sector: &Vec<Vec<Option<Tile>>>) -> f64{
    let n = (sector.len()*sector.len()) as f64;
    let w = get_w(sector.len()) as f64;
    let mean = 4.5;
    let mut enumerator_sum = 0.0;
    let mut denominator_sum = 0.0;
    for (i, row) in sector.iter().enumerate(){
        for (j, tile) in row.iter().enumerate(){

            let value = get_content_value_morans(tile);
            if i > 0{
                enumerator_sum += (value - mean)*(get_content_value_morans(&sector[i-1][j]) - mean);
            }
            if j > 0{
                enumerator_sum += (value - mean)*(get_content_value_morans(&sector[i][j-1]) - mean);
            }
            if i < sector.len() - 1{
                enumerator_sum += (value - mean)*(get_content_value_morans(&sector[i + 1][j]) - mean);
            }
            if j < sector[0].len() - 1{
                enumerator_sum += (value - mean)*(get_content_value_morans(&sector[i][j + 1]) - mean);
            }

            denominator_sum += (value - mean) * (value - mean);
        }
    }
    let i = n/(w*denominator_sum)*enumerator_sum;
    println!("{}", i);
    i
}
pub fn get_content_value_morans(tile: &Option<Tile>) -> f64{
    if tile.is_none(){
        return rand::thread_rng().gen_range(1..10) as f64;
        //return 0.;
    }
    let content = tile.as_ref().unwrap().content.to_default();
    match content{
        Content::Rock(_) => 1.,
        Content::Tree(_) => 2.,
        Content::Water(_) => 3.,
        Content::Coin(_) => 4.,
        Content::Fire => 5.,
        Content::Garbage(_) => 6.,
        Content::Fish(_) => 7.,
        Content::Bush(_) => 8.,
        Content::JollyBlock(_) => 9.,
        _ => rand::thread_rng().gen_range(1..10) as f64,
        //_ => 0.,
    }
}
pub fn get_w(n:usize) -> usize{
    if n == 1{
        return 0;
    }
    get_w(n - 1) + 8*(n - 1)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_w(){
        assert_eq!(get_w(1), 0);
        assert_eq!(get_w(2), 8);
        assert_eq!(get_w(3), 24);
        assert_eq!(get_w(4), 48);
    }
    #[test]
    fn test_morans_i(){
        let sector: Vec<Vec<Option<Tile>>> = vec![
            vec![
                Some(Tile{tile_type: TileType::Grass, content: Content::Coin(3), elevation: 0}),
                Some(Tile{tile_type: TileType::Grass, content: Content::Coin(3), elevation: 0}),
                Some(Tile{tile_type: TileType::Grass, content: Content::None, elevation: 0}),
                Some(Tile{tile_type: TileType::Grass, content: Content::None, elevation: 0}),
                Some(Tile{tile_type: TileType::Grass, content: Content::Water(1), elevation: 0}),
                Some(Tile{tile_type: TileType::Grass, content: Content::Water(1), elevation: 0}),
            ],
            vec![
                Some(Tile{tile_type: TileType::Grass, content: Content::Coin(3), elevation: 0}),
                Some(Tile{tile_type: TileType::Grass, content: Content::Coin(3), elevation: 0}),
                Some(Tile{tile_type: TileType::Grass, content: Content::None, elevation: 0}),
                Some(Tile{tile_type: TileType::Grass, content: Content::None, elevation: 0}),
                Some(Tile{tile_type: TileType::Grass, content: Content::Water(1), elevation: 0}),
                Some(Tile{tile_type: TileType::Grass, content: Content::Water(1), elevation: 0}),
            ],
            vec![
                Some(Tile{tile_type: TileType::Grass, content: Content::None, elevation: 0}),
                Some(Tile{tile_type: TileType::Grass, content: Content::None, elevation: 0}),
                Some(Tile{tile_type: TileType::Grass, content: Content::None, elevation: 0}),
                Some(Tile{tile_type: TileType::Grass, content: Content::None, elevation: 0}),
                Some(Tile{tile_type: TileType::Grass, content: Content::None, elevation: 0}),
                Some(Tile{tile_type: TileType::Grass, content: Content::None, elevation: 0}),
            ],
            vec![
                Some(Tile{tile_type: TileType::Grass, content: Content::None, elevation: 0}),
                Some(Tile{tile_type: TileType::Grass, content: Content::None, elevation: 0}),
                Some(Tile{tile_type: TileType::Grass, content: Content::None, elevation: 0}),
                Some(Tile{tile_type: TileType::Grass, content: Content::None, elevation: 0}),
                Some(Tile{tile_type: TileType::Grass, content: Content::None, elevation: 0}),
                Some(Tile{tile_type: TileType::Grass, content: Content::None, elevation: 0}),
            ],
            vec![
                Some(Tile{tile_type: TileType::Grass, content: Content::Tree(3), elevation: 0}),
                Some(Tile{tile_type: TileType::Grass, content: Content::Tree(3), elevation: 0}),
                Some(Tile{tile_type: TileType::Grass, content: Content::None, elevation: 0}),
                Some(Tile{tile_type: TileType::Grass, content: Content::None, elevation: 0}),
                Some(Tile{tile_type: TileType::Grass, content: Content::Garbage(1), elevation: 0}),
                Some(Tile{tile_type: TileType::Grass, content: Content::Garbage(1), elevation: 0}),
            ],
            vec![
                Some(Tile{tile_type: TileType::Grass, content: Content::Tree(3), elevation: 0}),
                Some(Tile{tile_type: TileType::Grass, content: Content::Tree(3), elevation: 0}),
                Some(Tile{tile_type: TileType::Grass, content: Content::None, elevation: 0}),
                Some(Tile{tile_type: TileType::Grass, content: Content::None, elevation: 0}),
                Some(Tile{tile_type: TileType::Grass, content: Content::Garbage(1), elevation: 0}),
                Some(Tile{tile_type: TileType::Grass, content: Content::Garbage(1), elevation: 0}),
            ],
        ];
        println!("{}", morans_i(&sector));
    }
}

