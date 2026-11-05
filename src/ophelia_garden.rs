use glam::{Mat4, Quat, Vec3};

use crate::obj_loader::{Model, Vertex};

// Incluye el agua, las islas y la parte superior del marco.
pub const BOUND_RADIUS: f32 = 1.85;

pub struct GardenPart {
    pub model: Model,
    pub color: u32,
}

pub fn create_garden() -> Vec<GardenPart> {
    let mut rock = empty_model();
    let mut marble = empty_model();
    let mut gold = empty_model();
    let mut leaves = empty_model();
    let mut pink_petals = empty_model();
    let mut pale_petals = empty_model();

    // Direcciones desde el centro del planeta.
    let island_directions = [
        Vec3::new(-0.48, 0.86, 0.18).normalize(),
        Vec3::new(0.30, 0.95, -0.10).normalize(),
        Vec3::new(0.08, 0.86, 0.50).normalize(),
    ];

    for (index, direction) in island_directions.iter().enumerate() {
        let transform = surface_transform(*direction);

        let radius = if index == 1 { 0.35 } else { 0.29 };

        // Base de roca que entra ligeramente en el agua.
        add_prism(&mut rock, transform, radius, -0.23, 0.01, 9);

        // Plataforma superior de mármol.
        add_prism(&mut marble, transform, radius * 0.92, 0.01, 0.08, 9);
    }

    // El marco se apoya sobre la segunda isla.
    let frame_transform = surface_transform(island_directions[1])
        * Mat4::from_translation(Vec3::Y * 0.08)
        * Mat4::from_rotation_z(-0.14);

    // Laterales del marco.
    for x in [-0.25, 0.25] {
        add_box(
            &mut gold,
            frame_transform * Mat4::from_translation(Vec3::new(x, 0.34, 0.0)),
            Vec3::new(0.065, 0.68, 0.085),
        );
    }

    // Parte inferior y superior.
    for y in [0.02, 0.66] {
        add_box(
            &mut gold,
            frame_transform * Mat4::from_translation(Vec3::new(0.0, y, 0.0)),
            Vec3::new(0.56, 0.065, 0.085),
        );
    }

    // Segundo borde para dar profundidad al marco.
    for x in [-0.205, 0.205] {
        add_box(
            &mut gold,
            frame_transform * Mat4::from_translation(Vec3::new(x, 0.34, 0.035)),
            Vec3::new(0.018, 0.57, 0.035),
        );
    }

    for y in [0.065, 0.615] {
        add_box(
            &mut gold,
            frame_transform * Mat4::from_translation(Vec3::new(0.0, y, 0.035)),
            Vec3::new(0.42, 0.018, 0.035),
        );
    }

    // Adornos geométricos en las cuatro esquinas.
    for x in [-0.25, 0.25] {
        for y in [0.02, 0.66] {
            add_box(
                &mut gold,
                frame_transform
                    * Mat4::from_translation(Vec3::new(x, y, 0.025))
                    * Mat4::from_rotation_z(std::f32::consts::FRAC_PI_4),
                Vec3::new(0.11, 0.11, 0.10),
            );
        }
    }

    // Adorno central superior.
    add_box(
        &mut gold,
        frame_transform
            * Mat4::from_translation(Vec3::new(0.0, 0.73, 0.0))
            * Mat4::from_rotation_z(std::f32::consts::FRAC_PI_4),
        Vec3::new(0.12, 0.12, 0.08),
    );

    // Arco de mármol sobre la primera isla.
    let arch_transform =
        surface_transform(island_directions[0]) * Mat4::from_translation(Vec3::Y * 0.08);

    // Dos columnas.
    for x in [-0.21, 0.21] {
        add_box(
            &mut marble,
            arch_transform * Mat4::from_translation(Vec3::new(x, 0.16, 0.0)),
            Vec3::new(0.075, 0.32, 0.12),
        );

        // Bases y capiteles.
        for y in [0.025, 0.31] {
            add_box(
                &mut marble,
                arch_transform * Mat4::from_translation(Vec3::new(x, y, 0.0)),
                Vec3::new(0.11, 0.05, 0.15),
            );
        }
    }

    // Piezas que forman la curva superior del arco.
    let segments = 9;
    let arch_radius = 0.21;
    let step = std::f32::consts::PI / segments as f32;

    for index in 0..segments {
        let angle = (index as f32 + 0.5) * step;

        let position = Vec3::new(
            angle.cos() * arch_radius,
            0.32 + angle.sin() * arch_radius,
            0.0,
        );

        let transform = arch_transform
            * Mat4::from_translation(position)
            * Mat4::from_rotation_z(angle + std::f32::consts::FRAC_PI_2);

        add_box(
            &mut marble,
            transform,
            Vec3::new(arch_radius * step * 1.05, 0.07, 0.12),
        );
    }

    // Nenúfares distribuidos en distintas zonas del agua.
    let flower_directions = [
        Vec3::new(-0.50, 0.20, 0.84).normalize(),
        Vec3::new(0.55, 0.12, 0.82).normalize(),
        Vec3::new(-0.30, 0.70, 0.65).normalize(),
    ];

    for direction in flower_directions {
        let leaf_transform = surface_transform(direction) * Mat4::from_translation(Vec3::Y * 0.035);

        // Hoja verde sobre el agua.
        add_prism(&mut leaves, leaf_transform, 0.22, 0.0, 0.015, 12);

        let flower_transform = leaf_transform * Mat4::from_translation(Vec3::Y * 0.025);

        // Ocho pétalos exteriores.
        for index in 0..8 {
            let angle = index as f32 / 8.0 * std::f32::consts::TAU;

            add_petal(
                &mut pink_petals,
                flower_transform * Mat4::from_rotation_y(angle),
                0.17,
                0.065,
                0.075,
            );
        }

        // Seis pétalos interiores, más levantados.
        for index in 0..6 {
            let angle = index as f32 / 6.0 * std::f32::consts::TAU + 0.3;

            add_petal(
                &mut pale_petals,
                flower_transform * Mat4::from_rotation_y(angle),
                0.11,
                0.045,
                0.12,
            );
        }

        // Centro dorado de la flor.
        add_prism(&mut gold, flower_transform, 0.035, 0.0, 0.07, 8);
    }

    // Vegetación y flores pequeñas alrededor de las islas.
    for direction in island_directions {
        let island_transform = surface_transform(direction);

        for index in 0..6 {
            let angle = index as f32 / 6.0 * std::f32::consts::TAU;

            // Borde de la plataforma.
            let border_position = Vec3::new(angle.cos() * 0.25, 0.09, angle.sin() * 0.25);

            let plant_transform = island_transform * Mat4::from_translation(border_position);

            // Tallo.
            add_box(
                &mut leaves,
                plant_transform * Mat4::from_translation(Vec3::Y * 0.035),
                Vec3::new(0.012, 0.07, 0.012),
            );

            // Dos hojas junto al tallo.
            for leaf_angle in [angle, angle + std::f32::consts::PI] {
                add_petal(
                    &mut leaves,
                    plant_transform
                        * Mat4::from_translation(Vec3::Y * 0.025)
                        * Mat4::from_rotation_y(leaf_angle),
                    0.075,
                    0.025,
                    0.025,
                );
            }

            let blossom_transform = plant_transform * Mat4::from_translation(Vec3::Y * 0.07);

            // Flor pequeña de seis pétalos claros.
            for petal in 0..6 {
                let petal_angle = petal as f32 / 6.0 * std::f32::consts::TAU;

                add_petal(
                    &mut pale_petals,
                    blossom_transform * Mat4::from_rotation_y(petal_angle),
                    0.045,
                    0.018,
                    0.02,
                );
            }

            add_prism(&mut gold, blossom_transform, 0.013, 0.0, 0.025, 6);
        }
    }

    // Hojas que bajan por la superficie desde las islas.
    for direction in island_directions {
        let destination = (direction + Vec3::new(0.0, -0.7, 0.45)).normalize();

        for index in 0..9 {
            let progress = index as f32 / 8.0;

            let leaf_direction = direction.lerp(destination, progress * 0.65).normalize();

            let leaf_transform =
                surface_transform(leaf_direction) * Mat4::from_translation(Vec3::Y * 0.04);

            let turn = if index % 2 == 0 { 0.6 } else { -0.6 };

            add_petal(
                &mut leaves,
                leaf_transform * Mat4::from_rotation_y(turn),
                0.095,
                0.032,
                0.018,
            );
        }
    }

    vec![
        GardenPart {
            model: rock,
            color: 0xB8AB91,
        },
        GardenPart {
            model: marble,
            color: 0xF4E7CD,
        },
        GardenPart {
            model: gold,
            color: 0xF2BC52,
        },
        GardenPart {
            model: leaves,
            color: 0x63976B,
        },
        GardenPart {
            model: pink_petals,
            color: 0xEDA6B8,
        },
        GardenPart {
            model: pale_petals,
            color: 0xFFE7DB,
        },
    ]
}

