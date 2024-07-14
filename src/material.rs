use crate::mesh;
use bevy::asset::{Asset, Assets, Handle};
use bevy::color::{LinearRgba, Srgba};
use bevy::math::Vec3;
use bevy::pbr::{Material, MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::mesh::MeshVertexBufferLayoutRef;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
};

pub fn make_voxel_material(mut commands: Commands, mut materials: ResMut<Assets<VoxelMaterial>>) {
    let handle = materials.add(VoxelMaterial {
        light_color: Srgba::WHITE.into(),
        light_dir: Vec3::new(1.0, 1.0, 1.0),
        selected_voxel: Vec3::ZERO,
        has_selected: 0,
    });
    commands.insert_resource(VoxelMaterialResource { handle });
}

#[derive(Resource)]
pub struct VoxelMaterialResource {
    pub(crate) handle: Handle<VoxelMaterial>,
}

#[derive(AsBindGroup, Reflect, Asset, Debug, Clone)]
pub struct VoxelMaterial {
    #[uniform(1)]
    light_color: LinearRgba,
    #[uniform(2)]
    light_dir: Vec3,
    #[uniform(3)]
    pub selected_voxel: Vec3,
    #[uniform(4)]
    pub has_selected: u32,
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
