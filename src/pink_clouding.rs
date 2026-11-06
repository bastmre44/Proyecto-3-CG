use glam::{Mat4, Quat, Vec3};

use crate::obj_loader::{Model, Vertex};

pub const BOUND_RADIUS: f32 = 1.90;

pub struct CloudPart {
    pub model: Model,
    pub color: u32,
    motion_axis: Vec3,
    motion_amount: f32,
    phase: f32,
}

impl CloudPart {
    // Movimiento rígido: la geometría se genera una sola vez.
    pub fn local_matrix(&self, time: f32) -> Mat4 {
        let offset = self.motion_axis * ((time * 0.65 + self.phase).sin() * self.motion_amount);
        Mat4::from_translation(offset)
    }
}

pub fn surface_color(position: Vec3, time: f32) -> Vec3 {
    surface_color_filtered(position, time, 0.0)
}
pub fn surface_color_filtered(position: Vec3, time: f32, footprint: f32) -> Vec3 {
    let p = position.normalize_or_zero();
    let pattern = (p.x * 3.2 + p.z * 2.5 + (p.y * 4.0).sin() * 0.5).sin() * 0.5 + 0.5;
    let rose = Vec3::new(0.88, 0.40, 0.65);
    let pink = Vec3::new(1.0, 0.69, 0.83);
    let cream = Vec3::new(1.0, 0.87, 0.84);
    let color = if pattern < 0.65 {
        rose.lerp(pink, pattern / 0.65)
    } else {
        pink.lerp(cream, (pattern - 0.65) / 0.35)
    };

    let sparkle = crate::sparkle::amount(p, time, footprint / position.length().max(0.2));
    color.lerp(Vec3::new(1.0, 0.97, 0.75), sparkle)
}

// Icosaedro subdividido: triángulos repartidos sin anillos horizontales.
pub fn create_surface() -> Model {
    let t = (1.0 + 5.0_f32.sqrt()) * 0.5;
    let vertices = [
        Vec3::new(-1.0, t, 0.0),
        Vec3::new(1.0, t, 0.0),
        Vec3::new(-1.0, -t, 0.0),
        Vec3::new(1.0, -t, 0.0),
        Vec3::new(0.0, -1.0, t),
        Vec3::new(0.0, 1.0, t),
        Vec3::new(0.0, -1.0, -t),
        Vec3::new(0.0, 1.0, -t),
        Vec3::new(t, 0.0, -1.0),
        Vec3::new(t, 0.0, 1.0),
        Vec3::new(-t, 0.0, -1.0),
        Vec3::new(-t, 0.0, 1.0),
    ]
    .map(|p| p.normalize());
    let faces = [
        [0, 11, 5],
        [0, 5, 1],
        [0, 1, 7],
        [0, 7, 10],
        [0, 10, 11],
        [1, 5, 9],
        [5, 11, 4],
        [11, 10, 2],
        [10, 7, 6],
        [7, 1, 8],
        [3, 9, 4],
        [3, 4, 2],
        [3, 2, 6],
        [3, 6, 8],
        [3, 8, 9],
        [4, 9, 5],
        [2, 4, 11],
        [6, 2, 10],
        [8, 6, 7],
        [9, 8, 1],
    ];
    let mut triangles: Vec<[Vec3; 3]> = faces
        .into_iter()
        .map(|face| face.map(|i| vertices[i]))
        .collect();
    for _ in 0..2 {
        let mut subdivided = Vec::with_capacity(triangles.len() * 4);
        for [a, b, c] in triangles {
            let ab = (a + b).normalize();
            let bc = (b + c).normalize();
            let ca = (c + a).normalize();
            subdivided.extend([[a, ab, ca], [b, bc, ab], [c, ca, bc], [ab, bc, ca]]);
        }
        triangles = subdivided;
    }
    let mut model = empty_model();
    for points in triangles {
        add_triangle(&mut model, Mat4::IDENTITY, points);
    }
    model
}

pub fn create_decorations() -> Result<Vec<CloudPart>, String> {
    let mut parts = Vec::new();
    let cloud_directions = [
        Vec3::new(-0.65, 0.65, 0.35).normalize(),
        Vec3::new(0.65, 0.25, 0.70).normalize(),
        Vec3::new(-0.35, -0.55, 0.75).normalize(),
        Vec3::new(0.20, 0.55, -0.80).normalize(),
        Vec3::new(-0.48, 0.83, 0.28).normalize(),
    ];

    for (index, direction) in cloud_directions.into_iter().enumerate() {
        let root = surface_transform(direction, 1.015) * Mat4::from_rotation_y(index as f32 * 0.8);
        let cloud = create_cloud(index as f32)?;
        let mut model = empty_model();
        append_model(&mut model, &cloud, root);
        parts.push(CloudPart {
            model,
            color: if index % 2 == 0 { 0xFFC6DD } else { 0xFFE0E9 },
            motion_axis: direction.cross(Vec3::Y).normalize_or_zero(),
            motion_amount: 0.025,
            phase: index as f32 * 1.4,
        });
    }

    let island_directions = [
        Vec3::new(-0.35, 0.92, 0.15).normalize(),
        Vec3::new(0.45, 0.85, -0.25).normalize(),
        Vec3::new(0.15, 0.65, 0.75).normalize(),
    ];
    for (index, direction) in island_directions.into_iter().enumerate() {
        let root = surface_transform(direction, 1.58) * Mat4::from_rotation_y(index as f32);
        let radius = if index == 0 { 0.22 } else { 0.17 };
        let mut top = empty_model();
        let mut rock = empty_model();
        for side in 0..6 {
            let a = side as f32 / 6.0 * std::f32::consts::TAU;
            let b = (side + 1) as f32 / 6.0 * std::f32::consts::TAU;
            let pa = Vec3::new(a.cos() * radius, 0.06, a.sin() * radius);
            let pb = Vec3::new(b.cos() * radius, 0.06, b.sin() * radius);
            add_triangle(&mut top, root, [Vec3::Y * 0.06, pb, pa]);
            add_triangle(&mut rock, root, [pa, pb, Vec3::Y * -0.25]);
        }
        // Ambas partes de cada isla comparten exactamente el mismo movimiento.
        for (model, color) in [(top, 0xFFE8D1), (rock, 0xD985B0)] {
            parts.push(CloudPart {
                model,
                color,
                motion_axis: direction,
                motion_amount: 0.045,
                phase: index as f32 * 1.3,
            });
        }
    }
    Ok(parts)
}

