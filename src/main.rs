mod camera;
mod flight;
mod framebuffer;
mod obj_loader;
mod pipeline;
mod renderer;
mod sphere;
mod triangle;

use std::time::Instant;

use camera::Camera;
use flight::{FlightController, FlightInput};
use framebuffer::Framebuffer;
use glam::{Mat4, Vec3};
use minifb::{Key, KeyRepeat, Window, WindowOptions};
use obj_loader::Model;

const WIDTH: usize = 1000;
const HEIGHT: usize = 750;
const BACKGROUND: u32 = 0x101522;

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

fn run() -> Result<(), String> {
    let ship = obj_loader::load_obj("Nave_final.obj")?;
    let sphere = sphere::create_sphere(12, 24)?;

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
    let ship_start = Vec3::new(0.0, 0.4, 7.5);

    let mut flight = FlightController::new(ship_start);

    let mut camera = Camera::new(
        flight.position - flight.forward() * 6.0 + Vec3::Y * 2.5,
        flight.position + flight.forward() * 2.0,
    );

    let sun_matrix = Mat4::from_scale(Vec3::splat(1.5));
    let mut buffer = Framebuffer::new(WIDTH, HEIGHT);

    let mut window = Window::new(
        "Midnights - Vuelo espacial",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .map_err(|error| format!("No se pudo crear la ventana: {error}"))?;

    window.set_target_fps(60);

    let start = Instant::now();
    let mut previous = start;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = Instant::now();
        let delta_time = now.duration_since(previous).as_secs_f32();
        let elapsed = now.duration_since(start).as_secs_f32();
        previous = now;

        // R permite regresar si nos alejamos demasiado.
        if window.is_key_pressed(Key::R, KeyRepeat::No) {
            flight = FlightController::new(ship_start);

            camera.position = flight.position - flight.forward() * 6.0 + Vec3::Y * 2.5;

            camera.target = flight.position + flight.forward() * 2.0;
        }

        let input = if window.is_active() {
            FlightInput {
                throttle: input_axis(&window, Key::W, Key::S),
                yaw: input_axis(&window, Key::D, Key::A),
                pitch: input_axis(&window, Key::Up, Key::Down),
            }
        } else {
            FlightInput::default()
        };

        flight.update(&input, delta_time);

        // Cámara detrás y ligeramente por encima de la nave.
        let desired_position = flight.position - flight.forward() * 6.0 + Vec3::Y * 2.5;

        let desired_target = flight.position + flight.forward() * 2.0;

        // Seguimiento suave calculado según el tiempo.
        let follow = 1.0 - (-8.0 * delta_time).exp();

        camera.position += (desired_position - camera.position) * follow;
        camera.target += (desired_target - camera.target) * follow;

        let ship_matrix = Mat4::from_scale_rotation_translation(
            Vec3::splat(0.45),
            flight.orientation(),
            flight.position,
        ) * Mat4::from_translation(-center);

        let orbit_angle = elapsed * 0.35;
        let rotation_angle = elapsed * 0.8;

        let planet_position = Vec3::new(orbit_angle.cos() * 5.5, 0.0, orbit_angle.sin() * 5.5);

        let planet_matrix = Mat4::from_translation(planet_position)
            * Mat4::from_rotation_y(rotation_angle)
            * Mat4::from_scale(Vec3::splat(1.1));

        buffer.clear(BACKGROUND);

        draw_object(&sphere, sun_matrix, &camera, &mut buffer, 0xFFD17A, true);

        draw_object(
            &sphere,
            planet_matrix,
            &camera,
            &mut buffer,
            0xB6A3E0,
            false,
        );

        draw_object(&ship, ship_matrix, &camera, &mut buffer, 0x385794, false);

        window
            .update_with_buffer(&buffer.pixels, WIDTH, HEIGHT)
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
