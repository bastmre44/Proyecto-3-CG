use glam::Vec3;

use crate::flight::FlightInput;

pub struct LocalOrbit {
    distance: f32,
    azimuth: f32,
    elevation: f32,
    min_distance: f32,
    max_distance: f32,
}

impl LocalOrbit {
    pub fn new(ship_position: Vec3, planet_center: Vec3, min_distance: f32) -> Self {
        let offset = ship_position - planet_center;
        let direction = offset.try_normalize().unwrap_or(Vec3::Z);

        let distance = offset.length().max(min_distance);

        Self {
            distance,
            azimuth: direction.z.atan2(direction.x),
            elevation: direction.y.clamp(-1.0, 1.0).asin(),
            min_distance,
            max_distance: distance.max(min_distance + 12.0),
        }
    }

    pub fn update(&mut self, input: &FlightInput, delta_time: f32) {
        if !delta_time.is_finite() || delta_time <= 0.0 {
            return;
        }

        self.azimuth = (self.azimuth + input.yaw.clamp(-1.0, 1.0) * 0.8 * delta_time)
            .rem_euclid(std::f32::consts::TAU);

        // Evitar pasar directamente por los polos.
        // Si entra cerca de uno, conservar la posición inicial
        // y permitir alejar  de él sin un salto.
        let limit = std::f32::consts::FRAC_PI_2 - 0.01;
        let lower = self.elevation.min(-limit);
        let upper = self.elevation.max(limit);

        self.elevation =
            (self.elevation + input.pitch.clamp(-1.0, 1.0) * 0.65 * delta_time).clamp(lower, upper);

        self.distance = (self.distance - input.throttle.clamp(-1.0, 1.0) * 2.0 * delta_time)
            .clamp(self.min_distance, self.max_distance);
    }

    fn radial_direction(&self) -> Vec3 {
        Vec3::new(
            self.elevation.cos() * self.azimuth.cos(),
            self.elevation.sin(),
            self.elevation.cos() * self.azimuth.sin(),
        )
    }

    pub fn position(&self, planet_center: Vec3) -> Vec3 {
        planet_center + self.radial_direction() * self.distance
    }

    pub fn forward(&self) -> Vec3 {
        -self.radial_direction()
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conserva_la_posicion_y_acompana_al_planeta() {
        let center = Vec3::new(5.0, 0.0, 0.0);
        let ship = center + Vec3::new(3.0, 1.0, 2.0);

        let orbit = LocalOrbit::new(ship, center, 2.0);

        assert!(orbit.position(center).distance(ship) < 0.00001);

        let displacement = Vec3::new(2.0, 0.5, -1.0);

        let moved_position = orbit.position(center + displacement);

        assert!(moved_position.distance(ship + displacement) < 0.00001);
    }

    #[test]
    fn acercarse_no_cruza_la_distancia_minima() {
        let mut orbit = LocalOrbit::new(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO, 2.5);

        orbit.update(
            &FlightInput {
                throttle: 1.0,
                ..Default::default()
            },
            100.0,
        );

        assert!((orbit.position(Vec3::ZERO).length() - 2.5).abs() < 0.00001);
    }

    #[test]
    fn rodear_el_planeta_conserva_la_distancia() {
        let mut orbit = LocalOrbit::new(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO, 2.0);

        let initial_position = orbit.position(Vec3::ZERO);

        orbit.update(
            &FlightInput {
                yaw: 1.0,
                ..Default::default()
            },
            1.0,
        );

        let position = orbit.position(Vec3::ZERO);

        assert!((position.length() - 5.0).abs() < 0.00001);
        assert!(position.distance(initial_position) > 0.5);

        // La nave mira hacia el centro del planeta.
        assert!(orbit.forward().dot(-position.normalize()) > 0.9999);
    }

    #[test]
    fn un_intervalo_invalido_no_modifica_la_orbita() {
        let mut orbit = LocalOrbit::new(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO, 2.0);

        let initial_position = orbit.position(Vec3::ZERO);

        let input = FlightInput {
            throttle: 1.0,
            yaw: 1.0,
            pitch: 1.0,
        };

        for delta_time in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            orbit.update(&input, delta_time);
        }

        assert_eq!(orbit.position(Vec3::ZERO), initial_position);
    }
}
