use crate::{obj_loader::Model, showgirl_mesh::Mesh};
use glam::{Mat4, Vec3};
pub const SUN_CENTER: Vec3 = Vec3::ZERO;
pub const CRUISE_SPEED: f32 = 8.0;
pub const BOOST_SPEED: f32 = 24.0;
pub fn orbital_radius(previous: f32, previous_extent: f32, extent: f32, ship_radius: f32) -> f32 {
    previous + previous_extent + extent + 2.0 * ship_radius + 0.2
}
pub fn orbit_ring(radius: f32) -> Model {
    let mut mesh = Mesh::new();
    for i in 0..512 {
        let a = i as f32 * std::f32::consts::TAU / 512.0;
        let b = (i + 1) as f32 * std::f32::consts::TAU / 512.0;
        let p = |angle: f32, r: f32| SUN_CENTER + Vec3::new(angle.cos() * r, 0.0, angle.sin() * r);
        let inner = radius - 0.012;
        let outer = radius + 0.012;
        mesh.triangle(Mat4::IDENTITY, [p(a, inner), p(a, outer), p(b, outer)]);
        mesh.triangle(Mat4::IDENTITY, [p(a, inner), p(b, outer), p(b, inner)]);
    }
    mesh.model
}
pub fn travel_speed(clearance: f32, boost: bool) -> f32 {
    let t = ((clearance - 1.0) / 5.0).clamp(0.0, 1.0);
    let t = t * t * (3.0 - 2.0 * t);
    2.0 + (if boost { BOOST_SPEED } else { CRUISE_SPEED } - 2.0) * t
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn separacion_minima_para_nave() {
        let (a, e1, e2, ship) = (9.0, 2.5, 4.9, 0.8);
        let b = orbital_radius(a, e1, e2, ship);
        assert!((b - a - e1 - e2 - 2.0 * ship - 0.2).abs() < 0.00001);
    }
    #[test]
    fn maniobras_lentas_con_impulso() {
        assert_eq!(travel_speed(0.0, true), 2.0);
        assert_eq!(travel_speed(10.0, true), 24.0);
        assert_eq!(travel_speed(10.0, false), 8.0);
    }
}
