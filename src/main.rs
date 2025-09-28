mod chunk;
mod chunk_store;
mod gravity;

fn main() {}

// use bevy::prelude::*;

// const SIZE: UVec2 = UVec2::splat(LEN as u32);
// const DISPLAY_FACTOR: u32 = 16;

// fn main() {
//     App::new()
//         .add_plugins(
//             DefaultPlugins
//                 .set(WindowPlugin {
//                     primary_window: Some(Window {
//                         resolution: ((SIZE + UVec2::splat(2)) * DISPLAY_FACTOR).into(),
//                         ..default()
//                     }),
//                     ..default()
//                 })
//                 .set(ImagePlugin::default_nearest()),
//         )
//         .insert_resource(Time::<Fixed>::from_hz(60.0))
//         .init_resource::<Handles>()
//         .add_systems(Startup, setup)
//         .add_systems(FixedUpdate, (tick_simulation, mesh_cells).chain())
//         .run();
// }

// fn setup(mut commands: Commands) {
//     fn mask_gen(i: usize) -> u64 {
//         if i > 5 { 1 << 32 | 1 << 30 } else { 0 }
//     }

//     commands.insert_resource(Chunk {
//         some_masks: array::from_fn(mask_gen),
//         gravity_masks: array::from_fn(mask_gen),
//         ..default()
//     });

//     commands.spawn(Camera2d);
// }

// fn tick_simulation(mut chunk: ResMut<Chunk>) {
//     chunk.tick_single()
// }

// fn mesh_cells(
//     mut commands: Commands,
//     cell_entities: Query<Entity, With<Cell>>,
//     handles: Res<Handles>,
//     chunk: Res<Chunk>,
// ) {
//     for cell_entity in cell_entities {
//         commands.entity(cell_entity).despawn();
//     }

//     for pos in chunk.iter_some() {
//         commands.spawn((
//             Transform::from_translation(
//                 (pos.as_vec2() - (SIZE.as_vec2() / 2.0)).extend(0.0) * DISPLAY_FACTOR as f32,
//             ),
//             Mesh2d(handles.0.clone()),
//             MeshMaterial2d(handles.1.clone()),
//             Cell,
//         ));
//     }
// }

// #[derive(Resource)]
// struct Handles(Handle<Mesh>, Handle<ColorMaterial>);

// impl FromWorld for Handles {
//     fn from_world(world: &mut World) -> Self {
//         let mesh = world
//             .resource_mut::<Assets<Mesh>>()
//             .add(Rectangle::from_length(DISPLAY_FACTOR as f32));
//         let color = world
//             .resource_mut::<Assets<ColorMaterial>>()
//             .add(Color::WHITE);
//         Self(mesh, color)
//     }
// }

// #[derive(Component)]
// struct Cell;
