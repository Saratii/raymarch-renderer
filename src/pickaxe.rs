use bevy::{
    asset::Assets,
    ecs::{
        query::With,
        system::{Res, ResMut, Single},
    },
    input::{mouse::MouseButton, ButtonInput},
    math::Vec3,
    render::storage::ShaderStorageBuffer,
    transform::components::Transform,
};

use crate::{
    camera::{get_buffer_data, FlyCam},
    sdf::{two_chunk_sdf, SphereSDF},
    ChunkMap, ChunkSphereBufferHandle,
};

const MAX_PICKAXE_DISTANCE: f32 = 5.0;
const BASE_PICKAGXE_INTERSECT_RADIUS: f32 = 0.5; //must be less than CHUNK_SIZE
const COLLISION_EPSILON: f32 = 0.001;

pub fn pickaxe_listener(
    buttons: Res<ButtonInput<MouseButton>>,
    camera_transform: Single<&Transform, With<FlyCam>>,
    mut chunk_map: ResMut<ChunkMap>,
    chunk_sphere_buffer_handle: Res<ChunkSphereBufferHandle>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        let camera_direction = camera_transform.forward();
        let mut current_position = camera_transform.translation;
        let current_chunk_index = (
            (current_position.x / 64.0).round() as i32,
            (current_position.z / 64.0).round() as i32,
        );
        let current_chunk = chunk_map.0.get(&current_chunk_index).unwrap();
        let possible_overflow_chunk_index = get_next_chunk(&current_chunk_index, &camera_direction);
        let overflow_chunk = chunk_map
            .0
            .get(&possible_overflow_chunk_index)
            .unwrap_or(current_chunk);
        while current_position.distance(camera_transform.translation) < MAX_PICKAXE_DISTANCE {
            let step_distance = two_chunk_sdf(
                &current_position,
                &current_chunk.box_sdfs,
                &current_chunk.sphere_sdfs,
                &overflow_chunk.box_sdfs,
                &overflow_chunk.sphere_sdfs,
            );
            if step_distance < COLLISION_EPSILON {
                let final_chunk_index = (
                    (current_position.x / 64.0).round() as i32,
                    (current_position.z / 64.0).round() as i32,
                );
                let final_chunk = chunk_map.0.get_mut(&final_chunk_index).unwrap();
                final_chunk.sphere_sdfs.push(SphereSDF {
                    center: current_position,
                    radius: BASE_PICKAGXE_INTERSECT_RADIUS,
                    negate: 1,
                });
                let (_, sphere_sdfs) = get_buffer_data(current_chunk_index, &mut chunk_map.0);
                let sphere_buffer_data = buffers.get_mut(&chunk_sphere_buffer_handle.0).unwrap();
                sphere_buffer_data.set_data(sphere_sdfs);
                return;
            }
            current_position += camera_direction * step_distance;
        }
    }
}

pub fn get_next_chunk(current_chunk: &(i32, i32), direction: &Vec3) -> (i32, i32) {
    if direction.length_squared() < f32::EPSILON {
        return *current_chunk;
    }
    let step_x = direction.x.signum() as i32;
    let step_z = direction.z.signum() as i32;
    if step_x == 0 && step_z == 0 {
        return *current_chunk;
    }
    if step_x == 0 {
        return (current_chunk.0, current_chunk.1 + step_z);
    }
    if step_z == 0 {
        return (current_chunk.0 + step_x, current_chunk.1);
    }
    let t_cross_x = direction.z.abs();
    let t_cross_z = direction.x.abs();

    if (t_cross_x - t_cross_z).abs() < f32::EPSILON {
        (current_chunk.0 + step_x, current_chunk.1 + step_z)
    } else if t_cross_x < t_cross_z {
        (current_chunk.0, current_chunk.1 + step_z)
    } else {
        (current_chunk.0 + step_x, current_chunk.1)
    }
}
