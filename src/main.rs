use bevy::{
    platform::collections::{HashMap, HashSet},
    prelude::*,
};
use std::{array, collections::BTreeSet, iter::from_fn};

const LEN: usize = u64::BITS as usize;

#[derive(Resource, Clone)]
struct Chunk {
    some_masks: [u64; LEN],
    gravity_masks: [u64; LEN],
}

impl Default for Chunk {
    fn default() -> Self {
        Self::EMPTY
    }
}

impl Chunk {
    const EMPTY: Self = Self {
        some_masks: [0; LEN],
        gravity_masks: [0; LEN],
    };

    fn tick_y0(&mut self, down_left: &mut Self, down_right: &mut Self, left: &Self, right: &Self) {
        let mut sim_gravity_masks = [left.gravity_masks[0], right.gravity_masks[0]];
        let mut sim_some_masks: ([u64; 2], [u64; 2]) = (
            [left.some_masks[0], right.some_masks[0]],
            [
                down_left.some_masks[LEN - 1] >> 32,
                down_right.some_masks[LEN - 1] << 32,
            ],
        );

        self.down_y0(down_left, down_right);
        Self::sim_down_y0(&mut sim_some_masks, &mut sim_gravity_masks);

        self.down_left_y0(
            down_left,
            down_right,
            sim_some_masks.0[0] << 63,
            sim_some_masks.1[0] << 63, 
        );

        Self::sim_down_left_y0(&mut sim_some_masks, &mut sim_gravity_masks);

        self.down_right_y0(
            down_left,
            down_right,
            sim_some_masks.0[1] >> 63,
            sim_some_masks.1[1] >> 63, 
        );
    }

    fn down_y0(&mut self, down_left: &mut Self, down_right: &mut Self) {
        let gravity_mask = self.gravity_masks[0];
        let down_some_mask =
            down_left.some_masks[LEN - 1] << 32 | down_right.some_masks[LEN - 1] >> 32;

        let fall_mask = gravity_mask & !down_some_mask;

        self.some_masks[0] &= !fall_mask;
        self.gravity_masks[0] &= !fall_mask;

        down_left.some_masks[LEN - 1] |= fall_mask << 32;
        down_left.gravity_masks[LEN - 1] |= fall_mask << 32;

        down_right.some_masks[LEN - 1] |= fall_mask >> 32;
        down_right.gravity_masks[LEN - 1] |= fall_mask >> 32;
    }

    fn sim_down_y0(sim_some_masks: &mut ([u64; 2], [u64; 2]), sim_gravity_masks: &mut [u64; 2]) {
        for i in 0..2 {
            let gravity_mask = sim_gravity_masks[i];
            let down_some_mask = sim_some_masks.1[i];

            let fall_mask = gravity_mask & !down_some_mask;

            sim_gravity_masks[i] &= !fall_mask;
            sim_some_masks.0[i] &= !fall_mask;

            sim_some_masks.1[i] |= fall_mask;
        }
    }

    fn down_left_y0(
        &mut self,
        down_left: &mut Self,
        down_right: &mut Self,
        left_some: u64,
        down_left_some: u64,
    ) {
        let gravity_mask = self.gravity_masks[0];
        let left_some_mask = (self.some_masks[0] >> 1) | left_some;
        let down_left_some_mask: u64 =
            (down_left.some_masks[LEN - 1] << 31) | (down_right.some_masks[LEN - 1] >> 33) | down_left_some;

        let fall_mask = gravity_mask & !left_some_mask & !down_left_some_mask;

        self.some_masks[0] &= !fall_mask;
        self.gravity_masks[0] &= !fall_mask;

        down_left.some_masks[LEN - 1] |= fall_mask << 33;
        down_left.gravity_masks[LEN - 1] |= fall_mask << 33;

        down_right.some_masks[LEN - 1] |= fall_mask >> 31; // 1 questional bit here
        down_right.gravity_masks[LEN - 1] |= fall_mask >> 31;
    }

    fn sim_down_left_y0(sim_some_masks: &mut ([u64; 2], [u64; 2]), sim_gravity_masks: &mut [u64; 2]) {
        let right_gravity_mask = sim_gravity_masks[1] << 1;
        let some_mask = sim_some_masks.0[1];
        let down_some_mask = sim_some_masks.1[1];

        let right_fall_mask = right_gravity_mask & !some_mask & !down_some_mask;
        sim_some_masks.1[1] |= right_fall_mask;
    }

