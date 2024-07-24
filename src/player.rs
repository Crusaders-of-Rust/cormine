use crate::{
    chunk::Chunk,
    input::{
        CameraVelocity,
        InputState,
    },
    ui,
    voxel::{
        Voxel,
        VoxelKind,
        VoxelPosition,
    },
    world::World,
};
use bevy::prelude::*;

const GRAVITY: f32 = 40.0;
const JUMP_VELOCITY: f32 = 10.0;
const FLY_SPEED_VERTICAL: f32 = 15.0;
const WATER_ACCELERATION: f32 = 0.5;
const MAX_SWIM_UP_SPEED: f32 = 4.0;
const PLAYER_HEIGHT: i32 = 2;

pub fn player_move(
    mut camera_velocity: ResMut<CameraVelocity>,
    mut camera_transform: Query<&mut Transform, With<Camera>>,
    time: Res<Time>,
    world: Res<World>,
    chunks: Query<&Chunk>,
    input_state: Res<InputState>,
    mut color_overlay: Query<&mut BackgroundColor, With<ui::ColorOverlay>>,
) {
    let vel = &mut camera_velocity.vel;
    let pos: &mut Vec3 = &mut camera_transform.single_mut().translation;
    let get_voxel = |player_pos: Vec3, offset: IVec3| -> Option<&Voxel> {
        let check_pos = player_pos.as_ivec3() + offset + IVec3::new(0, -PLAYER_HEIGHT, 0);
        let voxel_pos = VoxelPosition::new(check_pos);

        let chunk_ent = world.chunk_containing(voxel_pos)?;

        let chunk = chunks.get(chunk_ent).unwrap();
        Some(chunk.voxel(voxel_pos.into()))
    };

    let is_in_water = get_voxel(*pos, IVec3::new(0, -1, 0))
        .map_or(false, |voxel| matches!(voxel.kind, VoxelKind::Water));

    if !input_state.fly_hack {
        let (g_accel, max_vel) = if is_in_water {
            (10.0, -10.0)
        } else {
            (GRAVITY, -30.0)
        };
        vel.y = (vel.y - g_accel * time.delta_seconds()).max(max_vel);
    }

    *pos += *vel * time.delta_seconds();

    // velocity decay
    vel.x *= 0.9 * (time.delta_seconds() / 1000.0);
    vel.z *= 0.9 * (time.delta_seconds() / 1000.0);

    let has_collision = |player_pos: Vec3, offset: IVec3| -> bool {
        let Some(voxel) = get_voxel(player_pos, offset) else {
            return false;
        };

        voxel.has_collision()
    };

    let is_on_ground = has_collision(*pos, IVec3::new(0, -1, 0));

    if input_state.fly_hack {
        vel.y = if input_state.space_held {
            FLY_SPEED_VERTICAL
        } else if input_state.shift_held {
            -FLY_SPEED_VERTICAL
        } else {
            0.0
        };
    } else {
        // allow jumping if on ground or in water
        if input_state.space_pressed && is_on_ground {
            vel.y = JUMP_VELOCITY;
        }
        // Swim up, capping at a maximum speed
        if input_state.space_held && is_in_water {
            vel.y = (vel.y + WATER_ACCELERATION).clamp(-5.0, MAX_SWIM_UP_SPEED);
        }
    }

    let mut water_overlay = color_overlay.single_mut();
    let is_head_in_water = get_voxel(*pos, IVec3::new(0, PLAYER_HEIGHT, 0))
        .map_or(false, |voxel| matches!(voxel.kind, VoxelKind::Water));
    const WATER_OVERLAY_COLOR: Color = Color::linear_rgba(0.0, 0.0, 0.5, 0.5);
    if is_head_in_water {
        water_overlay.0 = WATER_OVERLAY_COLOR
    } else if water_overlay.0 == WATER_OVERLAY_COLOR {
        water_overlay.0 = Color::NONE;
    }

    // snap to ground
    if vel.y < 0.0 && is_on_ground {
        vel.y = 0.0;
        pos.y = (pos.y + 0.1).floor();
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
        pos.x = (pos.x - 0.1).ceil();
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
        pos.z = (pos.z - 0.1).ceil();
        vel.z = 0.0;
    }
}
