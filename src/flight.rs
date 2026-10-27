use glam::{Quat, Vec3};

#[derive(Default)]
pub struct FlightInput {
    pub throttle: f32,
    pub yaw: f32,
    pub pitch: f32,
}

pub struct FlightController {
    pub position: Vec3,
    yaw: f32,
    pitch: f32,
    speed: f32,
    yaw_speed: f32,
    pitch_speed: f32,
}

impl FlightController {
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            yaw: std::f32::consts::PI,
            pitch: 0.0,
            speed: 0.0,
            yaw_speed: 0.0,
            pitch_speed: 0.0,
        }
    }

    pub fn orientation(&self) -> Quat {
        Quat::from_rotation_y(self.yaw) * Quat::from_rotation_x(-self.pitch)
    }

    pub fn forward(&self) -> Vec3 {
        self.orientation() * Vec3::Z
    }

    pub fn set_pose(&mut self, position: Vec3, forward: Vec3) {
        let direction = forward.try_normalize().unwrap_or_else(|| self.forward());

        self.position = position;

        self.yaw = direction
            .x
            .atan2(direction.z)
            .rem_euclid(std::f32::consts::TAU);

        self.pitch = direction.y.clamp(-1.0, 1.0).asin();

        self.speed = 0.0;
        self.yaw_speed = 0.0;
        self.pitch_speed = 0.0;
    }

    pub fn update(&mut self, input: &FlightInput, delta_time: f32) {
        if !delta_time.is_finite() || delta_time <= 0.0 {
            return;
        }

        let yaw_change = integrate_axis(
            &mut self.yaw_speed,
            input.yaw.clamp(-1.0, 1.0) * 1.5,
            5.0,
            delta_time,
        );

        let pitch_change = integrate_axis(
            &mut self.pitch_speed,
            input.pitch.clamp(-1.0, 1.0) * 1.1,
            5.0,
            delta_time,
        );

        self.yaw = (self.yaw + yaw_change).rem_euclid(std::f32::consts::TAU);

        let lower = self.pitch.min(-1.15);
        let upper = self.pitch.max(1.15);

        self.pitch = (self.pitch + pitch_change).clamp(lower, upper);

        let distance = integrate_axis(
            &mut self.speed,
            input.throttle.clamp(-1.0, 1.0) * 6.0,
            3.0,
            delta_time,
        );

        self.position += self.forward() * distance;
    }
}

// Suaviza la velocidad y calcula el desplazamiento del intervalo.
fn integrate_axis(velocity: &mut f32, target: f32, response: f32, delta_time: f32) -> f32 {
    let decay = (-response * delta_time).exp();

    let displacement = target * delta_time + (*velocity - target) * (1.0 - decay) / response;

    *velocity = target + (*velocity - target) * decay;

    displacement
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn avanzar_un_segundo_es_consistente_con_distintos_pasos() {
        let input = FlightInput {
            throttle: 1.0,
            ..Default::default()
        };

        let mut single_step = FlightController::new(Vec3::ZERO);
        let mut many_steps = FlightController::new(Vec3::ZERO);

        single_step.update(&input, 1.0);

        for _ in 0..120 {
            many_steps.update(&input, 1.0 / 120.0);
        }

        assert!(single_step.position.distance(many_steps.position) < 0.0001);

        assert!((single_step.speed - many_steps.speed).abs() < 0.0001);
    }

    #[test]
    fn soltar_el_acelerador_reduce_la_velocidad_progresivamente() {
        let mut controller = FlightController::new(Vec3::ZERO);

        controller.update(
            &FlightInput {
                throttle: 1.0,
                ..Default::default()
            },
            1.0,
        );

        let previous_speed = controller.speed;

        controller.update(&FlightInput::default(), 0.5);

        assert!(controller.speed > 0.0);
        assert!(controller.speed < previous_speed);
    }

    #[test]
    fn un_intervalo_invalido_no_modifica_la_posicion() {
        let initial_position = Vec3::new(1.0, 2.0, 3.0);
        let mut controller = FlightController::new(initial_position);

        let input = FlightInput {
            throttle: 1.0,
            ..Default::default()
        };

        for delta_time in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            controller.update(&input, delta_time);
        }

        assert_eq!(controller.position, initial_position);
        assert_eq!(controller.speed, 0.0);
    }

    #[test]
    fn cambiar_de_modo_conserva_la_posicion_y_el_rumbo() {
        let mut controller = FlightController::new(Vec3::ZERO);

        let position = Vec3::new(3.0, 4.0, 5.0);
        let direction = Vec3::new(0.0, -1.0, 0.0);

        controller.set_pose(position, direction);
        controller.update(&FlightInput::default(), 0.1);

        assert_eq!(controller.position, position);
        assert!(controller.forward().dot(direction) > 0.9999);
        assert_eq!(controller.speed, 0.0);
    }
}
