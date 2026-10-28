use glam::Vec3;

#[derive(Clone, Copy)]
pub enum Material {
    Solid { color: u32, unlit: bool },
    LavenderHaze,
}

pub struct Fragment {
    pub local_position: Vec3,
    pub normal: Vec3,
    pub time: f32,
}

pub fn shade(material: Material, fragment: &Fragment) -> u32 {
    let (base_color, unlit) = match material {
        Material::Solid { color, unlit } => (unpack_color(color), unlit),

        Material::LavenderHaze => {
            let position = fragment.local_position;

            // Deforma las bandas para que no sean líneas rectas.
            let waves = (position.x * 5.0 + position.z * 4.0 + fragment.time * 0.45).sin() * 0.45;

            // El tiempo desplaza el patrón sobre la superficie.
            let bands = (position.y * 18.0 + waves + fragment.time * 0.7).sin() * 0.5 + 0.5;

            let dark = Vec3::new(0.25, 0.12, 0.43);
            let lavender = Vec3::new(0.64, 0.43, 0.83);
            let pale = Vec3::new(0.87, 0.74, 0.96);

            let color = if bands < 0.5 {
                dark.lerp(lavender, bands * 2.0)
            } else {
                lavender.lerp(pale, (bands - 0.5) * 2.0)
            };

            (color, false)
        }
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
