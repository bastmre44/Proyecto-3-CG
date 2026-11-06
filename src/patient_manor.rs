use glam::{Mat4, Quat, Vec3};

use crate::obj_loader::{Model, Vertex};

pub struct ManorPart {
    pub model: Model,
    pub color: u32,
    pub unlit: bool,
}

pub fn create_manor() -> Vec<ManorPart> {
    let settings = [
        (0x555B75, false), // Paredes.
        (0x292D46, false), // Techos.
        (0xBAC3D8, false), // Piedra y bordes.
        (0xFFCB78, true),  // Ventanas luminosas.
        (0x191D2E, false), // Puerta.
    ];

    let mut parts: Vec<ManorPart> = settings
        .into_iter()
        .map(|(color, unlit)| ManorPart {
            model: Model {
                vertices: Vec::new(),
                faces: Vec::new(),
            },
            color,
            unlit,
        })
        .collect();

    let direction = Vec3::new(0.0, 0.98, 0.20).normalize();

    let root = Mat4::from_translation(direction * 1.04)
        * Mat4::from_quat(Quat::from_rotation_arc(Vec3::Y, direction))
        * Mat4::from_scale(Vec3::new(1.30, 1.20, 1.15));

    // Plataforma de piedra sobre el terreno.
    add_box(
        &mut parts[2].model,
        root * translation(0.0, 0.035, 0.0),
        Vec3::new(0.98, 0.09, 0.70),
    );

    // Edificio central.
    add_box(
        &mut parts[0].model,
        root * translation(0.0, 0.28, 0.0),
        Vec3::new(0.48, 0.40, 0.36),
    );

    add_gable(
        &mut parts[1].model,
        root * translation(0.0, 0.48, 0.0),
        Vec3::new(0.55, 0.22, 0.44),
    );

    // Alas laterales.
    for x in [-0.32, 0.32] {
        add_box(
            &mut parts[0].model,
            root * translation(x, 0.205, 0.0),
            Vec3::new(0.23, 0.25, 0.30),
        );

        add_gable(
            &mut parts[1].model,
            root * translation(x, 0.33, 0.0),
            Vec3::new(0.29, 0.16, 0.37),
        );

        // Ventana del ala.
        add_window(
            &mut parts[3].model,
            root * translation(x, 0.15, 0.158),
            0.065,
            0.12,
        );
    }

    // Torres posteriores con techos puntiagudos.
    for x in [-0.34, 0.34] {
        add_box(
            &mut parts[0].model,
            root * translation(x, 0.33, -0.12),
            Vec3::new(0.16, 0.50, 0.17),
        );

        // Cornisa superior.
        add_box(
            &mut parts[2].model,
            root * translation(x, 0.58, -0.12),
            Vec3::new(0.19, 0.035, 0.20),
        );

        add_pyramid(
            &mut parts[1].model,
            root * translation(x, 0.595, -0.12),
            Vec3::new(0.19, 0.32, 0.20),
        );

        add_window(
            &mut parts[3].model,
            root * translation(x, 0.47, -0.027),
            0.055,
            0.09,
        );
    }

    // Ventanas del edificio central.
    for x in [-0.15, 0.15] {
        for y in [0.13, 0.31] {
            add_window(
                &mut parts[3].model,
                root * translation(x, y, 0.188),
                0.065,
                0.12,
            );
        }
    }

    // Puerta y pequeño cristal iluminado.
    add_box(
        &mut parts[4].model,
        root * translation(0.0, 0.18, 0.19),
        Vec3::new(0.115, 0.20, 0.025),
    );

    add_window(
        &mut parts[3].model,
        root * translation(0.0, 0.20, 0.207),
        0.045,
        0.055,
    );

    // Escalinata de entrada.
    for step in 0..3 {
        add_box(
            &mut parts[2].model,
            root * translation(0.0, 0.065 - step as f32 * 0.025, 0.38 + step as f32 * 0.065),
            Vec3::new(0.26, 0.035, 0.085),
        );
    }

    // Chimenea.
    add_box(
        &mut parts[0].model,
        root * translation(0.15, 0.63, -0.10),
        Vec3::new(0.065, 0.19, 0.065),
    );

    add_box(
        &mut parts[2].model,
        root * translation(0.15, 0.73, -0.10),
        Vec3::new(0.085, 0.025, 0.085),
    );

    // Entrada central que sobresale de la fachada.
    add_box(
        &mut parts[0].model,
        root * translation(0.0, 0.27, 0.255),
        Vec3::new(0.22, 0.38, 0.18),
    );

    // Techo alto y estrecho sobre la entrada.
    add_gable(
        &mut parts[1].model,
        root * translation(0.0, 0.46, 0.255),
        Vec3::new(0.27, 0.30, 0.23),
    );

    // Puerta iluminada, alta y con punta.
    add_window(
        &mut parts[3].model,
        root * translation(0.0, 0.085, 0.355),
        0.105,
        0.27,
    );

    // Ventana sobre la puerta.
    add_window(
        &mut parts[3].model,
        root * translation(0.0, 0.49, 0.378),
        0.06,
        0.12,
    );

    // Bordes de piedra del techo de la entrada.
    for side in [-1.0, 1.0] {
        add_beam(
            &mut parts[2].model,
            root,
            Vec3::new(side * 0.135, 0.46, 0.38),
            Vec3::new(0.0, 0.76, 0.38),
            0.018,
        );

        // Columnas a ambos lados de la entrada.
        add_box(
            &mut parts[2].model,
            root * translation(side * 0.12, 0.27, 0.35),
            Vec3::new(0.022, 0.38, 0.025),
        );
    }

    // Bordes del techo principal.
    for side in [-1.0, 1.0] {
        add_beam(
            &mut parts[2].model,
            root,
            Vec3::new(side * 0.275, 0.48, 0.228),
            Vec3::new(0.0, 0.70, 0.228),
            0.015,
        );
    }

    // Divisiones oscuras en las ventanas de la fachada.
    for x in [-0.15, 0.15] {
        for y in [0.13, 0.31] {
            add_box(
                &mut parts[4].model,
                root * translation(x, y + 0.055, 0.202),
                Vec3::new(0.008, 0.10, 0.006),
            );

            add_box(
                &mut parts[4].model,
                root * translation(x, y + 0.045, 0.202),
                Vec3::new(0.060, 0.009, 0.006),
            );
        }
    }

    // División de la ventana superior de la entrada.
    add_box(
        &mut parts[4].model,
        root * translation(0.0, 0.54, 0.392),
        Vec3::new(0.007, 0.09, 0.006),
    );

    parts
}

