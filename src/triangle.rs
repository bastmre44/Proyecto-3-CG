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
