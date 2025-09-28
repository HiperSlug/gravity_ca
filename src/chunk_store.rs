use bevy::{platform::collections::{HashMap, HashSet}, prelude::*, tasks::{ComputeTaskPool, ParallelSlice, ParallelSliceMut}};
use std::{collections::BTreeSet, ops::{Deref, DerefMut}};

use crate::chunk::Chunk;

struct SendPtr<T>(*mut T);

unsafe impl<T> Send for SendPtr<T> {}

impl<T> Deref for SendPtr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            &*self.0
        }
    }
}

impl<T> DerefMut for SendPtr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            &mut *self.0
        }
    }
}

impl<T> SendPtr<T> {
    fn new(value: &mut T) -> Self {
        Self(value as *mut T)
    }
}

struct Layer {
    map: HashMap<i32, Chunk>,
    simulate_odd: HashSet<i32>,
    simulate_even: HashSet<i32>,
}

impl Layer {
    fn gravity_step_non_zero(&mut self, state: bool) {
        let mut simulate = self.simulate_even.iter().chain(self.simulate_odd.iter()).map(|x| {
            let left = self.map.get(&(x - 1)).unwrap().clone();
            let right = self.map.get(&(x + 1)).unwrap().clone();

            let chunk = SendPtr::new(self.map.get_mut(x).unwrap());

            (chunk, left, right)
        }).collect::<Vec<_>>();

        simulate.par_chunk_map_mut(ComputeTaskPool::get(), 4, |_, slice| {
            for (chunk, left, right) in slice {
                chunk.gravity_step_non_zero(left, right, state);
            }
        });

        // let mut left_map = self.map.clone();
        // let mut right_map = self.map.clone();

        // for (x, chunk) in &mut self.map {
        //     let left = left_map.get_mut(&(x - 1)).unwrap();
        //     let right = right_map.get_mut(&(x + 1)).unwrap();

        //     chunk.gravity_step_non_zero(left, right, state);
        // }
    }

    fn gravity_step_zero(&mut self, prev: &mut Self, y_is_odd: bool, state: bool) {
        let mut simulate_odd = self.simulate_odd.iter().map(|x| {
            let left_gravity_masks = self.map.get(&(x - 1)).unwrap().gravity_masks();
            let right_gravity_masks = self.map.get(&(x + 1)).unwrap().gravity_masks();
            let chunk = SendPtr::new(self.map.get_mut(x).unwrap());

            let [down_left, down_right] = prev.get_down_adj_mut(x, y_is_odd).map(|c| SendPtr::new(c));

            (chunk, down_left, down_right, left_gravity_masks, right_gravity_masks)
        }).collect::<Vec<_>>();

        simulate_odd.par_chunk_map_mut(ComputeTaskPool::get(), 8, |_, slice| {
            for (chunk, down_left, down_right, left_gravity_masks, right_gravity_masks) in slice {
                chunk.gravity_step_zero(down_left, down_right, *left_gravity_masks, *right_gravity_masks, state);
            }
        });

        let mut simulate_even = self.simulate_even.iter().map(|x| {
            let left_gravity_masks = self.map.get(&(x - 1)).unwrap().gravity_masks();
            let right_gravity_masks = self.map.get(&(x + 1)).unwrap().gravity_masks();
            let chunk = SendPtr::new(self.map.get_mut(x).unwrap());

            let [down_left, down_right] = prev.get_down_adj_mut(x, y_is_odd).map(|c| SendPtr::new(c));

            (chunk, down_left, down_right, left_gravity_masks, right_gravity_masks)
        }).collect::<Vec<_>>();

        simulate_even.par_chunk_map_mut(ComputeTaskPool::get(), 8, |_, slice| {
            for (chunk, down_left, down_right, left_gravity_masks, right_gravity_masks) in slice {
                chunk.gravity_step_zero(down_left, down_right, *left_gravity_masks, *right_gravity_masks, state);
            }
        });
        
        // for x in self.simulate_odd.iter() {
        //     let left_gravity_masks = self.map.get(&(x - 1)).unwrap().gravity_masks();
        //     let right_gravity_masks = self.map.get(&(x + 1)).unwrap().gravity_masks();
        //     let chunk = self.map.get_mut(x).unwrap();

        //     let [down_left, down_right] = prev.get_down_adj_mut(x, y_is_odd);

        //     chunk.gravity_step_zero(
        //         down_left,
        //         down_right,
        //         left_gravity_masks,
        //         right_gravity_masks,
        //         state,
        //     );
        // }
        // for x in self.simulate_even.iter() {
        //     let left_gravity_masks = self.map.get(&(x - 1)).unwrap().gravity_masks();
        //     let right_gravity_masks = self.map.get(&(x + 1)).unwrap().gravity_masks();
        //     let chunk = self.map.get_mut(x).unwrap();

        //     let [down_left, down_right] = prev.get_down_adj_mut(x, y_is_odd);

        //     chunk.gravity_step_zero(
        //         down_left,
        //         down_right,
        //         left_gravity_masks,
        //         right_gravity_masks,
        //         state,
        //     );
        // }
    }

    fn get_down_adj_mut(&mut self, x: &i32, y_is_odd: bool) -> [&mut Chunk; 2] {
        if y_is_odd {
            self.map.get_many_mut([x, &(x + 1)]).map(|opt| opt.unwrap())
        } else {
            self.map.get_many_mut([&(x - 1), x]).map(|opt| opt.unwrap())
        }
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
