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

    // Keep allocations and depth unchanged until the dimensions actually change.
    pub fn resize(&mut self, width: usize, height: usize) -> bool {
        if width == 0 || height == 0 || (width, height) == (self.width, self.height) {
            return false;
        }
        self.width = width;
        self.height = height;
        self.pixels.resize(width * height, 0);
        self.depth.resize(width * height, f32::INFINITY);
        self.depth.fill(f32::INFINITY);
        true
    }

    pub fn clear(&mut self, color: u32) {
        self.pixels.fill(color);
        self.depth.fill(f32::INFINITY);
    }

    pub fn visible(&self, x: i32, y: i32, z: f32) -> bool {
        self.pixel_index(x, y, z)
            .is_some_and(|index| z < self.depth[index])
    }

    pub fn point(&mut self, x: i32, y: i32, z: f32, color: u32) {
        let Some(index) = self.pixel_index(x, y, z) else {
            return;
        };
        if z < self.depth[index] {
            self.depth[index] = z;
            self.pixels[index] = color;
        }
    }

    // Los efectos transparentes se dibujan después de todos los objetos opacos.
    // Consultan la profundidad, pero no la modifican.
    pub fn blend_point(&mut self, x: i32, y: i32, z: f32, color: u32, alpha: f32) {
        let Some(index) = self.pixel_index(x, y, z) else {
            return;
        };
        if !alpha.is_finite() || alpha <= 0.0 || z >= self.depth[index] {
            return;
        }

        let alpha = alpha.min(1.0);
        let background = self.pixels[index];
        let blend = |shift: u32| -> u32 {
            let foreground = ((color >> shift) & 255) as f32;
            let background = ((background >> shift) & 255) as f32;
            (foreground * alpha + background * (1.0 - alpha)).round() as u32
        };

        self.pixels[index] = (blend(16) << 16) | (blend(8) << 8) | blend(0);
    }

    fn pixel_index(&self, x: i32, y: i32, z: f32) -> Option<usize> {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 || !z.is_finite() {
            return None;
        }
        Some(y as usize * self.width + x as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::Framebuffer;

    #[test]
    fn redimensionar_minimizar_y_restaurar_conserva_buffers_validos() {
        let mut b = Framebuffer::new(800, 600);
        b.point(4, 4, 0.1, 0xFFFFFF);
        let pixels = b.pixels.as_ptr();
        assert!(!b.resize(800, 600));
        assert_eq!(pixels, b.pixels.as_ptr());
        assert_eq!(b.depth[4 * 800 + 4], 0.1);
        assert!(!b.resize(0, 0));
        for (w, h) in [(1000, 650), (1280, 720), (800, 600)] {
            assert!(b.resize(w, h));
            assert_eq!(b.pixels.len(), w * h);
            assert_eq!(b.depth.len(), w * h);
            assert!(b.depth.iter().all(|z| *z == f32::INFINITY));
            b.point((w - 1) as i32, (h - 1) as i32, 0.2, 0x123456);
            assert_eq!(b.pixels[w * h - 1], 0x123456);
        }
    }

    #[test]
    fn el_pixel_cercano_gana_sin_importar_el_orden() {
        for depths in [[0.8, 0.2], [0.2, 0.8]] {
            let mut buffer = Framebuffer::new(10, 10);
            for z in depths {
                let color = if z < 0.5 { 0xB6A3E0 } else { 0x17264D };
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
        buffer.point(5, 5, 0.9, 0x17264D);
        assert_eq!(buffer.pixels[55], 0x17264D);
    }

    #[test]
    fn la_niebla_se_oculta_detras_de_un_objeto_opaco() {
        let mut buffer = Framebuffer::new(1, 1);
        buffer.point(0, 0, 0.2, 0x204060);
        buffer.blend_point(0, 0, 0.7, 0xFFFFFF, 0.5);
        assert_eq!(buffer.pixels[0], 0x204060);
    }

    #[test]
    fn la_transparencia_mezcla_color_sin_escribir_profundidad() {
        let mut buffer = Framebuffer::new(1, 1);
        buffer.point(0, 0, 0.8, 0x000000);
        buffer.blend_point(0, 0, 0.3, 0xFFFFFF, 0.5);
        assert_eq!(buffer.pixels[0], 0x808080);
        assert_eq!(buffer.depth[0], 0.8);
    }
}
