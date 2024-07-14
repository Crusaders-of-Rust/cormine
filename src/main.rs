mod chunk;
mod mesh;
mod voxel;

#[cfg(feature = "debug")]
/// Debugging UI features
mod debug;

/// Keeps track of the whole world of chunks and voxels
mod world;

/// Handles defining and creating materials for rendering
mod material;

/// Handles finding the currently 'selected' voxel and highlighting it
mod highlight;
mod ui;

use chunk::{Chunk, CHUNK_SIZE};
use mesh::HasMesh;
use voxel::Voxel;

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
use ndarray::s as slice;

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

    #[cfg(feature = "wireframe")]
    {
        app.add_plugins(WireframePlugin);
    }

    app.add_systems(
        Startup,
        (
            make_camera,
            material::make_voxel_material,
            generate_chunks,
            ui::draw_ui,
        ),
    )
    .add_systems(Update, generate_chunk_meshes)
    .add_systems(Update, highlight::update_selected_voxel);

    #[cfg(feature = "flycam")]
    app.add_plugins(NoCameraPlayerPlugin);

    #[cfg(feature = "debug")]
    app.add_plugins(debug::DebugUiPlugins);

    app.run();
}

fn make_camera(mut commands: Commands) {
    let mut ent = commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-10.0, 4.5, -10.0).looking_at(
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
        ..default()
    });
    #[cfg(feature = "flycam")]
    ent.insert(FlyCam);
}

fn generate_chunks(mut commands: Commands, mut world: ResMut<world::World>) {
    for x in 0..8 {
        for z in 0..8 {
            let pos = IVec3 {
                x: x * CHUNK_SIZE as i32,
                y: 0,
                z: z * CHUNK_SIZE as i32,
            };
            let mut chunk = Chunk::new().with_position(pos);
            chunk
                .slice_mut(slice![0..CHUNK_SIZE, 0..x + z, 0..CHUNK_SIZE])
                .fill(Voxel::STONE);
            chunk
                .slice_mut(slice![0..CHUNK_SIZE, 5..x + z, 0..CHUNK_SIZE])
                .fill(Voxel::GRASS);
            world.add_chunk(pos, commands.spawn((Name::new("Chunk"), chunk)).id());
        }
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
