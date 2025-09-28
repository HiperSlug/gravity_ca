use bevy::prelude::*;
use std::iter::from_fn;

pub const LEN: usize = u64::BITS as usize;

#[derive(Clone)]
pub struct Chunk {
    pub some_masks: [u64; LEN],
    pub gravity_masks: [u64; LEN],
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            some_masks: [0; LEN],
            gravity_masks: [0; LEN],
        }
    }
}

impl Chunk {
    pub fn iter_some(&self, y_is_odd: bool) -> impl Iterator<Item = UVec2> {
        (0..LEN).flat_map(move |y| {
            let mut x_mask = self.some_masks[y];
            from_fn(move || {
                if x_mask == 0 {
                    None
                } else {
                    let x = x_mask.trailing_zeros();
                    x_mask &= x_mask - 1;
                    if y_is_odd {
                        Some(UVec2::new(x + 32, y as u32))
                    } else {
                        Some(UVec2::new(x, y as u32))
                    }
                }
            })
        })
    }
}
