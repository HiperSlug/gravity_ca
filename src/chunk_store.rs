use bevy::platform::collections::{HashMap, HashSet};
use std::collections::BTreeSet;

use crate::chunk::Chunk;

struct Layer {
    map: HashMap<i32, Chunk>,
    simulate_odd: HashSet<i32>,
    simulate_even: HashSet<i32>,
}

impl Layer {
    fn gravity_step_non_zero(&mut self, state: bool) {
        let mut left_map = self.map.clone();
        let mut right_map = self.map.clone();

        for (x, chunk) in &mut self.map {
            let left = left_map.get_mut(&(x - 1)).unwrap();
            let right = right_map.get_mut(&(x + 1)).unwrap();

            chunk.gravity_step_non_zero(left, right, state);
        }
    }

    fn gravity_step_zero(&mut self, prev: &mut Self, y_is_odd: bool, state: bool) {
        for x in self.simulate_odd.iter() {
            let left_gravity_masks = self.map.get(&(x - 1)).unwrap().gravity_masks();
            let right_gravity_masks = self.map.get(&(x + 1)).unwrap().gravity_masks();
            let chunk = self.map.get_mut(x).unwrap();

            let (down_left, down_right) = prev.get_down_adj_mut(x, y_is_odd);

            chunk.gravity_step_zero(
                down_left,
                down_right,
                left_gravity_masks,
                right_gravity_masks,
                state,
            );
        }
        for x in self.simulate_even.iter() {
            let left_gravity_masks = self.map.get(&(x - 1)).unwrap().gravity_masks();
            let right_gravity_masks = self.map.get(&(x + 1)).unwrap().gravity_masks();
            let chunk = self.map.get_mut(x).unwrap();

            let (down_left, down_right) = prev.get_down_adj_mut(x, y_is_odd);

            chunk.gravity_step_zero(
                down_left,
                down_right,
                left_gravity_masks,
                right_gravity_masks,
                state,
            );
        }
    }

    fn get_down_adj_mut(&mut self, x: &i32, y_is_odd: bool) -> (&mut Chunk, &mut Chunk) {
        let [down_left, down_right] = if y_is_odd {
            self.map.get_many_mut([x, &(x + 1)]).map(|opt| opt.unwrap())
        } else {
            self.map.get_many_mut([&(x - 1), x]).map(|opt| opt.unwrap())
        };

        (down_left, down_right)
    }
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

            let y_is_odd = y % 2 == 1;

            layer.gravity_step_zero(prev_layer, y_is_odd, self.state);
            layer.gravity_step_non_zero(self.state);
        }
        self.state = !self.state;
    }
}
