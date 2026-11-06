use crate::obj_loader::{Model, Vertex};
use glam::{Mat4, Quat, Vec3};

pub struct GardenPart {
    pub model: Model,
    pub color: u32,
}

pub fn create_garden() -> Vec<GardenPart> {
    let mut parts: Vec<GardenPart> = [0x817386, 0x59394B, 0x211D2B, 0x302637, 0x392D40, 0x44354A]
        .into_iter()
        .map(|color| GardenPart {
            model: Model {
                vertices: Vec::new(),
                faces: Vec::new(),
            },
            color,
        })
        .collect();

    let tree_directions = [
        Vec3::new(-0.48, 0.83, -0.28).normalize(),
        Vec3::new(0.50, 0.75, -0.40).normalize(),
        Vec3::new(-0.65, 0.50, 0.40).normalize(),
    ];
    for (index, direction) in tree_directions.into_iter().enumerate() {
        let root = surface_transform(direction) * Mat4::from_rotation_y(index as f32 * 1.7);
        let branches = [
            (
                Vec3::new(0.0, -0.04, 0.0),
                Vec3::new(0.015, 0.37, 0.0),
                0.043,
            ),
            (
                Vec3::new(0.015, 0.37, 0.0),
                Vec3::new(0.05, 0.74, 0.0),
                0.027,
            ),
            (
                Vec3::new(0.01, 0.24, 0.0),
                Vec3::new(-0.17, 0.43, 0.04),
                0.025,
            ),
            (
                Vec3::new(-0.17, 0.43, 0.04),
                Vec3::new(-0.24, 0.61, 0.03),
                0.014,
            ),
            (
                Vec3::new(-0.17, 0.43, 0.04),
                Vec3::new(-0.32, 0.49, 0.06),
                0.011,
            ),
            (
                Vec3::new(0.02, 0.38, 0.0),
                Vec3::new(0.21, 0.52, -0.03),
                0.022,
            ),
            (
                Vec3::new(0.21, 0.52, -0.03),
                Vec3::new(0.28, 0.69, -0.04),
                0.013,
            ),
            (
                Vec3::new(0.21, 0.52, -0.03),
                Vec3::new(0.34, 0.56, 0.04),
                0.011,
            ),
            (
                Vec3::new(0.035, 0.57, 0.0),
                Vec3::new(-0.10, 0.68, -0.12),
                0.014,
            ),
        ];
        for (start, end, radius) in branches {
            branch(&mut parts[0].model, root, start, end, radius);
        }
    }

    // Dos rosas protagonistas y una pequeña en el lado posterior.
    let roses = [
        (Vec3::new(-0.50, 0.13, 0.85).normalize(), 1.0),
        (Vec3::new(0.45, -0.35, 0.82).normalize(), 0.90),
    ];
    for (direction, scale) in roses {
        let root = surface_transform(direction) * Mat4::from_scale(Vec3::splat(scale));
        branch(
            &mut parts[1].model,
            root,
            Vec3::Y * -0.04,
            Vec3::Y * 0.18,
            0.017,
        );
        for side in [-1.0, 1.0] {
            let center = Vec3::new(side * 0.10, 0.07, 0.03);
            let points = [
                Vec3::new(0.0, 0.03, 0.0),
                Vec3::new(side * 0.18, 0.13, 0.03),
                Vec3::new(side * 0.14, 0.02, 0.09),
                Vec3::new(side * 0.10, 0.08, -0.04),
            ];
            for i in 0..4 {
                triangle(
                    &mut parts[1].model,
                    root,
                    [center, points[i], points[(i + 1) % 4]],
                );
            }
        }
        let flower = root * Mat4::from_translation(Vec3::Y * 0.22);
        // A closed receptacle hides terrain between overlapping petals.
        for i in 0..10 {
            let a = i as f32 * std::f32::consts::TAU / 10.0;
            let b = (i + 1) as f32 * std::f32::consts::TAU / 10.0;
            let pa = Vec3::new(a.cos() * 0.19, 0.07, a.sin() * 0.19);
            let pb = Vec3::new(b.cos() * 0.19, 0.07, b.sin() * 0.19);
            triangle(&mut parts[2].model, flower, [Vec3::Y * 0.19, pb, pa]);
            triangle(&mut parts[2].model, flower, [Vec3::Y * -0.045, pa, pb]);
        }
        // Broad cupped petals: outer lips open, inner lips rise around a bud.
        for (layer, (count, radius, height)) in [
            (8, 0.35, 0.13),
            (7, 0.265, 0.22),
            (6, 0.17, 0.29),
            (4, 0.075, 0.32),
        ]
        .into_iter()
        .enumerate()
        {
            for petal in 0..count {
                let angle =
                    petal as f32 / count as f32 * std::f32::consts::TAU + layer as f32 * 0.49;
                let transform = flower * Mat4::from_rotation_y(angle);
                let point = |row: usize, col: usize| {
                    let t = row as f32 / 3.0;
                    let u = col as f32 / 4.0 * 2.0 - 1.0;
                    let spread = std::f32::consts::PI / count as f32 * 1.5;
                    let theta = u * spread;
                    let r = radius * (0.24 + 0.76 * t);
                    Vec3::new(
                        theta.sin() * r,
                        height * t + 0.07 * (std::f32::consts::PI * t).sin() - 0.035 * u * u * t,
                        theta.cos() * r,
                    )
                };
                let model = &mut parts[2 + (petal + layer) % 4].model;
                for row in 0..3 {
                    for col in 0..4 {
                        let a = point(row, col);
                        let b = point(row + 1, col);
                        let c = point(row, col + 1);
                        let d = point(row + 1, col + 1);
                        triangle(model, transform, [a, b, c]);
                        triangle(model, transform, [c, b, d]);
                    }
                }
            }
        }
    }
    parts
}

