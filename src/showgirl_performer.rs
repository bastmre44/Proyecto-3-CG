use glam::{Mat4, Quat, Vec3};

use crate::obj_loader::Model;
use crate::showgirl_mesh::{Mesh, icosphere, translation};

pub const JOINT_COUNT: usize = 9;

pub struct PerformerPart {
    pub model: Model,
    pub color: u32,
    pub unlit: bool,
    pub joint: usize,
}

pub struct Performer {
    pub parts: Vec<PerformerPart>,
}

pub fn create_performer() -> Performer {
    let mut parts = Vec::new();
    let mut skin = Mesh::new();
    let mut boots = Mesh::new();
    let mut costume = Mesh::new();
    let mut trim = Mesh::new();
    for side in [-1.0, 1.0] {
        segment(
            &mut skin,
            Vec3::new(side * 0.028, 0.205, 0.0),
            Vec3::new(side * 0.036, 0.105, 0.005),
            0.028,
        );
        segment(
            &mut boots,
            Vec3::new(side * 0.036, 0.112, 0.005),
            Vec3::new(side * 0.042, 0.018, 0.0),
            0.036,
        );
        boots.cuboid(
            translation(side * 0.042, 0.010, 0.018),
            Vec3::new(0.044, 0.020, 0.065),
        );
    }
    skin.cylinder(Mat4::IDENTITY, 0.017, 0.345, 0.394, 16);
    // Torso del vestuario: cintura estrecha y hombros más anchos.
    let torso = [
        Vec3::new(-0.035, 0.215, -0.023),
        Vec3::new(0.035, 0.215, -0.023),
        Vec3::new(0.056, 0.345, -0.026),
        Vec3::new(-0.056, 0.345, -0.026),
        Vec3::new(-0.035, 0.215, 0.023),
        Vec3::new(0.035, 0.215, 0.023),
        Vec3::new(0.056, 0.345, 0.026),
        Vec3::new(-0.056, 0.345, 0.026),
    ];
    for face in [
        [0, 2, 1],
        [0, 3, 2],
        [4, 5, 6],
        [4, 6, 7],
        [0, 4, 7],
        [0, 7, 3],
        [1, 2, 6],
        [1, 6, 5],
        [0, 1, 5],
        [0, 5, 4],
        [3, 7, 6],
        [3, 6, 2],
    ] {
        costume.triangle(Mat4::IDENTITY, face.map(|i| torso[i]));
    }
    costume.cuboid(translation(0.0, 0.215, 0.0), Vec3::new(0.074, 0.035, 0.047));
    trim.cuboid(
        translation(0.0, 0.245, 0.025),
        Vec3::new(0.075, 0.009, 0.006),
    );
    for side in [-1.0, 1.0] {
        trim.cuboid(
            translation(side * 0.023, 0.32, 0.029) * Mat4::from_rotation_z(side * 0.5),
            Vec3::new(0.009, 0.055, 0.006),
        );
    }
    push(&mut parts, skin, 0xFFD0AE, false, 0);
    push(&mut parts, boots, 0xE86520, false, 0);
    push(&mut parts, costume, 0xFF7D27, false, 0);
    push(&mut parts, trim, 0xFFD76E, false, 0);

    // Cabeza y cabello, ligados a un pequeño giro independiente.
    let ball = icosphere(2);
    let mut head = Mesh::new();
    head.append(
        &ball,
        translation(0.0, 0.042, 0.0) * Mat4::from_scale(Vec3::new(0.046, 0.057, 0.041)),
    );
    push(&mut parts, head, 0xFFD0AE, false, 1);
    let mut hair = Mesh::new();
    for face in &ball.faces {
        let points = face.map(|i| {
            let v = ball.vertices[i];
            Vec3::new(v.x, v.y, v.z)
        });
        let center = (points[0] + points[1] + points[2]) / 3.0;
        if center.z < 0.05 || center.y > 0.45 {
            hair.triangle(
                translation(0.0, 0.047, -0.005) * Mat4::from_scale(Vec3::new(0.052, 0.062, 0.046)),
                points,
            );
        }
    }
    hair.cuboid(
        translation(0.0, 0.074, 0.034),
        Vec3::new(0.084, 0.022, 0.014),
    );
    for side in [-1.0, 1.0] {
        hair.cuboid(
            translation(side * 0.040, 0.005, -0.024),
            Vec3::new(0.021, 0.09, 0.035),
        );
    }
    push(&mut parts, hair, 0xEAB957, false, 1);
    let mut eyes = Mesh::new();
    for side in [-1.0, 1.0] {
        eyes.cuboid(
            translation(side * 0.015, 0.045, 0.040),
            Vec3::new(0.007, 0.006, 0.004),
        );
    }
    push(&mut parts, eyes, 0x282336, true, 1);
    let mut lips = Mesh::new();
    lips.cuboid(
        translation(0.0, 0.022, 0.041),
        Vec3::new(0.013, 0.005, 0.004),
    );
    push(&mut parts, lips, 0xCB3451, false, 1);

    let mut headdress = Mesh::new();
    headdress.cylinder(translation(0.0, 0.098, 0.0), 0.045, 0.0, 0.012, 16);
    for i in 0..3 {
        let angle = (i as f32 - 1.0) * 0.48;
        headdress.feather(
            translation(0.0, 0.103, -0.008) * Mat4::from_rotation_z(angle),
            0.085,
            0.020,
        );
    }
    push(&mut parts, headdress, 0xFF9E32, false, 1);

    // Brazos: cilindros unitarios, transformados según sus dos extremos.
    for joint in [2, 3, 6, 7] {
        let mut arm = Mesh::new();
        arm.cylinder(Mat4::IDENTITY, 0.5, -0.5, 0.5, 16);
        push(&mut parts, arm, 0xFFD0AE, false, joint);
    }
    for joint in [4, 8] {
        parts.push(PerformerPart {
            model: icosphere(1),
            color: 0xFFD0AE,
            unlit: false,
            joint,
        });
    }

    let mut microphone = Mesh::new();
    microphone.cylinder(Mat4::IDENTITY, 0.0045, 0.0, 0.05, 16);
    push(&mut parts, microphone, 0x353949, false, 5);
    let mut microphone_head = Mesh::new();
    microphone_head.append(
        &icosphere(1),
        translation(0.0, 0.055, 0.0) * Mat4::from_scale(Vec3::splat(0.011)),
    );
    push(&mut parts, microphone_head, 0xB9C1D4, false, 5);
    Performer { parts }
}

