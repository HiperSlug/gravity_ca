use std::collections::BTreeMap;

use bevy::{platform::collections::HashMap, prelude::*};
use itertools::Itertools;

const DISPLAY_FACTOR: u32 = 4;
const SIZE: UVec2 = UVec2::new(1280 / DISPLAY_FACTOR, 720 / DISPLAY_FACTOR);

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: (SIZE * DISPLAY_FACTOR).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        ))
        .run();
}

pub mod pad {
    pub const LEN: usize = 64;
}
pub mod unpad {
    use super::pad;

    pub const LEN: usize = pad::LEN - 4; // 60
}

pub struct Chunk {
    pub some_masks: [u64; pad::LEN],
    pub dynamic_masks: [u64; pad::LEN],
}

// TODO: Figure out a faster way to do this.
// BTree autosorts lowest y values first
// I'm splitting the maps because we need to 
// a. have keyed(positional) lookup
// b. being able to iterate over all chunks at a y level
// c. be able to iterate over y levels in sorted order
pub struct ChunkMap(pub HashMap<i32, HashMap<i32, Chunk>>);

pub fn do_gravity(chunk_map: &mut ChunkMap) {
    let mut keys = chunk_map.0.keys().copied().collect::<Vec<_>>();
    keys.sort_unstable();

    for chunk_y in keys {
        // BTreeMap doesnt have this function!!!!!!! 
        let [down_group_opt, group_opt, up_group_opt] = chunk_map.0.get_many_mut([&(chunk_y - 1), &chunk_y, &(chunk_y + 1)]);
        let group = group_opt.unwrap();
        // simulation
        for chunk in group.values_mut() {
            for y in 1..pad::LEN {
                // down
                {
                    let dynamic_mask = chunk.dynamic_masks[y];
                    let down_some_mask = chunk.some_masks[y - 1];

                    let fall_mask = dynamic_mask & !down_some_mask;

                    chunk.some_masks[y] &= !fall_mask;
                    chunk.dynamic_masks[y] &= !fall_mask;

                    chunk.some_masks[y - 1] |= fall_mask;
                    chunk.dynamic_masks[y - 1] |= fall_mask;
                }
                // down right
                {
                    let dynamic_mask = chunk.dynamic_masks[y];
                    let right_some_mask = chunk.some_masks[y] << 1;
                    let down_right_some_mask = chunk.some_masks[y - 1] << 1;

                    let fall_mask = dynamic_mask & !right_some_mask & !down_right_some_mask;

                    chunk.some_masks[y] &= !fall_mask;
                    chunk.dynamic_masks[y] &= !fall_mask;

                    chunk.some_masks[y - 1] |= fall_mask << 1;
                    chunk.dynamic_masks[y - 1] |= fall_mask << 1;
                }
                // down left
                {
                    let dynamic_mask = chunk.dynamic_masks[y];
                    let left_some_mask = chunk.some_masks[y] >> 1;
                    let down_left_some_mask = chunk.some_masks[y - 1] >> 1;

                    let fall_mask = dynamic_mask & !left_some_mask & !down_left_some_mask;

                    chunk.some_masks[y] &= !fall_mask;
                    chunk.dynamic_masks[y] &= !fall_mask;

                    chunk.some_masks[y - 1] |= fall_mask >> 1;
                    chunk.dynamic_masks[y - 1] |= fall_mask >> 1;
                }
            }
        }
        // syncronization of padding
        let chunk_x_keys = group.keys().copied().collect::<Vec<_>>();
        for chunk_x in chunk_x_keys.iter() {
            // horizontal
            let [left_chunk_opt, chunk_opt, right_chunk_opt] = group.get_many_mut([&(*chunk_x - 1), chunk_x, &(*chunk_x + 1)]);
            let chunk = chunk_opt.unwrap();

            if let Some(left_chunk) = left_chunk_opt {
                for y in 0..pad::LEN {
                    let dynamic_left = &mut left_chunk.dynamic_masks[y];
                    let dynamic_right = &mut chunk.dynamic_masks[y];
                    horizontal_sync_padding(dynamic_left, dynamic_right);

                    let some_left = &mut left_chunk.some_masks[y];
                    let some_right = &mut chunk.some_masks[y];
                    horizontal_sync_padding(some_left, some_right);
                }
            }
            if let Some(right_chunk) = right_chunk_opt {
                for y in 0..pad::LEN {
                    let dynamic_left = &mut chunk.dynamic_masks[y];
                    let dynamic_right = &mut right_chunk.dynamic_masks[y];
                    horizontal_sync_padding(dynamic_left, dynamic_right);

                    let some_left = &mut chunk.some_masks[y];
                    let some_right = &mut right_chunk.some_masks[y];
                    horizontal_sync_padding(some_left, some_right);
                }
            }
        }
        todo!()

        if let Some(down_group) = down_group_opt {
            for chunk_x in chunk_x_keys {

            }
        }
        if let Some(up_group) = up_group_opt {

        }
    }
}

fn horizontal_sync_padding(left: &mut u64, right: &mut u64) {
    const LEFT_MASK: u64 = 0x3FFFFFFFFFFFFFFF;
    *left &= LEFT_MASK;
    *left |= (*right << 60) & !LEFT_MASK;

    const RIGHT_MASK: u64 = 0xFFFFFFFFFFFFFFFC;
    *right &= RIGHT_MASK;
    *right |= (*left >> 60) & !RIGHT_MASK;
}
