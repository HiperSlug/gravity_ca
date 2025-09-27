use crate::*;

const TOP: usize = LEN - 1;
const BOTTOM: usize = 0;

const LEFT_HALF_MASK: u64 = u64::MAX >> 32;
const RIGHT_HALF_MASK: u64 = u64::MAX << 32;

#[derive(Clone, Copy)]
struct Adj {
    some_mask: u64,
    gravity_mask: u64,
}

impl Chunk {
    fn adj_y0(&self) -> Adj {
        Adj {
            some_mask: self.some_masks[0],
            gravity_mask: self.gravity_masks[0],
        }
    }
}

#[derive(Clone, Copy)]
struct Adjacent {
    adj: Adj,
    down_some_mask: u64,
}

impl Adjacent {
    fn down(&mut self) {
        let fall_mask = self.adj.gravity_mask & !self.down_some_mask;
        self.adj.gravity_mask &= !fall_mask;
        self.adj.some_mask &= !fall_mask;
        self.down_some_mask |= fall_mask;
    }

    fn down_left(&mut self) {
        let right_gravity_mask = self.adj.gravity_mask >> 1;
        let right_fall_mask = right_gravity_mask & !self.adj.some_mask & !self.down_some_mask;
        self.down_some_mask |= right_fall_mask;
    }

    fn down_right(&mut self) {
        let right_gravity_mask = self.adj.gravity_mask << 1;
        let right_fall_mask = right_gravity_mask & !self.adj.some_mask & !self.down_some_mask;
        self.down_some_mask |= right_fall_mask;
    }

    fn left_some(&self) -> u64 {
        self.adj.some_mask << 63
    }

    fn down_left_some(&self) -> u64 {
        self.down_some_mask << 63
    }

    fn right_some(&self) -> u64 {
        self.adj.some_mask >> 63
    }

    fn down_right_some(&self) -> u64 {
        self.down_some_mask >> 63
    }
}

impl Chunk {
    fn tick_y0(
        &mut self,
        down_left: &mut Self,
        down_right: &mut Self,
        left_adj: Adj,
        right_adj: Adj,
    ) {
        let mut left_adj = Adjacent {
            adj: left_adj,
            down_some_mask: down_left.some_masks[TOP] >> 32,
        };
        let mut right_adj = Adjacent {
            adj: right_adj,
            down_some_mask: down_right.some_masks[TOP] << 32,
        };

        self.down_y0(down_left, down_right);
        left_adj.down();
        right_adj.down();

        self.down_left_y0(down_left, down_right, left_adj);
        right_adj.down_left();

        self.down_right_y0(down_left, down_right, right_adj);
    }

    fn down_y0(&mut self, down_left: &mut Self, down_right: &mut Self) {
        let gravity_mask = self.gravity_masks[BOTTOM];
        let down_some_mask =
            down_left.some_masks[TOP] << 32 | down_right.some_masks[TOP] >> 32;

        let fall_mask = gravity_mask & !down_some_mask;

        self.some_masks[BOTTOM] &= !fall_mask;
        self.gravity_masks[BOTTOM] &= !fall_mask;

        down_left.some_masks[TOP] |= fall_mask >> 32;
        down_left.gravity_masks[TOP] |= fall_mask >> 32;

        down_right.some_masks[TOP] |= fall_mask << 32;
        down_right.gravity_masks[TOP] |= fall_mask << 32;
    }

    fn down_left_y0(&mut self, down_left: &mut Self, down_right: &mut Self, left_adj: Adjacent) {
        let gravity_mask = self.gravity_masks[BOTTOM];
        let left_some_mask = (self.some_masks[BOTTOM] >> 1) | left_adj.left_some();
        let down_left_some_mask = (down_left.some_masks[TOP] << 31)
            | (down_right.some_masks[TOP] >> 33)
            | left_adj.down_left_some();

        let fall_mask = gravity_mask & !left_some_mask & !down_left_some_mask;

        self.some_masks[BOTTOM] &= !fall_mask;
        self.gravity_masks[BOTTOM] &= !fall_mask;

        down_left.some_masks[TOP] |= (fall_mask >> 31) & RIGHT_HALF_MASK;
        down_left.gravity_masks[TOP] |= (fall_mask >> 31) & RIGHT_HALF_MASK;

        down_right.some_masks[TOP] |= fall_mask << 33;
        down_right.gravity_masks[TOP] |= fall_mask << 33;
    }

    fn down_right_y0(&mut self, down_left: &mut Self, down_right: &mut Self, right_adj: Adjacent) {
        let gravity_mask = self.gravity_masks[BOTTOM];
        let right_some_mask = (self.some_masks[BOTTOM] << 1) | right_adj.right_some();
        let down_right_some_mask = (down_right.some_masks[TOP] >> 31)
            | (down_left.some_masks[TOP] << 33)
            | right_adj.down_right_some();

        let fall_mask = gravity_mask & !right_some_mask & !down_right_some_mask;

        self.some_masks[BOTTOM] &= !fall_mask;
        self.gravity_masks[BOTTOM] &= !fall_mask;

        down_left.some_masks[TOP] |= fall_mask >> 33;
        down_left.gravity_masks[TOP] |= fall_mask >> 33;

        down_right.some_masks[TOP] |= (fall_mask << 31) & LEFT_HALF_MASK;
        down_right.gravity_masks[TOP] |= (fall_mask << 31) & LEFT_HALF_MASK;
    }
}