pub fn pose(time: f32) -> [Mat4; JOINT_COUNT] {
    let time = if time.is_finite() { time } else { 0.0 };
    let root = translation((time * 1.3).sin() * 0.007, 0.681, 0.25)
        * Mat4::from_rotation_y((time * 0.7).sin() * 0.10);
    let left_shoulder = Vec3::new(-0.062, 0.335, 0.0);
    let left_elbow = Vec3::new(-0.115, 0.360 + (time * 1.6).sin() * 0.005, 0.012);
    let left_hand = Vec3::new(-0.064, 0.405 + (time * 1.6).cos() * 0.004, 0.055);
    let right_shoulder = Vec3::new(0.062, 0.335, 0.0);
    let right_elbow = Vec3::new(0.115, 0.395 + (time * 1.8).sin() * 0.006, 0.0);
    let right_hand = Vec3::new(
        0.17 + (time * 2.3).sin() * 0.025,
        0.47 + (time * 2.3).cos() * 0.015,
        0.005,
    );
    let mouth = Vec3::new(0.0, 0.427, 0.043);
    let microphone_direction = mouth - left_hand;
    let microphone = root
        * Mat4::from_translation(left_hand)
        * Mat4::from_quat(Quat::from_rotation_arc(
            Vec3::Y,
            microphone_direction.normalize(),
        ))
        * Mat4::from_scale(Vec3::new(1.0, microphone_direction.length() / 0.066, 1.0));
    [
        root,
        root * translation(0.0, 0.385, 0.0) * Mat4::from_rotation_y((time * 0.9).sin() * 0.15),
        root * bone_matrix(left_shoulder, left_elbow, 0.024),
        root * bone_matrix(left_elbow, left_hand, 0.021),
        root * Mat4::from_translation(left_hand) * Mat4::from_scale(Vec3::splat(0.013)),
        microphone,
        root * bone_matrix(right_shoulder, right_elbow, 0.024),
        root * bone_matrix(right_elbow, right_hand, 0.021),
        root * Mat4::from_translation(right_hand) * Mat4::from_scale(Vec3::splat(0.013)),
    ]
}

fn bone_matrix(start: Vec3, end: Vec3, width: f32) -> Mat4 {
    let direction = end - start;
    Mat4::from_scale_rotation_translation(
        Vec3::new(width, direction.length(), width),
        Quat::from_rotation_arc(Vec3::Y, direction.normalize()),
        (start + end) * 0.5,
    )
}

fn segment(mesh: &mut Mesh, start: Vec3, end: Vec3, width: f32) {
    mesh.cylinder(bone_matrix(start, end, width), 0.5, -0.5, 0.5, 16);
}

fn push(parts: &mut Vec<PerformerPart>, mesh: Mesh, color: u32, unlit: bool, joint: usize) {
    parts.push(PerformerPart {
        model: mesh.model,
        color,
        unlit,
        joint,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn el_codo_y_el_microfono_permanecen_unidos_durante_la_animacion() {
        for step in 0..120 {
            let matrices = pose(step as f32 / 10.0);
            let elbow_a = matrices[2].transform_point3(Vec3::Y * 0.5);
            let elbow_b = matrices[3].transform_point3(Vec3::Y * -0.5);
            assert!(elbow_a.distance(elbow_b) < 0.00001);
            let hand = matrices[4].transform_point3(Vec3::ZERO);
            let microphone = matrices[5].transform_point3(Vec3::ZERO);
            assert!(hand.distance(microphone) < 0.00001);
        }
    }
}
