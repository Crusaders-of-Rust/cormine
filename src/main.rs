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

fn main() {
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
    app.init_resource::<input::JumpState>();

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
    .add_systems(Update, (input::check_input, generate_chunk_meshes))
    .add_systems(Update, highlight::update_selected_voxel);

    #[cfg(feature = "flycam")]
    app.add_plugins(NoCameraPlayerPlugin);

    #[cfg(not(feature = "flycam"))]
    {
        app.add_systems(Startup, input::hook_cursor);
        app.add_systems(Update, input::player_look);
        app.add_systems(Update, player::player_move);
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

/// Find any [`Chunk`]s which haven't yet had their meshes generated and add them.
fn generate_chunk_meshes(
    mut commands: Commands,
    query: Query<(Entity, &Chunk), Without<HasMesh>>,
    mut meshes: ResMut<Assets<Mesh>>,
    material: Res<VoxelMaterialResource>,
) {
    for (ent, chunk) in query.iter() {
        let mesh = mesh::from_chunk(chunk);
        commands
            .entity(ent)
            .insert(MaterialMeshBundle {
                mesh: meshes.add(mesh),
                transform: Transform::from_translation(chunk.position().as_vec3()),
                material: material.handle.clone(),
                ..default()
            })
            .insert(HasMesh);
    }
}