// A single closed bank, built from beveled cross sections, never spheres.
fn create_cloud(phase: f32) -> Result<Model, String> {
    let mut model = empty_model();
    let profiles = [
        (-0.72, 0.08, 0.10),
        (-0.53, 0.24, 0.21),
        (-0.28, 0.32, 0.36),
        (-0.02, 0.29, 0.25),
        (0.22, 0.35, 0.40),
        (0.46, 0.23, 0.24),
        (0.70, 0.07, 0.09),
    ];
    for (i, &(x, width, height)) in profiles.iter().enumerate() {
        let z = (i as f32 * 1.3 + phase).sin() * 0.065;
        // Clockwise around +X; bottom is flat, shoulders and ridge asymmetric.
        for (y, depth) in [
            (-0.055, -width * 0.65),
            (-0.055, width * 0.65),
            (0.025, width),
            (height * 0.72, width * 0.64),
            (height, -width * 0.12),
            (height * 0.58, -width * 0.83),
            (0.02, -width),
        ] {
            model.vertices.push(Vertex { x, y, z: z + depth });
        }
    }
    for i in 0..profiles.len() - 1 {
        for j in 0..7 {
            let a = i * 7 + j;
            let b = i * 7 + (j + 1) % 7;
            let c = a + 7;
            let d = b + 7;
            model.faces.extend([[a, b, c], [b, d, c]]);
        }
    }
    for j in 1..6 {
        model.faces.push([0, j + 1, j]);
        let end = (profiles.len() - 1) * 7;
        model.faces.push([end, end + j, end + j + 1]);
    }
    for face in &mut model.faces {
        face.swap(1, 2);
    }
    Ok(model)
}

fn empty_model() -> Model {
    Model {
        vertices: Vec::new(),
        faces: Vec::new(),
    }
}

fn surface_transform(direction: Vec3, distance: f32) -> Mat4 {
    Mat4::from_translation(direction * distance)
        * Mat4::from_quat(Quat::from_rotation_arc(Vec3::Y, direction))
}

fn append_model(target: &mut Model, source: &Model, transform: Mat4) {
    let first = target.vertices.len();
    for v in &source.vertices {
        let p = transform.transform_point3(Vec3::new(v.x, v.y, v.z));
        target.vertices.push(Vertex {
            x: p.x,
            y: p.y,
            z: p.z,
        });
    }
    for face in &source.faces {
        target.faces.push(face.map(|i| first + i));
    }
}

fn add_triangle(model: &mut Model, transform: Mat4, points: [Vec3; 3]) {
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
    fn bancos_cerrados_con_normales_exteriores_y_limite_analitico() {
        for phase in [0.0, 1.0, 2.0, 3.0] {
            let model = create_cloud(phase).unwrap();
            let mut edges = std::collections::HashMap::new();
            let mut volume = 0.0;
            for face in &model.faces {
                assert!(face.iter().all(|&i| i < model.vertices.len()));
                let [a, b, c] = face.map(|i| {
                    let v = model.vertices[i];
                    Vec3::new(v.x, v.y, v.z)
                });
                assert!((b - a).cross(c - a).length_squared() > 1e-12);
                volume += a.dot(b.cross(c));
                for (a, b) in [(face[0], face[1]), (face[1], face[2]), (face[2], face[0])] {
                    let entry = edges.entry((a.min(b), a.max(b))).or_insert((0, 0));
                    entry.0 += 1;
                    entry.1 += if a < b { 1 } else { -1 };
                }
            }
            assert!(volume > 0.0);
            assert!(
                edges
                    .values()
                    .all(|&(count, orientation)| count == 2 && orientation == 0)
            );
        }
        for part in create_decorations().unwrap() {
            for v in part.model.vertices {
                // Triangle inequality bounds every animation phase, not just samples.
                assert!(Vec3::new(v.x, v.y, v.z).length() + part.motion_amount < BOUND_RADIUS);
            }
        }
    }

    #[test]
    fn la_superficie_subdividida_tiene_caras_hacia_afuera() {
        let model = create_surface();
        assert_eq!(model.faces.len(), 320);
        for face in &model.faces {
            let [a, b, c] = face.map(|i| {
                let v = model.vertices[i];
                Vec3::new(v.x, v.y, v.z)
            });
            assert!((a.length() - 1.0).abs() < 0.00001);
            assert!((b - a).cross(c - a).dot(a) > 0.0);
        }
    }

    #[test]
    fn las_decoraciones_animadas_caben_en_el_limite_de_colision() {
        let parts = create_decorations().unwrap();
        for time in [0.0, 1.0, 3.0, 8.0] {
            for part in &parts {
                let transform = part.local_matrix(time);
                for v in &part.model.vertices {
                    let p = transform.transform_point3(Vec3::new(v.x, v.y, v.z));
                    assert!(p.is_finite() && p.length() < BOUND_RADIUS);
                }
            }
        }
    }
}
