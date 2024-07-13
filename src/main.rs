mod chunk;
mod mesh;
mod voxel;

use bevy::color::palettes::css::WHITE;
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use chunk::Chunk;
use mesh::HasMesh;
use voxel::Voxel;

use bevy::prelude::*;
#[cfg(feature = "wireframe")]
use bevy::render::{
    settings::{RenderCreation, WgpuFeatures, WgpuSettings},
    RenderPlugin,
};
#[cfg(feature = "flycam")]
use bevy_flycam::prelude::*;

use crate::chunk::CHUNK_SIZE;
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
            global: true,
            default_color: WHITE.into(),
        });
    }

    app.add_plugins(default_plugins);

    #[cfg(feature = "wireframe")]
    {
        app.add_plugins(WireframePlugin);
    }

    app.add_systems(Startup, generate_chunks)
        .add_systems(Update, generate_chunk_meshes);

    #[cfg(feature = "flycam")]
    app.add_plugins(PlayerPlugin);

    #[cfg(not(feature = "flycam"))]
    app.add_systems(Startup, make_camera);

    #[cfg(feature = "dev_tools")]
    app.add_plugins(bevy::dev_tools::fps_overlay::FpsOverlayPlugin::default());

    app.run();
}

#[cfg(not(feature = "flycam"))]
fn make_camera(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(
            Vec3 {
                y: 4.5,
                ..default()
            },
            Vec3::Y,
        ),
        ..default()
    });
}

fn generate_chunks(mut commands: Commands) {
    for x in 0..8 {
        for z in 0..8 {
            let mut chunk = Chunk::new().with_position(IVec3 {
                x: x * CHUNK_SIZE as i32,
                y: 0,
                z: z * CHUNK_SIZE as i32,
            });
            chunk
                .slice_mut(slice![0..chunk::CHUNK_SIZE, 0..x + z, 0..chunk::CHUNK_SIZE])
                .fill(Voxel::GRASS);
            commands.spawn((Name::new("Chunk"), chunk));
        }
    }
}

fn generate_chunk_meshes(
    mut commands: Commands,
    mut query: Query<(Entity, &Chunk), (Without<HasMesh>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let green_material = materials.add(StandardMaterial {
        base_color: Color::Srgba(Srgba::GREEN),
        ..default()
    });
    for (ent, chunk) in query.iter() {
        let mesh = mesh::from_chunk(chunk);
        commands
            .entity(ent)
            .insert(MaterialMeshBundle {
                mesh: meshes.add(mesh),
                transform: Transform::from_translation(chunk.position().as_vec3()),
                material: green_material.clone(),
                ..default()
            })
            .insert(HasMesh);
    }
}
