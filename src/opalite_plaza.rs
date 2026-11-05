use glam::{Mat4, Quat, Vec3};

use crate::obj_loader::{Model, Vertex};
use crate::sphere::create_sphere;

pub struct PlazaPart {
    pub model: Model,
    pub color: u32,
}

pub fn create_plaza() -> Result<Vec<PlazaPart>, String> {
    let colors = [
        0xB3A4D8, // Base lavanda.
        0xFFF3DE, // Cuadros crema.
        0x9472DA, // Cuadros violetas.
        0xFF85B4, // Arcos y escalones rosas.
        0x75D8EC, // Interior turquesa.
        0xFFE4F2, // Piedra perlada.
    ];

    let mut parts: Vec<PlazaPart> = colors
        .into_iter()
        .map(|color| PlazaPart {
            model: Model {
                vertices: Vec::new(),
                faces: Vec::new(),
            },
            color,
        })
        .collect();

    // Colocación sobre la superficie curva de Opalite.
    let direction = Vec3::new(0.05, 1.0, 0.30).normalize();

    let root = Mat4::from_translation(direction * 1.04)
        * Mat4::from_quat(Quat::from_rotation_arc(Vec3::Y, direction));

    // Plataforma bajo el piso.
    add_box(
        &mut parts[0].model,
        root * Mat4::from_translation(Vec3::new(0.0, -0.075, 0.0)),
        Vec3::new(0.94, 0.12, 0.94),
    );

    // Piso de cuadros, seis por seis.
    let tile_size = 0.14;

    for row in 0..6 {
        for column in 0..6 {
            let x = -0.42 + (column as f32 + 0.5) * tile_size;
            let z = -0.42 + (row as f32 + 0.5) * tile_size;

            let part_index = if (row + column) % 2 == 0 { 1 } else { 2 };

            add_box(
                &mut parts[part_index].model,
                root * Mat4::from_translation(Vec3::new(x, 0.0, z)),
                Vec3::new(tile_size, 0.04, tile_size),
            );
        }
    }

    // Tres portales: el central es más alto.
    add_arch(&mut parts, root, -0.30, 0.09, 0.45);
    add_arch(&mut parts, root, 0.0, 0.13, 0.65);
    add_arch(&mut parts, root, 0.30, 0.09, 0.45);

    // Escalones al frente de la plaza.
    for step in 0..3 {
        let y = -0.025 - step as f32 * 0.035;
        let z = 0.46 + step as f32 * 0.065;

        add_box(
            &mut parts[3].model,
            root * Mat4::from_translation(Vec3::new(0.0, y, z)),
            Vec3::new(0.32, 0.04, 0.085),
        );
    }

    // Pedestal de la piedra.
    add_box(
        &mut parts[3].model,
        root * Mat4::from_translation(Vec3::new(0.0, 0.045, 0.08)),
        Vec3::new(0.34, 0.05, 0.30),
    );

    add_box(
        &mut parts[1].model,
        root * Mat4::from_translation(Vec3::new(0.0, 0.085, 0.08)),
        Vec3::new(0.26, 0.04, 0.23),
    );

    // Piedra redondeada con caras visibles.
    let stone = create_sphere(6, 10)?;

    let stone_transform = root
        * Mat4::from_translation(Vec3::new(0.0, 0.27, 0.08))
        * Mat4::from_rotation_z(-0.12)
        * Mat4::from_scale(Vec3::new(0.14, 0.17, 0.13));

    append_model(&mut parts[5].model, &stone, stone_transform);

    Ok(parts)
}

fn add_arch(parts: &mut [PlazaPart], root: Mat4, x: f32, radius: f32, height: f32) {
    let base = root * Mat4::from_translation(Vec3::new(x, 0.02, -0.33));

    let straight_height = height - radius;
    let thickness = 0.035;

    // Laterales rosados.
    for side in [-1.0, 1.0] {
        add_box(
            &mut parts[3].model,
            base * Mat4::from_translation(Vec3::new(side * radius, straight_height * 0.5, 0.0)),
            Vec3::new(thickness, straight_height, 0.075),
        );
    }

    // Curva superior del marco.
    let segments = 12;
    let step = std::f32::consts::PI / segments as f32;

    for index in 0..segments {
        let angle = (index as f32 + 0.5) * step;

        let position = Vec3::new(
            angle.cos() * radius,
            straight_height + angle.sin() * radius,
            0.0,
        );

        add_box(
            &mut parts[3].model,
            base * Mat4::from_translation(position)
                * Mat4::from_rotation_z(angle + std::f32::consts::FRAC_PI_2),
            Vec3::new(radius * step * 1.08, thickness, 0.075),
        );
    }

    // Panel azul dentro del portal.
    add_box(
        &mut parts[4].model,
        base * Mat4::from_translation(Vec3::new(0.0, straight_height * 0.5, -0.025)),
        Vec3::new(radius * 2.0, straight_height, 0.02),
    );

    // Parte semicircular del panel.
    let center = Vec3::new(0.0, straight_height, -0.015);

    for index in 0..segments {
        let a = index as f32 * step;
        let b = (index + 1) as f32 * step;

        add_triangle(
            &mut parts[4].model,
            base,
            [
                center,
                center + Vec3::new(a.cos() * radius, a.sin() * radius, 0.0),
                center + Vec3::new(b.cos() * radius, b.sin() * radius, 0.0),
            ],
        );
    }
}

fn append_model(target: &mut Model, source: &Model, transform: Mat4) {
    let first = target.vertices.len();

    for vertex in &source.vertices {
        let position = transform.transform_point3(Vec3::new(vertex.x, vertex.y, vertex.z));

        target.vertices.push(Vertex {
            x: position.x,
            y: position.y,
            z: position.z,
        });
    }

    for face in &source.faces {
        target.faces.push(face.map(|index| first + index));
    }
}

fn add_triangle(model: &mut Model, transform: Mat4, positions: [Vec3; 3]) {
    let first = model.vertices.len();

    for position in positions {
        let position = transform.transform_point3(position);

        model.vertices.push(Vertex {
            x: position.x,
            y: position.y,
            z: position.z,
        });
    }

    model.faces.push([first, first + 1, first + 2]);
}

fn add_box(model: &mut Model, transform: Mat4, size: Vec3) {
    let corners = [
        Vec3::new(-0.5, -0.5, -0.5),
        Vec3::new(0.5, -0.5, -0.5),
        Vec3::new(0.5, 0.5, -0.5),
        Vec3::new(-0.5, 0.5, -0.5),
        Vec3::new(-0.5, -0.5, 0.5),
        Vec3::new(0.5, -0.5, 0.5),
        Vec3::new(0.5, 0.5, 0.5),
        Vec3::new(-0.5, 0.5, 0.5),
    ];

    let faces = [
        [0, 2, 1],
        [0, 3, 2],
        [4, 5, 6],
        [4, 6, 7],
        [0, 4, 7],
        [0, 7, 3],
        [1, 2, 6],
        [1, 6, 5],
        [0, 1, 5],
        [0, 5, 4],
        [3, 7, 6],
        [3, 6, 2],
    ];

    for face in faces {
        add_triangle(model, transform, face.map(|index| corners[index] * size));
    }
}
