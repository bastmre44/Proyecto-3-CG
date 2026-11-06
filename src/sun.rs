use crate::{
    obj_loader::Model,
    showgirl_mesh::{Mesh, icosphere},
};
use glam::{Mat4, Vec3};
pub const BOUND_RADIUS: f32 = 1.28;
pub fn create() -> Model {
    let mut mesh = Mesh::new();
    mesh.append(&icosphere(1), Mat4::IDENTITY);
    // Solid triangular pyramids around three great circles: visible from every approach.
    for plane in 0..3 {
        for i in 0..10 {
            let a = i as f32 * std::f32::consts::TAU / 10.0;
            let d = Vec3::new(a.cos(), a.sin(), 0.0);
            let tangent = Vec3::new(-a.sin(), a.cos(), 0.0);
            let rotate = match plane {
                0 => Mat4::IDENTITY,
                1 => Mat4::from_rotation_x(std::f32::consts::FRAC_PI_2),
                _ => Mat4::from_rotation_y(std::f32::consts::FRAC_PI_2),
            };
            let tip = d * BOUND_RADIUS;
            let p = d * 0.88 + tangent * 0.18;
            let q = d * 0.88 - tangent * 0.18;
            let r = d * 0.90 + Vec3::Z * 0.14;
            let s = d * 0.90 - Vec3::Z * 0.14;
            for tri in [
                [p, tip, r],
                [r, tip, q],
                [q, tip, s],
                [s, tip, p],
                [p, r, q],
                [p, q, s],
            ] {
                mesh.triangle(rotate, tri);
            }
        }
    }
    mesh.model
}
pub fn color(normal: Vec3, time: f32) -> Vec3 {
    let facet = normal
        .normalize_or_zero()
        .dot(Vec3::new(-0.3, 0.8, 0.5).normalize())
        * 0.5
        + 0.5;
    Vec3::new(1.0, 0.57, 0.10).lerp(Vec3::new(1.0, 0.86, 0.38), facet)
        * (0.94 + 0.06 * (time * 1.2).sin())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn corona_cabe_en_colision() {
        let m = create();
        for v in m.vertices {
            assert!(Vec3::new(v.x, v.y, v.z).length() <= BOUND_RADIUS + 0.00001);
        }
    }
}
