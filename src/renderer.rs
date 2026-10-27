use glam::{Mat4, Vec3, Vec4};

use crate::framebuffer::Framebuffer;
use crate::obj_loader::Model;
use crate::pipeline::{clip_triangle, to_screen};
use crate::triangle::triangle;

pub fn render(
    model: &Model,
    clip_vertices: &[Vec4],
    buffer: &mut Framebuffer,
    model_matrix: Mat4,
    base_color: u32,
    unlit: bool,
) {
    let normal_matrix = model_matrix.inverse().transpose();
    let light_direction = Vec3::new(-0.3, 0.8, 0.5).normalize();

    for face in &model.faces {
        let index_a = face[0];
        let index_b = face[1];
        let index_c = face[2];

        let position = |index: usize| {
            let vertex = model.vertices[index];
            Vec3::new(vertex.x, vertex.y, vertex.z)
        };

        let a = position(index_a);
        let b = position(index_b);
        let c = position(index_c);

        let original_normal = (b - a).cross(c - a);

        if original_normal.length_squared() < 0.000000000001 {
            continue;
        }

        let normal = normal_matrix
            .transform_vector3(original_normal)
            .normalize_or_zero();

        let intensity = if unlit {
            1.0
        } else {
            let diffuse = normal.dot(light_direction).clamp(0.0, 1.0);
            0.35 + 0.65 * diffuse
        };

        let red = ((base_color >> 16) & 255) as f32;
        let green = ((base_color >> 8) & 255) as f32;
        let blue = (base_color & 255) as f32;

        let color = (((red * intensity) as u32) << 16)
            | (((green * intensity) as u32) << 8)
            | ((blue * intensity) as u32);

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
