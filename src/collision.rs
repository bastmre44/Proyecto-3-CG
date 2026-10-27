use glam::Vec3;

pub fn resolve_sphere_collision(
    start: Vec3,
    end: Vec3,
    previous_center: Vec3,
    current_center: Vec3,
    radius: f32,
) -> Option<Vec3> {
    // Movimiento de la nave relativo al movimiento del cuerpo.
    let relative_start = start - previous_center;
    let relative_end = end - current_center;

    let radius_squared = radius * radius;
    let margin = 0.01;

    // Si comenzamos dentro, permitimos salir.
    // Si seguimos dentro, empujamos la nave hacia la superficie.
    if relative_start.length_squared() < radius_squared {
        if relative_end.length_squared() >= radius_squared {
            return None;
        }

        let normal = relative_end.try_normalize().unwrap_or(Vec3::Z);

        return Some(current_center + normal * (radius + margin));
    }

    let movement = relative_end - relative_start;
    let a = movement.length_squared();

    if a <= f32::EPSILON {
        return None;
    }

    let b = relative_start.dot(movement);
    let c = relative_start.length_squared() - radius_squared;

    let discriminant = b * b - a * c;

    if discriminant < 0.0 {
        return None;
    }

    // Primer punto donde el recorrido toca la esfera.
    let time = (-b - discriminant.sqrt()) / a;

    if !(0.0..=1.0).contains(&time) {
        return None;
    }

    let contact = relative_start + movement * time;
    let normal = contact.try_normalize().unwrap_or(Vec3::Z);

    Some(current_center + normal * (radius + margin))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detecta_un_cruce_aunque_ambos_extremos_esten_fuera() {
        let corrected = resolve_sphere_collision(
            Vec3::new(0.0, 0.0, 3.0),
            Vec3::new(0.0, 0.0, -3.0),
            Vec3::ZERO,
            Vec3::ZERO,
            1.0,
        )
        .unwrap();

        // Debe detenerse del lado por donde entraba.
        assert!(corrected.z > 1.0);
        assert!((corrected.length() - 1.01).abs() < 0.0001);
    }

    #[test]
    fn permite_un_recorrido_que_no_toca_la_esfera() {
        let result = resolve_sphere_collision(
            Vec3::new(0.0, 0.0, 3.0),
            Vec3::new(1.0, 0.0, 3.0),
            Vec3::ZERO,
            Vec3::ZERO,
            1.0,
        );

        assert!(result.is_none());
    }

    #[test]
    fn detecta_un_cuerpo_que_cruza_una_nave_quieta() {
        let current_center = Vec3::new(3.0, 0.0, 0.0);

        let corrected = resolve_sphere_collision(
            Vec3::ZERO,
            Vec3::ZERO,
            Vec3::new(-3.0, 0.0, 0.0),
            current_center,
            1.0,
        )
        .unwrap();

        assert!((corrected.distance(current_center) - 1.01).abs() < 0.0001);
    }

    #[test]
    fn permite_salir_si_la_nave_comenzo_dentro() {
        let result = resolve_sphere_collision(
            Vec3::new(0.0, 0.0, 0.5),
            Vec3::new(0.0, 0.0, 2.0),
            Vec3::ZERO,
            Vec3::ZERO,
            1.0,
        );

        assert!(result.is_none());
    }
}
