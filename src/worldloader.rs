use egui::Color32;
// use robotics_lib::world::environmental_conditions::{EnvironmentalConditions, WeatherType};
// use robotics_lib::world::tile::TileType;
// use robotics_lib::world::tile::{Content, Tile};
use serde::{Deserialize, Serialize};
// use std::collections::HashMap;
// use std::path::PathBuf;
use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
// use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen::UnwrapThrowExt;

use wasm_bindgen::prelude::*;


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Cell {
    pub(crate) color: Color32,
    pub(crate) tile_type: Option<TileType>,
    pub(crate) content: Content,
    pub(crate) elevation: usize,
}

impl Default for Cell {
    fn default() -> Self {
        Cell {
            color: Color32::WHITE,
            tile_type: None,
            elevation: 0,
            content: Content::None,
        }
    }
}
#[derive(Serialize, Deserialize)]
pub(crate) struct Grid {
    pub(crate) size: usize,
    pub(crate) cells: Vec<Vec<Cell>>,
    pub(crate) cell_dim: f32,
    pub(crate) brush_size: usize,
    pub(crate) hovered_tile: Option<(usize, usize)>,
    pub(crate) weather: Vec<WeatherType>,
    pub(crate) max_score: f32,
    pub(crate) robot_pos: (usize, usize),
}
impl Grid {
    pub(crate) fn new(size: usize) -> Self {
        let cells = vec![vec![Cell::default(); size]; size];
        Grid {
            size,
            cells,
            cell_dim: 1.5,
            brush_size: 3,
            hovered_tile: None,
            weather: Vec::new(),
            max_score: 100.0,
            robot_pos: (0, 0),
        }
    }
}

#[wasm_bindgen(module = "/src/file_reader.js")]
extern "C" {
    #[wasm_bindgen(catch)]
    pub fn read_file(path: String) -> Result<Uint8Array, JsValue>;
    // pub fn read_file(path: String) -> Result<Uint8Array, JsValue>; JsFuture
}

const WORLD_DATA: &'static [u8] = include_bytes!("world.bin");

pub(crate) fn load_as_grid(grid: &mut Grid, path: PathBuf) {
    // let uint8_array = match read_file(path.into_os_string().into_string().expect("NAPULE NAVVENTURA")) {
    //     Ok(array) => array,
    //     Err(_) => panic!("Failed to read file."),
    // };
    let msg = JsValue::from(format!("TEST {:?}", WORLD_DATA));
    info!("{}", msg.as_string().unwrap());
    // let bytes = Uint8Array::to_vec(&uint8_array);
    // let msg = JsValue::from(format!("TEST {:?}", bytes));
    // info!("{}", msg.as_string().unwrap());
    // let sliced = bytes.as_slice();
    let deserialized: Grid = bincode::deserialize(WORLD_DATA)
        .expect("Failed to read file. The file is not in the correct format. The only way to create a world is with the gui, maybe you tried doing it manually or you tried to load the wrong file.");
    *grid = deserialized;
}



pub(crate) fn load(
    path: PathBuf,
) -> (
    Vec<Vec<Tile>>,
    (usize, usize),
    EnvironmentalConditions,
    f32,
    Option<HashMap<Content, f32>>,
) {
    
    let mut grid = Grid::new(1);
    load_as_grid(&mut grid, path);

    let mut out = vec![
        vec![
            Tile {
                tile_type: TileType::Grass,
                elevation: 0,
                content: Content::None,
            };
            grid.size
        ];
        grid.size
    ];
    for (i, row) in grid.cells.iter().enumerate() {
        for (j, tile) in row.iter().enumerate() {
            out[i][j] = Tile {
                tile_type: tile.tile_type.unwrap(), // tile type is not empty, otherwise I cannot call load
                elevation: tile.elevation,
                content: tile.content.clone(),
            };
        }
    }
    (
        out,
        grid.robot_pos,
        EnvironmentalConditions::new(&grid.weather, 15, 12).unwrap(), // weather forecast is never empty, ensured by the gui
        grid.max_score,
        None,
    )
}

/// This struct is the world generator, by .unwrap().unwrap().unwrap() !
pub struct WorldgeneratorUnwrap {
    gui_start: bool,
    path: Option<PathBuf>,
    score_map: Option<HashMap<Content, f32>>,
}

impl WorldgeneratorUnwrap {
    /// This function creates an instance of our world generator
    /// # Arguments
    /// * `gui_start` - if true, the gui will start
    /// * `path` - if Some(path), the world will be loaded from this path, otherwise the world will
    /// be loaded from the default path, which is "world.bin", in the parent folder of the "src"
    /// binary directory.
    /// # Remarks
    /// If you choose to start the gui, the simulation will start right after you close it, loading
    /// the path specified here (or "world.bin").
    /// # Suggestions
    /// Compile with `cargo run --release`. Egui really likes to be compiled with optimizations,
    /// and it will be much faster.
    pub fn init(gui_start: bool, path: Option<PathBuf>) -> Self {
        Self {
            gui_start,
            path,
            score_map: None,
        }
    }
    /// This function sets the score hashmap for the world generator
    /// in case you want a custom one.
    /// # Arguments
    /// * `score_map` - the hashmap with the custom scores for each content
    /// # Remarks
    /// If not set, the default score hashmap will be used.
    pub fn set_score_hashmap(&mut self, score_map: HashMap<Content, f32>) {
        self.score_map = Some(score_map);
    }
}

impl robotics_lib::world::world_generator::Generator for WorldgeneratorUnwrap {
    fn gen(
        &mut self,
    ) -> (
        Vec<Vec<Tile>>,
        (usize, usize),
        EnvironmentalConditions,
        f32,
        Option<HashMap<Content, f32>>,
    ) {
        if self.gui_start {
            eprintln!("Gui not supported");
            panic!();
        }
        let mut loaded = match self.path.clone() {
            Some(path) => load(path),
            None => load(PathBuf::new().join("world.bin")),
        };
        loaded.4 = self.score_map.clone();
        loaded
    }
}
