use glam::Vec4;

use crate::framebuffer::Framebuffer;
use crate::obj_loader::Model;
use crate::pipeline::{clip_triangle, to_screen};
use crate::triangle::triangle;

pub fn render(model: &Model, clip_vertices: &[Vec4], buffer: &mut Framebuffer) {
    for face in &model.faces {
        let index_a = face[0];
        let index_b = face[1];
        let index_c = face[2];

        let a = model.vertices[index_a];
        let b = model.vertices[index_b];
        let c = model.vertices[index_c];

        let ux = b.x - a.x;
        let uy = b.y - a.y;
        let uz = b.z - a.z;

        let vx = c.x - a.x;
        let vy = c.y - a.y;
        let vz = c.z - a.z;

        let nx = uy * vz - uz * vy;
        let ny = uz * vx - ux * vz;
        let nz = ux * vy - uy * vx;

        let normal_length = (nx * nx + ny * ny + nz * nz).sqrt();

        if normal_length < 0.000001 {
            continue;
        }

        let light_length = 0.98_f32.sqrt();

        let diffuse =
            ((nx * -0.3 + ny * 0.8 + nz * 0.5) / (normal_length * light_length)).clamp(0.0, 1.0);

        let intensity = 0.35 + 0.65 * diffuse;

        let red = (182.0 * intensity) as u32;
        let green = (163.0 * intensity) as u32;
        let blue = (224.0 * intensity) as u32;

        let color = (red << 16) | (green << 8) | blue;

        // Primero recortar; después dividir por w y convertir a píxeles.
        let clipped_triangles = clip_triangle([
            clip_vertices[index_a],
            clip_vertices[index_b],
            clip_vertices[index_c],
        ]);

        for clipped in clipped_triangles {
            let screen = clipped.map(|position| to_screen(position, buffer.width, buffer.height));

            let [Some(screen_a), Some(screen_b), Some(screen_c)] = screen else {
                continue;
            };

            triangle(buffer, screen_a, screen_b, screen_c, color);
        }
    }
}
