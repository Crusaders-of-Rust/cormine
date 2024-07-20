use crate::chunk::Chunk;
use crate::highlight::SelectedVoxel;
use crate::mesh::HasMesh;
use crate::world;
use bevy::prelude::*;

pub fn check_input(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    selected: ResMut<SelectedVoxel>,
    world: Res<world::World>,
    mut chunks: Query<&mut Chunk>,
) {
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
    }
}
