mod camera;
mod framebuffer;
mod obj_loader;
mod pipeline;
mod renderer;
mod sphere;
mod triangle;

use std::time::Instant;

use camera::Camera;
use framebuffer::Framebuffer;
use glam::{Mat4, Vec3};
use minifb::{Key, Window, WindowOptions};
use obj_loader::Model;

const WIDTH: usize = 1000;
const HEIGHT: usize = 750;
const BACKGROUND: u32 = 0x101522;

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

    // Centro del modelo original de la nave.
    let mut minimum = Vec3::splat(f32::INFINITY);
    let mut maximum = Vec3::splat(f32::NEG_INFINITY);

    for vertex in &ship.vertices {
        let position = Vec3::new(vertex.x, vertex.y, vertex.z);
        minimum = minimum.min(position);
        maximum = maximum.max(position);
    }

    let center = (minimum + maximum) * 0.5;

    // Primero centrar, luego escalar y finalmente colocar en el mundo.
    let ship_matrix = Mat4::from_translation(Vec3::new(0.0, 0.4, 4.5))
        * Mat4::from_scale(Vec3::splat(0.45))
        * Mat4::from_translation(-center);

    let sun_matrix = Mat4::from_scale(Vec3::splat(1.5));

    let camera = Camera::new(Vec3::new(9.0, 7.0, 12.0), Vec3::ZERO);

    let mut buffer = Framebuffer::new(WIDTH, HEIGHT);

    let mut window = Window::new(
        "Midnights - Sol y Lavender Haze",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .map_err(|error| format!("No se pudo crear la ventana: {error}"))?;

    window.set_target_fps(60);

    let start = Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let elapsed = start.elapsed().as_secs_f32();

        // Ángulos calculados en radianes según los segundos transcurridos.
        let orbit_angle = elapsed * 0.35;
        let rotation_angle = elapsed * 0.8;

        let orbit_radius = 5.5;

        let planet_position = Vec3::new(
            orbit_angle.cos() * orbit_radius,
            0.0,
            orbit_angle.sin() * orbit_radius,
        );

        let planet_matrix = Mat4::from_translation(planet_position)
            * Mat4::from_rotation_y(rotation_angle)
            * Mat4::from_scale(Vec3::splat(1.1));

        // Limpiar una sola vez antes de dibujar toda la escena.
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

