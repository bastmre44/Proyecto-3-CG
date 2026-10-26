mod framebuffer;
mod obj_loader;
mod renderer;
mod triangle;

use framebuffer::Framebuffer;
use minifb::{Key, Window, WindowOptions};

const WIDTH: usize = 1000;
const HEIGHT: usize = 750;
const BACKGROUND: u32 = 0x101522;

fn run() -> Result<(), String> {
    let model = obj_loader::load_obj("Nave_final.obj")?;

    println!("Modelo cargado correctamente");
    println!("Vértices: {}", model.vertices.len());
    println!("Triángulos: {}", model.faces.len());

    let mut buffer = Framebuffer::new(WIDTH, HEIGHT);

    let screen_vertices = renderer::project_vertices(&model, WIDTH, HEIGHT);

    buffer.clear(BACKGROUND);
    renderer::render(&model, &screen_vertices, &mut buffer);

    let mut window = Window::new(
        "Lab 4 - Nave Midnights",
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
