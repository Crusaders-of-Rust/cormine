mod chunk;
mod mesh;
mod voxel;

#[cfg(feature = "debug")]
mod debug;

use chunk::{Chunk, CHUNK_SIZE};
use mesh::HasMesh;
use voxel::Voxel;

use bevy::color::palettes::css::WHITE;
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
};
use bevy::window::PresentMode;

use bevy::render::mesh::MeshVertexBufferLayoutRef;
#[cfg(feature = "wireframe")]
use bevy::render::{
    settings::{RenderCreation, WgpuFeatures, WgpuSettings},
    RenderPlugin,
};
#[cfg(feature = "flycam")]
use bevy_flycam::prelude::*;

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

    #[cfg(feature = "wireframe")]
    {
        app.add_plugins(WireframePlugin);
    }

    app.add_systems(Startup, (make_camera, make_light, generate_chunks))
        .add_systems(Update, generate_chunk_meshes);

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
        ..default()
    });
    #[cfg(feature = "flycam")]
    ent.insert(FlyCam);
}

fn make_light(mut commands: Commands) {
    commands.spawn(DirectionalLightBundle {
        transform: Transform::IDENTITY.looking_to(Vec3::new(-1.0, -0.6, -1.0), Vec3::Y),
        directional_light: DirectionalLight {
            color: Color::WHITE,
            shadows_enabled: true,
            ..default()
        },
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
                .slice_mut(slice![0..CHUNK_SIZE, 0..x + z, 0..CHUNK_SIZE])
                .fill(Voxel::GRASS);
            commands.spawn((Name::new("Chunk"), chunk));
        }
    }
}

#[derive(AsBindGroup, Reflect, Asset, Debug, Clone)]
pub struct VoxelMaterial {
    #[uniform(0)]
    base_color: LinearRgba,
    #[uniform(1)]
    light_color: LinearRgba,
    #[uniform(2)]
    light_dir: Vec3,
}

impl Material for VoxelMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/voxel.wgsl".into()
    }
    fn fragment_shader() -> ShaderRef {
        "shaders/voxel.wgsl".into()
    }
    fn specialize(
        _: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vtx_layout = layout.0.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            mesh::VOXEL_VERTEX_DATA.at_shader_location(1),
        ])?;
        descriptor.vertex.buffers = vec![vtx_layout];
        Ok(())
    }
}

fn generate_chunk_meshes(
    mut commands: Commands,
    query: Query<(Entity, &Chunk), Without<HasMesh>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<VoxelMaterial>>,
) {
    let voxel_material = materials.add(VoxelMaterial {
        base_color: Srgba::GREEN.into(),
        light_color: Srgba::WHITE.into(),
        light_dir: Vec3::new(1.0, 1.0, 1.0),
    });
    for (ent, chunk) in query.iter() {
        let mesh = mesh::from_chunk(chunk);
        commands
            .entity(ent)
            .insert(MaterialMeshBundle {
                mesh: meshes.add(mesh),
                transform: Transform::from_translation(chunk.position().as_vec3()),
                material: voxel_material.clone(),
                ..default()
            })
            .insert(HasMesh);
    }
}
