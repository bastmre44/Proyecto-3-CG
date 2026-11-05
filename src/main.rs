mod camera;
mod collision;
mod flight;
mod framebuffer;
mod koi;
mod local_orbit;
mod obj_loader;
mod opalite;
mod opalite_plaza;
mod ophelia;
mod ophelia_garden;
mod pipeline;
mod renderer;
mod shader;
mod sphere;
mod triangle;

use std::time::Instant;

use camera::Camera;
use flight::{FlightController, FlightInput};
use framebuffer::Framebuffer;
use glam::{Mat4, Quat, Vec3};
use local_orbit::LocalOrbit;
use minifb::{Key, KeyRepeat, Window, WindowOptions};
use obj_loader::Model;
use shader::Material;

const WIDTH: usize = 1000;
const HEIGHT: usize = 750;
const BACKGROUND: u32 = 0x101522;

// true: planetas quietos para revisar detalles.
// false: órbitas y rotación normales.
const INSPECTION_MODE: bool = true;

struct Planet {
    name: &'static str,
    radius: f32,
    orbit_radius: f32,
    orbit_speed: f32,
    orbit_phase: f32,
    rotation_speed: f32,
    material: Material,
}

impl Planet {
    fn position(&self, time: f32) -> Vec3 {
        let time = if INSPECTION_MODE { 0.0 } else { time };

        let angle = time * self.orbit_speed + self.orbit_phase;

        Vec3::new(
            angle.cos() * self.orbit_radius,
            0.0,
            angle.sin() * self.orbit_radius,
        )
    }

    fn collision_radius(&self) -> f32 {
        match self.material {
            Material::Ophelia => self.radius * ophelia_garden::BOUND_RADIUS,

            Material::Opalite => self.radius * opalite::BOUND_RADIUS,

            _ => self.radius,
        }
    }

    fn model_matrix(&self, time: f32) -> Mat4 {
        let time = if INSPECTION_MODE { 0.0 } else { time };

        Mat4::from_translation(self.position(time))
            * Mat4::from_rotation_y(time * self.rotation_speed)
            * Mat4::from_scale(Vec3::splat(self.radius))
    }
}

fn input_axis(window: &Window, positive: Key, negative: Key) -> f32 {
    let positive = window.is_key_down(positive) as i32;
    let negative = window.is_key_down(negative) as i32;

    (positive - negative) as f32
}

fn draw_object(
    model: &Model,
    model_matrix: Mat4,
    camera: &Camera,
    buffer: &mut Framebuffer,
    color: u32,
    unlit: bool,
) {
    let clip_vertices = pipeline::transform_vertices(
        model,
        model_matrix,
        camera.view_matrix(),
        camera.projection_matrix(buffer.width, buffer.height),
    );

    renderer::render(model, &clip_vertices, buffer, model_matrix, color, unlit);
}

fn draw_shaded_object(
    model: &Model,
    model_matrix: Mat4,
    camera: &Camera,
    buffer: &mut Framebuffer,
    material: Material,
    time: f32,
) {
    let clip_vertices = pipeline::transform_vertices(
        model,
        model_matrix,
        camera.view_matrix(),
        camera.projection_matrix(buffer.width, buffer.height),
    );

    renderer::render_shaded(model, &clip_vertices, buffer, model_matrix, material, time);
}

