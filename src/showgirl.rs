use glam::{Mat4, Vec3};

use crate::obj_loader::Model;
use crate::showgirl_mesh::icosphere;
use crate::showgirl_performer::{Performer, create_performer, pose};
use crate::showgirl_stage::{StagePiece, create_stage};

// Aproximadamente un 52% mayor que Opalite (radio 1.35).
pub const RECOMMENDED_RADIUS: f32 = 2.05;
// Cota conservadora del cuerpo, escenario, plumas y personaje animado.
pub const BOUND_RADIUS: f32 = 2.40;

pub struct ShowgirlWorld {
    pub surface: Model,
    pub stage: Vec<StagePiece>,
    pub performer: Performer,
}

impl ShowgirlWorld {
    pub fn new() -> Self {
        let mut surface = icosphere(2);
        // Integrar el escenario en una plataforma, en vez de apoyarlo en la cima.
        for v in &mut surface.vertices {
            v.y = v.y.min(0.60);
        }
        Self {
            surface,
            stage: create_stage(),
            performer: create_performer(),
        }
    }

    // Permite dibujar todos los accesorios con el draw_object existente.
    pub fn visit_parts<F>(&self, time: f32, planet_matrix: Mat4, mut draw: F)
    where
        F: FnMut(&Model, Mat4, u32, bool),
    {
        for part in &self.stage {
            draw(&part.model, planet_matrix, part.color_at(time), part.unlit);
        }
        let matrices = pose(time);
        for part in &self.performer.parts {
            draw(
                &part.model,
                planet_matrix * matrices[part.joint],
                part.color,
                part.unlit,
            );
        }
    }
}

pub fn surface_color(position: Vec3, time: f32) -> Vec3 {
    surface_color_filtered(position, time, 0.0)
}
pub fn surface_color_filtered(position: Vec3, time: f32, footprint: f32) -> Vec3 {
    let p = position.normalize_or_zero();
    let pattern = (p.x * 3.0 + p.y * 2.0 + (p.z * 5.0).sin() * 0.5).sin() * 0.5 + 0.5;
    let copper = Vec3::new(0.86, 0.19, 0.06);
    let orange = Vec3::new(1.0, 0.43, 0.08);
    let color = copper.lerp(orange, pattern);
    let sparkle = crate::sparkle::amount(p, time, footprint / position.length().max(0.2));
    color.lerp(Vec3::new(1.0, 0.85, 0.35), sparkle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn los_modelos_del_escenario_y_la_actuacion_caben_en_la_colision() {
        let world = ShowgirlWorld::new();
        for step in 0..24 {
            world.visit_parts(step as f32 * 0.5, Mat4::IDENTITY, |model, matrix, _, _| {
                for vertex in &model.vertices {
                    let p = matrix.transform_point3(Vec3::new(vertex.x, vertex.y, vertex.z));
                    assert!(p.is_finite() && p.length() < BOUND_RADIUS);
                }
                for face in &model.faces {
                    assert!(face.iter().all(|&i| i < model.vertices.len()));
                }
            });
        }
    }
}
