use glam::Vec3;

use crate::obj_loader::Model;
use crate::sphere::create_sphere;

// Incluye la mansión ampliada y los árboles.
pub const BOUND_RADIUS: f32 = 2.25;

pub fn create_surface(latitudes: usize, longitudes: usize) -> Result<Model, String> {
    let mut model = create_sphere(latitudes, longitudes)?;

    for vertex in &mut model.vertices {
        let direction = Vec3::new(vertex.x, vertex.y, vertex.z).normalize_or_zero();

        // Relieve más marcado para formar caras rocosas.
        let mountains = (direction.x * 5.0 + direction.z * 2.0).sin()
            * (direction.y * 4.0 - direction.z * 3.0).cos();

        let ridges = (direction.z * 8.0 - direction.x * 3.0 + direction.y * 2.0).sin();

        let rough_radius = 1.0 + mountains * 0.14 + ridges * 0.075;

        // Reservamos una zona estable para apoyar la mansión.
        let plateau = smooth_transition(0.65, 0.93, direction.y);

        let radius = rough_radius * (1.0 - plateau) + 1.06 * plateau;

        let position = direction * radius;

        vertex.x = position.x;
        vertex.y = position.y;
        vertex.z = position.z;
    }

    Ok(model)
}

pub fn surface_color(position: Vec3) -> Vec3 {
    let direction = position.normalize_or_zero();

    // Roca azul pizarra con variación moderada.
    let variation = (direction.x * 4.0 + direction.z * 3.0 - direction.y * 2.0).sin() * 0.5 + 0.5;

    let dark_rock = Vec3::new(0.25, 0.27, 0.36);
    let light_rock = Vec3::new(0.48, 0.49, 0.61);

    let rock = dark_rock.lerp(light_rock, variation);

    // Nieve con un borde definido en la zona superior.
    let snow_line = direction.y + (direction.x * 6.0 + direction.z * 7.0).sin() * 0.10;

    let snow_amount = smooth_transition(0.43, 0.49, snow_line);

    let snow = Vec3::new(0.91, 0.93, 1.0);

    rock.lerp(snow, snow_amount)
}

fn smooth_transition(start: f32, end: f32, value: f32) -> f32 {
    let t = ((value - start) / (end - start)).clamp(0.0, 1.0);

    t * t * (3.0 - 2.0 * t)
}
