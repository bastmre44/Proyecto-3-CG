use glam::Vec3;

#[derive(Clone, Copy)]
pub enum Material {
    Solid { color: u32, unlit: bool },
    LavenderHaze,
    Koi,
}

pub struct Fragment {
    pub local_position: Vec3,
    pub normal: Vec3,
    pub time: f32,
}

pub fn shade(material: Material, fragment: &Fragment) -> u32 {
    let (base_color, unlit) = match material {
        Material::Solid { color, unlit } => (unpack_color(color), unlit),

        Material::LavenderHaze => (lavender_haze(fragment.local_position, fragment.time), false),
        Material::Koi => (koi_color(fragment.local_position), false),
    };

    let intensity = if unlit {
        1.0
    } else {
        let light_direction = Vec3::new(-0.3, 0.8, 0.5).normalize();

        let normal = fragment.normal.normalize_or_zero();

        0.35 + 0.65 * normal.dot(light_direction).clamp(0.0, 1.0)
    };

    pack_color(base_color * intensity)
}

fn lavender_haze(position: Vec3, time: f32) -> Vec3 {
    let drift = time * 0.18;

    // Estas ondas deforman las coordenadas del patrón.
    let warp = Vec3::new(
        (position.y * 3.4 + position.z * 2.1 + drift).sin(),
        (position.z * 3.1 - position.x * 2.7 - drift * 0.8).sin(),
        (position.x * 3.6 + position.y * 2.3 + drift * 0.6).sin(),
    );

    let cloud_position = position * 3.0 + warp * 0.85;

    // Combinamos ondas en varias direcciones para formar
    // manchas suaves en lugar de franjas horizontales.
    let large_clouds =
        (cloud_position.x + cloud_position.y * 0.8 - cloud_position.z * 0.6 + drift).sin();

    let medium_clouds =
        (cloud_position.y * 2.1 + cloud_position.z * 1.7 + (cloud_position.x * 1.3).sin()
            - drift * 0.7)
            .sin();

    let small_clouds =
        (cloud_position.z * 4.2 - cloud_position.x * 3.0 + cloud_position.y * 2.4 + drift * 1.2)
            .sin();

    let density =
        (0.5 + large_clouds * 0.26 + medium_clouds * 0.16 + small_clouds * 0.08).clamp(0.0, 1.0);

    let midnight = Vec3::new(0.10, 0.13, 0.30);
    let violet = Vec3::new(0.40, 0.25, 0.62);
    let lavender = Vec3::new(0.70, 0.53, 0.85);
    let pink = Vec3::new(0.93, 0.67, 0.82);

    if density < 0.35 {
        midnight.lerp(violet, smooth_transition(0.0, 0.35, density))
    } else if density < 0.65 {
        violet.lerp(lavender, smooth_transition(0.35, 0.65, density))
    } else {
        lavender.lerp(pink, smooth_transition(0.65, 1.0, density))
    }
}

fn smooth_transition(start: f32, end: f32, value: f32) -> f32 {
    let t = ((value - start) / (end - start)).clamp(0.0, 1.0);

    t * t * (3.0 - 2.0 * t)
}

fn unpack_color(color: u32) -> Vec3 {
    Vec3::new(
        ((color >> 16) & 255) as f32,
        ((color >> 8) & 255) as f32,
        (color & 255) as f32,
    ) / 255.0
}

fn pack_color(color: Vec3) -> u32 {
    let color = color.clamp(Vec3::ZERO, Vec3::ONE) * 255.0;

    let red = color.x.round() as u32;
    let green = color.y.round() as u32;
    let blue = color.z.round() as u32;

    (red << 16) | (green << 8) | blue
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn un_material_sin_iluminacion_conserva_su_color() {
        let fragment = Fragment {
            local_position: Vec3::ZERO,
            normal: Vec3::ZERO,
            time: 0.0,
        };

        let color = shade(
            Material::Solid {
                color: 0xFFD17A,
                unlit: true,
            },
            &fragment,
        );

        assert_eq!(color, 0xFFD17A);
    }

    #[test]
    fn lavender_haze_cambia_con_el_tiempo() {
        let mut fragment = Fragment {
            local_position: Vec3::Y,
            normal: Vec3::Y,
            time: 0.0,
        };

        let initial = shade(Material::LavenderHaze, &fragment);

        fragment.time = 1.0;

        let animated = shade(Material::LavenderHaze, &fragment);

        assert_ne!(initial, animated);
    }
}

fn koi_color(position: Vec3) -> Vec3 {
    let white = Vec3::new(1.0, 0.88, 0.91);
    let pink = Vec3::new(0.92, 0.36, 0.56);
    let dark = Vec3::new(0.12, 0.12, 0.23);

    // Un ojo a cada lado de la cabeza.
    let eye_offset = Vec3::new(
        (position.x.abs() - 0.145) / 0.045,
        (position.y - 0.07) / 0.05,
        (position.z - 0.42) / 0.065,
    );

    if eye_offset.length_squared() < 1.0 {
        return Vec3::new(0.025, 0.02, 0.045);
    }

    // Aletas y cola rosadas.
    if position.z < -0.64 || position.x.abs() > 0.24 || position.y > 0.28 {
        return white.lerp(pink, 0.55);
    }

    // Manchas fijas sobre el cuerpo.
    let pink_pattern =
        (position.z * 13.0 + (position.y * 16.0).sin() * 1.4 + position.x * 9.0).sin();

    let dark_pattern = (position.z * 19.0 - position.y * 12.0 + (position.x * 17.0).sin()).sin();

    if dark_pattern > 0.72 && pink_pattern > 0.1 {
        dark
    } else if pink_pattern > 0.05 {
        pink
    } else {
        white
    }
}