fn empty_model() -> Model {
    Model {
        vertices: Vec::new(),
        faces: Vec::new(),
    }
}

fn surface_transform(direction: Vec3) -> Mat4 {
    // El eje Y de cada isla apunta hacia afuera del planeta.
    let rotation = Quat::from_rotation_arc(Vec3::Y, direction);

    Mat4::from_translation(direction * 0.98) * Mat4::from_quat(rotation)
}

fn add_vertex(model: &mut Model, transform: Mat4, position: Vec3) {
    let position = transform.transform_point3(position);

    model.vertices.push(Vertex {
        x: position.x,
        y: position.y,
        z: position.z,
    });
}

fn add_box(model: &mut Model, transform: Mat4, size: Vec3) {
    let first = model.vertices.len();

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

    for corner in corners {
        add_vertex(model, transform, corner * size);
    }

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
        model.faces.push(face.map(|index| first + index));
    }
}

fn add_prism(model: &mut Model, transform: Mat4, radius: f32, bottom: f32, top: f32, sides: usize) {
    let first = model.vertices.len();

    // Anillo superior y anillo inferior.
    for height in [top, bottom] {
        for index in 0..sides {
            let angle = index as f32 / sides as f32 * std::f32::consts::TAU;

            add_vertex(
                model,
                transform,
                Vec3::new(angle.cos() * radius, height, angle.sin() * radius),
            );
        }
    }

    let top_center = model.vertices.len();
    add_vertex(model, transform, Vec3::Y * top);

    let bottom_center = model.vertices.len();
    add_vertex(model, transform, Vec3::Y * bottom);

    for index in 0..sides {
        let next = (index + 1) % sides;

        let a = first + index;
        let b = first + next;
        let c = first + sides + index;
        let d = first + sides + next;

        model.faces.push([top_center, b, a]);
        model.faces.push([bottom_center, c, d]);

        model.faces.push([a, b, c]);
        model.faces.push([b, d, c]);
    }
}

fn add_petal(model: &mut Model, transform: Mat4, length: f32, half_width: f32, lift: f32) {
    let first = model.vertices.len();

    let positions = [
        // Base del pétalo.
        Vec3::ZERO,
        // Borde izquierdo.
        Vec3::new(-half_width, lift * 0.3, length * 0.45),
        // Punta levantada.
        Vec3::new(0.0, lift, length),
        // Borde derecho.
        Vec3::new(half_width, lift * 0.3, length * 0.45),
        // Centro que da volumen al pétalo.
        Vec3::new(0.0, lift * 0.55, length * 0.45),
    ];

    for position in positions {
        add_vertex(model, transform, position);
    }

    for index in 0..4 {
        let next = (index + 1) % 4;

        model.faces.push([first + 4, first + index, first + next]);
    }
}
