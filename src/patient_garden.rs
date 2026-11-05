use glam::{Mat4, Quat, Vec3};

use crate::obj_loader::{Model, Vertex};

pub struct GardenPart {
    pub model: Model,
    pub color: u32,
}

pub fn create_garden() -> Vec<GardenPart> {
    let colors = [
        0x817386, // Ramas.
        0x59394B, // Tallos y hojas color vino.
        0x20212E, // Pétalos exteriores.
        0x40364C, // Pétalos interiores.
    ];

    let mut parts: Vec<GardenPart> = colors
        .into_iter()
        .map(|color| GardenPart {
            model: Model {
                vertices: Vec::new(),
                faces: Vec::new(),
            },
            color,
        })
        .collect();

    // Árboles que acompañan la silueta de la mansión.
    let tree_directions = [
        Vec3::new(-0.48, 0.83, -0.28).normalize(),
        Vec3::new(0.50, 0.75, -0.40).normalize(),
        Vec3::new(-0.65, 0.50, 0.40).normalize(),
    ];

    for (index, direction) in tree_directions.iter().enumerate() {
        let root = surface_transform(*direction) * Mat4::from_rotation_y(index as f32 * 1.7);

        add_tree(&mut parts[0].model, root);
    }

    // Rosas grandes sobre diferentes zonas del planeta.
    let rose_directions = [
        Vec3::new(-0.45, 0.05, 0.90).normalize(),
        Vec3::new(0.40, -0.22, 0.87).normalize(),
        Vec3::new(0.15, 0.55, 0.82).normalize(),
    ];

    for direction in rose_directions {
        let root = surface_transform(direction);

        add_branch(
            &mut parts[1].model,
            root,
            Vec3::new(0.0, -0.03, 0.0),
            Vec3::new(0.0, 0.18, 0.0),
            0.014,
        );

        // Hojas a los lados del tallo.
        for side in [-1.0, 1.0] {
            add_triangle(
                &mut parts[1].model,
                root,
                [
                    Vec3::new(0.0, 0.06, 0.0),
                    Vec3::new(side * 0.14, 0.14, 0.025),
                    Vec3::new(side * 0.08, 0.04, 0.055),
                ],
            );

            add_triangle(
                &mut parts[1].model,
                root,
                [
                    Vec3::new(0.0, 0.06, 0.0),
                    Vec3::new(side * 0.08, 0.04, 0.055),
                    Vec3::new(side * 0.10, 0.09, -0.035),
                ],
            );
        }

        let flower = root * Mat4::from_translation(Vec3::Y * 0.18);

        // Tres capas superpuestas de pétalos.
        for layer in 0..3 {
            let radius = 0.26 * 0.72_f32.powi(layer);
            let height = layer as f32 * 0.065;
            let part_index = if layer == 0 { 2 } else { 3 };

            for petal in 0..7 {
                let angle = petal as f32 / 7.0 * std::f32::consts::TAU + layer as f32 * 0.42;

                add_rose_petal(&mut parts[part_index].model, flower, angle, radius, height);
            }
        }
    }

    parts
}

fn surface_transform(direction: Vec3) -> Mat4 {
    Mat4::from_translation(direction * 1.035)
        * Mat4::from_quat(Quat::from_rotation_arc(Vec3::Y, direction))
}

fn add_tree(model: &mut Model, transform: Mat4) {
    let segments = [
        (
            Vec3::new(0.0, -0.04, 0.0),
            Vec3::new(0.015, 0.37, 0.0),
            0.028,
        ),
        (
            Vec3::new(0.015, 0.37, 0.0),
            Vec3::new(0.05, 0.74, 0.0),
            0.018,
        ),
        (
            Vec3::new(0.01, 0.24, 0.0),
            Vec3::new(-0.17, 0.43, 0.04),
            0.017,
        ),
        (
            Vec3::new(-0.17, 0.43, 0.04),
            Vec3::new(-0.24, 0.61, 0.03),
            0.010,
        ),
        (
            Vec3::new(-0.17, 0.43, 0.04),
            Vec3::new(-0.32, 0.49, 0.06),
            0.009,
        ),
        (
            Vec3::new(0.02, 0.38, 0.0),
            Vec3::new(0.21, 0.52, -0.03),
            0.015,
        ),
        (
            Vec3::new(0.21, 0.52, -0.03),
            Vec3::new(0.28, 0.69, -0.04),
            0.009,
        ),
        (
            Vec3::new(0.21, 0.52, -0.03),
            Vec3::new(0.34, 0.56, 0.04),
            0.008,
        ),
        (
            Vec3::new(0.035, 0.57, 0.0),
            Vec3::new(-0.10, 0.68, -0.12),
            0.010,
        ),
    ];

    for (start, end, radius) in segments {
        add_branch(model, transform, start, end, radius * 1.6);
    }
}

fn add_branch(model: &mut Model, transform: Mat4, start: Vec3, end: Vec3, radius: f32) {
    let direction = end - start;
    let length = direction.length();

    let branch_transform = transform
        * Mat4::from_translation(start)
        * Mat4::from_quat(Quat::from_rotation_arc(Vec3::Y, direction / length));

    let sides = 6;

    for index in 0..sides {
        let angle = index as f32 / sides as f32 * std::f32::consts::TAU;

        let next = (index + 1) as f32 / sides as f32 * std::f32::consts::TAU;

        let a = Vec3::new(angle.cos() * radius, 0.0, angle.sin() * radius);

        let b = Vec3::new(next.cos() * radius, 0.0, next.sin() * radius);

        // La rama se estrecha hacia la punta.
        let c = Vec3::new(a.x * 0.6, length, a.z * 0.6);
        let d = Vec3::new(b.x * 0.6, length, b.z * 0.6);

        add_triangle(model, branch_transform, [a, c, b]);
        add_triangle(model, branch_transform, [b, c, d]);

        add_triangle(model, branch_transform, [Vec3::ZERO, a, b]);

        add_triangle(model, branch_transform, [Vec3::Y * length, d, c]);
    }
}

fn add_rose_petal(model: &mut Model, transform: Mat4, angle: f32, radius: f32, height: f32) {
    let radial = |angle: f32, distance: f32, y: f32| {
        Vec3::new(angle.cos() * distance, y, angle.sin() * distance)
    };

    let base = radial(angle, radius * 0.20, height);

    let left = radial(angle - 0.45, radius, height + 0.035);

    let right = radial(angle + 0.45, radius, height + 0.035);

    // Pliegue levantado que da volumen al pétalo.
    let fold = radial(angle, radius * 0.80, height + 0.13);

    // Borde exterior que vuelve a bajar.
    let lip = radial(angle, radius * 1.12, height + 0.06);

    add_triangle(model, transform, [base, fold, left]);
    add_triangle(model, transform, [base, right, fold]);

    add_triangle(model, transform, [left, fold, lip]);
    add_triangle(model, transform, [fold, right, lip]);
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
