use std::collections::HashMap;

use crate::{
    sdf::SphereSDF, BoxSDF, Chunk, ChunkBoxBufferHandle, ChunkMap, ChunkSphereBufferHandle,
    CustomMaterial, MaterialHandle, CHUNK_LOAD_SQUARE_RADIUS, CHUNK_SIZE,
};
use bevy::{
    input::mouse::MouseMotion,
    prelude::*,
    render::storage::ShaderStorageBuffer,
    window::{CursorGrabMode, PrimaryWindow},
};

#[derive(Resource)]
pub struct MovementSettings {
    pub sensitivity: f32,
    pub speed: f32,
}

impl Default for MovementSettings {
    fn default() -> Self {
        Self {
            sensitivity: 0.00012,
            speed: 12.,
        }
    }
}

#[derive(Component)]
pub struct FlyCam;

#[derive(Resource)]
pub struct KeyBindings {
    pub move_forward: KeyCode,
    pub move_backward: KeyCode,
    pub move_left: KeyCode,
    pub move_right: KeyCode,
    pub move_ascend: KeyCode,
    pub move_descend: KeyCode,
    pub toggle_grab_cursor: KeyCode,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            move_forward: KeyCode::KeyW,
            move_backward: KeyCode::KeyS,
            move_left: KeyCode::KeyA,
            move_right: KeyCode::KeyD,
            move_ascend: KeyCode::Space,
            move_descend: KeyCode::ShiftLeft,
            toggle_grab_cursor: KeyCode::Escape,
        }
    }
}

pub fn cursor_grab(
    keys: Res<ButtonInput<KeyCode>>,
    key_bindings: Res<KeyBindings>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
) {
    if let Ok(mut window) = primary_window.single_mut() {
        if keys.just_pressed(key_bindings.toggle_grab_cursor) {
            toggle_grab_cursor(&mut window);
        }
    }
}

pub fn initial_grab_cursor(mut primary_window: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = primary_window.single_mut() {
        toggle_grab_cursor(&mut window);
    }
}

pub fn player_move(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    settings: Res<MovementSettings>,
    key_bindings: Res<KeyBindings>,
    mut query: Query<(&FlyCam, &mut Transform)>,
    mut materials: ResMut<Assets<CustomMaterial>>,
    material_handle: Res<MaterialHandle>,
    mut chunk_map: ResMut<ChunkMap>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    chunk_box_buffer_handle: Res<ChunkBoxBufferHandle>,
    chunk_sphere_buffer_handle: Res<ChunkSphereBufferHandle>,
) {
    if let Ok(window) = primary_window.single() {
        for (_camera, mut transform) in query.iter_mut() {
            let mut velocity = Vec3::ZERO;
            let local_z = transform.local_z();
            let forward = -Vec3::new(local_z.x, 0., local_z.z);
            let right = Vec3::new(local_z.z, 0., -local_z.x);
            for key in keys.get_pressed() {
                match window.cursor_options.grab_mode {
                    CursorGrabMode::None => (),
                    _ => {
                        let key = *key;
                        if key == key_bindings.move_forward {
                            velocity += forward;
                        } else if key == key_bindings.move_backward {
                            velocity -= forward;
                        } else if key == key_bindings.move_left {
                            velocity -= right;
                        } else if key == key_bindings.move_right {
                            velocity += right;
                        } else if key == key_bindings.move_ascend {
                            velocity += Vec3::Y;
                        } else if key == key_bindings.move_descend {
                            velocity -= Vec3::Y;
                        }
                    }
                }
            }
            velocity = velocity.normalize_or_zero();
            let old_position = transform.translation;
            transform.translation += velocity * time.delta_secs() * settings.speed;
            let material = materials.get_mut(&material_handle.0).unwrap();
            material.pos = transform.translation;
            let old_chunk = (
                (old_position.x / CHUNK_SIZE).round() as i32,
                (old_position.z / CHUNK_SIZE).round() as i32,
            );
            let new_chunk = (
                (transform.translation.x / CHUNK_SIZE).round() as i32,
                (transform.translation.z / CHUNK_SIZE).round() as i32,
            );
            if old_chunk != new_chunk {
                let (box_sdfs, sphere_sdfs) = get_buffer_data(new_chunk, &mut chunk_map.0);
                let chunk_buffer_data = buffers.get_mut(&chunk_box_buffer_handle.0).unwrap();
                chunk_buffer_data.set_data(box_sdfs);
                let sphere_buffer_data = buffers.get_mut(&chunk_sphere_buffer_handle.0).unwrap();
                sphere_buffer_data.set_data(sphere_sdfs);
            }
        }
    }
}

