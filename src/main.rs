mod camera;
mod framebuffer;
mod obj_loader;
mod pipeline;
mod renderer;
mod triangle;

use camera::Camera;
use framebuffer::Framebuffer;
use glam::{Mat4, Vec3};
use minifb::{Key, Window, WindowOptions};

const WIDTH: usize = 1000;
const HEIGHT: usize = 750;
const BACKGROUND: u32 = 0x101522;

fn run() -> Result<(), String> {
    let model = obj_loader::load_obj("Nave_final.obj")?;

    if model.vertices.is_empty() || model.faces.is_empty() {
        return Err("El modelo no contiene geometría para dibujar".into());
    }

    println!("Vértices: {}", model.vertices.len());
    println!("Triángulos: {}", model.faces.len());

    // Encontrar el centro de la geometría de la nave.
    let mut minimum = Vec3::splat(f32::INFINITY);
    let mut maximum = Vec3::splat(f32::NEG_INFINITY);

    for vertex in &model.vertices {
        let position = Vec3::new(vertex.x, vertex.y, vertex.z);

        minimum = minimum.min(position);
        maximum = maximum.max(position);
    }

    let center = (minimum + maximum) * 0.5;

    // Modelo: colocar el centro de la nave en el origen del mundo.
    let model_matrix = Mat4::from_translation(-center);

    // Vista: observar el origen desde arriba y desde un costado.
    let camera = Camera::new(Vec3::new(5.0, 3.0, 7.0), Vec3::ZERO);

    let clip_vertices = pipeline::transform_vertices(
        &model,
        model_matrix,
        camera.view_matrix(),
        camera.projection_matrix(WIDTH, HEIGHT),
    );

    let mut buffer = Framebuffer::new(WIDTH, HEIGHT);
    buffer.clear(BACKGROUND);

    renderer::render(&model, &clip_vertices, &mut buffer);

    let mut window = Window::new(
        "Proyecto final - Midnights",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .map_err(|error| format!("No se pudo crear la ventana: {error}"))?;

    window.set_target_fps(60);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window
            .update_with_buffer(&buffer.pixels, WIDTH, HEIGHT)
            .map_err(|error| format!("Error al mostrar la imagen: {error}"))?;
    }

    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}
