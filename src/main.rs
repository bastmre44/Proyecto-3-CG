mod audio;
mod camera;
mod collision;
mod flight;
mod fog;
mod framebuffer;
mod koi;
mod local_orbit;
mod navigation;
mod obj_loader;
mod opalite;
mod opalite_plaza;
mod ophelia;
mod ophelia_garden;
mod patient_garden;
mod patient_manor;
mod patient_zero;
mod pink_clouding;
mod pipeline;
mod presentation;
mod quality;
mod renderer;
mod screens;
mod shader;
mod showgirl;
mod showgirl_mesh;
mod showgirl_performer;
mod showgirl_stage;
mod simulation;
mod sky;
mod sparkle;
mod sphere;
mod sun;
mod triangle;
mod ui;
mod window_size;

use std::time::Instant;

use camera::Camera;
use flight::{FlightController, FlightInput};
use framebuffer::Framebuffer;
use glam::{Mat4, Quat, Vec3};
use local_orbit::LocalOrbit;
use minifb::{Key, KeyRepeat, MouseButton, MouseMode, Window, WindowOptions};
use obj_loader::Model;
use screens::Screen;
use shader::Material;

const BACKGROUND: u32 = 0x101522;

// true: planetas quietos para revisar detalles.
// false: órbitas y rotación normales.
const INSPECTION_MODE: bool = false;

struct Planet {
    name: &'static str,
    radius: f32,
    orbit_radius: f32,
    orbit_speed: f32,
    orbit_phase: f32,
    rotation_speed: f32,
    material: Material,
}

impl Planet {
    fn position(&self, time: f32) -> Vec3 {
        let time = if INSPECTION_MODE { 0.0 } else { time };

        let angle = time * self.orbit_speed + self.orbit_phase;

        navigation::SUN_CENTER
            + Vec3::new(
                angle.cos() * self.orbit_radius,
                0.0,
                angle.sin() * self.orbit_radius,
            )
    }

    fn collision_radius(&self) -> f32 {
        match self.material {
            Material::Ophelia => self.radius * ophelia_garden::BOUND_RADIUS,

            Material::Opalite => self.radius * opalite::BOUND_RADIUS,

            Material::PatientZero => self.radius * patient_zero::BOUND_RADIUS,

            Material::PinkClouding => self.radius * pink_clouding::BOUND_RADIUS,

            Material::Showgirl => self.radius * showgirl::BOUND_RADIUS,

            _ => self.radius,
        }
    }

    fn model_matrix(&self, time: f32) -> Mat4 {
        let time = if INSPECTION_MODE { 0.0 } else { time };

        Mat4::from_translation(self.position(time))
            * Mat4::from_rotation_y(time * self.rotation_speed)
            * Mat4::from_scale(Vec3::splat(self.radius))
    }
}

fn input_axis(window: &Window, positive: Key, negative: Key) -> f32 {
    let positive = window.is_key_down(positive) as i32;
    let negative = window.is_key_down(negative) as i32;

    (positive - negative) as f32
}

fn draw_object(
    model: &Model,
    model_matrix: Mat4,
    camera: &Camera,
    buffer: &mut Framebuffer,
    color: u32,
    unlit: bool,
) {
    let clip_vertices = pipeline::transform_vertices(
        model,
        model_matrix,
        camera.view_matrix(),
        camera.projection_matrix(buffer.width, buffer.height),
    );

    renderer::render(model, &clip_vertices, buffer, model_matrix, color, unlit);
}

fn draw_shaded_object(
    model: &Model,
    model_matrix: Mat4,
    camera: &Camera,
    buffer: &mut Framebuffer,
    material: Material,
    time: f32,
) {
    let clip_vertices = pipeline::transform_vertices(
        model,
        model_matrix,
        camera.view_matrix(),
        camera.projection_matrix(buffer.width, buffer.height),
    );

    renderer::render_shaded(model, &clip_vertices, buffer, model_matrix, material, time);
}

fn draw_gallery(
    gallery: &screens::Gallery,
    planet: &Planet,
    bound: f32,
    sky: &sky::Sky,
    buffer: &mut Framebuffer,
    viewport: &mut Framebuffer,
    elapsed: f32,
    help: bool,
    draw_planet: &impl Fn(usize, Mat4, &Camera, &mut Framebuffer, f32),
) {
    let mut layout = screens::gallery_layout(buffer.width, buffer.height, gallery.selected, help);
    // A native window can briefly report a client smaller than its normal UI.
    // Keep the compositing rectangle valid even during those resize events.
    layout.viewport.x = layout.viewport.x.min(buffer.width.saturating_sub(1));
    layout.viewport.y = layout.viewport.y.min(buffer.height.saturating_sub(1));
    layout.viewport.w = layout
        .viewport
        .w
        .min(buffer.width - layout.viewport.x)
        .max(1);
    layout.viewport.h = layout
        .viewport
        .h
        .min(buffer.height - layout.viewport.y)
        .max(1);
    let factor = gallery.quality.factor();
    viewport.resize(layout.viewport.w * factor, layout.viewport.h * factor);
    let mut camera = gallery.camera(bound, layout.viewport.w, layout.viewport.h);
    // Keep the inspection point attached to its planet while preserving user controls.
    let rotation = Mat4::from_rotation_y(elapsed * planet.rotation_speed);
    camera.position = rotation.transform_point3(camera.position);
    camera.target = rotation.transform_point3(camera.target);
    let matrix = Mat4::from_rotation_y(elapsed * planet.rotation_speed)
        * Mat4::from_scale(Vec3::splat(planet.radius));
    viewport.clear(BACKGROUND);
    sky.draw(&camera, viewport, elapsed);
    draw_planet(gallery.selected, matrix, &camera, viewport, elapsed);
    if gallery.selected == 2 {
        fog::draw(&camera, viewport, matrix, elapsed);
    }
    quality::resolve(viewport, buffer, layout.viewport, gallery.quality);
    screens::gallery_ui(buffer, &layout);
    for (index, rect) in screens::view_buttons(&layout).iter().enumerate() {
        presentation::panel(buffer, rect.x, rect.y, rect.w, rect.h);
        ui::wrapped(
            buffer,
            *rect,
            if index == 0 {
                "1: VISTA GENERAL"
            } else {
                "2: DETALLE PRINCIPAL"
            },
            1,
            if gallery.detail_view == (index == 1) {
                ui::GOLD
            } else {
                ui::CREAM
            },
        );
    }
}