    fn down_right_y0(
        &mut self,
        down_left: &mut Self,
        down_right: &mut Self,
        right_some: u64,
        down_right_some: u64, 
    ) {
        let gravity_mask = self.gravity_masks[0];
        let right_some_mask = (self.some_masks[0] << 1) | right_some;
        let down_right_some_mask =
            (down_left.some_masks[LEN - 1] << 33) | (down_right.some_masks[LEN - 1] >> 31) | down_right_some;

        let fall_mask = gravity_mask & !right_some_mask & !down_right_some_mask;

        self.some_masks[0] &= !fall_mask;
        self.gravity_masks[0] &= !fall_mask;

        down_left.some_masks[LEN - 1] |= fall_mask << 31;
        down_left.gravity_masks[LEN - 1] |= fall_mask << 31;

        down_right.some_masks[LEN - 1] |= fall_mask >> 33;
        down_right.gravity_masks[LEN - 1] |= fall_mask >> 33;
    }

    fn tick(&mut self, left: &mut Self, right: &mut Self) {
        for y in 1..LEN {
            self.multi_down(left, right, y);

            if y % 2 == 0 {
                self.multi_down_left(left, right, y);
                self.multi_down_right(left, right, y);
            } else {
                self.multi_down_right(left, right, y);
                self.multi_down_left(left, right, y);
            }
        }
    }

    fn multi_down(&mut self, left: &mut Self, right: &mut Self, y: usize) {
        self.down(y);
        left.down(y);
        right.down(y);
    }

    fn multi_down_left(&mut self, left: &mut Self, right: &mut Self, y: usize) {
        right.down_left(y, self);
        self.down_left(y, left);
        left.down_left_void(y);
    }

    fn multi_down_right(&mut self, left: &mut Self, right: &mut Self, y: usize) {
        left.down_right(y, self);
        self.down_right(y, right);
        right.down_right_void(y);
    }

    fn down(&mut self, y: usize) {
        let gravity_mask = self.gravity_masks[y];
        let down_some_mask = self.some_masks[y - 1];

        let fall_mask = gravity_mask & !down_some_mask;

        self.some_masks[y] &= !fall_mask;
        self.gravity_masks[y] &= !fall_mask;

        self.some_masks[y - 1] |= fall_mask;
        self.gravity_masks[y - 1] |= fall_mask;
    }

    #[inline]
    fn down_left(&mut self, y: usize, left: &Self) {
        let left_some = left.some_masks[y] << 63;
        let down_left_some = left.some_masks[y - 1] << 63;

        self._down_left(y, left_some, down_left_some)
    }

    #[inline]
    fn down_left_void(&mut self, y: usize) {
        self._down_left(y, 0, 0);
    }

    fn _down_left(&mut self, y: usize, left_some: u64, down_left_some: u64) {
        let gravity_mask = self.gravity_masks[y];
        let left_some_mask = (self.some_masks[y] >> 1) | left_some;
        let down_left_some_mask = (self.some_masks[y - 1] >> 1) | down_left_some;

        let fall_mask = gravity_mask & !left_some_mask & !down_left_some_mask;

        self.some_masks[y] &= !fall_mask;
        self.gravity_masks[y] &= !fall_mask;

        self.some_masks[y - 1] |= fall_mask << 1;
        self.gravity_masks[y - 1] |= fall_mask << 1;
    }

    #[inline]
    fn down_right(&mut self, y: usize, right: &Self) {
        let right_some = right.some_masks[y] >> 63;
        let down_right_some = right.some_masks[y - 1] >> 63;

        self._down_right(y, right_some, down_right_some);
    }

    #[inline]
    fn down_right_void(&mut self, y: usize) {
        self._down_right(y, 0, 0);
    }

    fn _down_right(&mut self, y: usize, right_some: u64, down_right_some: u64) {
        let gravity_mask = self.gravity_masks[y];
        let right_some_mask = (self.some_masks[y] << 1) | right_some;
        let down_right_some_mask = (self.some_masks[y - 1] << 1) | down_right_some;

        let fall_mask = gravity_mask & !right_some_mask & !down_right_some_mask;

        self.some_masks[y] &= !fall_mask;
        self.gravity_masks[y] &= !fall_mask;

        self.some_masks[y - 1] |= fall_mask >> 1;
        self.gravity_masks[y - 1] |= fall_mask >> 1;
    }

