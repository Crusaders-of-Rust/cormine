use std::cmp::Ordering;

use crate::{
    chunk::{
        Chunk,
        ChunkPosition,
        CHUNK_SIZE,
        MAX_HEIGHT,
    },
    voxel::{
        LocalVoxelPosition,
        VoxelKind,
        VoxelPosition,
    },
};
use bevy::{
    math::ivec2,
    prelude::*,
};
use noise::{
    utils::{
        NoiseMap,
        NoiseMapBuilder,
        PlaneMapBuilder,
    },
    BasicMulti,
    Perlin,
    ScalePoint,
};

pub fn generate_noise_map(width: usize, height: usize, seed: u32) -> NoiseMap {
    let mut basic_multi = BasicMulti::<Perlin>::new(seed);
    basic_multi.octaves = 4;
    basic_multi.persistence = 0.5;
    basic_multi.lacunarity = 2.0;

    let mut scaled_bm: ScalePoint<_> = ScalePoint::new(basic_multi);
    scaled_bm.x_scale = 4.5;
    scaled_bm.y_scale = 4.5;

    PlaneMapBuilder::<_, 3>::new(&scaled_bm)
        .set_size(width, height)
        .build()
}

pub fn ground_height_to_voxel(height: usize, is_top_level: bool) -> VoxelKind {
    if height > 100 && is_top_level {
        return VoxelKind::Snow;
    }
    if !(50..=96).contains(&height) {
        return VoxelKind::Stone;
    }
    if is_top_level {
        VoxelKind::Grass
    } else {
        VoxelKind::Dirt
    }
}

pub fn spiral(max_x: isize, max_y: isize) -> impl Iterator<Item = (isize, isize)> {
    let mut x = 0;
    let mut y = 0;
    let directions = [(1, 0), (0, 1), (-1, 0), (0, -1)];
    let mut direction_index = 0;

    let mut steps = 1;
    let mut step_count = 0;
    let mut steps_in_current_direction = 0;
    std::iter::from_fn(move || {
        if !(x <= max_x && y <= max_y) {
            return None;
        }
        let ret = (x, y);
        x = x.checked_add(directions[direction_index].0).unwrap();
        y = y.checked_add(directions[direction_index].1).unwrap();
        steps_in_current_direction += 1;

        if steps_in_current_direction == steps {
            steps_in_current_direction = 0;
            direction_index = (direction_index + 1) % 4;
            step_count += 1;
            if step_count == 2 {
                step_count = 0;
                steps += 1;
            }
        }

        Some(ret)
    })
}

pub fn generate_chunks(
    mut commands: Commands,
    mut world: ResMut<crate::world::World>,
    player: Query<&Transform, With<Camera>>,
) {
    let pos: ChunkPosition = player.single().translation.as_ivec3().into();
    let chunk_count_x = 8;
    let chunk_count_z = 8;

    for (chunk_x, chunk_z) in spiral(chunk_count_x, chunk_count_z) {
        let chunk_pos = &pos
            + ivec2(
                (chunk_x * CHUNK_SIZE as isize) as i32,
                (chunk_z * CHUNK_SIZE as isize) as i32,
            );
        if world.chunk_at(chunk_pos).is_some() {
            continue;
        }
        let mut chunk = Chunk::new().with_position(chunk_pos);

        for x in 0..CHUNK_SIZE {
            for y in 0..MAX_HEIGHT {
                for z in 0..CHUNK_SIZE {
                    let local_pos = LocalVoxelPosition::new(x as _, y as _, z as _);
                    let global_pos = &chunk_pos + local_pos;
                    chunk.voxel_mut(local_pos).kind =
                        block_at_position(global_pos, &world.noise_map);
                }
            }
        }
        world.add_chunk(chunk_pos, commands.spawn((Name::new("Chunk"), chunk)).id());
    }
}

pub fn block_at_position(pos: VoxelPosition, noise: &NoiseMap) -> VoxelKind {
    let normalized_x = pos.x() + (noise.size().0 / 2) as i32;
    let normalized_z = pos.z() + (noise.size().1 / 2) as i32;
    let noise_val = noise.get_value(normalized_x as usize, normalized_z as usize);
    let mut height = (noise_val.powf(2.0) * MAX_HEIGHT as f64) as usize;
    let y = pos.y() as usize;
    height += 64;
    if height == 64 {
        let water_floor: usize = ((noise_val.powf(1.2) * MAX_HEIGHT as f64) as usize).max(32);
        if y < water_floor {
            VoxelKind::Stone
        } else if y < height + 2 {
            VoxelKind::Water
        } else {
            VoxelKind::Air
        }
    } else {
        if height > MAX_HEIGHT - 1 {
            height = MAX_HEIGHT - 1;
        }
        match y.cmp(&height) {
            Ordering::Less => ground_height_to_voxel(y, false),
            Ordering::Equal => ground_height_to_voxel(height, true),
            Ordering::Greater => VoxelKind::Air,
        }
    }
}
