mod chunk;
mod mesh;
mod voxel;

#[cfg(feature = "debug")]
mod debug;
mod world;

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
            make_light,
            make_voxel_material,
            generate_chunks,
        ),
    )
    .add_systems(Update, generate_chunk_meshes)
    .add_systems(Update, update_selected_voxel);

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

fn make_voxel_material(mut commands: Commands, mut materials: ResMut<Assets<VoxelMaterial>>) {
    let handle = materials.add(VoxelMaterial {
        base_color: Srgba::GREEN.into(),
        light_color: Srgba::WHITE.into(),
        light_dir: Vec3::new(1.0, 1.0, 1.0),
        selected_voxel: Vec3::ZERO,
        has_selected: 0,
    });
    commands.insert_resource(VoxelMaterialResource { handle });
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
                .fill(Voxel::GRASS);
            world.add_chunk(pos, commands.spawn((Name::new("Chunk"), chunk)).id());
        }
    }
}

#[derive(Resource)]
struct VoxelMaterialResource {
    handle: Handle<VoxelMaterial>,
}

#[derive(AsBindGroup, Reflect, Asset, Debug, Clone)]
pub struct VoxelMaterial {
    #[uniform(0)]
    base_color: LinearRgba,
    #[uniform(1)]
    light_color: LinearRgba,
    #[uniform(2)]
    light_dir: Vec3,
    #[uniform(3)]
    selected_voxel: Vec3,
    #[uniform(4)]
    has_selected: u32,
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

#[derive(Resource, Default)]
struct SelectedVoxel(Option<IVec3>);

const SELECT_DISTANCE: usize = 32;

fn update_selected_voxel(
    world: Res<world::World>,
    mut selected: ResMut<SelectedVoxel>,
    player: Query<&Transform, (With<Camera>, Changed<Transform>)>,
    chunks: Query<&Chunk>,
    material_handle: Res<VoxelMaterialResource>,
    mut materials: ResMut<Assets<VoxelMaterial>>,
) {
    let Ok(player_trans) = player.get_single() else {
        return;
    };
    let pos = player_trans.translation;
    let direction = player_trans.forward().as_vec3().normalize();
    for step in 0..SELECT_DISTANCE {
        let check = (pos + direction * step as f32).as_ivec3();
        match world.voxel_at(check, &chunks) {
            Some(voxel) if voxel.should_mesh() => {
                selected.0 = Some(check);
                let mat = materials.get_mut(&material_handle.handle).unwrap();
                mat.has_selected = 1;
                mat.selected_voxel = check.as_vec3();
                return;
            }
            _ => continue,
        }
    }
    if selected.0.is_some() {
        let mat = materials.get_mut(&material_handle.handle).unwrap();
        mat.has_selected = 0;
        selected.0 = None;
    }
}
