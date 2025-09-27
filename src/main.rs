use bevy::{
    platform::collections::{HashMap, HashSet},
    prelude::*,
    tasks::prelude::*,
};
use std::{array, collections::BTreeSet, iter::from_fn};

const LEN: usize = u64::BITS as usize;

#[derive(Resource, Clone)]
struct Chunk {
    some_masks: [u64; LEN],
    dynamic_masks: [u64; LEN],
}

impl Default for Chunk {
    fn default() -> Self {
        Self::EMPTY
    }
}

impl Chunk {
    const EMPTY: Self = Self {
        some_masks: [0; LEN],
        dynamic_masks: [0; LEN],
    };

    /// only `self`s simulation is be accurate
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
        right.down_left(self, y);
        self.down_left(left, y);
        left.down_left(&default(), y);
    }

    fn multi_down_right(&mut self, left: &mut Self, right: &mut Self, y: usize) {
        left.down_right(self, y);
        self.down_right(right, y);
        right.down_right(&default(), y);
    }

    fn down(&mut self, y: usize) {
        let dynamic_mask = self.dynamic_masks[y];
        let down_some_mask = self.some_masks[y - 1];

        let fall_mask = dynamic_mask & !down_some_mask;

        self.some_masks[y] &= !fall_mask;
        self.dynamic_masks[y] &= !fall_mask;

        self.some_masks[y - 1] |= fall_mask;
        self.dynamic_masks[y - 1] |= fall_mask;
    }

    fn down_left(&mut self, left: &Self, y: usize) {
        let dynamic_mask = self.dynamic_masks[y];
        let left_some_mask = (self.some_masks[y] >> 1) | (left.some_masks[y] << 63);
        let down_left_some_mask = (self.some_masks[y - 1] >> 1) | (left.some_masks[y - 1] << 63);

        let fall_mask = dynamic_mask & !left_some_mask & !down_left_some_mask;

        self.some_masks[y] &= !fall_mask;
        self.dynamic_masks[y] &= !fall_mask;

        self.some_masks[y - 1] |= fall_mask << 1;
        self.dynamic_masks[y - 1] |= fall_mask << 1;
    }

    fn down_right(&mut self, right: &Self, y: usize) {
        let dynamic_mask = self.dynamic_masks[y];
        let right_some_mask = (self.some_masks[y] << 1) | (right.some_masks[y] >> 63);
        let down_right_some_mask = (self.some_masks[y - 1] << 1) | (right.some_masks[y - 1] >> 63);

        let fall_mask = dynamic_mask & !right_some_mask & !down_right_some_mask;

        self.some_masks[y] &= !fall_mask;
        self.dynamic_masks[y] &= !fall_mask;

        self.some_masks[y - 1] |= fall_mask >> 1;
        self.dynamic_masks[y - 1] |= fall_mask >> 1;
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
        self.for_each_consecutive_simulted_window_mut(|layer, prev_layer| {
            let mut chunks = layer
                .map
                .keys()
                .copied()
                .map(|x| {
                    let left = layer.map.get(&(x - 1)).unwrap().clone();
                    let right = layer.map.get(&(x + 1)).unwrap().clone();
                    let chunk = layer.map.get_mut(&x).unwrap();
                    (x, left, chunk, right)
                })
                .collect::<Vec<_>>();

            chunks.par_chunk_map_mut(ComputeTaskPool::get(), 8, |_, slice| {
                for (x, left, chunk, right) in slice {
                    chunk.tick_multi(left, right);
                }
            });
        });
    }

    fn for_each_consecutive_simulted_window_mut(
        &mut self,
        mut f: impl FnMut(&mut Layer, &mut Layer),
    ) {
        for &y in &self.simulate {
            let prev_y = y - 1;

            let [layer, prev_layer] = self.map.get_many_mut([&y, &prev_y]);

            f(layer.unwrap(), prev_layer.unwrap())
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
        dynamic_masks: array::from_fn(mask_gen),
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
