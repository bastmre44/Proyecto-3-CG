use crate::{camera::Camera, framebuffer::Framebuffer};
use glam::Vec3;

struct Star {
    direction: Vec3,
    brightness: f32,
    phase: f32,
    color: Vec3,
}
pub struct Sky {
    stars: Vec<Star>,
}
fn pack(c: Vec3) -> u32 {
    let c = c.clamp(Vec3::ZERO, Vec3::splat(255.0));
    ((c.x as u32) << 16) | ((c.y as u32) << 8) | c.z as u32
}
impl Sky {
    pub fn new() -> Self {
        let mut seed = 0x53484f57u32;
        let mut random = || {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            (seed >> 8) as f32 / 16777216.0
        };
        let stars = (0..3600)
            .map(|_| {
                let y = random() * 2.0 - 1.0;
                let a = random() * std::f32::consts::TAU;
                let r = (1.0 - y * y).sqrt();
                let brightness = random();
                let phase = random() * std::f32::consts::TAU;
                let color = match (random() * 3.0) as u32 {
                    0 => Vec3::new(185.0, 209.0, 255.0),
                    1 => Vec3::new(255.0, 217.0, 231.0),
                    _ => Vec3::splat(248.0),
                };
                Star {
                    direction: Vec3::new(r * a.cos(), y, r * a.sin()),
                    brightness,
                    phase,
                    color,
                }
            })
            .collect();
        Self { stars }
    }
    pub fn draw(&self, camera: &Camera, buffer: &mut Framebuffer, time: f32) {
        let forward = (camera.target - camera.position).normalize();
        let right = forward.cross(camera.up).normalize();
        let up = right.cross(forward);
        let tangent = (camera.fov_y * 0.5).tan();
        let aspect = buffer.width as f32 / buffer.height as f32;
        // Sample directional clouds on a coarse grid, then bilinearly enlarge.
        let step = 8;
        let w = buffer.width / step + 2;
        let h = buffer.height / step + 2;
        let mut grid = vec![Vec3::ZERO; w * h];
        for y in 0..h {
            for x in 0..w {
                let u = (x * step) as f32 / buffer.width as f32 * 2.0 - 1.0;
                let v = 1.0 - (y * step) as f32 / buffer.height as f32 * 2.0;
                let d = (forward + right * u * tangent * aspect + up * v * tangent).normalize();
                let mut c = Vec3::new(5.0, 8.0, 23.0);
                for (center, tint) in [
                    (
                        Vec3::new(0.7, 0.3, -0.6).normalize(),
                        Vec3::new(18.0, 8.0, 30.0),
                    ),
                    (
                        Vec3::new(-0.4, 0.2, 0.9).normalize(),
                        Vec3::new(24.0, 8.0, 19.0),
                    ),
                    (
                        Vec3::new(0.2, -0.7, 0.6).normalize(),
                        Vec3::new(7.0, 15.0, 28.0),
                    ),
                ] {
                    let density = ((d.dot(center) - 0.72) / 0.28).clamp(0.0, 1.0);
                    let waves = 0.6 + 0.2 * (d.x * 19.0 + d.z * 13.0 + (d.y * 11.0).sin()).sin();
                    c += tint * density * density * waves;
                }
                grid[y * w + x] = c;
            }
        }
        for y in 0..buffer.height {
            for x in 0..buffer.width {
                let i = (y / step) * w + x / step;
                let u = (x % step) as f32 / step as f32;
                let v = (y % step) as f32 / step as f32;
                buffer.pixels[y * buffer.width + x] = pack(
                    grid[i]
                        .lerp(grid[i + 1], u)
                        .lerp(grid[i + w].lerp(grid[i + w + 1], u), v),
                );
            }
        }
        for star in &self.stars {
            let z = star.direction.dot(forward);
            if z <= 0.0 {
                continue;
            }
            let screen_x = star.direction.dot(right) / (z * tangent * aspect);
            let screen_y = star.direction.dot(up) / (z * tangent);
            if screen_x.abs() > 1.01 || screen_y.abs() > 1.01 {
                continue;
            }
            let x = ((star.direction.dot(right) / (z * tangent * aspect) + 1.0)
                * 0.5
                * buffer.width as f32) as i32;
            let y = ((1.0 - star.direction.dot(up) / (z * tangent)) * 0.5 * buffer.height as f32)
                as i32;
            let pulse = if star.brightness > 0.75 {
                0.90 + 0.10 * (time * 0.65 + star.phase).sin()
            } else {
                1.0
            };
            let c = star.color * (0.35 + 0.65 * star.brightness) * pulse;
            let radius = if star.brightness > 0.988 { 1i32 } else { 0 };
            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    if dx != 0 && dy != 0 {
                        continue;
                    }
                    let (px, py) = (x + dx, y + dy);
                    if px >= 0 && py >= 0 && px < buffer.width as i32 && py < buffer.height as i32 {
                        buffer.pixels[py as usize * buffer.width + px as usize] =
                            pack(c * if dx == 0 && dy == 0 { 1.0 } else { 0.38 });
                    }
                }
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cielo_repetible_independiente_de_traslacion_y_sensible_al_giro() {
        let sky = Sky::new();
        let mut a = Framebuffer::new(160, 120);
        let mut b = Framebuffer::new(160, 120);
        let mut camera = Camera::new(Vec3::ZERO, -Vec3::Z);
        sky.draw(&camera, &mut a, 0.0);
        camera.position += Vec3::X * 20.0;
        camera.target += Vec3::X * 20.0;
        Sky::new().draw(&camera, &mut b, 0.0);
        assert_eq!(a.pixels, b.pixels);
        camera.target = camera.position + Vec3::X;
        sky.draw(&camera, &mut b, 0.0);
        assert_ne!(a.pixels, b.pixels);
        a.point(0, 0, 0.99, 0x123456);
        assert_eq!(a.pixels[0], 0x123456);
    }
}
