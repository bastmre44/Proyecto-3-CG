use std::{collections::BTreeMap, path::Path};
#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
pub struct Model {
    pub vertices: Vec<Vertex>,
    pub faces: Vec<[usize; 3]>,
}
pub struct MaterialBatch {
    pub model: Model,
    pub color: u32,
}
pub struct ObjAsset {
    pub model: Model,
    pub batches: Vec<MaterialBatch>,
}
pub fn load_obj(path: &str) -> Result<Model, String> {
    Ok(load_obj_materials(path)?.model)
}
pub fn load_obj_materials(path: &str) -> Result<ObjAsset, String> {
    let contents =
        std::fs::read_to_string(path).map_err(|e| format!("No se pudo leer {path}: {e}"))?;
    let directory = Path::new(path).parent().unwrap_or(Path::new("."));
    parse_obj(&contents, |name| {
        let p = directory.join(name);
        std::fs::read_to_string(&p).map_err(|e| format!("No se pudo leer {}: {e}", p.display()))
    })
}
fn parse_obj(
    contents: &str,
    mut library: impl FnMut(&str) -> Result<String, String>,
) -> Result<ObjAsset, String> {
    let mut model = Model {
        vertices: Vec::new(),
        faces: Vec::new(),
    };
    let mut materials = BTreeMap::new();
    let mut names = Vec::new();
    let mut active = None;
    for (line_number, line) in contents.lines().enumerate() {
        let mut words = line.split('#').next().unwrap_or("").split_whitespace();
        match words.next() {
            Some("mtllib") => {
                for name in words {
                    materials.extend(parse_mtl(&library(name)?)?);
                }
            }
            Some("usemtl") => active = Some(words.collect::<Vec<_>>().join(" ")),
            Some("v") => {
                let mut v = [0.0f32; 3];
                for c in &mut v {
                    *c = words
                        .next()
                        .ok_or("Vertice incompleto")?
                        .parse()
                        .map_err(|_| "Coordenada invalida")?;
                    if !c.is_finite() {
                        return Err("Coordenada no finita".into());
                    }
                }
                model.vertices.push(Vertex {
                    x: v[0],
                    y: v[1],
                    z: v[2],
                });
            }
            Some("f") => {
                let refs = words.collect::<Vec<_>>();
                if refs.len() != 3 {
                    return Err(format!("Cara sin triangular en linea {}", line_number + 1));
                }
                let mut face = [0; 3];
                for (corner, r) in refs.iter().enumerate() {
                    let raw = r
                        .split('/')
                        .next()
                        .unwrap_or("")
                        .parse::<i64>()
                        .map_err(|_| "Indice invalido")?;
                    let count = model.vertices.len() as i64;
                    let i = if raw > 0 { raw - 1 } else { count + raw };
                    if raw == 0 || i < 0 || i >= count {
                        return Err("Indice fuera de rango".into());
                    }
                    face[corner] = i as usize;
                }
                model.faces.push(face);
                names.push(active.clone());
            }
            _ => {}
        }
    }
    let mut groups: BTreeMap<u32, Vec<[usize; 3]>> = BTreeMap::new();
    for (face, name) in model.faces.iter().zip(names) {
        let color = match name {
            Some(name) => *materials
                .get(&name)
                .ok_or_else(|| format!("Material sin Kd o no definido: {name}"))?,
            None => 0xFFFFFF,
        };
        groups.entry(color).or_default().push(*face);
    }
    let batches = groups
        .into_iter()
        .map(|(color, faces)| MaterialBatch {
            model: Model {
                vertices: model.vertices.clone(),
                faces,
            },
            color,
        })
        .collect();
    Ok(ObjAsset { model, batches })
}
fn linear_to_srgb(v: f32) -> u32 {
    let v = v.clamp(0.0, 1.0);
    let s = if v <= 0.0031308 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    };
    (s * 255.0).round() as u32
}
fn parse_mtl(contents: &str) -> Result<BTreeMap<String, u32>, String> {
    let mut materials = BTreeMap::new();
    let mut active = None;
    for line in contents.lines() {
        let mut words = line.split('#').next().unwrap_or("").split_whitespace();
        match words.next() {
            Some("newmtl") => active = Some(words.collect::<Vec<_>>().join(" ")),
            Some("Kd") => {
                let name = active.as_ref().ok_or("Kd sin newmtl")?;
                let mut c = [0; 3];
                for channel in &mut c {
                    let v: f32 = words
                        .next()
                        .ok_or("Kd incompleto")?
                        .parse()
                        .map_err(|_| "Kd invalido")?;
                    if !v.is_finite() {
                        return Err("Kd no finito".into());
                    }
                    *channel = linear_to_srgb(v);
                }
                materials.insert(name.clone(), (c[0] << 16) | (c[1] << 8) | c[2]);
            }
            _ => {}
        }
    }
    Ok(materials)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn varias_caras_y_materiales() {
        let obj = "mtllib colors.mtl\nv 0 0 0\nv 1 0 0\nv 0 1 0\nv 0 0 1\nusemtl Material\nf 1/1 2/2 3/3\nusemtl lavanda\nf -4 -2 -1\nusemtl Material\nf 2 3 4";
        let a=parse_obj(obj,|name|{assert_eq!(name,"colors.mtl");Ok("newmtl Material\nKd 0.008568 0.019382 0.074214\nnewmtl lavanda\nKd 0.467784 0.366253 0.745404".into())}).unwrap();
        assert_eq!(a.model.faces, vec![[0, 1, 2], [0, 2, 3], [1, 2, 3]]);
        assert_eq!(a.batches[0].color, 0x17264D);
        assert_eq!(a.batches[0].model.faces, vec![[0, 1, 2], [1, 2, 3]]);
        assert_eq!(a.batches[1].color, 0xB6A3E0);
        assert_eq!(a.batches[1].model.faces, vec![[0, 2, 3]]);
    }
    #[test]
    fn rechaza_material_ausente_e_indices_invalidos() {
        assert!(parse_mtl("newmtl x\nKd NaN 0 0").is_err());
        assert!(
            parse_obj("v 0 0 0\nv 1 0 0\nv 0 1 0\nusemtl falta\nf 1 2 3", |_| Ok(
                String::new()
            ))
            .is_err()
        );
        for face in ["0 1 2", "1 2 4", "-4 -2 -1"] {
            assert!(
                parse_obj(&format!("v 0 0 0\nv 1 0 0\nv 0 1 0\nf {face}"), |_| Ok(
                    String::new()
                ))
                .is_err()
            );
        }
    }
    #[test]
    fn mtl_relativo_al_obj_real() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("Nave_final.obj");
        let a = load_obj_materials(path.to_str().unwrap()).unwrap();
        assert_eq!(
            a.model.faces.len(),
            a.batches.iter().map(|b| b.model.faces.len()).sum::<usize>()
        );
        assert!(a.batches.iter().any(|b| b.color == 0x17264D));
        assert!(a.batches.iter().any(|b| b.color == 0xB6A3E0));
    }
}