fn run() -> Result<(), String> {
    let ship = obj_loader::load_obj("Nave_final.obj")?;
    let sphere = sphere::create_sphere(12, 24)?;
    let koi_model = koi::create_koi()?;
    let garden = ophelia_garden::create_garden();
    let opalite_crystals = opalite::create_crystals();
    let plaza = opalite_plaza::create_plaza()?;

    if ship.vertices.is_empty() || ship.faces.is_empty() {
        return Err("La nave no contiene geometría para dibujar".into());
    }

    let mut minimum = Vec3::splat(f32::INFINITY);
    let mut maximum = Vec3::splat(f32::NEG_INFINITY);

    for vertex in &ship.vertices {
        let position = Vec3::new(vertex.x, vertex.y, vertex.z);

        minimum = minimum.min(position);
        maximum = maximum.max(position);
    }

    let center = (minimum + maximum) * 0.5;

    let ship_scale = 0.45_f32;
    let sun_radius = 1.5_f32;
    let lavender_radius = 1.1_f32;
    let midnight_radius = 1.35_f32;

    let ship_radius = ship
        .vertices
        .iter()
        .map(|vertex| Vec3::new(vertex.x, vertex.y, vertex.z).distance(center))
        .fold(0.0_f32, f32::max)
        * ship_scale;

    // Separar las órbitas dejando espacio para la nave.
    let lavender_extent = lavender_radius * ophelia_garden::BOUND_RADIUS;

    let lavender_orbit = (sun_radius + lavender_extent + 2.0 * ship_radius + 0.5).max(5.5);

    let midnight_extent = midnight_radius * opalite::BOUND_RADIUS;

    let midnight_orbit =
        lavender_orbit + lavender_extent + midnight_extent + 2.0 * ship_radius + 0.8;

    let planets = [
        Planet {
            name: "The Fate of Ophelia",
            radius: lavender_radius,
            orbit_radius: lavender_orbit,
            orbit_speed: 0.35,
            orbit_phase: 0.0,
            rotation_speed: 0.12,
            material: Material::Ophelia,
        },
        Planet {
            name: "Opalite",
            radius: midnight_radius,
            orbit_radius: midnight_orbit,
            orbit_speed: 0.18,
            orbit_phase: 2.2,
            rotation_speed: 0.10,
            material: Material::Opalite,
        },
    ];

    let ship_start = Vec3::new(0.0, 0.4, 7.5);
    let mut flight = FlightController::new(ship_start);

    // Guarda el planeta elegido y su controlador local.
    let mut local_mode: Option<(usize, LocalOrbit)> = None;

    let mut camera = Camera::new(
        flight.position - flight.forward() * 6.0 + Vec3::Y * 2.5,
        flight.position + flight.forward() * 2.0,
    );

    let sun_matrix = Mat4::from_scale(Vec3::splat(sun_radius));
    let mut buffer = Framebuffer::new(WIDTH, HEIGHT);

    let mut window = Window::new(
        "Midnights - Sistema planetario",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .map_err(|error| format!("No se pudo crear la ventana: {error}"))?;

    window.set_target_fps(60);

    let start = Instant::now();
    let mut previous = start;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = Instant::now();
        let delta_time = now.duration_since(previous).as_secs_f32();
        let elapsed = now.duration_since(start).as_secs_f32();
        let previous_time = (elapsed - delta_time).max(0.0);

        previous = now;

        if window.is_key_pressed(Key::R, KeyRepeat::No) {
            flight = FlightController::new(ship_start);
            local_mode = None;

            camera.position = flight.position - flight.forward() * 6.0 + Vec3::Y * 2.5;

            camera.target = flight.position + flight.forward() * 2.0;

            camera.up = Vec3::Y;
        }

        let input = if window.is_active() {
            FlightInput {
                throttle: input_axis(&window, Key::W, Key::S),
                yaw: input_axis(&window, Key::A, Key::D),
                pitch: input_axis(&window, Key::Up, Key::Down),
            }
        } else {
            FlightInput::default()
        };

        // Buscar el planeta cuya superficie esté más cerca.
        let nearest_index = (0..planets.len())
            .min_by(|&a, &b| {
                let distance_a =
                    flight.position.distance(planets[a].position(elapsed)) - planets[a].radius;

                let distance_b =
                    flight.position.distance(planets[b].position(elapsed)) - planets[b].radius;

                distance_a.total_cmp(&distance_b)
            })
            .unwrap();

        if window.is_active() && window.is_key_pressed(Key::C, KeyRepeat::No) {
            if local_mode.is_some() {
                local_mode = None;

                let position = flight.position;
                let forward = flight.forward();

                flight.set_pose(position, forward);
            } else {
                let planet = &planets[nearest_index];
                let planet_position = planet.position(elapsed);
                let minimum_distance = planet.collision_radius() + ship_radius + 0.01;

                if flight.position.distance(planet_position) <= minimum_distance + 4.0 {
                    local_mode = Some((
                        nearest_index,
                        LocalOrbit::new(flight.position, planet_position, minimum_distance),
                    ));
                }
            }
        }

        let previous_ship_position = flight.position;

        if let Some((index, orbit)) = &mut local_mode {
            let planet_position = planets[*index].position(elapsed);

            orbit.update(&input, delta_time);

            flight.set_pose(orbit.position(planet_position), orbit.forward());
        } else {
            flight.update(&input, delta_time);
        }

        // El sol y ambos planetas participan en las colisiones.
        let mut bodies = vec![(Vec3::ZERO, Vec3::ZERO, sun_radius)];

        for planet in &planets {
            bodies.push((
                planet.position(previous_time),
                planet.position(elapsed),
                planet.collision_radius(),
            ));
        }

        let mut collided = false;

        for &(previous_center, current_center, body_radius) in &bodies {
            if let Some(corrected_position) = collision::resolve_sphere_collision(
                previous_ship_position,
                flight.position,
                previous_center,
                current_center,
                body_radius + ship_radius,
            ) {
                flight.position = corrected_position;
                collided = true;
            }
        }

        if let Some((index, orbit)) = &mut local_mode {
            let planet = &planets[*index];
            let planet_position = planet.position(elapsed);

            flight.set_pose(flight.position, planet_position - flight.position);

            if collided {
                *orbit = LocalOrbit::new(
                    flight.position,
                    planet_position,
                    planet.collision_radius() + ship_radius + 0.01,
                );
            }
        }

        let forward = flight.forward();
        let up = flight.orientation() * Vec3::Y;
        let right = forward.cross(up).normalize_or_zero();

        let (desired_position, desired_target) = if let Some((index, _)) = &local_mode {
            (
                flight.position - forward * 4.0 + up * 2.0 + right * 3.0,
                planets[*index].position(elapsed),
            )
        } else {
            (
                flight.position - forward * 6.0 + up * 2.5,
                flight.position + forward * 2.0,
            )
        };

        let previous_camera_position = camera.position;
        let follow = 1.0 - (-8.0 * delta_time).exp();

        camera.position += (desired_position - camera.position) * follow;

        camera.target += (desired_target - camera.target) * follow;

        camera.up = up;

        for &(previous_center, current_center, body_radius) in &bodies {
            if let Some(corrected_position) = collision::resolve_sphere_collision(
                previous_camera_position,
                camera.position,
                previous_center,
                current_center,
                body_radius + 0.2,
            ) {
                camera.position = corrected_position;
            }
        }

        let mut view_direction = camera.target - camera.position;

        if view_direction.length_squared() < 0.000001 {
            camera.target = camera.position + forward;
            view_direction = forward;
        }

        let view_direction = view_direction.normalize();

        if view_direction.cross(camera.up).length_squared() < 0.000001 {
            camera.up = if view_direction.y.abs() < 0.99 {
                Vec3::Y
            } else {
                Vec3::Z
            };
        }

        let selected_index = if let Some((index, _)) = &local_mode {
            *index
        } else {
            nearest_index
        };

        let selected = &planets[selected_index];

        let distance = flight.position.distance(selected.position(elapsed));

        let activation_distance = selected.collision_radius() + ship_radius + 0.01 + 4.0;

        let mode = if local_mode.is_some() {
            "LOCAL | C: salir"
        } else if distance <= activation_distance {
            "LIBRE | C: rodear planeta"
        } else {
            "LIBRE | Acercate para usar C"
        };

        window.set_title(&format!(
            "Midnights | {} | {mode} | Distancia: {distance:.1} | R: reiniciar",
            selected.name
        ));

        let ship_matrix = Mat4::from_scale_rotation_translation(
            Vec3::splat(ship_scale),
            flight.orientation(),
            flight.position,
        ) * Mat4::from_translation(-center);

        buffer.clear(BACKGROUND);

        draw_object(&sphere, sun_matrix, &camera, &mut buffer, 0xFFD17A, true);

        for planet in &planets {
            draw_shaded_object(
                &sphere,
                planet.model_matrix(elapsed),
                &camera,
                &mut buffer,
                planet.material,
                elapsed,
            );
        }

        // El jardín comparte la transformación de Ophelia.
        // Así orbita y gira junto con el planeta.
        let ophelia_matrix = planets[0].model_matrix(elapsed);

        for part in &garden {
            draw_object(
                &part.model,
                ophelia_matrix,
                &camera,
                &mut buffer,
                part.color,
                false,
            );
        }

        // Los cristales comparten la transformación de Opalite.
        let opalite_matrix = planets[1].model_matrix(elapsed);

        for part in &opalite_crystals {
            draw_object(
                &part.model,
                opalite_matrix,
                &camera,
                &mut buffer,
                part.color,
                false,
            );
        }

        for part in &plaza {
            draw_object(
                &part.model,
                opalite_matrix,
                &camera,
                &mut buffer,
                part.color,
                false,
            );
        }

        // Los koi pertenecen a Lavender Haze.
        let lavender_position = planets[0].position(elapsed);
        let koi_orbit_radius = planets[0].radius + 1.0;
        let orbit_tilt = Quat::from_rotation_x(0.3);

        for index in 0..2 {
            let phase = index as f32 * std::f32::consts::PI;
            let angle = elapsed * 0.5 + phase;

            let offset = Vec3::new(
                angle.cos() * koi_orbit_radius,
                0.0,
                angle.sin() * koi_orbit_radius,
            );

            let position = lavender_position + orbit_tilt * offset;

            let orientation = orbit_tilt * Quat::from_rotation_y(-angle);

            let koi_matrix =
                Mat4::from_scale_rotation_translation(Vec3::splat(0.65), orientation, position);

            let swimming_koi = koi::animate_koi(&koi_model, elapsed, phase);

            draw_shaded_object(
                &swimming_koi,
                koi_matrix,
                &camera,
                &mut buffer,
                Material::Koi,
                elapsed,
            );
        }

        draw_object(&ship, ship_matrix, &camera, &mut buffer, 0x385794, false);

        window
            .update_with_buffer(&buffer.pixels, WIDTH, HEIGHT)
            .map_err(|error| format!("Error al mostrar la escena: {error}"))?;
    }

    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}
