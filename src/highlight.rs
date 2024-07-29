use std::f32::EPSILON;

use bevy::prelude::*;

use crate::{
    chunk::ChunkVoxels,
    material::{
        VoxelMaterial,
        VoxelMaterialResource,
    },
    voxel::VoxelPosition,
    world,
};

#[derive(Resource, Default)]
pub struct SelectedVoxel {
    pub to_break: Option<VoxelPosition>,
    pub to_place: Option<VoxelPosition>,
}

/// Emitted when the player's highlighted block should be updated
#[derive(Event)]
pub struct UpdateHighlightedEvent;

const SELECT_DISTANCE: f32 = 16.0;

fn draw_line(start: Vec3, direction: Vec3, distance: f32) -> impl Iterator<Item = IVec3> {
    let end_pos = start + direction * distance;

    let mut voxel = start.floor();
    let end_voxel = end_pos.floor();

    let step = direction.signum();
    let t_delta = direction.recip().abs();

    let mask = Vec3::from(step.cmpge(Vec3::ZERO));
    let mut t_max = ((voxel - start + mask) / direction).abs();

    std::iter::from_fn(move || {
        if Vec3::abs_diff_eq(voxel, end_voxel, EPSILON) {
            return None;
        };
        if t_max.x < t_max.y {
            if t_max.x < t_max.z {
                voxel.x += step.x;
                t_max.x += t_delta.x;
            } else {
                voxel.z += step.z;
                t_max.z += t_delta.z;
            }
        } else {
            if t_max.y < t_max.z {
                voxel.y += step.y;
                t_max.y += t_delta.y;
            } else {
                voxel.z += step.z;
                t_max.z += t_delta.z;
            }
        }
        Some(voxel.as_ivec3())
    })
}

pub fn update_selected_voxel(
    world: Res<world::World>,
    mut selected: ResMut<SelectedVoxel>,
    player: Query<&Transform, With<Camera>>,
    chunks: Query<&ChunkVoxels>,
    material_handle: Res<VoxelMaterialResource>,
    mut materials: ResMut<Assets<VoxelMaterial>>,
) {
    let player_trans = player.get_single().expect("expected player object");
    let pos = player_trans.translation;
    let direction = player_trans.forward().as_vec3().normalize();

    let mut prev = None;
    for pos in draw_line(pos, direction, SELECT_DISTANCE) {
        let voxel_pos = VoxelPosition::new(pos);
        match world.voxel_at(voxel_pos, &chunks) {
            Some(voxel) if voxel.should_mesh() => {
                selected.to_break = Some(voxel_pos);
                let mat = materials.get_mut(&material_handle.handle).unwrap();
                mat.has_selected = 1;
                mat.selected_voxel = voxel_pos.as_vec3();
                selected.to_place = prev.map(VoxelPosition::new);
                return;
            }
            _ => (),
        }
        prev = Some(pos);
    }

    if selected.to_break.is_some() {
        let mat = materials.get_mut(&material_handle.handle).unwrap();
        mat.has_selected = 0;
    }
    selected.to_break = None;
}
