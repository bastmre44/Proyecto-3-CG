use glam::{Mat4, Quat, Vec3};

use crate::obj_loader::{Model, Vertex};

pub const BOUND_RADIUS: f32 = 1.85;

pub struct CrystalPart {
    pub model: Model,
    pub color: u32,
}

pub fn surface_color(position: Vec3, time: f32) -> Vec3 {
    let p = position.normalize_or_zero();

    let shimmer = (p.x * 4.0 + p.z * 3.0 + (p.y * 5.0).sin() * 0.8 + time * 0.12).sin() * 0.5 + 0.5;

    let pearl = Vec3::new(0.96, 0.94, 1.0);
    let lavender = Vec3::new(0.63, 0.40, 0.94);
    let pink = Vec3::new(1.0, 0.51, 0.76);

    let base = if shimmer < 0.5 {
        lavender.lerp(pearl, shimmer * 2.0)
    } else {
        pearl.lerp(pink, (shimmer - 0.5) * 2.0)
    };

    let blue_reflection = (p.z * 6.0 - p.y * 3.0 + time * 0.08).sin() * 0.5 + 0.5;

    base.lerp(Vec3::new(0.31, 0.87, 1.0), blue_reflection * 0.35)
}

pub fn create_crystals() -> Vec<CrystalPart> {
    let colors = [
        0xD2ABFF, // Violeta.
        0x75E5E8, // Turquesa.
        0xFF91BE, // Rosa.
        0xFFE29A, // Dorado claro.
        0x8DAFFF, // Azul.
        0xE5BFFF, // Lila.
    ];

    let mut parts: Vec<CrystalPart> = colors
        .into_iter()
        .map(|color| CrystalPart {
            model: Model {
                vertices: Vec::new(),
                faces: Vec::new(),
            },
            color,
        })
        .collect();

    // Cada grupo tiene un cristal principal y dos pequeños.
    let directions = [
        Vec3::new(-0.56, 0.78, -0.25).normalize(),
        Vec3::new(0.62, 0.54, 0.50).normalize(),
        Vec3::new(-0.45, -0.25, 0.86).normalize(),
        Vec3::new(0.15, -0.60, -0.78).normalize(),
    ];

    for (index, direction) in directions.iter().enumerate() {
        let rotation = Quat::from_rotation_arc(Vec3::Y, *direction);

        let root = Mat4::from_translation(*direction * 0.96) * Mat4::from_quat(rotation);

        add_crystal(&mut parts, root, 0.13, 0.60, index);

        add_crystal(
            &mut parts,
            root * Mat4::from_translation(Vec3::new(0.15, 0.0, 0.05))
                * Mat4::from_rotation_z(-0.25),
            0.085,
            0.36,
            index + 2,
        );

        add_crystal(
            &mut parts,
            root * Mat4::from_translation(Vec3::new(-0.12, 0.0, -0.07))
                * Mat4::from_rotation_z(0.30),
            0.07,
            0.27,
            index + 4,
        );
    }

    parts
}

fn add_crystal(
    parts: &mut [CrystalPart],
    transform: Mat4,
    radius: f32,
    height: f32,
    color_offset: usize,
) {
    let sides = 6;
    let shoulder_height = height * 0.70;
    let tip = Vec3::Y * height;

    for index in 0..sides {
        let angle = index as f32 / sides as f32 * std::f32::consts::TAU;

        let next_angle = (index + 1) as f32 / sides as f32 * std::f32::consts::TAU;

        let bottom_a = Vec3::new(angle.cos() * radius, 0.0, angle.sin() * radius);

        let bottom_b = Vec3::new(next_angle.cos() * radius, 0.0, next_angle.sin() * radius);

        let top_a = bottom_a + Vec3::Y * shoulder_height;
        let top_b = bottom_b + Vec3::Y * shoulder_height;

        let color_index = (index + color_offset) % parts.len();

        let model = &mut parts[color_index].model;

        add_triangle(model, transform, [bottom_a, top_a, bottom_b]);

        add_triangle(model, transform, [bottom_b, top_a, top_b]);

        add_triangle(model, transform, [top_b, top_a, tip]);

        add_triangle(model, transform, [Vec3::ZERO, bottom_a, bottom_b]);
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