pub fn player_look(
    settings: Res<MovementSettings>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut state: EventReader<MouseMotion>,
    mut query: Query<&mut Transform, With<FlyCam>>,
    mut materials: ResMut<Assets<CustomMaterial>>,
    material_handle: Res<MaterialHandle>,
) {
    if let Ok(window) = primary_window.single() {
        for mut transform in query.iter_mut() {
            for ev in state.read() {
                let (mut yaw, mut pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
                match window.cursor_options.grab_mode {
                    CursorGrabMode::None => (),
                    _ => {
                        let window_scale = window.height().min(window.width());
                        pitch -= (settings.sensitivity * ev.delta.y * window_scale).to_radians();
                        yaw -= (settings.sensitivity * ev.delta.x * window_scale).to_radians();
                    }
                }
                pitch = pitch.clamp(-1.54, 1.54);
                transform.rotation =
                    Quat::from_axis_angle(Vec3::Y, yaw) * Quat::from_axis_angle(Vec3::X, pitch);
            }
            let material = materials.get_mut(&material_handle.0).unwrap();
            material.pos = transform.translation;
            material.forward = *transform.forward();
            material.right = *transform.right();
            material.up = *transform.up();
        }
    }
}

fn toggle_grab_cursor(window: &mut Window) {
    match window.cursor_options.grab_mode {
        CursorGrabMode::None => {
            window.cursor_options.grab_mode = CursorGrabMode::Confined;
            window.cursor_options.visible = false;
        }
        _ => {
            window.cursor_options.grab_mode = CursorGrabMode::None;
            window.cursor_options.visible = true;
        }
    }
}

pub fn get_buffer_data(
    current_camera_chunk: (i32, i32),
    chunk_map: &mut HashMap<(i32, i32), Chunk>,
) -> (Vec<BoxSDF>, Vec<SphereSDF>) {
    let mut box_sdfs = Vec::new();
    let mut sphere_sdfs = Vec::new();
    let mut new_chunks = Vec::new();
    for i in -CHUNK_LOAD_SQUARE_RADIUS..=CHUNK_LOAD_SQUARE_RADIUS {
        for j in -CHUNK_LOAD_SQUARE_RADIUS..=CHUNK_LOAD_SQUARE_RADIUS {
            let chunk_x = current_camera_chunk.0 + i;
            let chunk_z = current_camera_chunk.1 + j;
            match chunk_map.get(&(chunk_x, chunk_z)) {
                Some(chunk) => {
                    box_sdfs.extend(chunk.box_sdfs.clone());
                    sphere_sdfs.extend(chunk.sphere_sdfs.clone());
                }
                None => {
                    let new_chunk = Chunk {
                        box_sdfs: vec![BoxSDF {
                            center: Vec3::new(
                                chunk_x as f32 * CHUNK_SIZE,
                                -20.0,
                                chunk_z as f32 * CHUNK_SIZE,
                            ),
                            half_extents: Vec3::new(CHUNK_SIZE / 2., 20.0, CHUNK_SIZE / 2.),
                        }],
                        sphere_sdfs: Vec::new(),
                    };
                    new_chunks.push(((chunk_x, chunk_z), new_chunk));
                }
            }
        }
    }
    for (chunk_coords, chunk) in new_chunks {
        let new_box_sdf = chunk.box_sdfs.clone();
        chunk_map.insert(chunk_coords, chunk);
        box_sdfs.extend(new_box_sdf);
    }
    (box_sdfs, sphere_sdfs)
}
