use glam::Vec3;

use crate::obj_loader::{Model, Vertex};
use crate::sphere::create_sphere;

pub const BOUND_RADIUS: f32 = 2.25;

fn empty_model() -> Model {
    Model {
        vertices: Vec::new(),
        faces: Vec::new(),
    }
}

pub fn create_surface(latitudes: usize, longitudes: usize) -> Result<Model, String> {
    let mut model = create_sphere(latitudes, longitudes)?;
    for vertex in &mut model.vertices {
        let direction = Vec3::new(vertex.x, vertex.y, vertex.z).normalize_or_zero();
        let p = direction * surface_radius(direction);
        vertex.x = p.x;
        vertex.y = p.y;
        vertex.z = p.z;
    }
    Ok(model)
}

pub fn surface_radius(direction: Vec3) -> f32 {
    let direction = direction.normalize_or_zero();
    let mountains = (direction.x * 5.0 + direction.z * 2.0).sin()
        * (direction.y * 4.0 - direction.z * 3.0).cos();
    let ridges = (direction.z * 8.0 - direction.x * 3.0 + direction.y * 2.0).sin();
    let rough_radius = 1.0 + mountains * 0.14 + ridges * 0.075;
    let plateau = smooth_transition(0.65, 0.93, direction.y);
    rough_radius * (1.0 - plateau) + 1.06 * plateau
}

// El material del terreno dibuja solamente roca. La nieve tiene su propia malla.
pub fn surface_color(position: Vec3) -> Vec3 {
    let p = position.normalize_or_zero();
    let variation = (p.x * 3.0 + p.z * 2.5 - p.y * 1.5).sin() * 0.5 + 0.5;
    Vec3::new(0.29, 0.30, 0.40).lerp(Vec3::new(0.49, 0.49, 0.61), variation)
}

// Recorta cada cara del terreno contra un límite irregular de acumulación.
// Después genera una cubierta elevada y paredes laterales para darle espesor.
pub fn create_snow(surface: &Model) -> Model {
    let mut snow = empty_model();
    for face in &surface.faces {
        let points = face.map(|i| position(surface.vertices[i]));
        let normal = (points[1] - points[0])
            .cross(points[2] - points[0])
            .normalize_or_zero();
        if normal.y < 0.18 {
            continue;
        }
        let mut polygon = Vec::new();
        let mut previous = points[2];
        let mut previous_score = snow_score(previous);
        for current in points {
            let score = snow_score(current);
            if (score >= 0.0) != (previous_score >= 0.0) {
                let t = previous_score / (previous_score - score);
                polygon.push(previous.lerp(current, t));
            }
            if score >= 0.0 {
                polygon.push(current);
            }
            previous = current;
            previous_score = score;
        }
        if polygon.len() >= 3 {
            add_snow_patch(&mut snow, &polygon);
        }
    }
    snow
}

pub fn create_roof_snow(roof: &Model) -> Model {
    let mut snow = empty_model();
    for face in &roof.faces {
        let points = face.map(|i| position(roof.vertices[i]));
        let normal = (points[1] - points[0])
            .cross(points[2] - points[0])
            .normalize_or_zero();
        if normal.y < 0.15 {
            continue;
        }
        let center = (points[0] + points[1] + points[2]) / 3.0;
        // Dejar un borde oscuro ayuda a leer la forma del techo.
        let top = points.map(|p| center + (p - center) * 0.96 + normal * 0.009);
        add_triangle(&mut snow, top);
    }
    snow
}

fn snow_score(position: Vec3) -> f32 {
    let p = position.normalize_or_zero();
    p.y - 0.40 + (p.x * 6.0 + p.z * 5.0).sin() * 0.15 + (p.z * 9.0 - p.x * 3.0).sin() * 0.07
}

fn add_snow_patch(model: &mut Model, polygon: &[Vec3]) {
    let base: Vec<Vec3> = polygon
        .iter()
        .map(|&p| p + p.normalize_or_zero() * 0.002)
        .collect();
    let top: Vec<Vec3> = polygon
        .iter()
        .map(|&p| p + p.normalize_or_zero() * 0.027)
        .collect();
    for i in 1..top.len() - 1 {
        add_triangle(model, [top[0], top[i], top[i + 1]]);
    }
    for i in 0..top.len() {
        let next = (i + 1) % top.len();
        add_triangle(model, [base[i], base[next], top[i]]);
        add_triangle(model, [base[next], top[next], top[i]]);
    }
}

fn add_triangle(model: &mut Model, points: [Vec3; 3]) {
    if (points[1] - points[0])
        .cross(points[2] - points[0])
        .length_squared()
        < 1e-14
    {
        return;
    }
    let first = model.vertices.len();
    for p in points {
        model.vertices.push(Vertex {
            x: p.x,
            y: p.y,
            z: p.z,
        });
    }
    model.faces.push([first, first + 1, first + 2]);
}

fn position(vertex: Vertex) -> Vec3 {
    Vec3::new(vertex.x, vertex.y, vertex.z)
}

fn smooth_transition(start: f32, end: f32, value: f32) -> f32 {
    let t = ((value - start) / (end - start)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_cubierta_de_nieve_tiene_geometria_valida_y_espesor() {
        let surface = create_surface(12, 24).unwrap();
        let snow = create_snow(&surface);
        assert!(!snow.faces.is_empty());
        for face in &snow.faces {
            assert!(face.iter().all(|&i| i < snow.vertices.len()));
            let p = face.map(|i| position(snow.vertices[i]));
            assert!(p.iter().all(|v| v.is_finite() && v.length() < BOUND_RADIUS));
            assert!((p[1] - p[0]).cross(p[2] - p[0]).length_squared() > 1e-14);
        }
        let snowy_north_pole = Vec3::Y * (1.06 + 0.027);
        assert!(
            snow.vertices
                .iter()
                .any(|&v| position(v).distance(snowy_north_pole) < 0.0001)
        );
    }
}
