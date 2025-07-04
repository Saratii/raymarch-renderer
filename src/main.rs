pub mod camera;
pub mod pickaxe;
pub mod sdf;

use std::collections::HashMap;

use bevy::{
    prelude::*,
    reflect::TypePath,
    render::{
        render_resource::{AsBindGroup, ShaderRef},
        storage::ShaderStorageBuffer,
    },
    window::{PresentMode, WindowResolution},
};
use iyes_perf_ui::{prelude::PerfUiDefaultEntries, PerfUiPlugin};

use crate::{
    camera::{
        cursor_grab, get_buffer_data, initial_grab_cursor, player_look, player_move, FlyCam,
        KeyBindings, MovementSettings,
    },
    pickaxe::pickaxe_listener,
    sdf::{BoxSDF, SphereSDF},
};

const CHUNK_LOAD_SQUARE_RADIUS: i32 = 3;
const SHADER_ASSET_PATH: &str = "shaders/custom_material.wgsl";
pub const CHUNK_SIZE: f32 = 64.0;

#[derive(Resource)]
pub struct ChunkBoxBufferHandle(pub Handle<ShaderStorageBuffer>);

#[derive(Resource)]
pub struct ChunkSphereBufferHandle(pub Handle<ShaderStorageBuffer>);

#[derive(Resource)]
pub struct ChunkMap(HashMap<(i32, i32), Chunk>);

pub struct Chunk {
    pub box_sdfs: Vec<BoxSDF>,
    pub sphere_sdfs: Vec<SphereSDF>,
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: PresentMode::AutoNoVsync,
                    resolution: WindowResolution::new(2000.0, 1200.0),
                    ..default()
                }),
                ..default()
            }),
            MaterialPlugin::<CustomMaterial>::default(),
            PerfUiPlugin,
            bevy::diagnostic::FrameTimeDiagnosticsPlugin::default(),
        ))
        .insert_resource(MovementSettings::default())
        .insert_resource(KeyBindings::default())
        .add_systems(Startup, (setup, initial_grab_cursor))
        .add_systems(
            Update,
            (player_look, cursor_grab, player_move, pickaxe_listener),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CustomMaterial>>,
    window: Single<&Window>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    commands.spawn(PerfUiDefaultEntries::default());
    let mut chunk_map = build_world();
    let (box_sdfs, sphere_sdfs) = get_buffer_data((0, 0), &mut chunk_map);
    let box_buffer_handle = buffers.add(ShaderStorageBuffer::from(box_sdfs));
    let sphere_buffer_handle = buffers.add(ShaderStorageBuffer::from(sphere_sdfs));
    let material_handle = materials.add(CustomMaterial {
        pos: Vec3::new(0.0, 0.0, 0.0),
        forward: Vec3::new(0.0, 0.0, -1.0),
        right: Vec3::new(1.0, 0.0, 0.0),
        up: Vec3::new(0.0, 1.0, 0.0),
        box_sdf_buffer: box_buffer_handle.clone(),
        sphere_sdf_buffer: sphere_buffer_handle.clone(),
    });
    commands.insert_resource(ChunkBoxBufferHandle(box_buffer_handle));
    commands.insert_resource(ChunkSphereBufferHandle(sphere_buffer_handle));
    commands.insert_resource(ChunkMap(chunk_map));
    commands
        .spawn((
            Camera3d::default(),
            Projection::Orthographic(OrthographicProjection::default_3d()),
            Transform::from_xyz(0.0, 0.0, 0.0).looking_at(Vec3::new(0., 0., -4.), Vec3::Y),
            FlyCam,
        ))
        .with_child((
            Mesh3d(meshes.add(Plane3d::new(
                Vec3::Z,
                Vec2 {
                    x: window.width() / 2.0,
                    y: window.height() / 2.0,
                },
            ))),
            MeshMaterial3d(material_handle.clone()),
            Transform::from_xyz(0.0, 0.0, -4.0),
            ViewPort,
        ));
    commands.insert_resource(MaterialHandle(material_handle));
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct CustomMaterial {
    #[uniform(0)]
    pos: Vec3,
    #[uniform(1)]
    forward: Vec3,
    #[uniform(2)]
    right: Vec3,
    #[uniform(3)]
    up: Vec3,
    #[storage(4, read_only)]
    box_sdf_buffer: Handle<ShaderStorageBuffer>,
    #[storage(5, read_only)]
    sphere_sdf_buffer: Handle<ShaderStorageBuffer>,
}

impl Material for CustomMaterial {
    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }
}

#[derive(Resource)]
pub struct MaterialHandle(Handle<CustomMaterial>);

#[derive(Component)]
pub struct ViewPort;

fn build_world() -> HashMap<(i32, i32), Chunk> {
    let mut chunk_map = HashMap::new();
    for i in -CHUNK_LOAD_SQUARE_RADIUS..=CHUNK_LOAD_SQUARE_RADIUS {
        for j in -CHUNK_LOAD_SQUARE_RADIUS..=CHUNK_LOAD_SQUARE_RADIUS {
            let chunk_x = i;
            let chunk_z = j;
            chunk_map.insert(
                (chunk_x, chunk_z),
                Chunk {
                    box_sdfs: vec![BoxSDF {
                        center: Vec3::new(i as f32 * CHUNK_SIZE, -20.0, j as f32 * CHUNK_SIZE),
                        half_extents: Vec3::new(CHUNK_SIZE / 2., 20.0, CHUNK_SIZE / 2.),
                    }],
                    sphere_sdfs: Vec::new(),
                },
            );
        }
    }
    chunk_map
}
