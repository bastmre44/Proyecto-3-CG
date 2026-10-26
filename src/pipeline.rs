use glam::{Mat4, Vec3, Vec4};

use crate::obj_loader::{Model, Vertex};

pub fn transform_vertices(
    model: &Model,
    model_matrix: Mat4,
    view_matrix: Mat4,
    projection_matrix: Mat4,
) -> Vec<Vec4> {
    // Se aplica primero modelo, luego vista y finalmente proyección.
    let mvp = projection_matrix * view_matrix * model_matrix;

    model
        .vertices
        .iter()
        .map(|vertex| {
            let position = Vec3::new(vertex.x, vertex.y, vertex.z);

            mvp * position.extend(1.0)
        })
        .collect()
}

pub fn to_screen(clip_position: Vec4, width: usize, height: usize) -> Option<Vertex> {
    // Evita dividir entre cero o proyectar posiciones inválidas.
    if !clip_position.is_finite() || clip_position.w <= f32::EPSILON {
        return None;
    }

    let ndc = clip_position.truncate() / clip_position.w;

    Some(Vertex {
        x: (ndc.x + 1.0) * 0.5 * width as f32,
        y: (1.0 - ndc.y) * 0.5 * height as f32,
        z: (ndc.z + 1.0) * 0.5,
    })
}
pub fn clip_triangle(vertices: [Vec4; 3]) -> Vec<[Vec4; 3]> {
    if vertices.iter().any(|vertex| !vertex.is_finite()) {
        return Vec::new();
    }

    // Planos izquierdo, derecho, inferior, superior, cercano y lejano.
    // Corresponden a la proyección perspective_rh_gl de la cámara.
    let planes = [
        Vec4::new(1.0, 0.0, 0.0, 1.0),
        Vec4::new(-1.0, 0.0, 0.0, 1.0),
        Vec4::new(0.0, 1.0, 0.0, 1.0),
        Vec4::new(0.0, -1.0, 0.0, 1.0),
        Vec4::new(0.0, 0.0, 1.0, 1.0),
        Vec4::new(0.0, 0.0, -1.0, 1.0),
    ];

    let mut polygon = vertices.to_vec();

    for plane in planes {
        if polygon.is_empty() {
            return Vec::new();
        }

        let mut output = Vec::new();

        let mut previous = polygon[polygon.len() - 1];
        let mut previous_distance = plane.dot(previous);

        for &current in &polygon {
            let current_distance = plane.dot(current);

            let previous_inside = previous_distance >= 0.0;
            let current_inside = current_distance >= 0.0;

            // Una arista cruza el plano: calculo la intersección.
            if previous_inside != current_inside {
                let t = previous_distance / (previous_distance - current_distance);

                let intersection = previous.lerp(current, t);
                output.push(intersection);
            }

            if current_inside {
                output.push(current);
            }

            previous = current;
            previous_distance = current_distance;
        }

        polygon = output;
    }

    if polygon.len() < 3 {
        return Vec::new();
    }

    // El recorte puede producir un polígono de más de tres vértices.
    // Lo divido nuevamente en triángulos.
    let mut triangles = Vec::new();

    for index in 1..polygon.len() - 1 {
        triangles.push([polygon[0], polygon[index], polygon[index + 1]]);
    }

    triangles
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conserva_un_triangulo_dentro_de_la_vista() {
        let input = [
            Vec4::new(-0.5, -0.5, 0.0, 1.0),
            Vec4::new(0.5, -0.5, 0.0, 1.0),
            Vec4::new(0.0, 0.5, 0.0, 1.0),
        ];

        let result = clip_triangle(input);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], input);
    }

    #[test]
    fn descarta_un_triangulo_completamente_fuera() {
        let input = [
            Vec4::new(2.0, 0.0, 0.0, 1.0),
            Vec4::new(3.0, 0.5, 0.0, 1.0),
            Vec4::new(3.0, -0.5, 0.0, 1.0),
        ];

        assert!(clip_triangle(input).is_empty());
    }

    #[test]
    fn recorta_un_triangulo_que_cruza_el_plano_cercano() {
        let input = [
            Vec4::new(-0.5, -0.5, 0.0, 1.0),
            Vec4::new(0.5, -0.5, 0.0, 1.0),
            Vec4::new(0.0, 0.5, -2.0, 1.0),
        ];

        let result = clip_triangle(input);

        // El triángulo recortado forma un cuadrilátero.
        assert_eq!(result.len(), 2);

        for triangle in result {
            for vertex in triangle {
                let limit = vertex.w + 0.00001;

                assert!(vertex.is_finite());
                assert!(vertex.x.abs() <= limit);
                assert!(vertex.y.abs() <= limit);
                assert!(vertex.z.abs() <= limit);
            }
        }
    }

    #[test]
    fn convierte_el_centro_y_rechaza_una_division_invalida() {
        let center = to_screen(Vec4::new(0.0, 0.0, 0.0, 1.0), 1000, 750).unwrap();

        assert_eq!(center.x, 500.0);
        assert_eq!(center.y, 375.0);
        assert_eq!(center.z, 0.5);

        assert!(to_screen(Vec4::ZERO, 1000, 750).is_none());
    }
}
