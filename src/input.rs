use crate::chunk::{Chunk, CHUNK_SIZE};
use crate::highlight::SelectedVoxel;
use crate::mesh::HasMesh;
use crate::world;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};

pub fn check_input(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    selected: ResMut<SelectedVoxel>,
    world: Res<world::World>,
    mut chunks: Query<&mut Chunk>,
    mut q_windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        if let Some(selected_voxel) = selected.0 {
            let chunk = world
                .chunk_containing(selected_voxel)
                .expect("Selected voxel is not in a chunk");
            let mut chunk_data = chunks.get_mut(chunk).expect("Chunk does not exist");
            let voxel = chunk_data.voxel_mut(
                selected_voxel
                    .rem_euclid(IVec3::new(
                        CHUNK_SIZE as _,
                        CHUNK_SIZE as _,
                        CHUNK_SIZE as _,
                    ))
                    .to_array()
                    .map(|i| i as usize),
            );
            voxel.clear();
            commands.entity(chunk).remove::<HasMesh>();
        }
    }

    // lock cursor to window
    let mut primary_window = q_windows.single_mut();
    primary_window.cursor.visible = true;
    primary_window.cursor.grab_mode = CursorGrabMode::Locked;
}
