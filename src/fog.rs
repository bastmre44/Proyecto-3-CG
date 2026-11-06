use glam::{Mat4, Vec3};

use crate::{
    camera::Camera,
    framebuffer::Framebuffer,
    obj_loader::Vertex,
    pipeline::{ShadedVertex, clip_shaded_triangle, to_screen},
};

// Esta pasada debe ejecutarse DESPUÉS de la nave y de todos los objetos opacos.
pub fn draw(camera: &Camera, buffer: &mut Framebuffer, planet_matrix: Mat4, time: f32) {
    if buffer.width == 0 || buffer.height == 0 {
        return;
    }
    let mvp = camera.projection_matrix(buffer.width, buffer.height)
        * camera.view_matrix()
        * planet_matrix;
    let segments = 72;
    let mut triangles = Vec::new();

    for band in 0..2 {
        let radius = 1.24 + band as f32 * 0.10;
        let tilt = Mat4::from_rotation_z(0.12 + band as f32 * 0.08);
        let vertex = |segment: usize, row: usize| {
            let angle = segment as f32 / segments as f32 * std::f32::consts::TAU;
            let drift = time * 0.16;
            let height =
                -0.26 + band as f32 * 0.34 + (angle * 2.0 + drift + band as f32).sin() * 0.08;
            let offset = [-0.085, 0.0, 0.085][row];
            let opacity = if row == 1 {
                0.20 * (0.5 + 0.5 * (angle * 3.0 - drift).sin().powi(2))
            } else {
                0.0
            };
            let position = tilt.transform_point3(Vec3::new(
                angle.cos() * radius,
                height + offset,
                angle.sin() * radius,
            ));
            ShadedVertex {
                clip_position: mvp * position.extend(1.0),
                // Se reutiliza el interpolador de atributos para la opacidad.
                local_position: Vec3::new(opacity, 0.0, 0.0),
                normal: Vec3::ZERO,
            }
        };
        for segment in 0..segments {
            for row in 0..2 {
                let a = vertex(segment, row);
                let b = vertex(segment + 1, row);
                let c = vertex(segment, row + 1);
                let d = vertex(segment + 1, row + 1);
                triangles.extend(clip_shaded_triangle([a, b, c]));
                triangles.extend(clip_shaded_triangle([b, d, c]));
            }
        }
    }

    // Mezclar desde atrás hacia adelante. El depth buffer conserva los opacos.
    triangles.sort_by(|a, b| {
        let depth_a: f32 = a.iter().map(|v| v.clip_position.w).sum();
        let depth_b: f32 = b.iter().map(|v| v.clip_position.w).sum();
        depth_b.total_cmp(&depth_a)
    });
    for triangle in triangles {
        rasterize(buffer, triangle, 0xBFC8E0);
    }
}

fn rasterize(buffer: &mut Framebuffer, mut vertices: [ShadedVertex; 3], color: u32) {
    let projected = vertices.map(|v| to_screen(v.clip_position, buffer.width, buffer.height));
    let [Some(a), Some(b), Some(c)] = projected else {
        return;
    };
    let mut screen = [a, b, c];
    let mut area = edge(a, b, c.x, c.y);
    if !area.is_finite() || area.abs() < 0.000001 {
        return;
    }
    if area < 0.0 {
        screen.swap(1, 2);
        vertices.swap(1, 2);
        area = -area;
    }
    let [a, b, c] = screen;
    let min_x = a.x.min(b.x).min(c.x).floor().max(0.0) as i32;
    let max_x = a.x.max(b.x).max(c.x).ceil().min((buffer.width - 1) as f32) as i32;
    let min_y = a.y.min(b.y).min(c.y).floor().max(0.0) as i32;
    let max_y = a.y.max(b.y).max(c.y).ceil().min((buffer.height - 1) as f32) as i32;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;
            let ea = edge(b, c, px, py);
            let eb = edge(c, a, px, py);
            let ec = edge(a, b, px, py);
            if !covered(ea, b, c) || !covered(eb, c, a) || !covered(ec, a, b) {
                continue;
            }
            let weights = [ea / area, eb / area, ec / area];
            let depth = weights[0] * a.z + weights[1] * b.z + weights[2] * c.z;
            let corrected: [f32; 3] =
                std::array::from_fn(|i| weights[i] / vertices[i].clip_position.w);
            let total: f32 = corrected.iter().sum();
            if !total.is_finite() || total <= 0.0 {
                continue;
            }
            let alpha: f32 = (0..3)
                .map(|i| corrected[i] * vertices[i].local_position.x)
                .sum::<f32>()
                / total;
            buffer.blend_point(x, y, depth, color, alpha);
        }
    }
}

fn edge(a: Vertex, b: Vertex, x: f32, y: f32) -> f32 {
    (b.x - a.x) * (y - a.y) - (b.y - a.y) * (x - a.x)
}

fn covered(value: f32, a: Vertex, b: Vertex) -> bool {
    // Cada arista compartida pertenece a un solo triángulo.
    // Evita mezclar dos veces la diagonal entre dos triángulos vecinos.
    value > 0.0 || (value == 0.0 && (b.y < a.y || (b.y == a.y && b.x > a.x)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec4;

    #[test]
    fn la_pasada_transparente_respeta_la_profundidad_de_la_escena() {
        let make_vertex = |x, y| ShadedVertex {
            clip_position: Vec4::new(x, y, 0.5, 1.0),
            local_position: Vec3::new(0.5, 0.0, 0.0),
            normal: Vec3::ZERO,
        };
        let triangle = [
            make_vertex(-0.9, -0.9),
            make_vertex(0.9, -0.9),
            make_vertex(0.0, 0.9),
        ];
        let mut buffer = Framebuffer::new(8, 8);
        buffer.point(4, 4, 0.2, 0x123456);
        rasterize(&mut buffer, triangle, 0xFFFFFF);
        assert_eq!(buffer.pixels[4 * 8 + 4], 0x123456);
        assert!(buffer.pixels.iter().any(|&p| p == 0x808080));
    }
}
