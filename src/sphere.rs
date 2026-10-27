use std::f32::consts::{PI, TAU};

use crate::obj_loader::{Model, Vertex};

pub fn create_sphere(latitudes: usize, longitudes: usize) -> Result<Model, String> {
    if latitudes < 2 || longitudes < 3 {
        return Err("La esfera necesita al menos 2 latitudes y 3 longitudes".into());
    }

    let mut vertices = Vec::new();
    let mut faces = Vec::new();

    // Polo norte.
    vertices.push(Vertex {
        x: 0.0,
        y: 1.0,
        z: 0.0,
    });

    // Anillos entre los dos polos.
    for latitude in 1..latitudes {
        let theta = PI * latitude as f32 / latitudes as f32;

        for longitude in 0..longitudes {
            let phi = TAU * longitude as f32 / longitudes as f32;

            vertices.push(Vertex {
                x: theta.sin() * phi.cos(),
                y: theta.cos(),
                z: theta.sin() * phi.sin(),
            });
        }
    }

    let south_pole = vertices.len();

    vertices.push(Vertex {
        x: 0.0,
        y: -1.0,
        z: 0.0,
    });

    // Triángulos alrededor del polo norte.
    for longitude in 0..longitudes {
        let current = 1 + longitude;
        let next = 1 + (longitude + 1) % longitudes;

        faces.push([0, next, current]);
    }

    // Dos triángulos por cada sección entre anillos.
    for ring in 0..latitudes - 2 {
        for longitude in 0..longitudes {
            let next = (longitude + 1) % longitudes;

            let a = 1 + ring * longitudes + longitude;
            let b = 1 + ring * longitudes + next;
            let c = a + longitudes;
            let d = b + longitudes;

            faces.push([a, b, c]);
            faces.push([b, d, c]);
        }
    }

    // Triángulos alrededor del polo sur.
    let last_ring = 1 + (latitudes - 2) * longitudes;

    for longitude in 0..longitudes {
        let current = last_ring + longitude;
        let next = last_ring + (longitude + 1) % longitudes;

        faces.push([south_pole, current, next]);
    }

    Ok(Model { vertices, faces })
}
#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    #[test]
    fn genera_una_esfera_valida_con_caras_hacia_afuera() {
        for (latitudes, longitudes) in [(2, 3), (12, 24)] {
            let model = create_sphere(latitudes, longitudes).unwrap();

            assert_eq!(model.vertices.len(), 2 + (latitudes - 1) * longitudes);

            assert_eq!(model.faces.len(), 2 * longitudes * (latitudes - 1));

            for vertex in &model.vertices {
                let position = Vec3::new(vertex.x, vertex.y, vertex.z);

                assert!(position.is_finite());
                assert!((position.length() - 1.0).abs() < 0.00001);
            }

            for face in &model.faces {
                assert!(face.iter().all(|&i| i < model.vertices.len()));

                let position = |index: usize| {
                    let vertex = model.vertices[index];
                    Vec3::new(vertex.x, vertex.y, vertex.z)
                };

                let a = position(face[0]);
                let b = position(face[1]);
                let c = position(face[2]);

                let normal = (b - a).cross(c - a);

                assert!(normal.dot(a) > 0.0);
            }
        }
    }

    #[test]
    fn rechaza_cantidades_insuficientes_de_divisiones() {
        assert!(create_sphere(0, 24).is_err());
        assert!(create_sphere(1, 24).is_err());
        assert!(create_sphere(12, 0).is_err());
        assert!(create_sphere(12, 2).is_err());
    }
}
