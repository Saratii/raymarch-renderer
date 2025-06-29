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

struct BoxSDF {
    center: vec3<f32>,
    half_extents: vec3<f32>,
}

fn sdf_box(p: vec3<f32>, center: vec3<f32>, b: vec3<f32>) -> f32 {
  let q = abs(p - center) - b; 
  return length(max(q, vec3<f32>(0.0))) + min(max(q.x, max(q.y, q.z)), 0.0);
}

fn sdf_sphere(p: vec3<f32>, center: vec3<f32>, radius: f32) -> f32 {
    return length(p - center) - radius;
}

fn scene_sdf(p: vec3<f32>) -> f32 {
    var min_dist = sdf_box(p, box_sdfs[0].center, box_sdfs[0].half_extents);
    for (var i: u32 = 1u; i < arrayLength(&box_sdfs); i++) {
        min_dist = min(min_dist, sdf_box(p, box_sdfs[i].center, box_sdfs[i].half_extents));
    }
    return min_dist;
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
    let max_steps = 2000;
    let max_dist = 500.0;
    let epsilon = 0.001;
    for (var i = 0; i < max_steps; i++) {
        let dist = scene_sdf(ray_pos);
        if (dist < epsilon) {
            let h = 0.001;
            let normal = normalize(vec3<f32>(
                scene_sdf(ray_pos + vec3<f32>(h, 0.0, 0.0)) - scene_sdf(ray_pos - vec3<f32>(h, 0.0, 0.0)),
                scene_sdf(ray_pos + vec3<f32>(0.0, h, 0.0)) - scene_sdf(ray_pos - vec3<f32>(0.0, h, 0.0)),
                scene_sdf(ray_pos + vec3<f32>(0.0, 0.0, h)) - scene_sdf(ray_pos - vec3<f32>(0.0, 0.0, h))
            ));
            if (sdf_sphere(ray_pos, vec3<f32>(0.0, 0.0, 0.0), 1.0) < 0.0) {
                return vec4<f32>(0.5, 0.5, 1.0, 1.0); // Sky color
            }
            else if (ray_pos.y < -2.0) {
                return vec4<f32>(139.0/255.0, 69.0/255.0, 19.0/255.0, 1.0);
            } else {
                return vec4<f32>(17.0/255, 124.0/255, 19.0/255, 1.0);
            }
        }
        ray_pos += ray_dir * dist;
        if (length(ray_pos - ray_origin) > max_dist) {
            break;
        }
    }
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}