fn translation(x: f32, y: f32, z: f32) -> Mat4 {
    Mat4::from_translation(Vec3::new(x, y, z))
}

fn add_window(model: &mut Model, transform: Mat4, width: f32, height: f32) {
    // Parte rectangular.
    add_box(
        model,
        transform * translation(0.0, height * 0.35, 0.0),
        Vec3::new(width, height * 0.70, 0.012),
    );

    // Punta de la ventana gótica.
    add_geometry(
        model,
        transform,
        &[
            Vec3::new(-width * 0.5, height * 0.70, 0.007),
            Vec3::new(width * 0.5, height * 0.70, 0.007),
            Vec3::new(0.0, height, 0.007),
        ],
        &[[0, 1, 2]],
    );
}

fn add_box(model: &mut Model, transform: Mat4, size: Vec3) {
    let positions = [
        Vec3::new(-0.5, -0.5, -0.5),
        Vec3::new(0.5, -0.5, -0.5),
        Vec3::new(0.5, 0.5, -0.5),
        Vec3::new(-0.5, 0.5, -0.5),
        Vec3::new(-0.5, -0.5, 0.5),
        Vec3::new(0.5, -0.5, 0.5),
        Vec3::new(0.5, 0.5, 0.5),
        Vec3::new(-0.5, 0.5, 0.5),
    ]
    .map(|position| position * size);

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

    add_geometry(model, transform, &positions, &faces);
}

fn add_gable(model: &mut Model, transform: Mat4, size: Vec3) {
    let positions = [
        Vec3::new(-0.5, 0.0, -0.5),
        Vec3::new(0.5, 0.0, -0.5),
        Vec3::new(0.5, 0.0, 0.5),
        Vec3::new(-0.5, 0.0, 0.5),
        Vec3::new(0.0, 1.0, -0.5),
        Vec3::new(0.0, 1.0, 0.5),
    ]
    .map(|position| position * size);

    let faces = [
        [0, 4, 1],
        [3, 2, 5],
        [0, 3, 5],
        [0, 5, 4],
        [1, 4, 5],
        [1, 5, 2],
        [0, 1, 2],
        [0, 2, 3],
    ];

    add_geometry(model, transform, &positions, &faces);
}

fn add_pyramid(model: &mut Model, transform: Mat4, size: Vec3) {
    let positions = [
        Vec3::new(-0.5, 0.0, -0.5),
        Vec3::new(0.5, 0.0, -0.5),
        Vec3::new(0.5, 0.0, 0.5),
        Vec3::new(-0.5, 0.0, 0.5),
        Vec3::new(0.0, 1.0, 0.0),
    ]
    .map(|position| position * size);

    let faces = [
        [0, 4, 1],
        [1, 4, 2],
        [2, 4, 3],
        [3, 4, 0],
        [0, 1, 2],
        [0, 2, 3],
    ];

    add_geometry(model, transform, &positions, &faces);
}

fn add_geometry(model: &mut Model, transform: Mat4, positions: &[Vec3], faces: &[[usize; 3]]) {
    let first = model.vertices.len();

    for position in positions {
        let position = transform.transform_point3(*position);

        model.vertices.push(Vertex {
            x: position.x,
            y: position.y,
            z: position.z,
        });
    }

    for face in faces {
        model.faces.push(face.map(|index| first + index));
    }
}

fn add_beam(model: &mut Model, transform: Mat4, start: Vec3, end: Vec3, thickness: f32) {
    let direction = end - start;
    let length = direction.length();

    let beam_transform = transform
        * Mat4::from_translation((start + end) * 0.5)
        * Mat4::from_quat(Quat::from_rotation_arc(Vec3::Y, direction / length));

    add_box(
        model,
        beam_transform,
        Vec3::new(thickness, length, thickness),
    );
}
