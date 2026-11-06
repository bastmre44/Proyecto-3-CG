use std::time::Duration;

// El render puede ir a 10, 30 o 60 FPS; la simulación usa pasos pequeños iguales.
// El sobrante se conserva para el siguiente cuadro, sin descartarlo.
pub const STEP_SECONDS: f64 = 1.0 / 120.0;

#[derive(Clone, Copy, Debug)]
pub struct Step {
    pub previous_time: f32,
    pub current_time: f32,
    pub delta_time: f32,
}

#[derive(Default)]
pub struct SimulationClock {
    ticks: u64,
    accumulator: f64,
}

impl SimulationClock {
    pub fn time(&self) -> f32 {
        (self.ticks as f64 * STEP_SECONDS) as f32
    }

    pub fn advance(&mut self, duration: Duration, mut update: impl FnMut(Step)) {
        self.accumulator += duration.as_secs_f64();
        // Esta tolerancia solo absorbe el redondeo de punto flotante.
        const EPSILON: f64 = 1e-12;
        while self.accumulator + EPSILON >= STEP_SECONDS {
            let previous_time = self.time();
            self.ticks += 1;
            self.accumulator = (self.accumulator - STEP_SECONDS).max(0.0);
            update(Step {
                previous_time,
                current_time: self.time(),
                delta_time: STEP_SECONDS as f32,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        collision,
        flight::{FlightController, FlightInput},
    };
    use glam::Vec3;

    fn render_frames(fps: u32, seconds: u32, mut frame: impl FnMut(Duration)) {
        let mut previous = Duration::ZERO;
        for index in 1..=fps * seconds {
            let now = Duration::from_secs_f64(index as f64 / fps as f64);
            frame(now - previous);
            previous = now;
        }
    }

    #[test]
    fn un_segundo_real_no_se_pierde_con_pocos_fps() {
        for fps in [5, 10, 20, 30, 60, 144] {
            let mut clock = SimulationClock::default();
            let mut steps = 0;
            render_frames(fps, 1, |duration| {
                clock.advance(duration, |step| {
                    steps += 1;
                    assert!(step.delta_time <= 0.01);
                    assert!(step.current_time > step.previous_time);
                });
            });
            assert_eq!(steps, 120, "FPS: {fps}");
            assert!((clock.time() - 1.0).abs() < 0.00001);
        }
    }

    #[test]
    fn conserva_el_sobrante_entre_cuadros() {
        let mut clock = SimulationClock::default();
        let mut steps = 0;
        clock.advance(Duration::from_millis(4), |_| steps += 1);
        assert_eq!(steps, 0);
        clock.advance(Duration::from_millis(5), |_| steps += 1);
        assert_eq!(steps, 1);
        assert!(clock.accumulator > 0.0 && clock.accumulator < STEP_SECONDS);
        clock.advance(Duration::from_millis(16), |_| steps += 1);
        assert_eq!(steps, 3);
        assert!((clock.time() - 0.025).abs() < 0.00001);
    }

    #[test]
    fn vuelo_con_giro_y_orbita_coinciden_con_distintos_fps() {
        let simulate = |fps| {
            let mut clock = SimulationClock::default();
            let mut flight = FlightController::new(Vec3::new(0.0, 3.0, 15.0));
            let input = FlightInput {
                throttle: 1.0,
                yaw: 0.4,
                pitch: 0.1,
            };
            let mut planet = Vec3::ZERO;
            render_frames(fps, 4, |duration| {
                clock.advance(duration, |step| {
                    let start = flight.position;
                    let position = |time: f32| {
                        Vec3::new((time * 0.2).cos() * 7.0, 0.0, (time * 0.2).sin() * 7.0)
                    };
                    let previous_planet = position(step.previous_time);
                    planet = position(step.current_time);
                    flight.update_with_speed(&input, step.delta_time, 8.0);
                    if let Some(corrected) = collision::resolve_sphere_collision(
                        start,
                        flight.position,
                        previous_planet,
                        planet,
                        2.0,
                    ) {
                        flight.position = corrected;
                    }
                });
            });
            (flight.position, flight.forward(), planet)
        };
        let reference = simulate(120);
        for fps in [5, 10, 20, 30, 60, 144] {
            let result = simulate(fps);
            assert!(
                reference.0.distance(result.0) < 0.0001,
                "Posicion a {fps} FPS"
            );
            assert!(reference.1.distance(result.1) < 0.0001, "Rumbo a {fps} FPS");
            assert!(
                reference.2.distance(result.2) < 0.0001,
                "Orbita a {fps} FPS"
            );
        }
    }
}
