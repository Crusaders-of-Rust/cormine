// Bevy queries are necessarily verbose
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

mod chunk;
mod mesh;
mod voxel;

#[cfg(feature = "debug")]
/// Debugging UI features
mod debug;

mod terrain;
/// Keeps track of the whole world of chunks and voxels
mod world;

/// Handles defining and creating materials for rendering
mod material;

/// Handles finding the currently 'selected' voxel and highlighting it
mod highlight;
mod input;
mod ui;

mod player;

use argh::FromArgs;

use bevy::ecs::system::SystemState;
use bevy::ecs::world::CommandQueue;
use bevy::tasks::futures_lite::future;
use bevy::tasks::{block_on, AsyncComputeTaskPool, Task};
use chunk::Chunk;
use mesh::HasMesh;

use bevy::color::palettes::css::WHITE;
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::prelude::*;
use bevy::window::PresentMode;

#[cfg(feature = "wireframe")]
use bevy::render::{
    settings::{RenderCreation, WgpuFeatures, WgpuSettings},
    RenderPlugin,
};
#[cfg(feature = "flycam")]
use bevy_flycam::prelude::*;

use highlight::SelectedVoxel;
use material::{VoxelMaterial, VoxelMaterialResource};
use voxel::VoxelPosition;

/// CoRmine.
#[derive(FromArgs, Resource)]
struct Arguments {
    #[argh(subcommand)]
    commands: ArgumentsCommands,
}

#[derive(FromArgs)]
#[argh(subcommand)]
enum ArgumentsCommands {
    Generate(ArgumentsGenerate),
}

/// Generate a new world
#[derive(FromArgs)]
#[argh(subcommand, name = "generate")]
struct ArgumentsGenerate {
    /// seed to use for world generation
    #[argh(option)]
    seed: Option<u32>,

    /// width and length of the world (in chunks)
    #[argh(option)]
    size: Option<usize>,
}

fn main() {
    let args = argh::from_env::<Arguments>();
    let mut app = App::new();
    let mut default_plugins = DefaultPlugins.build();
    #[cfg(feature = "wireframe")]
    {
        default_plugins = default_plugins.set(RenderPlugin {
            render_creation: RenderCreation::Automatic(WgpuSettings {
                features: WgpuFeatures::POLYGON_MODE_LINE,
                ..default()
            }),
            ..default()
        });
        app.insert_resource(WireframeConfig {
            global: false,
            default_color: WHITE.into(),
        });
    }

    default_plugins = default_plugins.set(WindowPlugin {
        primary_window: Some(Window {
            present_mode: PresentMode::AutoNoVsync,
            ..default()
        }),
        ..default()
    });

    app.add_plugins(default_plugins);
    app.add_plugins(MaterialPlugin::<VoxelMaterial>::default());
    app.init_resource::<world::World>();
    app.init_resource::<SelectedVoxel>();
    app.init_resource::<input::CameraVelocity>();
    app.init_resource::<input::InputState>();
    app.insert_resource(args);

    #[cfg(feature = "wireframe")]
    {
        app.add_plugins(WireframePlugin);
    }

    app.add_systems(
        Startup,
        (
            make_camera,
            material::make_voxel_material,
            terrain::generate_chunks,
            ui::draw_ui,
        ),
    )
    .add_systems(Update, material::process_block_texture)
    .add_systems(
        Update,
        (input::check_input, queue_chunk_meshes, handle_mesh_tasks),
    )
    .add_systems(Update, highlight::update_selected_voxel);

    #[cfg(feature = "flycam")]
    app.add_plugins(NoCameraPlayerPlugin);

    #[cfg(not(feature = "flycam"))]
    {
        app.add_systems(Startup, input::hook_cursor);
        app.add_systems(Update, input::player_look);
        app.add_systems(
            Update,
            player::player_move
                .after(input::player_look)
                .after(input::check_input),
        );
    }

    #[cfg(feature = "debug")]
    app.add_plugins(debug::DebugUiPlugins);

    app.run();
}

fn make_camera(mut commands: Commands) {
    let bundle = Camera3dBundle {
        transform: Transform::from_xyz(8.0, 4.5 + 128.0, 8.0).looking_at(
            Vec3 {
                y: 4.5,
                ..default()
            },
            Vec3::Y,
        ),
        projection: Projection::Perspective(PerspectiveProjection {
            near: 0.1,
            far: 4096.0,
            ..default()
        }),
        camera: Camera {
            clear_color: ClearColorConfig::Custom(Color::linear_rgb(0.13, 0.65, 0.92)),
            ..default()
        },
        ..default()
    };

    #[cfg(feature = "flycam")]
    {
        let mut ent = commands.spawn(bundle);
        ent.insert(FlyCam);
    }

    #[cfg(not(feature = "flycam"))]
    {
        commands.spawn(bundle);
    }
}

#[derive(Component)]
struct ChunkMeshingTask(Task<CommandQueue>);

fn queue_chunk_meshes(
    mut commands: Commands,
    dirty_chunks: Query<(Entity, &Chunk), (Without<HasMesh>, Without<ChunkMeshingTask>)>,
    all_chunks: Query<&Chunk>,
    world: Res<world::World>,
) {
    info_once!("Started queuing chunk tasks");
    let task_pool = AsyncComputeTaskPool::get();
    for (ent, chunk) in dirty_chunks.iter().map(|(e, c)| (e, c.clone())).take(32) {
        // get all adjacent chunks
        let mut adj_chunks = Vec::with_capacity(4);
        let chunk_pos = chunk.position();
        for dx in -1..=1 {
            for dz in -1..=1 {
                if dx == 0 && dz == 0 {
                    continue;
                }
                let pos = VoxelPosition::new(IVec3 {
                    x: chunk_pos.x() + dx * 16,
                    y: 0,
                    z: chunk_pos.z() + dz * 16,
                });
                let Some(chunk) = world
                    .chunk_containing(pos)
                    .and_then(|e| all_chunks.get(e).ok().cloned())
                else {
                    continue;
                };
                adj_chunks.push(chunk);
            }
        }
        let task = task_pool.spawn(async move {
            let mesh = mesh::from_chunk(chunk.clone(), adj_chunks);
            let mut cmd_queue = CommandQueue::default();
            cmd_queue.push(move |world: &mut World| {
                let mut system_state =
                    SystemState::<(ResMut<Assets<Mesh>>, Res<VoxelMaterialResource>)>::new(world);
                let (mut meshes, material) = system_state.get_mut(world);
                let mesh = meshes.add(mesh);
                let material = material.handle.clone();
                world
                    .entity_mut(ent)
                    .insert(MaterialMeshBundle {
                        mesh,
                        transform: Transform::from_translation(chunk.position().as_vec3()),
                        material,
                        ..default()
                    })
                    .insert(HasMesh)
                    .remove::<ChunkMeshingTask>();
            });
            cmd_queue
        });
        commands.entity(ent).insert(ChunkMeshingTask(task));
    }
    info_once!("Finished queuing chunk tasks");
}

fn handle_mesh_tasks(mut commands: Commands, mut tasks: Query<&mut ChunkMeshingTask>) {
    let mut completed = 0;
    for mut task in tasks.iter_mut() {
        if let Some(mut cmd_queue) = block_on(future::poll_once(&mut task.0)) {
            completed += 1;
            commands.append(&mut cmd_queue);
        }
    }
    if completed > 0 {
        debug!("Completed {completed} meshes this frame");
    }
}
