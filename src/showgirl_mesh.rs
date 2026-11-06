use glam::{Mat4, Vec3};

use crate::obj_loader::{Model, Vertex};

pub struct Mesh {
    pub model: Model,
}

impl Mesh {
    pub fn new() -> Self {
        Self {
            model: Model {
                vertices: Vec::new(),
                faces: Vec::new(),
            },
        }
    }

    pub fn triangle(&mut self, transform: Mat4, points: [Vec3; 3]) {
        if (points[1] - points[0])
            .cross(points[2] - points[0])
            .length_squared()
            < 1e-14
        {
            return;
        }
        let first = self.model.vertices.len();
        for point in points {
            let point = transform.transform_point3(point);
            self.model.vertices.push(Vertex {
                x: point.x,
                y: point.y,
                z: point.z,
            });
        }
        self.model.faces.push([first, first + 1, first + 2]);
    }

    pub fn append(&mut self, source: &Model, transform: Mat4) {
        let first = self.model.vertices.len();
        for v in &source.vertices {
            let p = transform.transform_point3(Vec3::new(v.x, v.y, v.z));
            self.model.vertices.push(Vertex {
                x: p.x,
                y: p.y,
                z: p.z,
            });
        }
        for face in &source.faces {
            self.model.faces.push(face.map(|i| first + i));
        }
    }

    pub fn cuboid(&mut self, transform: Mat4, size: Vec3) {
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
        .map(|p| p * size);
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
            self.triangle(transform, face.map(|i| positions[i]));
        }
    }

    pub fn cylinder(&mut self, transform: Mat4, radius: f32, bottom: f32, top: f32, sides: usize) {
        for i in 0..sides {
            let angle = i as f32 / sides as f32 * std::f32::consts::TAU;
            let next = (i + 1) as f32 / sides as f32 * std::f32::consts::TAU;
            let a = Vec3::new(angle.cos() * radius, bottom, angle.sin() * radius);
            let b = Vec3::new(next.cos() * radius, bottom, next.sin() * radius);
            let c = Vec3::new(a.x, top, a.z);
            let d = Vec3::new(b.x, top, b.z);
            self.triangle(transform, [a, c, b]);
            self.triangle(transform, [b, c, d]);
            self.triangle(transform, [Vec3::Y * top, d, c]);
            self.triangle(transform, [Vec3::Y * bottom, a, b]);
        }
    }

    // Banda circular sólida: permite construir escalones curvos.
    pub fn curved_step(&mut self, inner: f32, outer: f32, bottom: f32, top: f32, half_angle: f32) {
        let segments = 64;
        let point = |radius: f32, angle: f32, height: f32| {
            Vec3::new(angle.sin() * radius, height, angle.cos() * radius)
        };
        for i in 0..segments {
            let a = -half_angle + i as f32 / segments as f32 * half_angle * 2.0;
            let b = -half_angle + (i + 1) as f32 / segments as f32 * half_angle * 2.0;
            let points = [
                point(inner, a, bottom),
                point(outer, a, bottom),
                point(outer, b, bottom),
                point(inner, b, bottom),
                point(inner, a, top),
                point(outer, a, top),
                point(outer, b, top),
                point(inner, b, top),
            ];
            // La parametrización x=sin(a),z=cos(a) conserva este orden.
            let faces = [
                [4, 5, 6],
                [4, 6, 7],
                [0, 2, 1],
                [0, 3, 2],
                [1, 2, 6],
                [1, 6, 5],
                [0, 4, 7],
                [0, 7, 3],
            ];
            for face in faces {
                self.triangle(Mat4::IDENTITY, face.map(|j| points[j]));
            }
            if i == 0 {
                self.triangle(Mat4::IDENTITY, [points[0], points[1], points[5]]);
                self.triangle(Mat4::IDENTITY, [points[0], points[5], points[4]]);
            }
            if i + 1 == segments {
                self.triangle(Mat4::IDENTITY, [points[3], points[7], points[6]]);
                self.triangle(Mat4::IDENTITY, [points[3], points[6], points[2]]);
            }
        }
    }

    pub fn feather(&mut self, transform: Mat4, length: f32, width: f32) {
        let outline = [
            Vec3::new(-width * 0.14, 0.0, 0.0),
            Vec3::new(-width * 0.65, length * 0.24, 0.0),
            Vec3::new(-width, length * 0.57, 0.0),
            Vec3::new(-width * 0.60, length * 0.81, 0.0),
            Vec3::new(0.0, length, 0.0),
            Vec3::new(width * 0.60, length * 0.81, 0.0),
            Vec3::new(width, length * 0.57, 0.0),
            Vec3::new(width * 0.65, length * 0.24, 0.0),
            Vec3::new(width * 0.14, 0.0, 0.0),
        ];
        let front = Vec3::new(0.0, length * 0.5, width * 0.28);
        let back = Vec3::new(0.0, length * 0.5, -width * 0.15);
        for i in 0..outline.len() {
            let next = (i + 1) % outline.len();
            self.triangle(transform, [front, outline[next], outline[i]]);
            self.triangle(transform, [back, outline[i], outline[next]]);
        }
    }
}

pub fn icosphere(subdivisions: usize) -> Model {
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
    let mut triangles: Vec<[Vec3; 3]> = faces.into_iter().map(|f| f.map(|i| vertices[i])).collect();
    for _ in 0..subdivisions.min(3) {
        let mut next = Vec::with_capacity(triangles.len() * 4);
        for [a, b, c] in triangles {
            let ab = (a + b).normalize();
            let bc = (b + c).normalize();
            let ca = (c + a).normalize();
            next.extend([[a, ab, ca], [b, bc, ab], [c, ca, bc], [ab, bc, ca]]);
        }
        triangles = next;
    }
    let mut mesh = Mesh::new();
    for triangle in triangles {
        mesh.triangle(Mat4::IDENTITY, triangle);
    }
    mesh.model
}

pub fn translation(x: f32, y: f32, z: f32) -> Mat4 {
    Mat4::from_translation(Vec3::new(x, y, z))
}
