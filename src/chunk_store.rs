use std::collections::BTreeSet;
use bevy::platform::collections::{HashMap, HashSet};

use crate::chunk::Chunk;

struct Layer {
    map: HashMap<i32, Chunk>,
    simulate: HashSet<i32>,
}

struct ChunkStore {
    map: HashMap<i32, Layer>,
    simulate: BTreeSet<i32>,
	state: bool,
}

impl ChunkStore {
    fn gravity_step(&mut self) {
        for &y in &self.simulate {
            let [layer, prev_layer] = self
                .map
                .get_many_mut([&y, &(y - 1)])
                .map(|opt| opt.unwrap());

            for x in layer.simulate.iter().copied().filter(|x| *x % 2 == 0) {
                let left_adj = layer.map.get(&(x - 1)).unwrap().gravity_masks();
                let right_adj = layer.map.get(&(x + 1)).unwrap().gravity_masks();
                let chunk = layer.map.get_mut(&x).unwrap();

                let [down_left, down_right] = if y % 2 == 0 {
                    prev_layer
                        .map
                        .get_many_mut([&(x - 1), &x])
                        .map(|opt| opt.unwrap())
                } else {
                    prev_layer
                        .map
                        .get_many_mut([&x, &(x + 1)])
                        .map(|opt| opt.unwrap())
                };
                
                chunk.gravity_step_zero(down_left, down_right, left_adj, right_adj, self.state);
            }

            for x in layer.simulate.iter().copied().filter(|x| *x % 2 == 1) {
                let left_adj = layer.map.get(&(x - 1)).unwrap().gravity_masks();
                let right_adj = layer.map.get(&(x + 1)).unwrap().gravity_masks();
                let chunk = layer.map.get_mut(&x).unwrap();

                let [down_left, down_right] = if y % 2 == 0 {
                    prev_layer
                        .map
                        .get_many_mut([&(x - 1), &x])
                        .map(|opt| opt.unwrap())
                } else {
                    prev_layer
                        .map
                        .get_many_mut([&x, &(x + 1)])
                        .map(|opt| opt.unwrap())
                };
                
                chunk.gravity_step_zero(down_left, down_right, left_adj, right_adj, self.state);
            }

            let mut left_map = layer.map.clone();
            let mut right_map = layer.map.clone();

            for (x, chunk) in &mut layer.map {
                let left = left_map.get_mut(&(x - 1)).unwrap();
                let right = right_map.get_mut(&(x + 1)).unwrap();

                chunk.gravity_step_non_zero(left, right, self.state);
            }
        }
		self.state = !self.state;
    }
}