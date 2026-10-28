mod camera;
mod collision;
mod flight;
mod framebuffer;
mod koi;
mod local_orbit;
mod obj_loader;
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

const WIDTH: usize = 1000;
const HEIGHT: usize = 750;
const BACKGROUND: u32 = 0x101522;

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
    material: shader::Material,
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

fn orbital_position(time: f32, radius: f32) -> Vec3 {
    let angle = time * 0.35;

    Vec3::new(angle.cos() * radius, 0.0, angle.sin() * radius)
}

fn run() -> Result<(), String> {
    let ship = obj_loader::load_obj("Nave_final.obj")?;
    let sphere = sphere::create_sphere(12, 24)?;
    let koi_model = koi::create_koi()?;

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
    let planet_radius = 1.1_f32;

    // Esfera que contiene toda la geometría de la nave.
    let ship_radius = ship
        .vertices
        .iter()
        .map(|vertex| Vec3::new(vertex.x, vertex.y, vertex.z).distance(center))
        .fold(0.0_f32, f32::max)
        * ship_scale;

    let orbit_radius = (sun_radius + planet_radius + 2.0 * ship_radius + 0.5).max(5.5);

    let local_min_distance = planet_radius + ship_radius + 0.01;
    let local_activation_distance = local_min_distance + 4.0;

    let ship_start = Vec3::new(0.0, 0.4, 7.5);

    let mut flight = FlightController::new(ship_start);
    let mut local_orbit: Option<LocalOrbit> = None;

    let mut camera = Camera::new(
        flight.position - flight.forward() * 6.0 + Vec3::Y * 2.5,
        flight.position + flight.forward() * 2.0,
    );

    let sun_matrix = Mat4::from_scale(Vec3::splat(sun_radius));
    let mut buffer = Framebuffer::new(WIDTH, HEIGHT);

    let mut window = Window::new(
        "Midnights - Vuelo libre",
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

        previous = now;

        let planet_position = orbital_position(elapsed, orbit_radius);

        let previous_planet_position =
            orbital_position((elapsed - delta_time).max(0.0), orbit_radius);

        // Reiniciar la nave y la cámara.
        if window.is_key_pressed(Key::R, KeyRepeat::No) {
            flight = FlightController::new(ship_start);
            local_orbit = None;

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

        // Entrar al modo local únicamente cerca del planeta.
        if window.is_active() && window.is_key_pressed(Key::C, KeyRepeat::No) {
            if local_orbit.is_some() {
                local_orbit = None;

                let position = flight.position;
                let forward = flight.forward();

                flight.set_pose(position, forward);
            } else if flight.position.distance(planet_position) <= local_activation_distance {
                local_orbit = Some(LocalOrbit::new(
                    flight.position,
                    planet_position,
                    local_min_distance,
                ));
            }
        }

        let previous_ship_position = flight.position;

        if let Some(orbit) = &mut local_orbit {
            orbit.update(&input, delta_time);

            flight.set_pose(orbit.position(planet_position), orbit.forward());
        } else {
            flight.update(&input, delta_time);
        }

        let bodies = [
            (Vec3::ZERO, Vec3::ZERO, sun_radius),
            (previous_planet_position, planet_position, planet_radius),
        ];

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

        if local_orbit.is_some() {
            let forward = planet_position - flight.position;

            flight.set_pose(flight.position, forward);

            // Actualizar el modo local si una colisión movió la nave.
            if collided {
                local_orbit = Some(LocalOrbit::new(
                    flight.position,
                    planet_position,
                    local_min_distance,
                ));
            }
        }

        let forward = flight.forward();
        let up = flight.orientation() * Vec3::Y;
        let right = forward.cross(up).normalize_or_zero();

        let (desired_position, desired_target) = if local_orbit.is_some() {
            // Cámara lateral para observar el planeta.
            (
                flight.position - forward * 4.0 + up * 2.0 + right * 3.0,
                planet_position,
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

        // Evitar que la cámara termine dentro del sol o planeta.
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

        let distance = flight.position.distance(planet_position);

        let mode = if local_orbit.is_some() {
            "LOCAL | C: volver a vuelo libre"
        } else if distance <= local_activation_distance {
            "LIBRE | C: activar modo local"
        } else {
            "LIBRE | Acercate al planeta para activar C"
        };

        window.set_title(&format!(
            "Midnights | {mode} | Distancia: {distance:.1} | R: reiniciar"
        ));

        let ship_matrix = Mat4::from_scale_rotation_translation(
            Vec3::splat(ship_scale),
            flight.orientation(),
            flight.position,
        ) * Mat4::from_translation(-center);

        let planet_matrix = Mat4::from_translation(planet_position)
            * Mat4::from_rotation_y(elapsed * 0.8)
            * Mat4::from_scale(Vec3::splat(planet_radius));

        buffer.clear(BACKGROUND);

        // Sol.
        draw_object(&sphere, sun_matrix, &camera, &mut buffer, 0xFFD17A, true);

        // Lavender Haze.
        draw_shaded_object(
            &sphere,
            planet_matrix,
            &camera,
            &mut buffer,
            shader::Material::LavenderHaze,
            elapsed,
        );

        // Dos koi nadando alrededor del planeta.
        let koi_orbit_radius = planet_radius + 1.0;
        let orbit_tilt = Quat::from_rotation_x(0.3);

        for index in 0..2 {
            let phase = index as f32 * std::f32::consts::PI;
            let angle = elapsed * 0.5 + phase;

            let offset = Vec3::new(
                angle.cos() * koi_orbit_radius,
                0.0,
                angle.sin() * koi_orbit_radius,
            );

            let position = planet_position + orbit_tilt * offset;

            // Orientación tangente al recorrido.
            let orientation = orbit_tilt * Quat::from_rotation_y(-angle);

            let koi_matrix =
                Mat4::from_scale_rotation_translation(Vec3::splat(0.65), orientation, position);

            let swimming_koi = koi::animate_koi(&koi_model, elapsed, phase);

            draw_shaded_object(
                &swimming_koi,
                koi_matrix,
                &camera,
                &mut buffer,
                shader::Material::Koi,
                elapsed,
            );
        }

        // Nave.
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