    fn iter_some(&self) -> impl Iterator<Item = UVec2> {
        (0..LEN).flat_map(|y| {
            let mut x_mask = self.some_masks[y];
            from_fn(move || {
                if x_mask == 0 {
                    None
                } else {
                    let x = x_mask.trailing_zeros();
                    x_mask &= x_mask - 1;
                    Some(UVec2::new(x, y as u32))
                }
            })
        })
    }
}

#[derive(Clone)]
struct Layer {
    map: HashMap<i32, Chunk>,
    simulate: HashSet<i32>,
}

struct ChunkStore {
    map: HashMap<i32, Layer>,
    simulate: BTreeSet<i32>,
}

impl ChunkStore {
    fn tick(&mut self) {
        for &y in &self.simulate {
            let [layer, prev_layer] = self
                .map
                .get_many_mut([&y, &(y - 1)])
                .map(|opt| opt.unwrap());

            let mut left_map = layer.map.clone();
            let mut right_map = layer.map.clone();

            let (left_shift, right_shift) = if y % 2 == 0 { (-1, 0) } else { (0, 1) };

            for (x, chunk) in layer.map.iter().filter(|(x, _)| **x % 2 == 1) {
                let left = left_map.get(&(x - 1)).unwrap();
                let right = right_map.get(&(x + 1)).unwrap();

                let [down_left, down_right] = if y % 2 == 0 {
                    prev_layer
                        .map
                        .get_many_mut([&(x - 1), x])
                        .map(|opt| opt.unwrap())
                } else {
                    prev_layer
                        .map
                        .get_many_mut([x, &(x + 1)])
                        .map(|opt| opt.unwrap())
                };
            }

            for (x, chunk) in layer.map.iter().filter(|(x, _)| **x % 2 == 0) {}

            for (x, chunk) in &mut layer.map {
                let left = left_map.get_mut(&(x - 1)).unwrap();
                let right = right_map.get_mut(&(x + 1)).unwrap();

                chunk.tick(left, right);
            }
        }
    }
}

const SIZE: UVec2 = UVec2::splat(LEN as u32);
const DISPLAY_FACTOR: u32 = 16;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: ((SIZE + UVec2::splat(2)) * DISPLAY_FACTOR).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .init_resource::<Handles>()
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, (tick_simulation, mesh_cells).chain())
        .run();
}

fn setup(mut commands: Commands) {
    fn mask_gen(i: usize) -> u64 {
        if i > 5 { 1 << 32 | 1 << 30 } else { 0 }
    }

    commands.insert_resource(Chunk {
        some_masks: array::from_fn(mask_gen),
        gravity_masks: array::from_fn(mask_gen),
        ..default()
    });

    commands.spawn(Camera2d);
}

fn tick_simulation(mut chunk: ResMut<Chunk>) {
    chunk.tick_single()
}

fn mesh_cells(
    mut commands: Commands,
    cell_entities: Query<Entity, With<Cell>>,
    handles: Res<Handles>,
    chunk: Res<Chunk>,
) {
    for cell_entity in cell_entities {
        commands.entity(cell_entity).despawn();
    }

    for pos in chunk.iter_some() {
        commands.spawn((
            Transform::from_translation(
                (pos.as_vec2() - (SIZE.as_vec2() / 2.0)).extend(0.0) * DISPLAY_FACTOR as f32,
            ),
            Mesh2d(handles.0.clone()),
            MeshMaterial2d(handles.1.clone()),
            Cell,
        ));
    }
}

#[derive(Resource)]
struct Handles(Handle<Mesh>, Handle<ColorMaterial>);

impl FromWorld for Handles {
    fn from_world(world: &mut World) -> Self {
        let mesh = world
            .resource_mut::<Assets<Mesh>>()
            .add(Rectangle::from_length(DISPLAY_FACTOR as f32));
        let color = world
            .resource_mut::<Assets<ColorMaterial>>()
            .add(Color::WHITE);
        Self(mesh, color)
    }
}

#[derive(Component)]
struct Cell;