fn surface_transform(direction: Vec3) -> Mat4 {
    let radius = crate::patient_zero::surface_radius(direction) - 0.025;
    Mat4::from_translation(direction * radius)
        * Mat4::from_quat(Quat::from_rotation_arc(Vec3::Y, direction))
}

fn branch(model: &mut Model, root: Mat4, start: Vec3, end: Vec3, radius: f32) {
    let direction = end - start;
    let length = direction.length();
    let transform = root
        * Mat4::from_translation(start)
        * Mat4::from_quat(Quat::from_rotation_arc(Vec3::Y, direction / length));
    for i in 0..6 {
        let angle = i as f32 / 6.0 * std::f32::consts::TAU;
        let next = (i + 1) as f32 / 6.0 * std::f32::consts::TAU;
        let a = Vec3::new(angle.cos() * radius, 0.0, angle.sin() * radius);
        let b = Vec3::new(next.cos() * radius, 0.0, next.sin() * radius);
        let c = Vec3::new(a.x * 0.6, length, a.z * 0.6);
        let d = Vec3::new(b.x * 0.6, length, b.z * 0.6);
        triangle(model, transform, [a, c, b]);
        triangle(model, transform, [b, c, d]);
        triangle(model, transform, [Vec3::ZERO, a, b]);
        triangle(model, transform, [Vec3::Y * length, d, c]);
    }
}

fn triangle(model: &mut Model, transform: Mat4, points: [Vec3; 3]) {
    if (points[1] - points[0])
        .cross(points[2] - points[0])
        .length_squared()
        < 1e-14
    {
        return;
    }
    let first = model.vertices.len();
    for p in points {
        let p = transform.transform_point3(p);
        model.vertices.push(Vertex {
            x: p.x,
            y: p.y,
            z: p.z,
        });
    }
    model.faces.push([first, first + 1, first + 2]);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn jardin_con_caras_validas_y_dentro_del_limite() {
        let parts = create_garden();
        let mut count = 0;
        for part in parts {
            for v in &part.model.vertices {
                let p = Vec3::new(v.x, v.y, v.z);
                assert!(p.is_finite() && p.length() < crate::patient_zero::BOUND_RADIUS);
            }
            for face in &part.model.faces {
                assert!(face.iter().all(|&i| i < part.model.vertices.len()));
                let [a, b, c] = face.map(|i| {
                    let v = part.model.vertices[i];
                    Vec3::new(v.x, v.y, v.z)
                });
                assert!((b - a).cross(c - a).length_squared() > 1e-14);
            }
            count += part.model.faces.len();
        }
        assert!(count < 2500);
    }
}
