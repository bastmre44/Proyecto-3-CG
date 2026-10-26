use crate::framebuffer::Framebuffer;
use crate::obj_loader::{Model, Vertex};
use crate::triangle::triangle;

pub fn project_vertices(model: &Model, width: usize, height: usize) -> Vec<Vertex> {
    let yaw = 0.65_f32;
    let pitch = 0.45_f32;

    let rotated: Vec<Vertex> = model
        .vertices
        .iter()
        .map(|vertex| {
            // Giro horizontal para observar un costado.
            let x = vertex.x * yaw.cos() + vertex.z * yaw.sin();
            let z = -vertex.x * yaw.sin() + vertex.z * yaw.cos();

            // Inclinación para observar la parte superior.
            let y = vertex.y * pitch.cos() - z * pitch.sin();
            let depth = vertex.y * pitch.sin() + z * pitch.cos();

            Vertex { x, y, z: depth }
        })
        .collect();

    let Some(first) = rotated.first() else {
        return Vec::new();
    };

    let mut min_x = first.x;
    let mut max_x = first.x;
    let mut min_y = first.y;
    let mut max_y = first.y;

    for vertex in &rotated {
        min_x = min_x.min(vertex.x);
        max_x = max_x.max(vertex.x);
        min_y = min_y.min(vertex.y);
        max_y = max_y.max(vertex.y);
    }

    let center_x = (min_x + max_x) * 0.5;
    let center_y = (min_y + max_y) * 0.5;

    let model_width = (max_x - min_x).max(0.001);
    let model_height = (max_y - min_y).max(0.001);

    // Mantiene las proporciones y deja margen alrededor.
    let scale = (width as f32 * 0.8 / model_width).min(height as f32 * 0.8 / model_height);

    rotated
        .iter()
        .map(|vertex| Vertex {
            x: width as f32 * 0.5 + (vertex.x - center_x) * scale,
            y: height as f32 * 0.5 - (vertex.y - center_y) * scale,
            z: -vertex.z,
        })
        .collect()
}

pub fn render(model: &Model, screen_vertices: &[Vertex], buffer: &mut Framebuffer) {
    for face in &model.faces {
        // Obtener los tres índices de la cara.
        let index_a = face[0];
        let index_b = face[1];
        let index_c = face[2];

        // Buscar los vértices originales.
        let a = model.vertices[index_a];
        let b = model.vertices[index_b];
        let c = model.vertices[index_c];

        // Dos vectores sobre la superficie del triángulo.
        let ux = b.x - a.x;
        let uy = b.y - a.y;
        let uz = b.z - a.z;

        let vx = c.x - a.x;
        let vy = c.y - a.y;
        let vz = c.z - a.z;

        // Producto cruz: obtiene la normal de la superficie.
        let nx = uy * vz - uz * vy;
        let ny = uz * vx - ux * vz;
        let nz = ux * vy - uy * vx;

        let normal_length = (nx * nx + ny * ny + nz * nz).sqrt();

        if normal_length < 0.000001 {
            continue;
        }

        // Luz direccional normalizada.
        let light_length = 0.98_f32.sqrt();

        let diffuse =
            ((nx * -0.3 + ny * 0.8 + nz * 0.5) / (normal_length * light_length)).clamp(0.0, 1.0);

        let intensity = 0.35 + 0.65 * diffuse;

        // Color lavanda ajustado según la iluminación.
        let red = (182.0 * intensity) as u32;
        let green = (163.0 * intensity) as u32;
        let blue = (224.0 * intensity) as u32;

        let color = (red << 16) | (green << 8) | blue;

        // Buscar los mismos vértices en coordenadas de pantalla.
        let screen_a = screen_vertices[index_a];
        let screen_b = screen_vertices[index_b];
        let screen_c = screen_vertices[index_c];

        // Dibujar con nuestra función de triángulos.
        triangle(buffer, screen_a, screen_b, screen_c, color);
    }
}
