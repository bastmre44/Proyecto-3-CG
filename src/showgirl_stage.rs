use glam::{Mat4, Vec3};

use crate::obj_loader::Model;
use crate::showgirl_mesh::{Mesh, icosphere, translation};

pub struct StagePiece {
    pub model: Model,
    pub color: u32,
    pub unlit: bool,
    pulse_phase: Option<f32>,
}

impl StagePiece {
    pub fn color_at(&self, time: f32) -> u32 {
        let Some(phase) = self.pulse_phase else {
            return self.color;
        };
        let brightness = 0.80 + 0.20 * (time * 3.0 - phase).sin();
        let channel = |shift: u32| -> u32 {
            ((((self.color >> shift) & 255) as f32) * brightness).round() as u32
        };
        (channel(16) << 16) | (channel(8) << 8) | channel(0)
    }
}

pub fn create_stage() -> Vec<StagePiece> {
    let colors = [0xC4472D, 0xFFCB62, 0x8E2135, 0xFF7A18, 0xFFD16A];
    let mut meshes: Vec<Mesh> = colors.iter().map(|_| Mesh::new()).collect();
    let mut bulbs = Vec::new();

    // Escenario integrado en el casquete plano del planeta.
    meshes[0].cylinder(Mat4::IDENTITY, 0.86, 0.55, 0.667, 96);
    meshes[1].curved_step(0.835, 0.875, 0.655, 0.680, std::f32::consts::PI);
    for i in 0..3 {
        let inner = 0.86 + i as f32 * 0.06;
        let top = 0.64 - i as f32 * 0.05;
        meshes[0].curved_step(inner, inner + 0.065, 0.47, top, 1.1);
        meshes[1].curved_step(inner + 0.049, inner + 0.068, top - 0.012, top + 0.004, 1.1);
    }

    // Tres portales, el central más alto.
    for (x, radius, height) in [(-0.54, 0.19, 0.68), (0.0, 0.28, 0.93), (0.54, 0.19, 0.68)] {
        let base = translation(x, 0.68, -0.32);
        let straight = height - radius;
        for side in [-1.0, 1.0] {
            meshes[1].cuboid(
                base * translation(side * radius, straight * 0.5, 0.0),
                Vec3::new(0.055, straight, 0.075),
            );
            meshes[1].cuboid(
                base * translation(side * radius, 0.04, 0.015),
                Vec3::new(0.11, 0.08, 0.12),
            );
        }
        let segments = 48;
        let arc_point =
            |angle: f32, r: f32, z: f32| Vec3::new(angle.cos() * r, straight + angle.sin() * r, z);
        for i in 0..segments {
            let a = i as f32 / segments as f32 * std::f32::consts::PI;
            let b = (i + 1) as f32 / segments as f32 * std::f32::consts::PI;
            let inner = radius - 0.028;
            let outer = radius + 0.028;
            let points = [
                arc_point(a, inner, 0.038),
                arc_point(a, outer, 0.038),
                arc_point(b, outer, 0.038),
                arc_point(b, inner, 0.038),
                arc_point(a, inner, -0.038),
                arc_point(a, outer, -0.038),
                arc_point(b, outer, -0.038),
                arc_point(b, inner, -0.038),
            ];
            for face in [
                [0, 1, 2],
                [0, 2, 3],
                [4, 6, 5],
                [4, 7, 6],
                [1, 5, 6],
                [1, 6, 2],
                [0, 3, 7],
                [0, 7, 4],
            ] {
                meshes[1].triangle(base, face.map(|j| points[j]));
            }
            meshes[2].triangle(
                base,
                [
                    Vec3::new(0.0, straight, -0.035),
                    arc_point(a, radius, -0.035),
                    arc_point(b, radius, -0.035),
                ],
            );
        }
        meshes[2].cuboid(
            base * translation(0.0, straight * 0.5, -0.05),
            Vec3::new(radius * 2.0, straight, 0.03),
        );
        // Pliegues geométricos anchos del fondo rojo.
        for k in [-1.0, 0.0, 1.0] {
            meshes[0].cuboid(
                base * translation(k * radius * 0.5, straight * 0.45, -0.028),
                Vec3::new(0.012, straight * 0.9, 0.008),
            );
        }

        for i in 0..11 {
            let angle = i as f32 / 10.0 * std::f32::consts::PI;
            bulbs.push(make_bulb(
                base.transform_point3(arc_point(angle, radius, 0.066)),
                i as f32 * 0.38 + x,
            ));
        }
        for side in [-1.0, 1.0] {
            for i in 1..=3 {
                let p = base.transform_point3(Vec3::new(
                    side * radius,
                    straight * i as f32 / 4.0,
                    0.066,
                ));
                bulbs.push(make_bulb(p, i as f32 * 0.6 + side + x));
            }
        }
    }

    // Siete plumas grandes, con un pliegue central y espesor.
    for i in 0..7 {
        let angle = (i as f32 - 3.0) * 0.34;
        let length = 1.40 - (i as f32 - 3.0).abs() * 0.07;
        let root = translation(0.0, 0.82, -0.50) * Mat4::from_rotation_z(angle);
        let color_index = if i % 2 == 0 { 3 } else { 4 };
        meshes[color_index].feather(root, length, 0.17);
        meshes[1].cuboid(
            root * translation(0.0, length * 0.44, 0.05),
            Vec3::new(0.014, length * 0.88, 0.014),
        );
    }

    // Adornos art déco al frente del escenario.
    for side in [-1.0, 1.0] {
        for i in 0..4 {
            let angle = (i as f32 - 1.5) * 0.19;
            meshes[1].cuboid(
                translation(side * 0.56, 0.55, 0.65) * Mat4::from_rotation_z(side * angle),
                Vec3::new(0.012, 0.14, 0.018),
            );
        }
    }

    let mut pieces: Vec<StagePiece> = meshes
        .into_iter()
        .zip(colors)
        .map(|(mesh, color)| StagePiece {
            model: mesh.model,
            color,
            unlit: false,
            pulse_phase: None,
        })
        .collect();
    pieces.extend(bulbs);
    pieces
}

fn make_bulb(position: Vec3, phase: f32) -> StagePiece {
    let mut mesh = Mesh::new();
    mesh.append(
        &icosphere(1),
        Mat4::from_translation(position) * Mat4::from_scale(Vec3::splat(0.018)),
    );
    StagePiece {
        model: mesh.model,
        color: 0xFFF0AF,
        unlit: true,
        pulse_phase: Some(phase),
    }
}
