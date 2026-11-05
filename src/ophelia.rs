use glam::Vec3;

pub fn water_color(position: Vec3, time: f32) -> Vec3 {
    let p = position.normalize_or_zero();

    // Variaciones amplias de profundidad aparente.
    let depth_pattern = (p.x * 3.5 + p.z * 2.8 + (p.y * 4.0).sin() * 0.7).sin() * 0.5 + 0.5;

    let deep = Vec3::new(0.02, 0.42, 0.45);
    let turquoise = Vec3::new(0.03, 0.85, 0.77);

    let mut color = deep.lerp(turquoise, depth_pattern);

    // Deformación del patrón para evitar líneas rectas.
    let drift = time * 0.35;

    let warped = p * 13.0
        + Vec3::new(
            (p.z * 7.0 + drift).sin(),
            (p.x * 6.0 - drift * 0.8).sin(),
            (p.y * 8.0 + drift * 0.6).sin(),
        ) * 0.7;

    // Dos familias de curvas simulan reflejos sobre el agua.
    let first = (warped.x + warped.y * 0.65 + (warped.z * 0.7).sin() + drift)
        .sin()
        .abs();

    let second = (warped.z - warped.y * 0.8 + (warped.x * 0.8).sin() - drift * 0.7)
        .sin()
        .abs();

    let thin_lines = 1.0 - smooth_transition(0.025, 0.14, first.min(second));

    let reflection = Vec3::new(0.64, 0.96, 0.87);

    color = color.lerp(reflection, thin_lines * 0.65);

    // Ondas suaves adicionales.
    let ripple = (p.y * 24.0 + p.x * 8.0 + (p.z * 5.0).sin() - time * 1.1).sin() * 0.5 + 0.5;

    color * (0.92 + ripple * 0.08)
}

fn smooth_transition(start: f32, end: f32, value: f32) -> f32 {
    let t = ((value - start) / (end - start)).clamp(0.0, 1.0);

    t * t * (3.0 - 2.0 * t)
}
