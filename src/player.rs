use crate::{
    chunk::Chunk,
    input::{CameraVelocity, InputState},
    voxel::{Voxel, VoxelKind, VoxelPosition},
    world::World,
};
use bevy::prelude::*;

const GRAVITY: f32 = 40.0;
const JUMP_VELOCITY: f32 = 10.0;

pub fn player_move(
    mut camera_velocity: ResMut<CameraVelocity>,
    mut camera_transform: Query<&mut Transform, With<Camera>>,
    time: Res<Time>,
    world: Res<World>,
    chunks: Query<&Chunk>,
    input_state: Res<InputState>,
    mut water_overlay: Query<(&mut crate::ui::WaterOverlay, &mut BackgroundColor)>,
) {
    let vel = &mut camera_velocity.vel;
    let pos: &mut Vec3 = &mut camera_transform.single_mut().translation;

    if !input_state.fly_hack {
        vel.y -= GRAVITY * time.delta_seconds();
    }

    *pos += *vel * time.delta_seconds();

    // velocity decay
    vel.x *= 0.9 * (time.delta_seconds() / 1000.0);
    vel.z *= 0.9 * (time.delta_seconds() / 1000.0);

    let get_voxel = |player_pos: Vec3, offset: IVec3| -> Option<&Voxel> {
        let check_pos = player_pos.as_ivec3() + offset + IVec3::new(0, -2, 0);
        let voxel_pos = VoxelPosition::new(check_pos);

        let chunk_ent = world.chunk_containing(voxel_pos)?;

        let chunk = chunks.get(chunk_ent).unwrap();
        Some(chunk.voxel(voxel_pos.into()))
    };

    let has_collision = |player_pos: Vec3, offset: IVec3| -> bool {
        let Some(voxel) = get_voxel(player_pos, offset) else {
            return false;
        };

        voxel.has_collision()
    };

    let is_on_ground = has_collision(*pos, IVec3::new(0, 0, 0));
    let is_in_water = get_voxel(*pos, IVec3::new(0, -1, 0))
        .map_or(false, |voxel| matches!(voxel.kind, VoxelKind::Water));

    // snap to ground
    if vel.y < 0.0 && is_on_ground {
        vel.y = 0.0;
        pos.y = (pos.y + 0.1).floor();
    }

    if input_state.fly_hack {
        vel.y = if input_state.space_held {
            15.0
        } else if input_state.shift_held {
            -15.0
        } else {
            0.0
        };
    } else {
        // allow jumping if on ground or in water
        if input_state.space_pressed && is_on_ground {
            vel.y = JUMP_VELOCITY;
        }
        if input_state.space_held && is_in_water {
            vel.y += 0.5;
        }

        // cap vertical speed in water
        if is_in_water {
            vel.y = vel.y.clamp(-5.0, 2.0);
        }
    }

    let mut water_overlay = water_overlay.single_mut();
    let is_head_in_wate = get_voxel(*pos, IVec3::new(0, 2, 0))
        .map_or(false, |voxel| matches!(voxel.kind, VoxelKind::Water));
    if is_head_in_wate {
        water_overlay.1 .0 = Color::linear_rgba(0.0, 0.0, 0.5, 0.5);
    } else {
        water_overlay.1 .0 = Color::NONE;
    }

    // Collision above head
    if vel.y > 0.0 && has_collision(*pos, IVec3::new(0, 2, 0)) {
        pos.y = pos.y.floor();
        vel.y = 0.0;
    }

    // Collision in 4 cardinal directions
    if vel.x > 0.0
        && (has_collision(*pos, IVec3::new(1, 0, 0)) || has_collision(*pos, IVec3::new(1, 1, 0)))
    {
        pos.x = (pos.x + 0.1).floor();
        vel.x = 0.0;
    }
    if vel.x < 0.0
        && (has_collision(*pos, IVec3::new(-1, 0, 0)) || has_collision(*pos, IVec3::new(-1, 1, 0)))
    {
        pos.x = (pos.x + 0.1).floor();
        vel.x = 0.0;
    }

    if vel.z > 0.0
        && (has_collision(*pos, IVec3::new(0, 0, 1)) || has_collision(*pos, IVec3::new(0, 1, 1)))
    {
        pos.z = (pos.z + 0.1).floor();
        vel.z = 0.0;
    }
    if vel.z < 0.0
        && (has_collision(*pos, IVec3::new(0, 0, -1)) || has_collision(*pos, IVec3::new(0, 1, -1)))
    {
        pos.z = (pos.z + 0.1).floor();
        vel.z = 0.0;
    }
}
