pub mod non_zero {
    use crate::chunk::{Chunk, LEN};

    impl Chunk {
        pub fn gravity_step_non_zero(&mut self, left: &mut Self, right: &mut Self, state: bool) {
            for y in 1..LEN {
                self.down_nz(y);
                left.down_nz(y);
                right.down_nz(y);

                if (y % 2 == 0) ^ state {
                    self.multi_down_left_nz(left, right, y);
                    self.multi_down_right_nz(left, right, y);
                } else {
                    self.multi_down_right_nz(left, right, y);
                    self.multi_down_left_nz(left, right, y);
                }
            }
        }

        fn multi_down_left_nz(&mut self, left: &mut Self, right: &mut Self, y: usize) {
            right.down_left_nz(y, self);
            self.down_left_nz(y, left);
            left.down_left_void_nz(y);
        }

        fn multi_down_right_nz(&mut self, left: &mut Self, right: &mut Self, y: usize) {
            left.down_right_nz(y, self);
            self.down_right_nz(y, right);
            right.down_right_void_nz(y);
        }

        fn down_nz(&mut self, y: usize) {
            let gravity_mask = self.gravity_masks[y];
            let down_some_mask = self.some_masks[y - 1];

            let fall_mask = gravity_mask & !down_some_mask;

            self.some_masks[y] &= !fall_mask;
            self.gravity_masks[y] &= !fall_mask;

            self.some_masks[y - 1] |= fall_mask;
            self.gravity_masks[y - 1] |= fall_mask;
        }

        fn down_left_nz(&mut self, y: usize, left: &Self) {
            let left_some = left.some_masks[y] << 63;
            let down_left_some = left.some_masks[y - 1] << 63;

            self.base_down_left_nz(y, left_some, down_left_some)
        }

        fn down_left_void_nz(&mut self, y: usize) {
            self.base_down_left_nz(y, 0, 0);
        }

        fn base_down_left_nz(&mut self, y: usize, left_some: u64, down_left_some: u64) {
            let gravity_mask = self.gravity_masks[y];
            let left_some_mask = (self.some_masks[y] >> 1) | left_some;
            let down_left_some_mask = (self.some_masks[y - 1] >> 1) | down_left_some;

            let fall_mask = gravity_mask & !left_some_mask & !down_left_some_mask;

            self.some_masks[y] &= !fall_mask;
            self.gravity_masks[y] &= !fall_mask;

            self.some_masks[y - 1] |= fall_mask << 1;
            self.gravity_masks[y - 1] |= fall_mask << 1;
        }

        fn down_right_nz(&mut self, y: usize, right: &Self) {
            let right_some = right.some_masks[y] >> 63;
            let down_right_some = right.some_masks[y - 1] >> 63;

            self.base_down_right_nz(y, right_some, down_right_some);
        }

        fn down_right_void_nz(&mut self, y: usize) {
            self.base_down_right_nz(y, 0, 0);
        }

        fn base_down_right_nz(&mut self, y: usize, right_some: u64, down_right_some: u64) {
            let gravity_mask = self.gravity_masks[y];
            let right_some_mask = (self.some_masks[y] << 1) | right_some;
            let down_right_some_mask = (self.some_masks[y - 1] << 1) | down_right_some;

            let fall_mask = gravity_mask & !right_some_mask & !down_right_some_mask;

            self.some_masks[y] &= !fall_mask;
            self.gravity_masks[y] &= !fall_mask;

            self.some_masks[y - 1] |= fall_mask >> 1;
            self.gravity_masks[y - 1] |= fall_mask >> 1;
        }
    }
}

pub mod zero {
    use crate::chunk::{Chunk, LEN};

    const TOP: usize = LEN - 1;
    const BOTTOM: usize = 0;

    const LEFT_HALF_MASK: u64 = u64::MAX >> 32;
    const RIGHT_HALF_MASK: u64 = u64::MAX << 32;

    #[derive(Clone, Copy)]
    pub struct GravityMasks {
        some_mask: u64,
        gravity_mask: u64,
    }

    impl Chunk {
        pub fn gravity_masks(&self) -> GravityMasks {
            GravityMasks {
                some_mask: self.some_masks[0],
                gravity_mask: self.gravity_masks[0],
            }
        }
    }

    #[derive(Clone, Copy)]
    struct AdjSim {
        masks: GravityMasks,
        down_some_mask: u64,
    }

    impl AdjSim {
        fn down(&mut self) {
            let fall_mask = self.masks.gravity_mask & !self.down_some_mask;
            self.masks.gravity_mask &= !fall_mask;
            self.masks.some_mask &= !fall_mask;
            self.down_some_mask |= fall_mask;
        }

