use std::{
    cmp::Ordering,
    sync::Arc,
};

use crate::{
    chunk::{
        ChunkPosition,
        ChunkVoxels,
        CHUNK_SIZE,
        MAX_HEIGHT,
    },
    player::PlayerMovedEvent,
    voxel::{
        LocalVoxelPosition,
        VoxelKind,
        VoxelPosition,
    },
};
use bevy::{
    math::ivec2,
    prelude::*,
    tasks::{
        block_on,
        futures_lite::future,
        AsyncComputeTaskPool,
        Task,
    },
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

#[derive(Component)]
pub struct TerrainGenerationTask(Task<(Entity, ChunkVoxels)>);

pub fn queue_generate_chunk_terrain(
    mut commands: Commands,
    mut world: ResMut<crate::world::World>,
    mut ev_movement: EventReader<PlayerMovedEvent>,
    settings: Res<crate::Settings>,
    player: Query<&Transform, With<Camera>>,
) {
    if !ev_movement.is_empty() && !ev_movement.read().any(|mvmnt| mvmnt.changed_chunk()) {
        return;
    }
    let pos: ChunkPosition = player.single().translation.as_ivec3().into();
    let radius = (settings.load_distance as isize) / 2;
    let task_pool = AsyncComputeTaskPool::get();

    // This all leads to a lot of hitching. Can we make it so the player has to be
    // further than `load_distance` to make a chunk unload?
    let mut chunks_to_despawn = world.chunk_map.clone();

    for (chunk_x, chunk_z) in spiral(radius, radius) {
        let chunk_pos = &pos
            + ivec2(
                (chunk_x * CHUNK_SIZE as isize) as i32,
                (chunk_z * CHUNK_SIZE as isize) as i32,
            );
        if world.chunk_at(chunk_pos).is_some() {
            chunks_to_despawn.remove(&chunk_pos);
            continue;
        }
        let mut chunk = commands.spawn((Name::new("Chunk"), chunk_pos));
        let chunk_id = chunk.id();
        let noise_map = Arc::clone(&world.noise_map);
        let task = async move {
            let mut voxels = ChunkVoxels::new();
            for x in 0..CHUNK_SIZE {
                for y in 0..MAX_HEIGHT {
                    for z in 0..CHUNK_SIZE {
                        let local_pos = LocalVoxelPosition::new(x as _, y as _, z as _);
                        let global_pos = &chunk_pos + local_pos;
                        voxels.voxel_mut(local_pos).kind =
                            block_at_position(global_pos, &noise_map);
                    }
                }
            }
            (chunk_id, voxels)
        };
        chunk.insert(TerrainGenerationTask(task_pool.spawn(task)));
        world.add_chunk(chunk_pos, chunk_id);
    }

    for (pos, ent) in chunks_to_despawn {
        world.remove_chunk(pos);
        commands.entity(ent).despawn();
    }
}

pub fn handle_generated_chunk_terrain(
    mut commands: Commands,
    mut tasks: Query<&mut TerrainGenerationTask>,
) {
    for mut task in tasks.iter_mut() {
        if let Some((ent, voxels)) = block_on(future::poll_once(&mut task.0)) {
            commands
                .entity(ent)
                .remove::<TerrainGenerationTask>()
                .insert(voxels);
        }
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
