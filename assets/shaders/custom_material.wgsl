#import bevy_pbr::{
    mesh_view_bindings::view,
    mesh_view_bindings::globals,
    forward_io::VertexOutput,
}

@group(2) @binding(0) var<uniform> position: vec3<f32>;
@group(2) @binding(1) var<uniform> forward: vec3<f32>;
@group(2) @binding(2) var<uniform> right: vec3<f32>;
@group(2) @binding(3) var<uniform> up: vec3<f32>;
@group(2) @binding(4) var<storage, read> box_sdfs: array<BoxSDF>;
@group(2) @binding(5) var<storage, read> sphere_sdfs: array<SphereSDF>;

struct BoxSDF {
    center: vec3<f32>,
    half_extents: vec3<f32>,
}

struct SphereSDF {
    center: vec3<f32>,
    radius: f32,
    negate: i32, // 1 for negate (subtract), 0 for normal (add/union)
}

struct SceneHit {
    dist: f32,
    // 0 = Background
    // 1 = Green material (for boxes)
    // 2 = Brown material (for spheres)
    material_id: i32,
}

fn sdf_box(p: vec3<f32>, center: vec3<f32>, b: vec3<f32>) -> f32 {
    let q = abs(p - center) - b;
    return length(max(q, vec3<f32>(0.0))) + min(max(q.x, max(q.y, q.z)), 0.0);
}

fn sdf_sphere(p: vec3<f32>, center: vec3<f32>, radius: f32) -> f32 {
    return length(p - center) - radius;
}

fn map_scene(p: vec3<f32>) -> SceneHit {
    var closest_hit = SceneHit(1e9, 0); 
    for (var i: u32 = 0u; i < arrayLength(&box_sdfs); i++) {
        let dist = sdf_box(p, box_sdfs[i].center, box_sdfs[i].half_extents);
        if (dist < closest_hit.dist) {
            closest_hit.dist = dist;
            closest_hit.material_id = 1;
        }
    }
    for (var i: u32 = 0u; i < arrayLength(&sphere_sdfs); i++) {
        let sphere_dist = sdf_sphere(p, sphere_sdfs[i].center, sphere_sdfs[i].radius);
        if (sphere_sdfs[i].negate == 1) {
            closest_hit.dist = max(-sphere_dist, closest_hit.dist);
        } else {
            if (sphere_dist < closest_hit.dist) {
                closest_hit.dist = sphere_dist;
                closest_hit.material_id = 2;
            }
        }
    }

    return closest_hit;
}

fn calculate_normal(p: vec3<f32>) -> vec3<f32> {
    let h = 0.001;
    let k = vec2<f32>(1.0, -1.0);
    return normalize(
        k.xyy * map_scene(p + k.xyy * h).dist +
        k.yyx * map_scene(p + k.yyx * h).dist +
        k.yxy * map_scene(p + k.yxy * h).dist +
        k.xxx * map_scene(p + k.xxx * h).dist
    );
}


@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let uv = mesh.uv * 2.0 - 1.0;
    let resolution = view.viewport.zw;
    let aspect = resolution.x / resolution.y;
    let corrected_uv = vec2<f32>(uv.x * aspect, uv.y);
    let ray_origin = position;
    let fov_scale = tan(radians(30.0));
    let ray_dir = normalize(
        forward +
        corrected_uv.x * right * fov_scale +
        -corrected_uv.y * up * fov_scale
    );
    var ray_pos = ray_origin;
    var total_dist_traveled = 0.0;
    let max_steps = 128; 
    let max_dist = 500.0;
    let epsilon = 0.001;

    for (var i = 0; i < max_steps; i++) {
        let hit = map_scene(ray_pos);
        if (hit.dist < epsilon) {
            let normal = calculate_normal(ray_pos);
            let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
            
            let diffuse = max(0.2, dot(normal, light_dir));
            if (hit.material_id == 1) {
                return vec4<f32>(17.0/255.0 * diffuse, 124.0/255.0 * diffuse, 19.0/255.0 * diffuse, 1.0);
            } else if (hit.material_id == 2) {
                 return vec4<f32>(139.0/255.0 * diffuse, 69.0/255.0 * diffuse, 19.0/255.0 * diffuse, 1.0);
            } else {
                 return vec4<f32>(0.1 * diffuse, 0.1 * diffuse, 0.1 * diffuse, 1.0);
            }
        }
        ray_pos += ray_dir * hit.dist;
        total_dist_traveled += hit.dist;
        if (total_dist_traveled > max_dist) {
            break;
        }
    }
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}