use glam::{Mat4, Vec3, Vec4};

use crate::{
    framebuffer::Framebuffer,
    obj_loader::Model,
    pipeline::{ShadedVertex, clip_shaded_triangle},
    shader::Material,
    triangle::shaded_triangle,
};

// Conserva la forma de dibujar la nave y el sol.
pub fn render(
    model: &Model,
    clip_vertices: &[Vec4],
    buffer: &mut Framebuffer,
    model_matrix: Mat4,
    base_color: u32,
    unlit: bool,
) {
    render_shaded(
        model,
        clip_vertices,
        buffer,
        model_matrix,
        Material::Solid {
            color: base_color,
            unlit,
        },
        0.0,
    );
}

pub fn render_shaded(
    model: &Model,
    clip_vertices: &[Vec4],
    buffer: &mut Framebuffer,
    model_matrix: Mat4,
    material: Material,
    time: f32,
) {
    let normal_matrix = model_matrix.inverse().transpose();

    for face in &model.faces {
        let positions = face.map(|index| {
            let vertex = model.vertices[index];

            Vec3::new(vertex.x, vertex.y, vertex.z)
        });

        let face_normal = (positions[1] - positions[0]).cross(positions[2] - positions[0]);

        if face_normal.length_squared() < 0.000000000001 {
            continue;
        }

        let vertices = std::array::from_fn(|corner| {
            let local_position = positions[corner];

            let local_normal = match material {
                // Para una esfera centrada en el origen,
                // la normal apunta desde el centro hacia su superficie.
                Material::LavenderHaze => local_position.normalize_or_zero(),

                Material::Solid { .. } | Material::Koi => face_normal.normalize_or_zero(),
            };

            let normal = normal_matrix
                .transform_vector3(local_normal)
                .normalize_or_zero();

            ShadedVertex {
                clip_position: clip_vertices[face[corner]],
                local_position,
                normal,
            }
        });

        for triangle in clip_shaded_triangle(vertices) {
            shaded_triangle(buffer, triangle, material, time);
        }
    }
}
