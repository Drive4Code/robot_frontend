use robotics_lib::world::tile::Tile;
use Classification::{Core, Edge, Noise};

use crate::morans_i::get_content_value_morans;

/// Classification according to the DBSCAN algorithm
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum Classification {
    /// A point with at least `min_points` neighbors within `eps` diameter
    Core((usize, usize), usize),
    /// A point within `eps` of a core point, but has less than `min_points` neighbors
    Edge((usize, usize), usize),
    /// A point with no connections
    Noise,
}

// Cluster datapoints using the DBSCAN algorithm
//
// # Arguments
// * `eps` - maximum distance between datapoints within a cluster
// * `min_points` - minimum number of datapoints to make a cluster
// * `input` - a Vec<Vec<f64>> of datapoints, organized by row
pub fn cluster(eps: f64, min_points: usize, input: &Vec<(usize, usize, usize)>) -> Vec<Classification>{
        Model::new(eps, min_points).run(input)
    }

// DBSCAN parameters
pub struct Model{
    /// Epsilon value - maximum distance between points in a cluster
    pub eps: f64,
    /// Minimum number of points in a cluster
    pub mpt: usize,

    distance: fn(&(usize, usize, usize), &(usize, usize, usize)) -> f64,
    c: Vec<Classification>,
    v: Vec<bool>,
}

pub fn get_distance(a: &(usize, usize, usize), b: &(usize, usize, usize)) -> f64 {
    if a.2 != b.2 {
        return 100.0;
    }
    let dx = (a.0 as f64 - b.0 as f64).powi(2);
    let dy = (a.1 as f64 - b.1 as f64).powi(2);
    (dx + dy).sqrt()
}
impl Model{
    pub fn new(eps: f64, min_points: usize) -> Model{
        Model {
            eps,
            mpt: min_points,
            c: Vec::new(),
            v: Vec::new(),
            distance: get_distance,
        }
    }

    pub fn set_distance_fn<F>(mut self, func: fn(&(usize, usize, usize), &(usize, usize, usize)) -> f64) -> Model {
        self.distance = func;
        self
    }

    fn expand_cluster(
        &mut self,
        population: &Vec<(usize, usize, usize)>,
        queue: &mut Vec<(usize,(usize, usize, usize)) >,
        cluster: usize,
    ) -> bool {
        let mut new_cluster = false;
        while let Some(ind) = queue.pop() {
            let neighbors = self.range_query(&ind.1, population);
            if neighbors.len() < self.mpt {
                continue;
            }
            new_cluster = true;
            self.c[ind.0] = Core((ind.1.0, ind.1.1),cluster);
            for n_idx in neighbors {
                // n_idx is at least an edge point
                if self.c[n_idx.0] == Noise {
                    self.c[n_idx.0] = Edge((n_idx.1.0, n_idx.1.1), cluster);
                }

                if self.v[n_idx.0] {
                    continue;
                }

                self.v[n_idx.0] = true;
                queue.push(n_idx);
            }
        }
        new_cluster
    }

    #[inline]
    fn range_query(&self, sample: &(usize, usize, usize), population: &Vec<(usize, usize, usize)>) -> Vec<(usize, (usize, usize, usize))> {
        population
            .iter()
            .enumerate()
            .filter(|(_, pt)| (self.distance)(sample, pt) < self.eps)
            .map(|(idx, item)| (idx, (item.0, item.1, item.2)))
            .collect()
    }

    pub fn run(mut self, population: &Vec<(usize, usize, usize)>) -> Vec<Classification> {
        self.c = vec![Noise; population.len()];
        self.v = vec![false; population.len()];

        let mut cluster = 0;
        let mut queue: Vec<(usize, (usize, usize, usize))> = Vec::new();

        for idx in 0..population.len() {
            if self.v[idx] {
                continue;
            }

            self.v[idx] = true;

            queue.push((idx, population[idx]));

            if self.expand_cluster(population, &mut queue, cluster) {
                cluster += 1;
            }
        }
        self.c
    }

}
pub fn map_into_db_input(input: &Vec<Vec<Option<Tile>>>) -> Vec<(usize, usize, usize)>{
    let mut output = Vec::new();
    for (i, row) in input.iter().enumerate(){
        for (j, tile) in row.iter().enumerate(){
            output.push((i, j, get_content_value_morans(tile) as usize));
        }
    }
    output
}