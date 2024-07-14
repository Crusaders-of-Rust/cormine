use bevy::prelude::*;

use crate::chunk::Chunk;
use crate::material::{VoxelMaterial, VoxelMaterialResource};
use crate::world;

#[derive(Resource, Default)]
pub struct SelectedVoxel(pub Option<IVec3>);

const SELECT_DISTANCE: usize = 32;

pub fn update_selected_voxel(
    world: Res<world::World>,
    mut selected: ResMut<SelectedVoxel>,
    player: Query<&Transform, With<Camera>>,
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
