use crate::{
    camera::Camera,
    framebuffer::Framebuffer,
    ui::{self, CREAM, Card, GOLD, Rect},
};
use glam::Vec3;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Menu,
    Gallery,
    Flight,
    Exit,
}
pub struct Screens {
    pub screen: Screen,
    pub menu: usize,
    pub gallery: Gallery,
}
impl Screens {
    pub fn new() -> Self {
        Self {
            screen: Screen::Menu,
            menu: 0,
            gallery: Gallery::new(),
        }
    }
    pub fn navigate(&mut self, step: i32) {
        match self.screen {
            Screen::Menu => self.menu = (self.menu as i32 + step).rem_euclid(3) as usize,
            Screen::Gallery => {
                self.gallery.selected =
                    (self.gallery.selected as i32 + step).rem_euclid(5) as usize;
                self.gallery.reset();
            }
            _ => {}
        }
    }
    pub fn enter(&mut self) {
        if self.screen == Screen::Menu {
            self.screen = match self.menu {
                0 => Screen::Flight,
                1 => Screen::Gallery,
                _ => Screen::Exit,
            };
        }
    }
    pub fn escape(&mut self) {
        self.screen = match self.screen {
            Screen::Gallery | Screen::Flight => Screen::Menu,
            _ => Screen::Exit,
        };
        self.gallery.end_drag();
    }
}
pub struct Gallery {
    pub selected: usize,
    yaw: f32,
    elevation: f32,
    zoom: f32,
    zoom_target: f32,
    focus: Vec3,
    pub detail_view: bool,
    pub quality: crate::quality::Quality,
    automatic_quality: bool,
    slow_frames: u8,
    reduced_for_speed: bool,
    pub render_ms: f32,
    pub drag: Option<(f32, f32)>,
    pan_drag: Option<(f32, f32)>,
    pub left_down: bool,
}
#[derive(Default)]
pub struct GalleryInput {
    pub mouse: Option<(f32, f32)>,
    pub left: bool,
    pub right: bool,
    pub wheel: f32,
    pub horizontal: f32,
    pub vertical: f32,
    pub dt: f32,
    pub height: usize,
}
impl Gallery {
    pub fn new() -> Self {
        Self {
            selected: 0,
            yaw: std::f32::consts::FRAC_PI_2,
            elevation: 0.20,
            zoom: 1.0,
            zoom_target: 1.0,
            focus: Vec3::ZERO,
            detail_view: false,
            quality: crate::quality::Quality::High,
            automatic_quality: true,
            slow_frames: 0,
            reduced_for_speed: false,
            render_ms: 0.0,
            drag: None,
            pan_drag: None,
            left_down: false,
        }
    }
    pub fn reset(&mut self) {
        let selected = self.selected;
        let quality = self.quality;
        let automatic = self.automatic_quality;
        let reduced = self.reduced_for_speed;
        *self = Self::new();
        self.automatic_quality = automatic;
        self.reduced_for_speed = reduced;
        self.selected = selected;
        self.quality = quality;
    }
    pub fn toggle_quality(&mut self) {
        self.quality.toggle();
        self.automatic_quality = false;
        self.reduced_for_speed = false;
        self.slow_frames = 0;
    }
    pub fn observe_render(&mut self, ms: f32) {
        self.render_ms = ms;
        if self.automatic_quality && self.quality == crate::quality::Quality::High {
            self.slow_frames = if ms > 80.0 {
                self.slow_frames.saturating_add(1)
            } else {
                0
            };
            if self.slow_frames >= 5 {
                self.quality = crate::quality::Quality::Normal;
                self.reduced_for_speed = true;
            }
        }
    }
    pub fn quality_label(&self) -> String {
        format!(
            "{}{}",
            self.quality.label(),
            if self.reduced_for_speed {
                " (AUTO: RENDIMIENTO)"
            } else {
                ""
            }
        )
    }
    pub fn detail(&mut self, radius: f32) {
        self.reset();
        self.detail_view = true;
        self.focus = match self.selected {
            0 => Vec3::new(0.0, 1.05, 0.15),
            1 => Vec3::new(0.0, 1.04, 0.10),
            2 => Vec3::new(0.0, 1.45, 0.0),
            3 => Vec3::new(0.25, 1.20, 0.0),
            _ => Vec3::new(0.0, 0.98, 0.25),
        } * radius;
        self.zoom = if self.selected == 4 { 0.13 } else { 0.26 };
        self.zoom_target = self.zoom;
        self.elevation = 0.08;
    }
    pub fn end_drag(&mut self) {
        self.drag = None;
        self.pan_drag = None;
        self.left_down = false;
    }
    pub fn inspect(&mut self, input: GalleryInput, camera: &Camera) {
        if !input.dt.is_finite() || input.dt <= 0.0 {
            return;
        }
        let mouse = input.mouse.filter(|(x, y)| x.is_finite() && y.is_finite());
        if input.left {
            if let (Some((x, y)), Some((px, py))) = (mouse, self.drag) {
                self.yaw -= (x - px) * 0.008;
                self.elevation += (y - py) * 0.008;
            }
            self.drag = mouse;
        } else {
            self.drag = None;
        }
        if input.right {
            if let (Some((x, y)), Some((px, py))) = (mouse, self.pan_drag) {
                let forward = (camera.target - camera.position).normalize_or_zero();
                let right = forward.cross(camera.up).normalize_or_zero();
                let up = right.cross(forward);
                let units =
                    2.0 * camera.position.distance(camera.target) * (camera.fov_y * 0.5).tan()
                        / input.height.max(1) as f32;
                self.focus += (-right * (x - px) + up * (y - py)) * units;
            }
            self.pan_drag = mouse;
        } else {
            self.pan_drag = None;
        }
        self.yaw = (self.yaw + input.horizontal.clamp(-1.0, 1.0) * input.dt * 1.2)
            .rem_euclid(std::f32::consts::TAU);
        self.elevation =
            (self.elevation + input.vertical.clamp(-1.0, 1.0) * input.dt).clamp(-1.48, 1.48);
        let wheel = if input.wheel.is_finite() {
            input.wheel.clamp(-12.0, 12.0)
        } else {
            0.0
        };
        self.zoom_target = (self.zoom_target * (-wheel * 0.12).exp()).clamp(0.008, 8.0);
        self.zoom += (self.zoom_target - self.zoom) * (1.0 - (-12.0 * input.dt).exp());
        self.left_down = input.left;
    }
    pub fn camera(&self, bound: f32, width: usize, height: usize) -> Camera {
        let fov = 45.0f32.to_radians();
        let aspect = width.max(1) as f32 / height.max(1) as f32;
        let half = (fov * 0.5).min(((fov * 0.5).tan() * aspect).atan());
        let distance = (bound / half.sin() * 1.08 * self.zoom).max(0.002);
        let direction = Vec3::new(
            self.yaw.cos() * self.elevation.cos(),
            self.elevation.sin(),
            self.yaw.sin() * self.elevation.cos(),
        );
        let mut camera = Camera::new(self.focus + direction * distance, self.focus);
        camera.fov_y = fov;
        camera.near = (distance * 0.008).clamp(0.0005, 0.08);
        camera
    }
}
pub const NAMES: [&str; 5] = [
    "THE FATE OF OPHELIA",
    "OPALITE",
    "PATIENT ZERO",
    "PINK CLOUDING",
    "THE LIFE OF A SHOWGIRL",
];
const DESCRIPTIONS: [&str; 5] = [
    "Un jardín de mármol y flores sobre un océano turquesa.",
    "Cristales perlados rodean una plaza de colores y arcos pastel.",
    "Una mansión entre nieve, niebla y rosas negras.",
    "Nubes rosadas e islas suspendidas en un mundo de ensueño.",
    "Un escenario naranja y dorado donde el espectáculo sigue vivo.",
];
pub fn menu(b: &mut Framebuffer, selected: usize, help: bool) {
    let m = ui::margin(b.width, b.height);
    let width = b.width.saturating_sub(2 * m).min(760);
    let mut card = Card::new(width);
    card.padding = if b.height > 500 { 20 } else { 10 };
    card.row(
        "THE SHOWGIRL UNIVERSE",
        if width >= 600 { 4 } else { 2 },
        CREAM,
    );
    card.row(
        "Cinco mundos, un universo de escenarios",
        if width >= 500 { 2 } else { 1 },
        GOLD,
    );
    for (i, name) in ["Explorar el sistema", "Galer\u{00ed}a de planetas", "Salir"]
        .iter()
        .enumerate()
    {
        card.row(
            format!("{} {}", if i == selected { "-" } else { " " }, name),
            if b.height >= 500 { 3 } else { 2 },
            if i == selected { GOLD } else { 0xAAA5B3 },
        );
    }
    card.row("ARRIBA/ABAJO: ELEGIR | ENTER: ABRIR | ESC: SALIR", 1, CREAM);
    card.row("H: AYUDA | M: SILENCIO | -/+: VOLUMEN", 1, GOLD);
    if help {
        card.row(
            "EXPLORAR: VUELO CONTINUO | GALERIA: FLECHAS Y RATON",
            1,
            CREAM,
        );
    }
    let available = b.height.saturating_sub(ui::footer_height(b.width));
    card.draw(
        b,
        (b.width - width) / 2,
        available.saturating_sub(card.height()) / 2,
    );
}