fn save_preview(buffer: &Framebuffer, path: &std::path::Path) -> Result<(), String> {
    use std::io::Write;
    let mut file = std::io::BufWriter::new(std::fs::File::create(path).map_err(|e| e.to_string())?);
    write!(file, "P6\n{} {}\n255\n", buffer.width, buffer.height).map_err(|e| e.to_string())?;
    for color in &buffer.pixels {
        file.write_all(&[(color >> 16) as u8, (color >> 8) as u8, *color as u8])
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn run() -> Result<(), String> {
    run_scene(None)
}

fn run_scene(export_gallery: Option<&std::path::Path>) -> Result<(), String> {
    let ship_asset = obj_loader::load_obj_materials("Nave_final.obj")?;
    let ship = &ship_asset.model;
    let sphere = sphere::create_sphere(12, 24)?;
    let sun_model = sun::create();
    let sky = sky::Sky::new();
    let mut presentation = presentation::Presentation::new();
    let mut screens = screens::Screens::new();
    let mut gallery_buffer = Framebuffer::new(1, 1);
    let mut scene_buffer = Framebuffer::new(1, 1);
    let mut world_quality = quality::Quality::Normal;
    let mut system_time = 0.0_f32;
    let mut simulation_clock = simulation::SimulationClock::default();
    let koi_model = koi::create_koi()?;
    let garden = ophelia_garden::create_garden();
    let opalite_crystals = opalite::create_crystals();
    let plaza = opalite_plaza::create_plaza()?;
    let patient_surface = patient_zero::create_surface(12, 24)?;
    let patient_snow = patient_zero::create_snow(&patient_surface);
    let manor = patient_manor::create_manor();
    let roof_snow = patient_zero::create_roof_snow(&manor[1].model);
    let patient_garden = patient_garden::create_garden();
    let pink_decorations = pink_clouding::create_decorations()?;
    let pink_surface = pink_clouding::create_surface();
    let showgirl_world = showgirl::ShowgirlWorld::new();

    if ship.vertices.is_empty() || ship.faces.is_empty() {
        return Err("La nave no contiene geometría para dibujar".into());
    }

    let mut minimum = Vec3::splat(f32::INFINITY);
    let mut maximum = Vec3::splat(f32::NEG_INFINITY);

    for vertex in &ship.vertices {
        let position = Vec3::new(vertex.x, vertex.y, vertex.z);

        minimum = minimum.min(position);
        maximum = maximum.max(position);
    }

    let center = (minimum + maximum) * 0.5;

    let ship_scale = 0.45_f32;
    let sun_radius = 1.5_f32;
    let lavender_radius = 1.1_f32;
    let midnight_radius = 1.35_f32;

    let ship_radius = ship
        .vertices
        .iter()
        .map(|vertex| Vec3::new(vertex.x, vertex.y, vertex.z).distance(center))
        .fold(0.0_f32, f32::max)
        * ship_scale;

    // Separar las órbitas dejando espacio para la nave.
    let lavender_extent = lavender_radius * ophelia_garden::BOUND_RADIUS;

    let lavender_orbit = navigation::orbital_radius(
        0.0,
        sun_radius * sun::BOUND_RADIUS,
        lavender_extent,
        ship_radius,
    );

    let midnight_extent = midnight_radius * opalite::BOUND_RADIUS;

    let midnight_orbit = navigation::orbital_radius(
        lavender_orbit,
        lavender_extent,
        midnight_extent,
        ship_radius,
    );

    let patient_radius = 1.2_f32;

    let patient_extent = patient_radius * patient_zero::BOUND_RADIUS;

    let patient_orbit =
        navigation::orbital_radius(midnight_orbit, midnight_extent, patient_extent, ship_radius);

    let pink_radius = 1.25_f32;
    let pink_extent = pink_radius * pink_clouding::BOUND_RADIUS;
    let pink_orbit =
        navigation::orbital_radius(patient_orbit, patient_extent, pink_extent, ship_radius);

    let showgirl_radius = showgirl::RECOMMENDED_RADIUS;
    let showgirl_extent = showgirl_radius * showgirl::BOUND_RADIUS;
    let showgirl_orbit =
        navigation::orbital_radius(pink_orbit, pink_extent, showgirl_extent, ship_radius);

    let planets = [
        Planet {
            name: "The Fate of Ophelia",
            radius: lavender_radius,
            orbit_radius: lavender_orbit,
            orbit_speed: 0.55 / lavender_orbit,
            orbit_phase: std::f32::consts::TAU * 0.0 / 5.0,
            rotation_speed: 0.12,
            material: Material::Ophelia,
        },
        Planet {
            name: "Opalite",
            radius: midnight_radius,
            orbit_radius: midnight_orbit,
            orbit_speed: 0.55 / midnight_orbit,
            orbit_phase: std::f32::consts::TAU * 1.0 / 5.0,
            rotation_speed: 0.10,
            material: Material::Opalite,
        },
        Planet {
            name: "Patient Zero",
            radius: patient_radius,
            orbit_radius: patient_orbit,
            orbit_speed: 0.55 / patient_orbit,
            orbit_phase: std::f32::consts::TAU * 2.0 / 5.0,
            rotation_speed: 0.08,
            material: Material::PatientZero,
        },
        Planet {
            name: "Pink Clouding",
            radius: pink_radius,
            orbit_radius: pink_orbit,
            orbit_speed: 0.55 / pink_orbit,
            orbit_phase: std::f32::consts::TAU * 3.0 / 5.0,
            rotation_speed: 0.07,
            material: Material::PinkClouding,
        },
        Planet {
            name: "The Life of a Showgirl",
            radius: showgirl_radius,
            orbit_radius: showgirl_orbit,
            orbit_speed: 0.55 / showgirl_orbit,
            orbit_phase: std::f32::consts::TAU * 4.0 / 5.0,
            rotation_speed: 0.055,
            material: Material::Showgirl,
        },
    ];

    let koi_extent = koi_model
        .vertices
        .iter()
        .map(|v| Vec3::new(v.x, v.y, v.z).length())
        .fold(0.0_f32, f32::max)
        * 0.65
        + 0.15;
    let mut gallery_bounds = planets.each_ref().map(|p| p.collision_radius());
    gallery_bounds[0] = gallery_bounds[0].max(planets[0].radius + 1.0 + koi_extent);
    let draw_planet =
        |index: usize, matrix: Mat4, camera: &Camera, buffer: &mut Framebuffer, elapsed: f32| {
            {
                let planet = &planets[index];
                let model = match planet.material {
                    Material::PatientZero => &patient_surface,
                    Material::PinkClouding => &pink_surface,
                    Material::Showgirl => &showgirl_world.surface,
                    _ => &sphere,
                };

                draw_shaded_object(model, matrix, camera, buffer, planet.material, elapsed);
            }

            // El jardín comparte la transformación de Ophelia.
            // Así orbita y gira junto con el planeta.

            match index {
                0 => {
                    let ophelia_matrix = matrix;

                    for part in &garden {
                        draw_object(
                            &part.model,
                            ophelia_matrix,
                            camera,
                            buffer,
                            part.color,
                            false,
                        );
                    }

                    let lavender_position = matrix.transform_point3(Vec3::ZERO);
                    let koi_orbit_radius = planets[0].radius + 1.0;
                    let orbit_tilt = Quat::from_rotation_x(0.3);

                    for index in 0..2 {
                        let phase = index as f32 * std::f32::consts::PI;
                        let angle = elapsed * 0.5 + phase;

                        let offset = Vec3::new(
                            angle.cos() * koi_orbit_radius,
                            0.0,
                            angle.sin() * koi_orbit_radius,
                        );

                        let position = lavender_position + orbit_tilt * offset;

                        let orientation = orbit_tilt * Quat::from_rotation_y(-angle);

                        let koi_matrix = Mat4::from_scale_rotation_translation(
                            Vec3::splat(0.65),
                            orientation,
                            position,
                        );

                        let swimming_koi = koi::animate_koi(&koi_model, elapsed, phase);

                        draw_shaded_object(
                            &swimming_koi,
                            koi_matrix,
                            camera,
                            buffer,
                            Material::Koi,
                            elapsed,
                        );
                    }
                }
                1 => {
                    let opalite_matrix = matrix;

                    for part in &opalite_crystals {
                        draw_object(
                            &part.model,
                            opalite_matrix,
                            camera,
                            buffer,
                            part.color,
                            false,
                        );
                    }

                    for part in &plaza {
                        draw_object(
                            &part.model,
                            opalite_matrix,
                            camera,
                            buffer,
                            part.color,
                            false,
                        );
                    }
                }
                2 => {
                    let patient_matrix = matrix;

                    for part in &manor {
                        draw_object(
                            &part.model,
                            patient_matrix,
                            camera,
                            buffer,
                            part.color,
                            part.unlit,
                        );
                    }

                    for part in &patient_garden {
                        draw_object(
                            &part.model,
                            patient_matrix,
                            camera,
                            buffer,
                            part.color,
                            false,
                        );
                    }

                    // Cubierta de nieve sobre la roca y los techos.
                    draw_object(
                        &patient_snow,
                        patient_matrix,
                        camera,
                        buffer,
                        0xE7ECFA,
                        false,
                    );
                    draw_object(&roof_snow, patient_matrix, camera, buffer, 0xEDF0FA, false);
                }
                3 => {
                    let pink_matrix = matrix;

                    for part in &pink_decorations {
                        draw_object(
                            &part.model,
                            pink_matrix * part.local_matrix(elapsed),
                            camera,
                            buffer,
                            part.color,
                            false,
                        );
                    }
                }
                4 => {
                    showgirl_world.visit_parts(elapsed, matrix, |model, matrix, color, unlit| {
                        draw_object(model, matrix, camera, buffer, color, unlit);
                    });
                }
                _ => unreachable!(),
            }
        };

    if let Some(directory) = export_gallery {
        std::fs::create_dir_all(directory).map_err(|e| e.to_string())?;
        let mut preview = Framebuffer::new(1, 1);
        let mut gallery = screens::Gallery::new();
        for (width, height) in [(800, 600), (1000, 650), (1280, 720), (480, 640)] {
            preview.resize(width, height);
            for index in 0..planets.len() {
                gallery.selected = index;
                preview.clear(BACKGROUND);
                draw_gallery(
                    &gallery,
                    &planets[index],
                    gallery_bounds[index],
                    &sky,
                    &mut preview,
                    &mut gallery_buffer,
                    1.0,
                    true,
                    &draw_planet,
                );
                ui::footer(
                    &mut preview,
                    "AUDIO: LISTO | VOL 25% | M: SILENCIO -/+: VOLUMEN",
                );
                save_preview(
                    &preview,
                    &directory.join(format!("showgirl_gallery_{width}x{height}_{index}.ppm")),
                )?;
            }
            let overview = Camera::new(
                Vec3::new(
                    showgirl_orbit * 0.25,
                    showgirl_orbit * 0.85,
                    showgirl_orbit * 1.55,
                ),
                Vec3::ZERO,
            );
            preview.clear(BACKGROUND);
            preview.clear(BACKGROUND);
            sky.draw(&overview, &mut preview, 1.0);
            for (index, planet) in planets.iter().enumerate() {
                draw_planet(
                    index,
                    planet.model_matrix(0.0),
                    &overview,
                    &mut preview,
                    1.0,
                );
            }
            screens::menu(&mut preview, 0, true);
            ui::footer(
                &mut preview,
                "AUDIO: LISTO | VOL 25% | M: SILENCIO -/+: VOLUMEN",
            );
            save_preview(
                &preview,
                &directory.join(format!("showgirl_menu_{width}x{height}.ppm")),
            )?;
            preview.clear(BACKGROUND);
            sky.draw(&overview, &mut preview, 1.0);
            for (index, planet) in planets.iter().enumerate() {
                draw_planet(
                    index,
                    planet.model_matrix(0.0),
                    &overview,
                    &mut preview,
                    1.0,
                );
            }
            let top = presentation::hud(
                &mut preview,
                "The Life of a Showgirl",
                12.5,
                "LIBRE | C: rodear planeta",
                true,
            );
            presentation::target(
                &mut preview,
                &overview,
                "The Fate of Ophelia",
                planets[0].position(0.0),
                28.5,
                0,
                top,
            );
            ui::footer(
                &mut preview,
                "AUDIO: LISTO | VOL 25% | M: SILENCIO -/+: VOLUMEN",
            );
            save_preview(
                &preview,
                &directory.join(format!("showgirl_flight_{width}x{height}.ppm")),
            )?;
        }
        preview.resize(1000, 650);
        gallery.selected = 4;
        gallery.detail(planets[4].radius);
        for quality in [quality::Quality::Normal, quality::Quality::High] {
            gallery.quality = quality;
            preview.clear(BACKGROUND);
            draw_gallery(
                &gallery,
                &planets[4],
                gallery_bounds[4],
                &sky,
                &mut preview,
                &mut gallery_buffer,
                1.0,
                false,
                &draw_planet,
            );
            let start = std::time::Instant::now();
            for _ in 0..8 {
                preview.clear(BACKGROUND);
                draw_gallery(
                    &gallery,
                    &planets[4],
                    gallery_bounds[4],
                    &sky,
                    &mut preview,
                    &mut gallery_buffer,
                    1.0,
                    false,
                    &draw_planet,
                );
            }
            let ms = start.elapsed().as_secs_f64() * 1000.0 / 8.0;
            let layout = screens::gallery_layout(preview.width, preview.height, 4, false);
            println!(
                "Showgirl detalle {}: viewport {}x{}, render {}x{}, {:.2} ms/frame ({:.1} fps de render CPU)",
                quality.label(),
                layout.viewport.w,
                layout.viewport.h,
                gallery_buffer.width,
                gallery_buffer.height,
                ms,
                1000.0 / ms
            );
            ui::footer(
                &mut preview,
                &format!(
                    "Q: {} | COMPARACION MISMO ANGULO Y TIEMPO | 1000X650",
                    quality.label()
                ),
            );
            save_preview(
                &preview,
                &directory.join(format!(
                    "showgirl_detail_{}.ppm",
                    quality.label().replace(' ', "_")
                )),
            )?;
        }
        return Ok(());
    }
    let mut audio = audio::Audio::start(audio::music_path());

    // Above the orbital plane: clear of every collision sphere at every phase.
    let orbit_rings: Vec<_> = planets
        .iter()
        .map(|p| navigation::orbit_ring(p.orbit_radius))
        .collect();
    let mut show_orbits = false;
    let mut target_index = 0;
    let mut ship_view = 0;
    let ship_start = Vec3::new(0.0, 10.0 + ship_radius, showgirl_orbit * 0.65);
    assert!(ship_start.length() > sun_radius * sun::BOUND_RADIUS + ship_radius);
    assert!(
        planets
            .iter()
            .all(|p| ship_start.y > p.collision_radius() + ship_radius)
    );
    let initial_forward = (Vec3::ZERO - ship_start).normalize();
    let mut flight = FlightController::new(ship_start);
    flight.set_pose(ship_start, initial_forward);

    // Guarda el planeta elegido y su controlador local.
    let mut local_mode: Option<(usize, LocalOrbit)> = None;

    let mut camera = Camera::new(
        flight.position - flight.forward() * 6.0 + Vec3::Y * 2.5,
        flight.position + flight.forward() * 2.0,
    );

    let overview_position = Vec3::new(
        showgirl_orbit * 0.25,
        showgirl_orbit * 0.85,
        showgirl_orbit * 1.55,
    );
    let overview_target = Vec3::ZERO;
    camera.position = overview_position;
    camera.target = overview_target;

    let sun_matrix = Mat4::from_scale(Vec3::splat(sun_radius));
    let (initial_width, initial_height) = window_size::initial_size();
    let mut buffer = Framebuffer::new(initial_width, initial_height);

    let mut window = Window::new(
        "The Showgirl Universe",
        initial_width,
        initial_height,
        WindowOptions {
            resize: true,
            scale: minifb::Scale::X1,
            ..WindowOptions::default()
        },
    )
    .map_err(|error| format!("No se pudo crear la ventana: {error}"))?;

    window.set_target_fps(60);

    let start = Instant::now();
    let mut previous = start;

    while window.is_open() && screens.screen != Screen::Exit {
        let now = Instant::now();
        let frame_duration = now.duration_since(previous);
        let delta_time = frame_duration.as_secs_f32();
        let elapsed = now.duration_since(start).as_secs_f32();
        let previous_time = system_time;

        previous = now;

        let (width, height) = window.get_size();
        if width == 0 || height == 0 {
            // Pump native messages while minimized, without allocating zero-sized buffers
            // or updating flight. Audio owns an independent playback thread.
            window.update();
            audio.update(false, 0.0);
            screens.gallery.end_drag();
            std::thread::sleep(std::time::Duration::from_millis(16));
            continue;
        }
        buffer.resize(width, height);
        let active = window.is_active();
        if active && window.is_key_pressed(Key::Q, KeyRepeat::No) {
            if screens.screen == Screen::Gallery {
                screens.gallery.toggle_quality();
            } else {
                world_quality.toggle();
            }
        }
        if active && window.is_key_pressed(Key::H, KeyRepeat::No) {
            presentation.help = !presentation.help;
        }
        if active {
            let volume_delta = if window.is_key_pressed(Key::Equal, KeyRepeat::Yes)
                || window.is_key_pressed(Key::NumPadPlus, KeyRepeat::Yes)
            {
                0.05
            } else {
                0.0
            } - if window.is_key_pressed(Key::Minus, KeyRepeat::Yes)
                || window.is_key_pressed(Key::NumPadMinus, KeyRepeat::Yes)
            {
                0.05
            } else {
                0.0
            };
            audio.update(window.is_key_pressed(Key::M, KeyRepeat::No), volume_delta);
            if window.is_key_pressed(Key::Escape, KeyRepeat::No) {
                screens.escape();
            }
        } else {
            audio.update(false, 0.0);
        }
        if screens.screen == Screen::Exit {
            break;
        }
        if screens.screen == Screen::Menu && active {
            if window.is_key_pressed(Key::Up, KeyRepeat::Yes) {
                screens.navigate(-1);
            }
            if window.is_key_pressed(Key::Down, KeyRepeat::Yes) {
                screens.navigate(1);
            }
            if window.is_key_pressed(Key::Enter, KeyRepeat::No) {
                screens.enter();
                if screens.screen == Screen::Flight {
                    presentation.started = true;
                }
            }
        }
        if screens.screen != Screen::Flight {
            buffer.clear(BACKGROUND);
            if screens.screen == Screen::Gallery {
                if active {
                    if window.is_key_pressed(Key::Left, KeyRepeat::Yes) {
                        screens.navigate(-1);
                    }
                    if window.is_key_pressed(Key::Right, KeyRepeat::Yes) {
                        screens.navigate(1);
                    }
                    let index = screens.gallery.selected;
                    let layout = screens::gallery_layout(
                        buffer.width,
                        buffer.height,
                        index,
                        presentation.help,
                    );
                    let mouse = window.get_mouse_pos(MouseMode::Discard);
                    let left = window.get_mouse_down(MouseButton::Left);
                    let right = window.get_mouse_down(MouseButton::Right);
                    let click = left && !screens.gallery.left_down;
                    let buttons = screens::view_buttons(&layout);
                    if window.is_key_pressed(Key::R, KeyRepeat::No)
                        || window.is_key_pressed(Key::Key1, KeyRepeat::No)
                        || (click
                            && mouse.is_some_and(|(x, y)| screens::contains(buttons[0], x, y)))
                    {
                        screens.gallery.reset();
                    }
                    if window.is_key_pressed(Key::Key2, KeyRepeat::No)
                        || (click
                            && mouse.is_some_and(|(x, y)| screens::contains(buttons[1], x, y)))
                    {
                        screens.gallery.detail(planets[index].radius);
                    }
                    let in_view =
                        mouse.is_some_and(|(x, y)| screens::contains(layout.viewport, x, y));
                    let camera = screens.gallery.camera(
                        gallery_bounds[index],
                        layout.viewport.w,
                        layout.viewport.h,
                    );
                    screens.gallery.inspect(
                        screens::GalleryInput {
                            mouse: if in_view || screens.gallery.drag.is_some() {
                                mouse
                            } else {
                                None
                            },
                            left,
                            right: right && in_view,
                            wheel: if in_view {
                                window.get_scroll_wheel().map_or(0.0, |(_, y)| y)
                            } else {
                                0.0
                            },
                            horizontal: input_axis(&window, Key::D, Key::A),
                            vertical: input_axis(&window, Key::W, Key::S),
                            dt: delta_time,
                            height: layout.viewport.h,
                        },
                        &camera,
                    );
                } else {
                    screens.gallery.end_drag();
                }
                let index = screens.gallery.selected;
                let planet = &planets[index];
                let render_start = std::time::Instant::now();
                draw_gallery(
                    &screens.gallery,
                    planet,
                    gallery_bounds[index],
                    &sky,
                    &mut buffer,
                    &mut gallery_buffer,
                    elapsed,
                    presentation.help,
                    &draw_planet,
                );
                screens
                    .gallery
                    .observe_render(render_start.elapsed().as_secs_f32() * 1000.0);
                window.set_title(&format!(
                    "The Showgirl Universe | Galeria | {}",
                    planet.name
                ));
            } else {
                let factor = world_quality.factor();
                scene_buffer.resize(buffer.width * factor, buffer.height * factor);
                scene_buffer.clear(BACKGROUND);
                let overview = Camera::new(overview_position, overview_target);
                sky.draw(&overview, &mut scene_buffer, elapsed);
                draw_shaded_object(
                    &sun_model,
                    sun_matrix,
                    &overview,
                    &mut scene_buffer,
                    Material::Sun,
                    elapsed,
                );
                for (index, planet) in planets.iter().enumerate() {
                    draw_planet(
                        index,
                        planet.model_matrix(system_time),
                        &overview,
                        &mut scene_buffer,
                        elapsed,
                    );
                }
                fog::draw(
                    &overview,
                    &mut scene_buffer,
                    planets[2].model_matrix(system_time),
                    elapsed,
                );
                quality::resolve(
                    &scene_buffer,
                    &mut buffer,
                    ui::Rect {
                        x: 0,
                        y: 0,
                        w: width,
                        h: height,
                    },
                    world_quality,
                );
                screens::menu(&mut buffer, screens.menu, presentation.help);
                window.set_title("The Showgirl Universe | Menu");
            }
            ui::footer(
                &mut buffer,
                &format!(
                    "{} | Q: {}",
                    audio.label(),
                    if screens.screen == Screen::Gallery {
                        screens.gallery.quality_label()
                    } else {
                        world_quality.label().to_string()
                    }
                ),
            );
            window
                .update_with_buffer(&buffer.pixels, buffer.width, buffer.height)
                .map_err(|e| format!("Error al mostrar pantalla: {e}"))?;
            continue;
        }
        presentation.update(false, false, delta_time);
        if presentation.started && window.is_key_pressed(Key::R, KeyRepeat::No) {
            flight = FlightController::new(ship_start);
            flight.set_pose(ship_start, initial_forward);
            local_mode = None;

            camera.position = flight.position - flight.forward() * 6.0 + Vec3::Y * 2.5;

            camera.target = flight.position + flight.forward() * 2.0;

            camera.up = Vec3::Y;
        }

        if presentation.started && window.is_active() {
            if window.is_key_pressed(Key::O, KeyRepeat::No) {
                show_orbits = !show_orbits;
            }
            if window.is_key_pressed(Key::Tab, KeyRepeat::No) {
                target_index = (target_index + 1) % planets.len();
            }
            if window.is_key_pressed(Key::V, KeyRepeat::No) {
                ship_view = (ship_view + 1) % 3;
            }
        }
        let input = if window.is_active() && presentation.started {
            FlightInput {
                throttle: input_axis(&window, Key::W, Key::S),
                yaw: input_axis(&window, Key::A, Key::D),
                pitch: input_axis(&window, Key::Up, Key::Down),
            }
        } else {
            FlightInput::default()
        };

        // Buscar el planeta cuya superficie esté más cerca.
        let nearest_index = (0..planets.len())
            .min_by(|&a, &b| {
                let distance_a =
                    flight.position.distance(planets[a].position(system_time)) - planets[a].radius;

                let distance_b =
                    flight.position.distance(planets[b].position(system_time)) - planets[b].radius;

                distance_a.total_cmp(&distance_b)
            })
            .unwrap();

        if presentation.started
            && window.is_active()
            && window.is_key_pressed(Key::C, KeyRepeat::No)
        {
            if local_mode.is_some() {
                local_mode = None;

                let position = flight.position;
                let forward = flight.forward();

                flight.set_pose(position, forward);
            } else {
                let planet = &planets[nearest_index];
                let planet_position = planet.position(system_time);
                let minimum_distance = planet.collision_radius() + ship_radius + 0.01;

                if flight.position.distance(planet_position) <= minimum_distance + 4.0 {
                    local_mode = Some((
                        nearest_index,
                        LocalOrbit::new(flight.position, planet_position, minimum_distance),
                    ));
                }
            }
        }

        let boost = window.is_active()
            && (window.is_key_down(Key::LeftShift) || window.is_key_down(Key::RightShift));

        // El render no determina la rapidez de la simulacion. Conservamos todo
        // el tiempo de vuelo y revisamos colisiones en cada paso de 1/120 s.
        simulation_clock.advance(frame_duration, |step| {
            let previous_ship_position = flight.position;

            if let Some((index, orbit)) = &mut local_mode {
                let planet_position = planets[*index].position(step.current_time);
                orbit.update(&input, step.delta_time);
                flight.set_pose(orbit.position(planet_position), orbit.forward());
            } else {
                let clearance = planets
                    .iter()
                    .map(|p| {
                        flight.position.distance(p.position(step.previous_time))
                            - p.collision_radius()
                            - ship_radius
                    })
                    .fold(
                        flight.position.length() - sun_radius * sun::BOUND_RADIUS - ship_radius,
                        f32::min,
                    );
                flight.update_with_speed(
                    &input,
                    step.delta_time,
                    navigation::travel_speed(clearance, boost),
                );
            }

            let bodies = std::iter::once((
                navigation::SUN_CENTER,
                navigation::SUN_CENTER,
                sun_radius * sun::BOUND_RADIUS,
            ))
            .chain(planets.iter().map(|planet| {
                (
                    planet.position(step.previous_time),
                    planet.position(step.current_time),
                    planet.collision_radius(),
                )
            }));
            let mut collided = false;
            for (previous_center, current_center, body_radius) in bodies {
                if let Some(corrected_position) = collision::resolve_sphere_collision(
                    previous_ship_position,
                    flight.position,
                    previous_center,
                    current_center,
                    body_radius + ship_radius,
                ) {
                    flight.position = corrected_position;
                    collided = true;
                }
            }

            if let Some((index, orbit)) = &mut local_mode {
                let planet = &planets[*index];
                let planet_position = planet.position(step.current_time);
                flight.set_pose(flight.position, planet_position - flight.position);
                if collided {
                    *orbit = LocalOrbit::new(
                        flight.position,
                        planet_position,
                        planet.collision_radius() + ship_radius + 0.01,
                    );
                }
            }
        });
        system_time = simulation_clock.time();

        // La camara conserva el seguimiento por cuadro, con el intervalo completo.
        let mut bodies = vec![(
            navigation::SUN_CENTER,
            navigation::SUN_CENTER,
            sun_radius * sun::BOUND_RADIUS,
        )];
        for planet in &planets {
            bodies.push((
                planet.position(previous_time),
                planet.position(system_time),
                planet.collision_radius(),
            ));
        }

        let forward = flight.forward();
        let up = flight.orientation() * Vec3::Y;
        let right = forward.cross(up).normalize_or_zero();

        let (desired_position, desired_target) = if let Some((index, _)) = &local_mode {
            (
                flight.position - forward * 4.0 + up * 2.0 + right * 3.0,
                planets[*index].position(system_time),
            )
        } else {
            (
                flight.position - forward * 6.0 + up * 2.5,
                flight.position + forward * 2.0,
            )
        };

        let (desired_position, desired_target) = match ship_view {
            1 => (
                flight.position + right * (ship_radius * 3.0 + 1.0) + up * ship_radius * 0.3,
                flight.position,
            ),
            2 => (
                flight.position - up * (ship_radius * 3.0 + 1.0) + forward * ship_radius * 0.3,
                flight.position,
            ),
            _ => (desired_position, desired_target),
        };
        let blend = presentation.blend();
        let desired_position = overview_position.lerp(desired_position, blend);
        let desired_target = overview_target.lerp(desired_target, blend);
        let previous_camera_position = camera.position;
        let follow = 1.0 - (-8.0 * delta_time).exp();

        camera.position += (desired_position - camera.position) * follow;

        camera.target += (desired_target - camera.target) * follow;

        camera.up = up;

        for &(previous_center, current_center, body_radius) in &bodies {
            if let Some(corrected_position) = collision::resolve_sphere_collision(
                previous_camera_position,
                camera.position,
                previous_center,
                current_center,
                body_radius + 0.2,
            ) {
                camera.position = corrected_position;
            }
        }

        let mut view_direction = camera.target - camera.position;

        if view_direction.length_squared() < 0.000001 {
            camera.target = camera.position + forward;
            view_direction = forward;
        }

        let view_direction = view_direction.normalize();

        if view_direction.cross(camera.up).length_squared() < 0.000001 {
            camera.up = if view_direction.y.abs() < 0.99 {
                Vec3::Y
            } else {
                Vec3::Z
            };
        }

        let selected_index = if let Some((index, _)) = &local_mode {
            *index
        } else {
            nearest_index
        };

        let selected = &planets[selected_index];

        let distance = flight.position.distance(selected.position(system_time));

        let activation_distance = selected.collision_radius() + ship_radius + 0.01 + 4.0;

        let mode = if local_mode.is_some() {
            "LOCAL | C: salir"
        } else if distance <= activation_distance {
            "LIBRE | C: rodear planeta"
        } else {
            "LIBRE | Acercate para usar C"
        };

        window.set_title(&format!(
            "The Showgirl Universe | {} | {mode} | Distancia: {distance:.1} | R: reiniciar",
            selected.name
        ));

        let ship_matrix = Mat4::from_scale_rotation_translation(
            Vec3::splat(ship_scale),
            flight.orientation(),
            flight.position,
        ) * Mat4::from_translation(-center);

        buffer.clear(BACKGROUND);
        let factor = world_quality.factor();
        scene_buffer.resize(buffer.width * factor, buffer.height * factor);
        scene_buffer.clear(BACKGROUND);

        sky.draw(&camera, &mut scene_buffer, elapsed);
        draw_shaded_object(
            &sun_model,
            sun_matrix,
            &camera,
            &mut scene_buffer,
            Material::Sun,
            elapsed,
        );

        if show_orbits {
            for ring in &orbit_rings {
                draw_object(
                    ring,
                    Mat4::IDENTITY,
                    &camera,
                    &mut scene_buffer,
                    0x35415C,
                    true,
                );
            }
        }
        for (index, planet) in planets.iter().enumerate() {
            draw_planet(
                index,
                planet.model_matrix(system_time),
                &camera,
                &mut scene_buffer,
                elapsed,
            );
        }
        for batch in &ship_asset.batches {
            draw_object(
                &batch.model,
                ship_matrix,
                &camera,
                &mut scene_buffer,
                batch.color,
                false,
            );
        }

        fog::draw(
            &camera,
            &mut scene_buffer,
            planets[2].model_matrix(system_time),
            elapsed,
        );

        quality::resolve(
            &scene_buffer,
            &mut buffer,
            ui::Rect {
                x: 0,
                y: 0,
                w: width,
                h: height,
            },
            world_quality,
        );

        if presentation.started {
            let nearest = planets
                .iter()
                .min_by(|a, b| {
                    (flight.position.distance(a.position(system_time)) - a.radius)
                        .total_cmp(&(flight.position.distance(b.position(system_time)) - b.radius))
                })
                .unwrap();
            let nearest_distance = flight.position.distance(nearest.position(system_time));
            let target_top = presentation::hud(
                &mut buffer,
                nearest.name,
                nearest_distance,
                mode,
                presentation.help,
            );
            let target = &planets[target_index];
            presentation::target(
                &mut buffer,
                &camera,
                target.name,
                target.position(system_time),
                flight.position.distance(target.position(system_time)),
                ship_view,
                target_top,
            );
        }

        ui::footer(
            &mut buffer,
            &format!(
                "{} | Q: {}",
                audio.label(),
                if screens.screen == Screen::Gallery {
                    screens.gallery.quality_label()
                } else {
                    world_quality.label().to_string()
                }
            ),
        );

        window
            .update_with_buffer(&buffer.pixels, buffer.width, buffer.height)
            .map_err(|error| format!("Error al mostrar la escena: {error}"))?;
    }

    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod navigation_tests {
    use super::*;
    #[test]
    fn orbitas_centradas_en_sol_y_sin_doble_traslacion() {
        for material in [
            Material::Ophelia,
            Material::Opalite,
            Material::PatientZero,
            Material::PinkClouding,
            Material::Showgirl,
        ] {
            let p = Planet {
                name: "test",
                radius: 2.0,
                orbit_radius: 20.0,
                orbit_speed: 0.55 / 20.0,
                orbit_phase: 1.0,
                rotation_speed: 0.1,
                material,
            };
            for time in [0.0, 12.0, 100.0, 500.0] {
                let position = p.position(time);
                assert!(
                    (position.distance(navigation::SUN_CENTER) - p.orbit_radius).abs() < 0.00001
                );
                assert!(
                    p.model_matrix(time)
                        .transform_point3(Vec3::ZERO)
                        .distance(position)
                        < 0.00001
                );
                assert!(p.orbit_speed * p.orbit_radius < navigation::CRUISE_SPEED * 0.1);
            }
        }
    }
    #[test]
    #[ignore = "Exporta vistas de la nave para revision visual con el renderer real"]
    fn exportar_vistas_nave() {
        use std::io::Write;
        let asset = obj_loader::load_obj_materials("Nave_final.obj").unwrap();
        let mut lo = Vec3::splat(f32::INFINITY);
        let mut hi = Vec3::splat(f32::NEG_INFINITY);
        for v in &asset.model.vertices {
            let p = Vec3::new(v.x, v.y, v.z);
            lo = lo.min(p);
            hi = hi.max(p);
        }
        let center = (lo + hi) * 0.5;
        let matrix = Mat4::from_scale(Vec3::splat(0.45)) * Mat4::from_translation(-center);
        for (name, position, up) in [
            ("inferior", Vec3::new(0.0, -3.0, 0.001), Vec3::Z),
            ("lateral", Vec3::new(3.0, 0.3, 0.0), Vec3::Y),
            ("superior", Vec3::new(0.0, 3.0, 0.001), Vec3::Z),
        ] {
            let mut camera = Camera::new(position, Vec3::ZERO);
            camera.up = up;
            let mut buffer = Framebuffer::new(800, 600);
            buffer.clear(0x101522);
            for b in &asset.batches {
                draw_object(&b.model, matrix, &camera, &mut buffer, b.color, false);
            }
            let path = std::env::temp_dir().join(format!("showgirl_nave_{name}.ppm"));
            let mut file = std::fs::File::create(&path).unwrap();
            write!(file, "P6\n800 600\n255\n").unwrap();
            for c in buffer.pixels {
                file.write_all(&[(c >> 16) as u8, (c >> 8) as u8, c as u8])
                    .unwrap();
            }
            println!("{}", path.display());
        }
    }
}

#[cfg(test)]
mod gallery_visual_tests {
    #[test]
    #[ignore = "Exporta los cinco planetas de galeria con el renderizador real, sin ventana ni audio"]
    fn exportar_galeria() {
        super::run_scene(Some(&std::env::temp_dir())).unwrap();
    }
}

#[cfg(test)]
mod resize_safety_tests {
    #[test]
    fn galeria_tolera_dimensiones_transitorias_pequenas() {
        let sky = crate::sky::Sky::new();
        let gallery = crate::screens::Gallery::new();
        let planet = super::Planet {
            name: "test",
            radius: 2.05,
            orbit_radius: 30.0,
            orbit_speed: 0.02,
            orbit_phase: 0.0,
            rotation_speed: 0.1,
            material: crate::shader::Material::Showgirl,
        };
        let mut viewport = crate::framebuffer::Framebuffer::new(1, 1);
        for (w, h) in [(1, 1), (16, 30), (160, 200), (320, 240)] {
            let mut buffer = crate::framebuffer::Framebuffer::new(w, h);
            super::draw_gallery(
                &gallery,
                &planet,
                4.92,
                &sky,
                &mut buffer,
                &mut viewport,
                0.0,
                true,
                &|_, _, _, _, _| {},
            );
            assert_eq!(buffer.pixels.len(), w * h);
        }
    }
}
