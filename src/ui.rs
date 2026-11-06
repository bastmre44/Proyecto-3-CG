//! Shared pixel layout. Text is measured with the same 6x7 font used for drawing.
use crate::{
    framebuffer::Framebuffer,
    presentation::{panel, text},
};
pub const CREAM: u32 = 0xFFF0CC;
pub const GOLD: u32 = 0xDDB566;
#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}
impl Rect {
    pub fn bottom(self) -> usize {
        self.y + self.h
    }
}
pub fn margin(width: usize, height: usize) -> usize {
    if width < 600 || height < 450 { 8 } else { 14 }
}
pub fn wrap(value: &str, width: usize, scale: usize) -> Vec<String> {
    let columns = (width / (6 * scale.max(1))).max(1);
    let mut lines = Vec::new();
    for paragraph in value.split('\n') {
        let mut line = String::new();
        for word in paragraph.split_whitespace() {
            if !line.is_empty() && line.chars().count() + 1 + word.chars().count() > columns {
                lines.push(std::mem::take(&mut line));
            }
            if !line.is_empty() {
                line.push(' ');
            }
            for c in word.chars() {
                if line.chars().count() == columns {
                    lines.push(std::mem::take(&mut line));
                }
                line.push(c);
            }
        }
        if !line.is_empty() || paragraph.is_empty() {
            lines.push(line);
        }
    }
    lines
}
pub fn line_height(scale: usize) -> usize {
    9 * scale
}
pub fn text_height(value: &str, width: usize, scale: usize) -> usize {
    wrap(value, width, scale).len() * line_height(scale)
}
pub fn wrapped(b: &mut Framebuffer, rect: Rect, value: &str, scale: usize, color: u32) {
    for (i, line) in wrap(value, rect.w, scale).iter().enumerate() {
        let y = rect.y + i * line_height(scale);
        if y + 7 * scale <= rect.bottom() {
            text(b, rect.x, y, line, scale, color);
        }
    }
}
pub struct Card {
    pub width: usize,
    pub padding: usize,
    pub rows: Vec<(String, usize, u32)>,
}
impl Card {
    pub fn new(width: usize) -> Self {
        Self {
            width,
            padding: 8,
            rows: Vec::new(),
        }
    }
    pub fn row(&mut self, value: impl Into<String>, scale: usize, color: u32) {
        self.rows.push((value.into(), scale, color));
    }
    pub fn height(&self) -> usize {
        self.padding * 2
            + self
                .rows
                .iter()
                .map(|(s, scale, _)| {
                    text_height(s, self.width.saturating_sub(2 * self.padding), *scale) + 4
                })
                .sum::<usize>()
    }
    pub fn draw(&self, b: &mut Framebuffer, x: usize, y: usize) {
        panel(b, x, y, self.width, self.height());
        let mut top = y + self.padding;
        for (s, scale, color) in &self.rows {
            let h = text_height(s, self.width.saturating_sub(2 * self.padding), *scale);
            wrapped(
                b,
                Rect {
                    x: x + self.padding,
                    y: top,
                    w: self.width.saturating_sub(2 * self.padding),
                    h,
                },
                s,
                *scale,
                *color,
            );
            top += h + 4;
        }
    }
}
pub fn footer_height(width: usize) -> usize {
    if width < 640 { 36 } else { 24 }
}
pub fn footer(b: &mut Framebuffer, label: &str) {
    let m = margin(b.width, b.height);
    let h = footer_height(b.width);
    let r = Rect {
        x: m,
        y: b.height.saturating_sub(h),
        w: b.width.saturating_sub(2 * m),
        h,
    };
    panel(b, r.x, r.y, r.w, r.h);
    wrapped(
        b,
        Rect {
            x: r.x + 4,
            y: r.y + 5,
            w: r.w.saturating_sub(8),
            h: r.h.saturating_sub(5),
        },
        label,
        1,
        GOLD,
    );
}
pub fn help_card(width: usize, expanded: bool) -> Card {
    let mut c = Card::new(width);
    c.row(
        "ESC: MENU | H: AYUDA | Q: NORMAL/ALTA | M: SILENCIO | -/+: VOLUMEN",
        1,
        GOLD,
    );
    if expanded {
        let scale = if width >= 740 { 2 } else { 1 };
        c.row(
            "W/S: VUELO O ZOOM LOCAL | A/D: GIRAR | ARRIBA/ABAJO: INCLINAR",
            scale,
            CREAM,
        );
        c.row(
            "SHIFT: IMPULSO | C: MODO LOCAL | TAB: OBJETIVO | O: ORBITAS",
            scale,
            CREAM,
        );
        c.row("V: LADO/ABAJO/VUELO | R: REINICIAR VUELO", 1, CREAM);
    }
    c
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ajuste_de_texto_no_pierde_palabras_ni_excede_panel() {
        let value = "Una mansion entre nieve, niebla y rosas negras.";
        for width in [60, 120, 400] {
            let lines = wrap(value, width, 1);
            assert!(lines.iter().all(|l| l.chars().count() * 6 <= width));
            assert_eq!(lines.join(" "), value);
        }
    }
}
