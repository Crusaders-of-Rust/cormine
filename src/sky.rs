use bevy::{
    pbr::{
        NotShadowCaster,
        NotShadowReceiver,
    },
    prelude::*,
};

use crate::material::{
    SunMaterial,
    VoxelMaterial,
    VoxelMaterialResource,
};

#[derive(Component)]
pub struct Sun;

pub fn add_sun(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SunMaterial>>,
) {
    commands
        .spawn(MaterialMeshBundle {
            mesh: meshes.add(Cuboid::new(80.0, 80.0, 1.0)),
            material: materials.add(SunMaterial {
                color: Color::WHITE.into(),
            }),
            ..default()
        })
        .insert(Sun)
        .insert((NotShadowReceiver, NotShadowCaster));
}

pub fn update_sun_position(
    time: Res<Time>,
    mut sun: Query<&mut Transform, (With<Sun>, Without<Camera>)>,
    player: Query<&Transform, With<Camera>>,
    material_handle: Res<VoxelMaterialResource>,
    mut materials: ResMut<Assets<VoxelMaterial>>,
) {
    let mut sun = sun.single_mut();
    let player = player.single();
    let mut sun_dir = Transform::from_translation(Vec3::new(0.0, 1.0, 1.0));
    // Rotate around the 'planet' every minute
    sun_dir.rotate_x(f32::to_radians((time.elapsed_seconds() + 15.0) * 6.0 * 3.0));
    sun.translation = player.translation + sun_dir.forward().as_vec3() * 1000.0;
    let up = sun.up();
    sun.look_at(player.translation, up);

    // FIXME: We don't really need to update this every frame
    let material = materials.get_mut(&material_handle.handle).unwrap();
    material.set_light_dir(sun_dir.forward().as_vec3());
}
