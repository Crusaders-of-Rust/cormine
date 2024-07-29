// Bevy queries are necessarily verbose
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

mod args;
mod chunk;
mod mesh;
mod voxel;

#[cfg(feature = "debug")]
/// Debugging UI features
mod debug;

#[cfg(feature = "renderdoc")]
mod renderdoc;

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
mod save;

use bevy::{
    asset::embedded_asset,
    ecs::{
        system::SystemState,
        world::CommandQueue,
    },
    tasks::{
        block_on,
        futures_lite::future,
        AsyncComputeTaskPool,
        ComputeTaskPool,
        Task,
    },
};
use chunk::{
    ChunkPosition,
    ChunkVoxels,
};
use mesh::HasMesh;

use bevy::{
    color::palettes::css::WHITE,
    pbr::wireframe::{
        WireframeConfig,
        WireframePlugin,
    },
    prelude::*,
    window::PresentMode,
};

#[cfg(feature = "wireframe")]
use bevy::render::{
    settings::{
        RenderCreation,
        WgpuFeatures,
        WgpuSettings,
    },
    RenderPlugin,
};

use material::{
    VoxelMaterial,
    VoxelMaterialResource,
};
use rand::{
    thread_rng,
    Rng,
};

#[derive(Resource)]
struct Settings {
    load_distance: usize,
}

fn main() {
    let args = argh::from_env::<args::Arguments>();
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
            present_mode: if args.enable_vsync {
                PresentMode::AutoVsync
            } else {
                PresentMode::AutoNoVsync
            },
            ..default()
        }),
        ..default()
    });

    app.add_plugins(default_plugins);

    embedded_asset!(app, "../assets/images/blocks.png");
    embedded_asset!(app, "../assets/images/crosshair.png");
    embedded_asset!(app, "../assets/images/toolbar.png");
    embedded_asset!(app, "../assets/images/selected.png");
    embedded_asset!(app, "../assets/shaders/voxel.wgsl");

    app.add_plugins(MaterialPlugin::<VoxelMaterial>::default());
    app.init_resource::<highlight::SelectedVoxel>();
    app.init_resource::<input::CameraVelocity>();
    app.init_resource::<input::InputState>();
    app.init_resource::<input::QuitCounter>();
    app.insert_resource(Settings {
        load_distance: args.load_distance,
    });

    app.add_systems(
        Update,
        terrain::generate_chunks.run_if(run_once().or_else(on_event::<player::PlayerMovedEvent>())),
    );

    if let Some(save) = &args.save_file {
        app.insert_resource(save::SaveData::from_file(save));
    }

    let seed = if let Some(seed) = args.seed {
        if args.save_file.is_some() {
            error!("Both `seed` and `load` are set");
            return;
        }
        seed
    } else {
        thread_rng().gen()
    };
    app.insert_resource(world::World::from_seed(seed));

    #[cfg(feature = "wireframe")]
    {
        app.add_plugins(WireframePlugin);
    }

    #[cfg(feature = "renderdoc")]
    {
        app.add_plugins(renderdoc::RenderDocPlugin);
    }

    app.add_systems(
        Startup,
        (make_camera, material::make_voxel_material, ui::draw_ui),
    )
    .add_systems(Update, material::process_block_texture)
    .add_systems(
        Update,
        (
            input::handle_lmb,
            input::handle_rmb,
            input::handle_movement_keys,
            input::handle_special_keys,
            input::player_look,
        )
            .in_set(input::InputSet),
    )
    .add_event::<input::SaveEvent>()
    .add_systems(
        PostUpdate,
        (
            queue_chunk_meshes,
            handle_mesh_tasks,
            world::process_save_events.run_if(on_event::<input::SaveEvent>()),
        ),
    )
    .add_event::<highlight::UpdateHighlightedEvent>()
    .add_systems(
        Update,
        highlight::update_selected_voxel.run_if(on_event::<highlight::UpdateHighlightedEvent>()),
    )
    .add_systems(Startup, input::hook_cursor)
    .add_systems(Update, input::player_look)
    .add_event::<player::PlayerMovedEvent>()
    .add_systems(Update, player::player_move.after(input::InputSet));

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
    commands.spawn(bundle);
}

#[derive(Component)]
struct ChunkMeshingTask(Task<CommandQueue>);

/// Marker component for chunks indicating they should be updated synchronously
/// (before the next frame)
#[derive(Component)]
struct UpdateSync;

fn queue_chunk_meshes(
    mut commands: Commands,
    dirty_chunks: Query<
        (Entity, &ChunkPosition, &ChunkVoxels, Option<&UpdateSync>),
        (Without<HasMesh>, Without<ChunkMeshingTask>),
    >,
    all_chunks: Query<&ChunkVoxels>,
    world: Res<world::World>,
) {
    info_once!("Started queuing chunk tasks");
    let task_pool = AsyncComputeTaskPool::get();
    let sync_task_pool = ComputeTaskPool::get();
    for (ent, chunk_pos, chunk, sync) in dirty_chunks
        .iter()
        .map(|(e, pos, c, sync)| (e, *pos, c.clone(), sync.is_some()))
    {
        // get all adjacent chunks
        let mut adj_chunks = Vec::with_capacity(4);
        for chunk_pos in chunk_pos.neighbouring_chunks().all() {
            let Some(chunk) = world
                .chunk_at(chunk_pos)
                .and_then(|e| all_chunks.get(e).ok().cloned())
            else {
                continue;
            };
            adj_chunks.push((chunk_pos, chunk));
        }

        let task = async move {
            let mesh = mesh::from_chunk((chunk_pos, chunk.clone()), adj_chunks);
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
                        transform: Transform::from_translation(chunk_pos.as_vec3()),
                        material,
                        ..default()
                    })
                    .insert(HasMesh)
                    .remove::<ChunkMeshingTask>();
            });
            cmd_queue
        };

        let task = if sync || chunk_pos.in_range_of_spawn(2) {
            sync_task_pool.spawn(task)
        } else {
            task_pool.spawn(task)
        };
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