        fn down_left(&mut self) {
            let right_gravity_mask = self.masks.gravity_mask >> 1;
            let right_fall_mask = right_gravity_mask & !self.masks.some_mask & !self.down_some_mask;
            self.down_some_mask |= right_fall_mask;
        }

        fn down_right(&mut self) {
            let right_gravity_mask = self.masks.gravity_mask << 1;
            let right_fall_mask = right_gravity_mask & !self.masks.some_mask & !self.down_some_mask;
            self.down_some_mask |= right_fall_mask;
        }

        fn left_some(&self) -> u64 {
            self.masks.some_mask << 63
        }

        fn down_left_some(&self) -> u64 {
            self.down_some_mask << 63
        }

        fn right_some(&self) -> u64 {
            self.masks.some_mask >> 63
        }

        fn down_right_some(&self) -> u64 {
            self.down_some_mask >> 63
        }
    }

    impl Chunk {
        pub fn gravity_step_zero(
            &mut self,
            down_left: &mut Self,
            down_right: &mut Self,
            left_gravity_masks: GravityMasks,
            right_gravity_masks: GravityMasks,
            state: bool,
        ) {
            let mut left_sim = AdjSim {
                masks: left_gravity_masks,
                down_some_mask: down_left.some_masks[TOP] >> 32,
            };
            let mut right_sim = AdjSim {
                masks: right_gravity_masks,
                down_some_mask: down_right.some_masks[TOP] << 32,
            };

            self.down_z(down_left, down_right);
            left_sim.down();
            right_sim.down();

            if state {
                self.down_left_z(down_left, down_right, left_sim);
                right_sim.down_left();

                self.down_right_z(down_left, down_right, right_sim);
            } else {
                self.down_left_z(down_left, down_right, left_sim);
                left_sim.down_right();

                self.down_right_z(down_left, down_right, right_sim);
            }
        }

        fn down_z(&mut self, down_left: &mut Self, down_right: &mut Self) {
            let gravity_mask = self.gravity_masks[BOTTOM];
            let down_some_mask = down_left.some_masks[TOP] << 32 | down_right.some_masks[TOP] >> 32;

            let fall_mask = gravity_mask & !down_some_mask;

            self.some_masks[BOTTOM] &= !fall_mask;
            self.gravity_masks[BOTTOM] &= !fall_mask;

            down_left.some_masks[TOP] |= fall_mask >> 32;
            down_left.gravity_masks[TOP] |= fall_mask >> 32;

            down_right.some_masks[TOP] |= fall_mask << 32;
            down_right.gravity_masks[TOP] |= fall_mask << 32;
        }

        fn down_left_z(&mut self, down_left: &mut Self, down_right: &mut Self, left_sim: AdjSim) {
            let gravity_mask = self.gravity_masks[BOTTOM];
            let left_some_mask = (self.some_masks[BOTTOM] >> 1) | left_sim.left_some();
            let down_left_some_mask = (down_left.some_masks[TOP] << 31)
                | (down_right.some_masks[TOP] >> 33)
                | left_sim.down_left_some();

            let fall_mask = gravity_mask & !left_some_mask & !down_left_some_mask;

            self.some_masks[BOTTOM] &= !fall_mask;
            self.gravity_masks[BOTTOM] &= !fall_mask;

            down_left.some_masks[TOP] |= (fall_mask >> 31) & RIGHT_HALF_MASK;
            down_left.gravity_masks[TOP] |= (fall_mask >> 31) & RIGHT_HALF_MASK;

            down_right.some_masks[TOP] |= fall_mask << 33;
            down_right.gravity_masks[TOP] |= fall_mask << 33;
        }

        fn down_right_z(&mut self, down_left: &mut Self, down_right: &mut Self, right_sim: AdjSim) {
            let gravity_mask = self.gravity_masks[BOTTOM];
            let right_some_mask = (self.some_masks[BOTTOM] << 1) | right_sim.right_some();
            let down_right_some_mask = (down_right.some_masks[TOP] >> 31)
                | (down_left.some_masks[TOP] << 33)
                | right_sim.down_right_some();

            let fall_mask = gravity_mask & !right_some_mask & !down_right_some_mask;

            self.some_masks[BOTTOM] &= !fall_mask;
            self.gravity_masks[BOTTOM] &= !fall_mask;

            down_left.some_masks[TOP] |= fall_mask >> 33;
            down_left.gravity_masks[TOP] |= fall_mask >> 33;

            down_right.some_masks[TOP] |= (fall_mask << 31) & LEFT_HALF_MASK;
            down_right.gravity_masks[TOP] |= (fall_mask << 31) & LEFT_HALF_MASK;
        }
    }
}
