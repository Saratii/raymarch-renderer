use bevy::{math::Vec3, render::render_resource::ShaderType};

#[derive(ShaderType, Clone)]
pub struct BoxSDF {
    pub center: Vec3,
    pub half_extents: Vec3,
}

#[derive(ShaderType, Clone)]
pub struct SphereSDF {
    pub center: Vec3,
    pub radius: f32,
    pub negate: i32, // 0 for normal, 1 for negated
}

fn sdf_sphere(p: &Vec3, center: &Vec3, radius: f32) -> f32 {
    return (p - center).length() - radius;
}

pub fn chunk_sdf(p: &Vec3, box_sdfs: &Vec<BoxSDF>, sphere_sdfs: &Vec<SphereSDF>) -> f32 {
    let mut min_dist = sdf_box(p, &box_sdfs[0].center, &box_sdfs[0].half_extents);
    for i in 1..box_sdfs.len() {
        min_dist = min_dist.min(sdf_box(p, &box_sdfs[i].center, &box_sdfs[i].half_extents));
    }
    for i in 0..sphere_sdfs.len() {
        min_dist = min_dist.min(sdf_sphere(p, &sphere_sdfs[i].center, sphere_sdfs[i].radius));
    }
    return min_dist;
}

fn sdf_box(p: &Vec3, center: &Vec3, b: &Vec3) -> f32 {
    let q = (p - center).abs() - b;
    return q.max(Vec3::ZERO).length() + q.x.max(q.y.max(q.z).min(0.0));
}

pub fn two_chunk_sdf(
    p: &Vec3,
    box_sdfs_1: &Vec<BoxSDF>,
    sphere_sdfs_1: &Vec<SphereSDF>,
    box_sdfs_2: &Vec<BoxSDF>,
    sphere_sdfs_2: &Vec<SphereSDF>,
) -> f32 {
    let mut min_dist = sdf_box(p, &box_sdfs_1[0].center, &box_sdfs_1[0].half_extents);
    for i in 1..box_sdfs_1.len() {
        min_dist = min_dist.min(sdf_box(
            p,
            &box_sdfs_1[i].center,
            &box_sdfs_1[i].half_extents,
        ));
    }
    for i in 0..sphere_sdfs_1.len() {
        if sphere_sdfs_1[i].negate == 1 {
            min_dist = min_dist.max(-sdf_sphere(
                p,
                &sphere_sdfs_1[i].center,
                sphere_sdfs_1[i].radius,
            ));
        } else {
            min_dist = min_dist.min(sdf_sphere(
                p,
                &sphere_sdfs_1[i].center,
                sphere_sdfs_1[i].radius,
            ));
        }
    }
    for i in 1..box_sdfs_2.len() {
        min_dist = min_dist.min(sdf_box(
            p,
            &box_sdfs_2[i].center,
            &box_sdfs_2[i].half_extents,
        ));
    }
    for i in 0..sphere_sdfs_2.len() {
        if sphere_sdfs_2[i].negate == 1 {
            if sphere_sdfs_2[i].negate == 1 {
                min_dist = min_dist.max(-sdf_sphere(
                    p,
                    &sphere_sdfs_2[i].center,
                    sphere_sdfs_2[i].radius,
                ));
            } else {
                min_dist = min_dist.min(sdf_sphere(
                    p,
                    &sphere_sdfs_2[i].center,
                    sphere_sdfs_2[i].radius,
                ));
            }
        }
    }
    return min_dist;
}
