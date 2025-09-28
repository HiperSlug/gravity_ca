use bevy::{prelude::*, window::PrimaryWindow};
use std::{array, iter::from_fn};

const LEN: usize = u64::BITS as usize;

const FIRST_BIT: u64 = 1;
const LAST_BIT: u64 = 1 << (u64::BITS - 1);

#[derive(Resource)]
struct Chunk {
    some_masks: [u64; LEN],
    dynamic_masks: [u64; LEN],
    state: bool,
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            some_masks: [0; LEN],
            dynamic_masks: [0; LEN],
            state: default(),
        }
    }
}

impl Chunk {
    fn set_dynamic(&mut self, pos: UVec2) {
        let mask = 1 << pos.x;

        self.some_masks[pos.y as usize] |= mask;
        self.dynamic_masks[pos.y as usize] |= mask;
    }

    fn set_none(&mut self, pos: UVec2) {
        let mask = 1 << pos.x;

        self.some_masks[pos.y as usize] &= !mask;
        self.dynamic_masks[pos.y as usize] &= !mask;
    }
}

impl Chunk {
    fn tick_single(&mut self) {
        self.state = !self.state;

        for y in 1..LEN {
            self.down(y);

            if (y % 2 == 0) ^ self.state {
                self.down_left(y);
                self.down_right(y);
            } else {
                self.down_left(y);
                self.down_right(y);
            }
        }
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

    fn down_left(&mut self, y: usize) {
        let dynamic_mask = self.dynamic_masks[y];
        let left_some_mask = (self.some_masks[y] >> 1) | LAST_BIT;
        let down_left_some_mask = (self.some_masks[y - 1] >> 1) | LAST_BIT;

        let fall_mask = dynamic_mask & !left_some_mask & !down_left_some_mask;

        self.some_masks[y] &= !fall_mask;
        self.dynamic_masks[y] &= !fall_mask;

        self.some_masks[y - 1] |= fall_mask << 1;
        self.dynamic_masks[y - 1] |= fall_mask << 1;
    }

    fn down_right(&mut self, y: usize) {
        let dynamic_mask = self.dynamic_masks[y];
        let right_some_mask = (self.some_masks[y] << 1) | FIRST_BIT;
        let down_right_some_mask = (self.some_masks[y - 1] << 1) | FIRST_BIT;

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
        .insert_resource(Time::<Fixed>::from_hz(30.0))
        .init_resource::<CursorCellPos>()
        .init_resource::<Handles>()
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, (tick_simulation, mesh_cells).chain())
        .add_systems(Update, (update_cursors_cell_pos, input_set_cells).chain())
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

#[derive(Resource, Default)]
struct CursorCellPos(Option<UVec2>);

fn update_cursors_cell_pos(mut cursor_cell_pos: ResMut<CursorCellPos>, window: Single<&Window, With<PrimaryWindow>>, cam_query: Single<(&Camera, &GlobalTransform)>) {
    let (camera, camera_transform) = cam_query.into_inner();

    if let Some(cursor_position) = window.cursor_position()
        && let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_position)
    {
        let cell_pos = (world_pos.div_euclid(Vec2::splat(DISPLAY_FACTOR as f32)).as_ivec2() + (SIZE.as_ivec2() / 2)).as_uvec2();
        if cell_pos.cmplt(SIZE).all() {
            cursor_cell_pos.0 = Some(cell_pos);
        } else {
            cursor_cell_pos.0 = None;
        }
    }
}

fn input_set_cells(mb_state: Res<ButtonInput<MouseButton>>, world_cursor_pos: Res<CursorCellPos>, mut chunk: ResMut<Chunk>) {
    if let Some(cell_pos) = world_cursor_pos.0 {
        if mb_state.pressed(MouseButton::Left) {
            chunk.set_dynamic(cell_pos);
        } else if mb_state.pressed(MouseButton::Right) {
            chunk.set_none(cell_pos);
        }
    }
}