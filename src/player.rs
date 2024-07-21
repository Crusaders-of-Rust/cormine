use bevy::prelude::*;
use crate::{chunk::Chunk, input::{CameraVelocity, JumpState}, voxel::{Voxel, VoxelKind, VoxelPosition}, world::World};

pub fn player_move(
    mut camera_velocity: ResMut<CameraVelocity>,
    mut camera_transform: Query<&mut Transform, With<Camera>>,
    time: Res<Time>,
    world: Res<World>,
    chunks: Query<&Chunk>,
    jump_state: Res<JumpState>
) {
    let vel = &mut camera_velocity.vel;
    let pos: &mut Vec3 = &mut camera_transform.single_mut().translation;

    // apply gravity
    vel.y -= 0.01;

    *pos += *vel * time.delta_seconds();

    // velocity decay
    vel.x *= 0.9 * (time.delta_seconds() / 1000.0);
    vel.z *= 0.9 * (time.delta_seconds() / 1000.0);

    let get_voxel = |player_pos: Vec3, offset: IVec3| -> Option<&Voxel> {
        let check_pos = player_pos.as_ivec3() + offset + IVec3::new(0, -2, 0);
        let voxel_pos = VoxelPosition::new(check_pos);

        let Some(chunk_ent) = world.chunk_containing(voxel_pos) else {
            return None;
        };
        
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
    let is_in_water = get_voxel(*pos, IVec3::new(0, -1, 0)).map_or(false, |voxel| matches!(voxel.kind, VoxelKind::Water));
    
    // snap to ground
    if vel.y < 0.0 && is_on_ground {
        vel.y = 0.0;
        pos.y = (pos.y + 0.05).floor();
    }

    // allow jumping if on ground or in water
    if jump_state.pressed && is_on_ground {
        vel.y = 5.0;
    }
    if jump_state.holding && is_in_water {
        vel.y += 0.5;
    }

    // cap vertical speed in water
    if vel.y < -5.0 && is_in_water {
        vel.y = -5.0;
    }
    if vel.y > 2.0 && is_in_water {
        vel.y = 2.0;
    }

    if vel.x > 0.0 && (has_collision(*pos, IVec3::new(1, 0, 0)) || has_collision(*pos, IVec3::new(1, 1, 0))) {
        pos.x = (pos.x + 0.1).floor();
        vel.x = 0.0;
    } 
    if vel.x < 0.0 && (has_collision(*pos, IVec3::new(-1, 0, 0)) || has_collision(*pos, IVec3::new(-1, 1, 0))) {
        pos.x = (pos.x + 0.1).floor();
        vel.x = 0.0;
    }

    if vel.z > 0.0 && (has_collision(*pos, IVec3::new(0, 0, 1)) || has_collision(*pos, IVec3::new(0, 1, 1))) {
        pos.z = (pos.z + 0.1).floor();
        vel.z = 0.0;
    }
    if vel.z < 0.0 && (has_collision(*pos, IVec3::new(0, 0, -1)) || has_collision(*pos, IVec3::new(0, 1, -1))) {
        pos.z = (pos.z + 0.1).floor();
        vel.z = 0.0;
    }
}