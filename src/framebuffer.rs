pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u32>,
    depth: Vec<f32>,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![0; width * height],
            depth: vec![f32::INFINITY; width * height],
        }
    }

    pub fn clear(&mut self, color: u32) {
        self.pixels.fill(color);
        self.depth.fill(f32::INFINITY);
    }

    pub fn point(&mut self, x: i32, y: i32, z: f32, color: u32) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 || !z.is_finite() {
            return;
        }

        let index = y as usize * self.width + x as usize;

        if z < self.depth[index] {
            self.depth[index] = z;
            self.pixels[index] = color;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Framebuffer;

    #[test]
    fn el_pixel_cercano_gana_sin_importar_el_orden() {
        for depths in [[0.8, 0.2], [0.2, 0.8]] {
            let mut buffer = Framebuffer::new(10, 10);

            for z in depths {
                let color = if z < 0.5 {
                    0xB6A3E0 // Lavanda: superficie cercana.
                } else {
                    0x17264D // Azul: superficie lejana.
                };

                buffer.point(5, 5, z, color);
            }

            assert_eq!(buffer.pixels[55], 0xB6A3E0);
        }
    }

    #[test]
    fn limpiar_reinicia_el_color_y_la_profundidad() {
        let mut buffer = Framebuffer::new(10, 10);

        buffer.point(5, 5, 0.1, 0xB6A3E0);
        buffer.clear(0x101522);

        assert!(buffer.pixels.iter().all(|&color| color == 0x101522));

        // Después de limpiar, una superficie más lejana puede dibujarse.
        buffer.point(5, 5, 0.9, 0x17264D);

        assert_eq!(buffer.pixels[55], 0x17264D);
    }
}
