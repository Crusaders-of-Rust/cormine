use crate::chunk::Chunk;
use crate::highlight::SelectedVoxel;
use crate::mesh::HasMesh;
use crate::voxel::VoxelKind;
use crate::world;
use bevy::prelude::*;

use bevy::input::mouse::MouseMotion;
use bevy::window::{CursorGrabMode, PrimaryWindow};

#[derive(Resource, Default)]
pub struct CameraVelocity {
    pub vel: Vec3,
}
#[derive(Resource, Default)]
pub struct JumpState {
    pub pressed: bool,
    pub holding: bool,
}

pub fn hook_cursor(mut qwindow: Query<&mut Window, With<PrimaryWindow>>) {
    let window = &mut qwindow.single_mut();
    window.cursor.grab_mode = CursorGrabMode::Confined;
    window.cursor.visible = false;
}

pub fn player_look(
    qwindow: Query<&Window, With<PrimaryWindow>>,
    mut mouse: EventReader<MouseMotion>,
    mut camera_transform: Query<&mut Transform, With<Camera>>,
) {
    let window = qwindow.single();
    let mut camera_transform = camera_transform.single_mut();

    for ev in mouse.read() {
        let (mut yaw, mut pitch, mut _roll) = camera_transform.rotation.to_euler(EulerRot::YXZ);
        if window.cursor.grab_mode != CursorGrabMode::None {
            yaw -= ev.delta.x * 0.002;
            pitch -= ev.delta.y * 0.002;
            pitch = pitch.clamp(-1.54, 1.54);
            camera_transform.rotation =
                Quat::from_axis_angle(Vec3::Y, yaw) * Quat::from_axis_angle(Vec3::X, pitch);
        }
    }
}

pub fn check_input(
    mut qwindow: Query<&mut Window, With<PrimaryWindow>>,
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    selected: ResMut<SelectedVoxel>,
    world: Res<world::World>,
    mut chunks: Query<&mut Chunk>,
    mut camera_velocity: ResMut<CameraVelocity>,
    mut jump_state: ResMut<JumpState>,
    camera_transform: Query<&Transform, With<Camera>>,
) {
    let window = &mut qwindow.single_mut();
    let camera_transform = camera_transform.single();

    if buttons.just_pressed(MouseButton::Left) {
        if let Some(selected_voxel) = selected.0 {
            let chunk = world
                .chunk_containing(selected_voxel)
                .expect("Selected voxel is not in a chunk");
            let mut chunk_data = chunks.get_mut(chunk).expect("Chunk does not exist");
            let voxel = chunk_data.voxel_mut(selected_voxel.into());
            voxel.clear();
            commands.entity(chunk).remove::<HasMesh>();
        }

        #[cfg(not(feature = "flycam"))]
        {
            if window.cursor.grab_mode == CursorGrabMode::None {
                window.cursor.grab_mode = CursorGrabMode::Confined;
                window.cursor.visible = false;
            }
        }
    }

    if buttons.just_pressed(MouseButton::Right) {
        if let Some(selected_voxel) = selected.1 {
            let chunk = world
                .chunk_containing(selected_voxel)
                .expect("Selected voxel is not in a chunk");
            let mut chunk_data = chunks.get_mut(chunk).expect("Chunk does not exist");
            let voxel = chunk_data.voxel_mut(selected_voxel.into());
            voxel.kind = VoxelKind::Stone;
            commands.entity(chunk).remove::<HasMesh>();
        }
    }

    #[cfg(not(feature = "flycam"))]
    {
        if window.cursor.grab_mode == CursorGrabMode::Confined {
            if keys.just_pressed(KeyCode::Escape) {
                window.cursor.grab_mode = CursorGrabMode::None;
                window.cursor.visible = true;
            }

            let camera_velocity = &mut camera_velocity.vel;
            let looking_at = camera_transform.local_z();
            let camera_forward = -Vec3::new(looking_at.x, 0., looking_at.z);
            let camera_right = Vec3::new(looking_at.z, 0., -looking_at.x);

            jump_state.holding = false;
            for key in keys.get_pressed() {
                if *key == KeyCode::KeyW {
                    *camera_velocity += 7.5 * camera_forward;
                } else if *key == KeyCode::KeyS {
                    *camera_velocity -= 7.5 * camera_forward;
                } else if *key == KeyCode::KeyA {
                    *camera_velocity -= 7.5 * camera_right;
                } else if *key == KeyCode::KeyD {
                    *camera_velocity += 7.5 * camera_right;
                } else if *key == KeyCode::Space {
                    jump_state.holding = true;
                }
            }
            jump_state.pressed = keys.just_pressed(KeyCode::Space);
        }
    }
}
