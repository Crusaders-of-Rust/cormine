use crate::chunk::Chunk;
use crate::highlight::SelectedVoxel;
use crate::mesh::HasMesh;
use crate::voxel::{VoxelKind, VoxelPosition};
use crate::world;
use bevy::prelude::*;

use bevy::input::mouse::MouseMotion;
use bevy::window::{CursorGrabMode, PrimaryWindow};

#[derive(Resource, Default)]
pub struct CameraVelocity {
    pub vel: Vec3,
}
#[derive(Resource, Default)]
pub struct InputState {
    pub space_pressed: bool,
    pub space_held: bool,
    pub shift_held: bool,
    pub fly_hack: bool,
    pub selected_voxel: u8,
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
    mut input_state: ResMut<InputState>,
    camera_transform: Query<&Transform, With<Camera>>,
    mut selected_pos: Query<(&mut crate::ui::SelectedPosition, &mut Style)>,
) {
    let window = &mut qwindow.single_mut();
    let camera_transform = camera_transform.single();

    if buttons.just_pressed(MouseButton::Left) {
        if let Some(selected_voxel) = selected.to_break {
            let chunk = world
                .chunk_containing(selected_voxel)
                .expect("Selected voxel is not in a chunk");
            let mut chunk_data = chunks.get_mut(chunk).expect("Chunk does not exist");
            let voxel = chunk_data.voxel_mut(selected_voxel.into());
            voxel.clear();

            commands.entity(chunk).remove::<HasMesh>();
            // clear HasMesh flag from potential adjacent chunk
            // check if selected voxel is on chunk boundary
            if selected_voxel.x() % 16 == 0 {
                let adj_pos = chunk_data.position().as_ivec3() + IVec3::new(-16, 0, 0);
                if let Some(adj_chunk) = world.chunk_containing(VoxelPosition::new(adj_pos)) {
                    commands.entity(adj_chunk).remove::<HasMesh>();
                }
            }
            if selected_voxel.x() % 16 == 15 {
                let adj_pos = chunk_data.position().as_ivec3() + IVec3::new(16, 0, 0);
                if let Some(adj_chunk) = world.chunk_containing(VoxelPosition::new(adj_pos)) {
                    commands.entity(adj_chunk).remove::<HasMesh>();
                }
            }
            if selected_voxel.z() % 16 == 0 {
                let adj_pos = chunk_data.position().as_ivec3() + IVec3::new(0, 0, -16);
                if let Some(adj_chunk) = world.chunk_containing(VoxelPosition::new(adj_pos)) {
                    commands.entity(adj_chunk).remove::<HasMesh>();
                }
            }
            if selected_voxel.z() % 16 == 15 {
                let adj_pos = chunk_data.position().as_ivec3() + IVec3::new(0, 0, 16);
                if let Some(adj_chunk) = world.chunk_containing(VoxelPosition::new(adj_pos)) {
                    commands.entity(adj_chunk).remove::<HasMesh>();
                }
            }
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
        if let Some(selected_voxel) = selected.to_place {
            let chunk = world
                .chunk_containing(selected_voxel)
                .expect("Selected voxel is not in a chunk");
            let mut chunk_data = chunks.get_mut(chunk).expect("Chunk does not exist");
            let voxel = chunk_data.voxel_mut(selected_voxel.into());
            voxel.kind = match input_state.selected_voxel {
                0 => VoxelKind::Stone,
                1 => VoxelKind::Grass,
                2 => VoxelKind::Water,
                3 => VoxelKind::Snow,
                4 => VoxelKind::Dirt,
                _ => panic!("Invalid selected voxel"),
            };

            commands.entity(chunk).remove::<HasMesh>();
            // clear HasMesh flag from potential adjacent chunk
            // check if selected voxel is on chunk boundary
            if selected_voxel.x() % 16 == 0 {
                let adj_pos = chunk_data.position().as_ivec3() + IVec3::new(-16, 0, 0);
                if let Some(adj_chunk) = world.chunk_containing(VoxelPosition::new(adj_pos)) {
                    commands.entity(adj_chunk).remove::<HasMesh>();
                }
            }
            if selected_voxel.x() % 16 == 15 {
                let adj_pos = chunk_data.position().as_ivec3() + IVec3::new(16, 0, 0);
                if let Some(adj_chunk) = world.chunk_containing(VoxelPosition::new(adj_pos)) {
                    commands.entity(adj_chunk).remove::<HasMesh>();
                }
            }
            if selected_voxel.z() % 16 == 0 {
                let adj_pos = chunk_data.position().as_ivec3() + IVec3::new(0, 0, -16);
                if let Some(adj_chunk) = world.chunk_containing(VoxelPosition::new(adj_pos)) {
                    commands.entity(adj_chunk).remove::<HasMesh>();
                }
            }
            if selected_voxel.z() % 16 == 15 {
                let adj_pos = chunk_data.position().as_ivec3() + IVec3::new(0, 0, 16);
                if let Some(adj_chunk) = world.chunk_containing(VoxelPosition::new(adj_pos)) {
                    commands.entity(adj_chunk).remove::<HasMesh>();
                }
            }
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
            let camera_forward = camera_transform.forward().with_y(0.0);
            let camera_right = camera_transform.right().with_y(0.0);

            let mut speed_factor = if keys.pressed(KeyCode::ControlLeft) {
                12.5
            } else {
                7.5
            };

            input_state.space_held = keys.pressed(KeyCode::Space);
            input_state.shift_held = keys.pressed(KeyCode::ShiftLeft);
            input_state.space_pressed = keys.just_pressed(KeyCode::Space);

            // TODO: make this cheat only?
            if keys.just_pressed(KeyCode::KeyF) {
                input_state.fly_hack = !input_state.fly_hack;
            }
            if input_state.fly_hack {
                speed_factor *= 2.0;
            }

            for key in keys.get_pressed() {
                if *key == KeyCode::KeyW {
                    *camera_velocity += speed_factor * camera_forward;
                } else if *key == KeyCode::KeyS {
                    *camera_velocity -= speed_factor * camera_forward;
                } else if *key == KeyCode::KeyA {
                    *camera_velocity -= speed_factor * camera_right;
                } else if *key == KeyCode::KeyD {
                    *camera_velocity += speed_factor * camera_right;
                }
            }
        }
    }

    if keys.just_pressed(KeyCode::Digit1) {
        input_state.selected_voxel = 0;
    } else if keys.just_pressed(KeyCode::Digit2) {
        input_state.selected_voxel = 1;
    } else if keys.just_pressed(KeyCode::Digit3) {
        input_state.selected_voxel = 2;
    } else if keys.just_pressed(KeyCode::Digit4) {
        input_state.selected_voxel = 3;
    } else if keys.just_pressed(KeyCode::Digit5) {
        input_state.selected_voxel = 4;
    }

    let mut selected_pos = selected_pos.single_mut();
    selected_pos.1.margin.left = Val::Px((input_state.selected_voxel as f32 - 2.0) * 156.0 + 8.0);
}
