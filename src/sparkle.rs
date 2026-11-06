//! Smooth, sparse, surface-anchored glitter. Subpixel cells fade instead of flickering.
use glam::Vec3;
pub fn amount(direction: Vec3, time: f32, footprint: f32) -> f32 {
    let frequency = 42.0;
    let p = direction * frequency;
    let cell = p.floor();
    let mut hash = (cell.x as i32 as u32).wrapping_mul(0x8da6b343)
        ^ (cell.y as i32 as u32).wrapping_mul(0xd8163841)
        ^ (cell.z as i32 as u32).wrapping_mul(0xcb1ab31f);
    hash ^= hash >> 16;
    hash = hash.wrapping_mul(0x7feb352d);
    hash ^= hash >> 15;
    let seed = (hash & 65535) as f32 / 65535.0;
    if seed < 0.984 {
        return 0.0;
    }
    let local = p - cell - Vec3::splat(0.5);
    let d = local.length();
    let t = ((0.28 - d) / 0.22).clamp(0.0, 1.0);
    let spot = t * t * (3.0 - 2.0 * t);
    let coverage = ((footprint * frequency - 0.15) / 0.65).clamp(0.0, 1.0);
    let visible = 1.0 - coverage * coverage * (3.0 - 2.0 * coverage);
    spot * visible * (0.5 + 0.5 * (time * 0.9 + seed * 80.0).sin()).powi(4) * 0.26
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn destellos_subpixel_se_apagan_y_son_discretos() {
        for i in 0..5000 {
            let p = Vec3::new(i as f32 * 0.013, 0.7, 0.3).normalize();
            let a = amount(p, 1.0, 0.0);
            assert!(a >= 0.0 && a <= 0.26);
            assert_eq!(amount(p, 1.0, 1.0), 0.0);
        }
    }
}
