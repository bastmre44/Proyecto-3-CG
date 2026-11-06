use crate::{framebuffer::Framebuffer, ui::Rect};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Quality {
    Normal,
    High,
}
impl Quality {
    pub fn factor(self) -> usize {
        match self {
            Self::Normal => 1,
            Self::High => 2,
        }
    }
    pub fn toggle(&mut self) {
        *self = match self {
            Self::Normal => Self::High,
            Self::High => Self::Normal,
        };
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Normal => "NORMAL",
            Self::High => "ALTA 2X",
        }
    }
}
// Resolve all opaque geometry and transparency together. UI never enters this buffer.
pub fn resolve(source: &Framebuffer, target: &mut Framebuffer, rect: Rect, quality: Quality) {
    let factor = quality.factor();
    assert_eq!(
        (source.width, source.height),
        (rect.w * factor, rect.h * factor)
    );
    for y in 0..rect.h {
        for x in 0..rect.w {
            let color = if factor == 1 {
                source.pixels[y * source.width + x]
            } else {
                let i = (y * 2) * source.width + x * 2;
                let samples = [
                    source.pixels[i],
                    source.pixels[i + 1],
                    source.pixels[i + source.width],
                    source.pixels[i + source.width + 1],
                ];
                let channel = |shift: u32| -> u32 {
                    (samples.iter().map(|c| (c >> shift) & 255).sum::<u32>() + 2) / 4
                };
                (channel(16) << 16) | (channel(8) << 8) | channel(0)
            };
            target.pixels[(rect.y + y) * target.width + rect.x + x] = color;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn promedio_cuatro_muestras_y_transparencia_antes_de_reducir() {
        let mut src = Framebuffer::new(2, 2);
        src.pixels = vec![0xFF0000, 0x00FF00, 0x0000FF, 0xFFFFFF];
        let mut out = Framebuffer::new(1, 1);
        resolve(
            &src,
            &mut out,
            Rect {
                x: 0,
                y: 0,
                w: 1,
                h: 1,
            },
            Quality::High,
        );
        assert_eq!(out.pixels[0], 0x808080);
        src.clear(0);
        for y in 0..2 {
            for x in 0..2 {
                src.point(x, y, 0.4, 0x204060);
                src.blend_point(x, y, 0.8, 0xFFFFFF, 0.5);
            }
        }
        resolve(
            &src,
            &mut out,
            Rect {
                x: 0,
                y: 0,
                w: 1,
                h: 1,
            },
            Quality::High,
        );
        assert_eq!(out.pixels[0], 0x204060);
        src.blend_point(0, 0, 0.2, 0xFFFFFF, 0.5);
        resolve(
            &src,
            &mut out,
            Rect {
                x: 0,
                y: 0,
                w: 1,
                h: 1,
            },
            Quality::High,
        );
        assert_ne!(out.pixels[0], 0x204060);
    }
}
