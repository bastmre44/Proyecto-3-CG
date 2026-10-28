use crate::framebuffer::Framebuffer;
use crate::obj_loader::Vertex;

// Indica de qué lado de una arista está un punto.
fn edge(a: Vertex, b: Vertex, x: f32, y: f32) -> f32 {
    (b.x - a.x) * (y - a.y) - (b.y - a.y) * (x - a.x)
}

pub fn triangle(buffer: &mut Framebuffer, a: Vertex, b: Vertex, c: Vertex, color: u32) {
    let area = edge(a, b, c.x, c.y);

    // Un triángulo sin área no se puede rellenar.
    if area.abs() < 0.000001 {
        return;
    }

    let min_x = a.x.min(b.x).min(c.x).floor().max(0.0) as i32;
    let max_x = a.x.max(b.x).max(c.x).ceil().min(buffer.width as f32 - 1.0) as i32;

    let min_y = a.y.min(b.y).min(c.y).floor().max(0.0) as i32;
    let max_y = a.y.max(b.y).max(c.y).ceil().min(buffer.height as f32 - 1.0) as i32;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            // Evaluamos el centro del píxel.
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;

            let weight_a = edge(b, c, px, py) / area;
            let weight_b = edge(c, a, px, py) / area;
            let weight_c = edge(a, b, px, py) / area;

            if weight_a >= 0.0 && weight_b >= 0.0 && weight_c >= 0.0 {
                let depth = weight_a * a.z + weight_b * b.z + weight_c * c.z;

                buffer.point(x, y, depth, color);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vertex(x: f32, y: f32) -> Vertex {
        Vertex { x, y, z: 0.5 }
    }

    #[test]
    fn dibuja_el_mismo_triangulo_en_ambos_sentidos() {
        let mut first = Framebuffer::new(10, 10);
        let mut second = Framebuffer::new(10, 10);

        let a = vertex(2.0, 2.0);
        let b = vertex(8.0, 2.0);
        let c = vertex(2.0, 8.0);
        let color = 0xB6A3E0;

        triangle(&mut first, a, b, c, color);
        triangle(&mut second, a, c, b, color);

        // Un píxel interior debe pintarse.
        assert_eq!(first.pixels[3 * 10 + 3], color);

        // Un píxel exterior debe conservar el fondo.
        assert_eq!(first.pixels[8 * 10 + 8], 0);

        // Cambiar el orden no debe cambiar la imagen.
        assert_eq!(first.pixels, second.pixels);
    }

    #[test]
    fn un_triangulo_sin_area_no_pinta_pixeles() {
        let mut buffer = Framebuffer::new(10, 10);
        buffer.clear(0x101522);

        triangle(
            &mut buffer,
            vertex(2.0, 2.0),
            vertex(4.0, 4.0),
            vertex(6.0, 6.0),
            0xB6A3E0,
        );

        assert!(buffer.pixels.iter().all(|&color| color == 0x101522));
    }
}
fn interpolate_fragment(
    vertices: &[crate::pipeline::ShadedVertex; 3],
    weights: [f32; 3],
    time: f32,
) -> Option<crate::shader::Fragment> {
    // Corregimos los pesos usando la profundidad de cada vértice.
    let corrected = [
        weights[0] / vertices[0].clip_position.w,
        weights[1] / vertices[1].clip_position.w,
        weights[2] / vertices[2].clip_position.w,
    ];

    let total = corrected[0] + corrected[1] + corrected[2];

    if !total.is_finite() || total <= 0.0 {
        return None;
    }

    let weights = [
        corrected[0] / total,
        corrected[1] / total,
        corrected[2] / total,
    ];

    let local_position = vertices[0].local_position * weights[0]
        + vertices[1].local_position * weights[1]
        + vertices[2].local_position * weights[2];

    let normal = vertices[0].normal * weights[0]
        + vertices[1].normal * weights[1]
        + vertices[2].normal * weights[2];

    Some(crate::shader::Fragment {
        local_position,
        normal,
        time,
    })
}

pub fn shaded_triangle(
    buffer: &mut Framebuffer,
    vertices: [crate::pipeline::ShadedVertex; 3],
    material: crate::shader::Material,
    time: f32,
) {
    if buffer.width == 0 || buffer.height == 0 {
        return;
    }

    let screen = vertices.map(|vertex| {
        crate::pipeline::to_screen(vertex.clip_position, buffer.width, buffer.height)
    });

    let [Some(a), Some(b), Some(c)] = screen else {
        return;
    };

    let area = edge(a, b, c.x, c.y);

    if !area.is_finite() || area.abs() < 0.000001 {
        return;
    }

    let min_x = a.x.min(b.x).min(c.x).floor().max(0.0) as i32;
    let max_x = a.x.max(b.x).max(c.x).ceil().min((buffer.width - 1) as f32) as i32;

    let min_y = a.y.min(b.y).min(c.y).floor().max(0.0) as i32;
    let max_y = a.y.max(b.y).max(c.y).ceil().min((buffer.height - 1) as f32) as i32;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let sample_x = x as f32 + 0.5;
            let sample_y = y as f32 + 0.5;

            let wa = edge(b, c, sample_x, sample_y) / area;
            let wb = edge(c, a, sample_x, sample_y) / area;
            let wc = edge(a, b, sample_x, sample_y) / area;

            if wa < 0.0 || wb < 0.0 || wc < 0.0 {
                continue;
            }

            // La profundidad de pantalla usa los pesos originales.
            let depth = a.z * wa + b.z * wb + c.z * wc;

            let Some(fragment) = interpolate_fragment(&vertices, [wa, wb, wc], time) else {
                continue;
            };

            let color = crate::shader::shade(material, &fragment);

            buffer.point(x, y, depth, color);
        }
    }
}

#[cfg(test)]
mod shaded_tests {
    use super::*;
    use crate::pipeline::ShadedVertex;
    use crate::shader::Material;
    use glam::{Vec3, Vec4};

    #[test]
    fn los_atributos_se_interpolan_con_correccion_de_perspectiva() {
        let vertices = [
            ShadedVertex {
                clip_position: Vec4::new(0.0, 0.0, 0.0, 1.0),
                local_position: Vec3::ZERO,
                normal: Vec3::Z,
            },
            ShadedVertex {
                clip_position: Vec4::new(0.0, 0.0, 0.0, 2.0),
                local_position: Vec3::X,
                normal: Vec3::Z,
            },
            ShadedVertex {
                clip_position: Vec4::new(0.0, 0.0, 0.0, 4.0),
                local_position: Vec3::Y,
                normal: Vec3::Z,
            },
        ];

        let fragment = interpolate_fragment(&vertices, [1.0 / 3.0; 3], 2.0).unwrap();

        let expected = Vec3::new(2.0 / 7.0, 1.0 / 7.0, 0.0);

        assert!(fragment.local_position.distance(expected) < 0.00001);
        assert!(fragment.normal.distance(Vec3::Z) < 0.00001);
        assert_eq!(fragment.time, 2.0);
    }

    #[test]
    fn un_triangulo_sin_area_no_ejecuta_el_dibujo_del_shader() {
        let mut buffer = Framebuffer::new(8, 8);
        buffer.clear(0x101522);

        let original = buffer.pixels.clone();

        let vertex = ShadedVertex {
            clip_position: Vec4::new(0.0, 0.0, 0.0, 1.0),
            local_position: Vec3::ZERO,
            normal: Vec3::Z,
        };

        shaded_triangle(&mut buffer, [vertex; 3], Material::LavenderHaze, 0.0);

        assert_eq!(buffer.pixels, original);
    }
}
