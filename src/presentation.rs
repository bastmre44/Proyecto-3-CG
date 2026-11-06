use crate::framebuffer::Framebuffer;
const CREAM: u32 = 0xFFF0CC;
const GOLD: u32 = 0xDDB566;
fn glyph(c: char) -> [u8; 7] {
    match c {
        '\u{00c1}' => [4, 8, 14, 17, 31, 17, 17],
        '\u{00c9}' => [4, 8, 31, 16, 30, 16, 31],
        '\u{00cd}' => [2, 4, 14, 4, 4, 4, 14],
        '\u{00d3}' => [4, 8, 14, 17, 17, 17, 14],
        '\u{00da}' => [4, 8, 17, 17, 17, 17, 14],
        '\u{00d1}' => [10, 20, 17, 25, 21, 19, 17],
        '+' => [0, 4, 4, 31, 4, 4, 0],
        'A' => [14, 17, 17, 31, 17, 17, 17],
        'B' => [30, 17, 17, 30, 17, 17, 30],
        'C' => [14, 17, 16, 16, 16, 17, 14],
        'D' => [30, 17, 17, 17, 17, 17, 30],
        'E' => [31, 16, 16, 30, 16, 16, 31],
        'F' => [31, 16, 16, 30, 16, 16, 16],
        'G' => [14, 17, 16, 23, 17, 17, 14],
        'H' => [17, 17, 17, 31, 17, 17, 17],
        'I' => [14, 4, 4, 4, 4, 4, 14],
        'J' => [7, 2, 2, 2, 18, 18, 12],
        'K' => [17, 18, 20, 24, 20, 18, 17],
        'L' => [16, 16, 16, 16, 16, 16, 31],
        'M' => [17, 27, 21, 21, 17, 17, 17],
        'N' => [17, 25, 21, 19, 17, 17, 17],
        'O' => [14, 17, 17, 17, 17, 17, 14],
        'P' => [30, 17, 17, 30, 16, 16, 16],
        'Q' => [14, 17, 17, 17, 21, 18, 13],
        'R' => [30, 17, 17, 30, 20, 18, 17],
        'S' => [15, 16, 16, 14, 1, 1, 30],
        'T' => [31, 4, 4, 4, 4, 4, 4],
        'U' => [17, 17, 17, 17, 17, 17, 14],
        'V' => [17, 17, 17, 17, 17, 10, 4],
        'W' => [17, 17, 17, 21, 21, 21, 10],
        'X' => [17, 17, 10, 4, 10, 17, 17],
        'Y' => [17, 17, 10, 4, 4, 4, 4],
        'Z' => [31, 1, 2, 4, 8, 16, 31],
        '0' => [14, 17, 19, 21, 25, 17, 14],
        '1' => [4, 12, 4, 4, 4, 4, 14],
        '2' => [14, 17, 1, 2, 4, 8, 31],
        '3' => [30, 1, 1, 14, 1, 1, 30],
        '4' => [2, 6, 10, 18, 31, 2, 2],
        '5' => [31, 16, 16, 30, 1, 1, 30],
        '6' => [14, 16, 16, 30, 17, 17, 14],
        '7' => [31, 1, 2, 4, 8, 8, 8],
        '8' => [14, 17, 17, 14, 17, 17, 14],
        '9' => [14, 17, 17, 15, 1, 1, 14],
        ':' => [0, 4, 4, 0, 4, 4, 0],
        '.' => [0, 0, 0, 0, 0, 6, 6],
        '-' => [0, 0, 0, 31, 0, 0, 0],
        '/' => [1, 2, 2, 4, 8, 8, 16],
        '|' => [4; 7],
        _ => [0; 7],
    }
}
pub fn panel(b: &mut Framebuffer, x: usize, y: usize, w: usize, h: usize) {
    for py in y..(y + h).min(b.height) {
        for px in x..(x + w).min(b.width) {
            let i = py * b.width + px;
            let c = b.pixels[i];
            b.pixels[i] = if py == y {
                GOLD
            } else {
                (((c >> 16) & 255) / 4 + 4) << 16
                    | (((c >> 8) & 255) / 4 + 5) << 8
                    | (c & 255) / 4 + 13
            };
        }
    }
}
pub fn text(b: &mut Framebuffer, x: usize, y: usize, s: &str, scale: usize, color: u32) {
    for (index, c) in s.to_uppercase().chars().enumerate() {
        for (row, bits) in glyph(c).iter().enumerate() {
            for col in 0..5 {
                if bits & (1 << (4 - col)) != 0 {
                    for dy in 0..scale {
                        for dx in 0..scale {
                            let px = x + (index * 6 + col) * scale + dx;
                            let py = y + row * scale + dy;
                            if px < b.width && py < b.height {
                                b.pixels[py * b.width + px] = color;
                            }
                        }
                    }
                }
            }
        }
    }
}
pub fn hud(b: &mut Framebuffer, name: &str, distance: f32, mode: &str, help: bool) -> usize {
    let m = crate::ui::margin(b.width, b.height);
    let width = if b.width >= 1000 {
        440
    } else {
        b.width.saturating_sub(2 * m).min(520)
    };
    let mut card = crate::ui::Card::new(width);
    let scale = if b.width >= 600 { 2 } else { 1 };
    card.row(format!("CERCANO: {name}"), scale, CREAM);
    card.row(
        format!("DISTANCIA AL CENTRO: {distance:.1} U"),
        scale,
        CREAM,
    );
    card.row(mode, 1, GOLD);
    card.draw(b, m, m);
    let help_card = crate::ui::help_card(b.width.saturating_sub(2 * m).min(800), help);
    let y = b
        .height
        .saturating_sub(crate::ui::footer_height(b.width) + m + help_card.height());
    help_card.draw(b, m, y);
    m + card.height() + 6
}
pub struct Presentation {
    pub started: bool,
    pub help: bool,
    pub transition: f32,
}
impl Presentation {
    pub fn new() -> Self {
        Self {
            started: false,
            help: true,
            transition: 0.0,
        }
    }
    pub fn update(&mut self, enter: bool, h: bool, dt: f32) {
        if enter {
            self.started = true;
        }
        if h {
            self.help = !self.help;
        }
        if self.started {
            self.transition = (self.transition + dt / 2.5).min(1.0);
        }
    }
    pub fn blend(&self) -> f32 {
        let t = self.transition;
        t * t * (3.0 - 2.0 * t)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn enter_h_y_transicion() {
        let mut p = Presentation::new();
        p.update(false, false, 1.0);
        assert!(!p.started);
        assert_eq!(p.blend(), 0.0);
        p.update(true, true, 0.0);
        assert!(p.started);
        assert!(!p.help);
        assert_eq!(p.blend(), 0.0);
        p.update(false, false, 1.25);
        assert!((p.blend() - 0.5).abs() < 0.001);
        p.update(true, true, 5.0);
        assert_eq!(p.blend(), 1.0);
        assert!(p.help);
    }
}
pub fn target(
    b: &mut Framebuffer,
    camera: &crate::camera::Camera,
    name: &str,
    position: glam::Vec3,
    distance: f32,
    view: u32,
    top: usize,
) {
    use glam::Vec2;
    let m = crate::ui::margin(b.width, b.height);
    let width = if b.width >= 1000 {
        440
    } else {
        b.width.saturating_sub(2 * m).min(520)
    };
    let x = if b.width >= 1000 {
        b.width - width - m
    } else {
        m
    };
    let y = if b.width >= 1000 { m } else { top };
    let mut card = crate::ui::Card::new(width);
    card.row(
        format!("OBJETIVO TAB: {name}"),
        if b.width >= 600 { 2 } else { 1 },
        GOLD,
    );
    card.row(format!("AL CENTRO: {distance:.1} U"), 1, CREAM);
    card.row(
        match view {
            1 => "V: LATERAL / INFERIOR / VUELO",
            2 => "V: INFERIOR / VUELO",
            _ => "V: INSPECCION NAVE | O: ORBITAS",
        },
        1,
        GOLD,
    );
    card.draw(b, x, y);
    let (point, behind) = target_marker(camera, position, b.width, b.height);
    let center = Vec2::new(b.width as f32 * 0.5, b.height as f32 * 0.5);
    let direction = (point - center).try_normalize().unwrap_or(Vec2::Y);
    let perpendicular = Vec2::new(-direction.y, direction.x);
    let tip = point + direction * 9.0;
    for side in [-1.0, 1.0] {
        let end = point - direction * 6.0 + perpendicular * side * 6.0;
        for step in 0..25 {
            let p = tip.lerp(end, step as f32 / 24.0);
            let x = p.x.round() as i32;
            let y = p.y.round() as i32;
            if x >= 0 && y >= 0 && x < b.width as i32 && y < b.height as i32 {
                b.pixels[y as usize * b.width + x as usize] = GOLD;
            }
        }
    }
    if behind {
        text(
            b,
            point.x.max(4.0) as usize,
            (point.y + 15.0).min(b.height.saturating_sub(10) as f32) as usize,
            "DETRAS",
            1,
            CREAM,
        );
    }
}
fn target_marker(
    camera: &crate::camera::Camera,
    position: glam::Vec3,
    width: usize,
    height: usize,
) -> (glam::Vec2, bool) {
    use glam::Vec2;
    let d = position - camera.position;
    let forward = (camera.target - camera.position).normalize();
    let right = forward.cross(camera.up).normalize();
    let up = right.cross(forward);
    let z = d.dot(forward);
    let tangent = (camera.fov_y * 0.5).tan();
    let aspect = width as f32 / height as f32;
    let mut offset =
        Vec2::new(d.dot(right) / (tangent * aspect), -d.dot(up) / tangent) / z.abs().max(0.001);
    if z <= 0.0 {
        offset = offset.try_normalize().unwrap_or(Vec2::Y) * 10.0;
    }
    let bound = Vec2::new(0.90, 0.86);
    let ratio = (offset.x.abs() / bound.x)
        .max(offset.y.abs() / bound.y)
        .max(1.0);
    offset /= ratio;
    (
        Vec2::new(
            (offset.x + 1.0) * width as f32 * 0.5,
            (offset.y + 1.0) * height as f32 * 0.5,
        ),
        z <= 0.0,
    )
}
#[cfg(test)]
mod target_tests {
    use super::*;
    use glam::Vec3;
    #[test]
    fn objetivo_fuera_de_pantalla_y_detras_sigue_visible() {
        let camera = crate::camera::Camera::new(Vec3::ZERO, -Vec3::Z);
        for p in [
            Vec3::new(100.0, 0.0, -1.0),
            Vec3::Z,
            Vec3::new(0.0, 100.0, -1.0),
        ] {
            let (m, behind) = target_marker(&camera, p, 1000, 750);
            assert!(m.x > 10.0 && m.x < 990.0 && m.y > 10.0 && m.y < 740.0);
            assert_eq!(behind, p.z > 0.0);
        }
    }
}
