use bevy::{
    pbr::{
        NotShadowCaster,
        NotShadowReceiver,
    },
    prelude::*,
};

use crate::material::SunMaterial;

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
    mut sun: Query<&mut Transform, (With<Sun>, Without<Camera>)>,
    player: Query<&Transform, With<Camera>>,
) {
    let mut sun = sun.single_mut();
    let player = player.single();
    sun.translation = player.translation + Vec3::new(0.0, 50.0, 1000.0);
}
