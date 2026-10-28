use glam::Vec3;

use crate::obj_loader::{Model, Vertex};
use crate::sphere::create_sphere;

pub fn create_koi() -> Result<Model, String> {
    let mut model = create_sphere(8, 12)?;

    // Convertimos la esfera en un cuerpo alargado.
    // La cabeza mira hacia +Z y la cola hacia -Z.
    for vertex in &mut model.vertices {
        let original_z = vertex.z;

        // Estrechamos ligeramente la parte trasera.
        let taper = 0.75 + 0.25 * ((original_z + 1.0) * 0.5);

        vertex.x *= 0.22 * taper;
        vertex.y *= 0.25 * taper;
        vertex.z *= 0.65;
    }

    // Cola dividida en dos lóbulos.
    let tail_root = Vec3::new(0.0, 0.0, -0.58);
    let tail_top = Vec3::new(0.0, 0.36, -1.05);
    let tail_notch = Vec3::new(0.0, 0.0, -0.88);
    let tail_bottom = Vec3::new(0.0, -0.36, -1.05);

    add_triangle(&mut model, [tail_root, tail_top, tail_notch]);

    add_triangle(&mut model, [tail_root, tail_notch, tail_bottom]);

    // Aleta superior.
    add_triangle(
        &mut model,
        [
            Vec3::new(0.0, 0.18, 0.28),
            Vec3::new(0.0, 0.43, -0.08),
            Vec3::new(0.0, 0.15, -0.40),
        ],
    );

    // Aleta lateral izquierda.
    add_triangle(
        &mut model,
        [
            Vec3::new(-0.16, -0.04, 0.18),
            Vec3::new(-0.43, -0.12, -0.22),
            Vec3::new(-0.15, -0.08, -0.18),
        ],
    );

    // Aleta lateral derecha.
    add_triangle(
        &mut model,
        [
            Vec3::new(0.16, -0.04, 0.18),
            Vec3::new(0.15, -0.08, -0.18),
            Vec3::new(0.43, -0.12, -0.22),
        ],
    );

    Ok(model)
}

fn add_triangle(model: &mut Model, positions: [Vec3; 3]) {
    let first = model.vertices.len();

    for position in positions {
        model.vertices.push(Vertex {
            x: position.x,
            y: position.y,
            z: position.z,
        });
    }

    model.faces.push([first, first + 1, first + 2]);
}
pub fn animate_koi(model: &Model, time: f32, phase: f32) -> Model {
    let vertices = model
        .vertices
        .iter()
        .map(|vertex| {
            // La cabeza permanece estable.
            // La parte trasera se mueve progresivamente más.
            let influence = ((0.15 - vertex.z) / 1.20).clamp(0.0, 1.0);

            let wave = (time * 6.0 + vertex.z * 3.5 + phase).sin();

            let displacement = wave * 0.18 * influence * influence;

            Vertex {
                x: vertex.x + displacement,
                y: vertex.y,
                z: vertex.z,
            }
        })
        .collect();

    Model {
        vertices,
        faces: model.faces.clone(),
    }
}