pub struct GalleryLayout {
    pub viewport: Rect,
    pub toolbar: Rect,
    pub information: Rect,
    pub list: Option<Rect>,
    pub info_card: Card,
    pub list_card: Card,
}
pub fn gallery_layout(width: usize, height: usize, selected: usize, help: bool) -> GalleryLayout {
    let m = ui::margin(width, height);
    let content_width = width.saturating_sub(2 * m).max(1);
    let wide = width >= 960;
    let mut info = Card::new(content_width);
    if !wide {
        info.row(NAMES.join(" | "), 1, 0xAAA5B3);
    }
    info.row(NAMES[selected], if height >= 420 { 2 } else { 1 }, GOLD);
    info.row(
        DESCRIPTIONS[selected],
        if height >= 500 { 2 } else { 1 },
        CREAM,
    );
    info.row(
        "Interpretaciones visuales inspiradas en las canciones.",
        1,
        GOLD,
    );
    info.row(
        "IZQ/DER: MUNDO | IZQUIERDO: GIRAR | DERECHO: DESPLAZAR | RUEDA: ZOOM | R: VISTA GENERAL | ESC: MENU",
        1,
        CREAM,
    );
    if help {
        info.row("A/D: GIRAR | W/S: INCLINAR | 1: GENERAL | 2: DETALLE | Q: NORMAL/ALTA | H: AYUDA | M: SILENCIO | -/+: VOLUMEN",1,GOLD);
    }
    let info_y = height.saturating_sub(ui::footer_height(width) + m + info.height());
    let list_width = 230.min(content_width);
    let mut list = Card::new(list_width);
    list.row("GALERIA DE PLANETAS", 2, CREAM);
    for (i, name) in NAMES.iter().enumerate() {
        list.row(
            *name,
            if height >= 520 { 2 } else { 1 },
            if i == selected { GOLD } else { 0xAAA5B3 },
        );
    }
    let view_x = if wide { m + list_width + m } else { m };
    let toolbar = Rect {
        x: view_x,
        y: m,
        w: width.saturating_sub(view_x + m).max(1),
        h: 28,
    };
    let viewport = Rect {
        x: view_x,
        y: m + toolbar.h + 6,
        w: width.saturating_sub(view_x + m).max(1),
        h: info_y.saturating_sub(2 * m + toolbar.h + 6).max(1),
    };
    let information = Rect {
        x: m,
        y: info_y,
        w: content_width,
        h: info.height(),
    };
    GalleryLayout {
        viewport,
        toolbar,
        information,
        list: wide.then_some(Rect {
            x: m,
            y: m,
            w: list_width,
            h: list.height(),
        }),
        info_card: info,
        list_card: list,
    }
}
pub fn contains(rect: Rect, x: f32, y: f32) -> bool {
    x >= rect.x as f32
        && y >= rect.y as f32
        && x < (rect.x + rect.w) as f32
        && y < (rect.y + rect.h) as f32
}
pub fn view_buttons(layout: &GalleryLayout) -> [Rect; 2] {
    let r = layout.toolbar;
    let half = r.w / 2;
    [
        Rect {
            x: r.x + 4,
            y: r.y + 6,
            w: half.saturating_sub(6),
            h: 18,
        },
        Rect {
            x: r.x + half + 4,
            y: r.y + 6,
            w: r.w.saturating_sub(half + 8),
            h: 18,
        },
    ]
}
pub fn gallery_ui(b: &mut Framebuffer, layout: &GalleryLayout) {
    if let Some(r) = layout.list {
        layout.list_card.draw(b, r.x, r.y);
    }
    layout
        .info_card
        .draw(b, layout.information.x, layout.information.y);
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn alta_automatica_respeta_rendimiento_y_eleccion_manual() {
        let mut g = Gallery::new();
        for _ in 0..4 {
            g.observe_render(100.0);
            assert_eq!(g.quality, crate::quality::Quality::High);
        }
        g.observe_render(100.0);
        assert_eq!(g.quality, crate::quality::Quality::Normal);
        g.toggle_quality();
        assert_eq!(g.quality, crate::quality::Quality::High);
        for _ in 0..10 {
            g.observe_render(200.0);
        }
        assert_eq!(g.quality, crate::quality::Quality::High);
        g.reset();
        g.observe_render(200.0);
        assert_eq!(g.quality, crate::quality::Quality::High);
    }

    #[test]
    fn inspeccion_persistente_pan_zoom_detalle_y_restauracion() {
        let mut g = Gallery::new();
        g.selected = 4;
        g.detail(2.05);
        let c = g.camera(4.92, 730, 450);
        assert!(c.target.y > 1.9 && c.target.y < 2.1);
        assert!(c.position.distance(c.target) < 3.0);
        g.inspect(
            GalleryInput {
                mouse: Some((20.0, 20.0)),
                right: true,
                dt: 0.016,
                height: 450,
                ..Default::default()
            },
            &c,
        );
        g.inspect(
            GalleryInput {
                mouse: Some((80.0, 60.0)),
                right: true,
                dt: 0.016,
                height: 450,
                ..Default::default()
            },
            &c,
        );
        let panned = g.camera(4.92, 730, 450);
        assert!(panned.target.distance(c.target) > 0.01);
        let before = g.yaw;
        g.inspect(
            GalleryInput {
                horizontal: 1.0,
                vertical: 1.0,
                dt: 0.1,
                height: 450,
                ..Default::default()
            },
            &panned,
        );
        assert_ne!(g.yaw, before);
        let focus = g.focus;
        g.inspect(
            GalleryInput {
                dt: 0.016,
                ..Default::default()
            },
            &panned,
        );
        assert_eq!(g.focus, focus);
        for _ in 0..200 {
            g.inspect(
                GalleryInput {
                    wheel: 12.0,
                    vertical: 1.0,
                    dt: 0.016,
                    height: 450,
                    ..Default::default()
                },
                &panned,
            );
        }
        let close = g.camera(4.92, 730, 450);
        assert!(close.position.distance(close.target) < 0.2);
        assert!(close.near > 0.0 && close.near < 0.01);
        assert!(close.view_matrix().is_finite());
        g.inspect(
            GalleryInput {
                mouse: Some((f32::NAN, 0.0)),
                wheel: f32::NAN,
                dt: 0.016,
                ..Default::default()
            },
            &close,
        );
        assert!(g.camera(4.92, 730, 450).view_matrix().is_finite());
        g.reset();
        assert_eq!(g.focus, Vec3::ZERO);
        assert_eq!(g.zoom, 1.0);
        assert!(!g.detail_view);
        assert_eq!(g.selected, 4);
    }

    #[test]
    fn paneles_y_encuadre_caben_en_tamanos_de_entrega() {
        for (width, height) in [(800, 600), (1000, 650), (1280, 720), (480, 640)] {
            for selected in 0..5 {
                for help in [false, true] {
                    let layout = gallery_layout(width, height, selected, help);
                    let mut rects = vec![layout.viewport, layout.information];
                    if let Some(list) = layout.list {
                        rects.push(list);
                        assert!(list.x + list.w <= layout.viewport.x);
                    } else {
                        assert!(layout.information.y >= layout.viewport.bottom());
                    }
                    for r in rects {
                        assert!(r.w > 0 && r.h > 0);
                        assert!(
                            r.x + r.w <= width && r.bottom() <= height - ui::footer_height(width)
                        );
                    }
                    let c = Gallery::new().camera(4.92, layout.viewport.w, layout.viewport.h);
                    let vertical = c.fov_y * 0.5;
                    let horizontal = (vertical.tan() * layout.viewport.w as f32
                        / layout.viewport.h as f32)
                        .atan();
                    assert!(c.position.length() * vertical.min(horizontal).sin() > 4.92);
                }
            }
        }
    }

    #[test]
    fn pantallas_y_controles_aislados() {
        let mut s = Screens::new();
        s.navigate(1);
        s.enter();
        assert_eq!(s.screen, Screen::Gallery);
        s.navigate(-1);
        assert_eq!(s.gallery.selected, 4);
        s.escape();
        assert_eq!(s.screen, Screen::Menu);
        s.navigate(-1);
        s.enter();
        assert_eq!(s.screen, Screen::Flight);
        let index = s.gallery.selected;
        s.navigate(1);
        assert_eq!(s.gallery.selected, index);
        s.escape();
        assert_eq!(s.screen, Screen::Menu);
        s.escape();
        assert_eq!(s.screen, Screen::Exit);
    }
    #[test]
    fn encuadre_completo_y_zoom_acotado() {
        let mut g = Gallery::new();
        for bound in [2.035, 2.4975, 2.7, 2.375, 4.92] {
            let c = g.camera(bound, 730, 560);
            assert!(c.position.length() * (c.fov_y * 0.5).sin() > bound);
        }
        let camera = g.camera(4.92, 730, 560);
        for _ in 0..120 {
            g.inspect(
                GalleryInput {
                    wheel: 12.0,
                    dt: 1.0 / 60.0,
                    height: 560,
                    ..Default::default()
                },
                &camera,
            );
        }
        assert!(g.zoom >= 0.008 && g.zoom < 0.009);
        for _ in 0..120 {
            g.inspect(
                GalleryInput {
                    wheel: -12.0,
                    dt: 1.0 / 60.0,
                    height: 560,
                    ..Default::default()
                },
                &camera,
            );
        }
        assert!(g.zoom <= 8.0 && g.zoom > 7.99);
        g.reset();
        assert_eq!(g.zoom, 1.0);
        assert!(g.drag.is_none());
    }
}
