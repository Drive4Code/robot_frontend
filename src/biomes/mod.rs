use ohcrab_weather::weather_tool::WeatherPredictionTool;
use robotics_lib::world::environmental_conditions::WeatherType;
use robotics_lib::world::tile::TileType::{Sand, Grass, Hill, Mountain, Snow, Street, ShallowWater, DeepWater};
use robotics_lib::world::tile::Tile;



#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Biome{
    Beach,
    Field,
    Highlands,
    Unknown,
}
pub fn detect_biome(robot_view: &Vec<Vec<Option<Tile>>>) -> Biome{
    let mut beach = 0;
    let mut field = 0;

    let mut highlands = 0;
    for tile in robot_view.iter().flatten(){
        if let Some(tile) = tile{
            match tile.tile_type {
                | Sand | ShallowWater | DeepWater  => beach += 1,
                | Grass | Street => field += 1,
                | Hill | Mountain | Snow => highlands += 1,
                _ => {},
            }
        }
    }
    if beach >= 5{
        return Biome::Beach;
    }
    if field >= 5{
        return Biome::Field;
    }
    if highlands >= 5{
        return Biome::Highlands;
    }
    else{
        return Biome::Unknown;
    }
}

pub fn is_weather_nice(biome: Biome, weather: WeatherType) -> bool{
    match (biome, weather){
        | (Biome::Highlands, WeatherType::TrentinoSnow) => false,
        | (_, WeatherType::Rainy) => false,
        | (_, WeatherType::TropicalMonsoon) => false,
        | (_, _) => true,
    }
}
pub fn is_weather_gonna_be_nice_n(tool: &WeatherPredictionTool, biome: Biome, bound: usize) -> bool{
    for n_of_ticks in 0..bound{
        if let Ok(weather) = tool.predict(n_of_ticks){
            if is_weather_nice( biome, weather) {
                return true;
            }
        }
    }
    false
}
#[cfg(test)]
mod biome_tests{
    use robotics_lib::world::tile::{Content, Tile};

    use super::*;
    #[test]
    fn test_detect_biome(){
        let robot_view = vec![vec![Some(Tile{tile_type:Grass,content:Content::None, elevation: 0 }); 3]; 3];
        assert_eq!(detect_biome(&robot_view), Biome::Field);
        let robot_view = 
                vec![vec![Some(Tile{tile_type:Sand,content:Content::None, elevation: 0 }); 3], 
                    vec![Some(Tile{tile_type:Grass,content:Content::None, elevation: 0 }); 3],
                    vec![Some(Tile{tile_type:Street,content:Content::None, elevation: 0 }); 3],
                ];
        assert_eq!(detect_biome(&robot_view), Biome::Field);
        let robot_view = 
                vec![vec![Some(Tile{tile_type:Sand,content:Content::None, elevation: 0 }); 3], 
                    vec![Some(Tile{tile_type:Grass,content:Content::None, elevation: 0 }); 3],
                    vec![Some(Tile{tile_type:Hill,content:Content::None, elevation: 0 }); 3],
                ];
        assert_eq!(detect_biome(&robot_view), Biome::Unknown);
    }
}