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

pub fn load_obj(path: &str) -> Result<Model, String> {
    let contents = std::fs::read_to_string(path)
        .map_err(|error| format!("No se pudo leer {path}: {error}"))?;

    let mut model = Model {
        vertices: Vec::new(),
        faces: Vec::new(),
    };

    for (index, line) in contents.lines().enumerate() {
        let mut parts = line.split('#').next().unwrap_or("").split_whitespace();

        match parts.next() {
            Some("v") => {
                let mut coordinates = [0.0_f32; 3];

                for coordinate in &mut coordinates {
                    let value = parts
                        .next()
                        .ok_or_else(|| format!("Vértice incompleto en la línea {}", index + 1))?;

                    *coordinate = value
                        .parse::<f32>()
                        .map_err(|_| format!("Coordenada inválida en la línea {}", index + 1))?;

                    if !coordinate.is_finite() {
                        return Err(format!("Coordenada no finita en la línea {}", index + 1));
                    }
                }

                model.vertices.push(Vertex {
                    x: coordinates[0],
                    y: coordinates[1],
                    z: coordinates[2],
                });
            }

            Some("f") => {
                let references: Vec<&str> = parts.collect();

                if references.len() != 3 {
                    return Err(format!("Cara sin triangular en la línea {}", index + 1));
                }

                let mut face = [0_usize; 3];

                for (corner, reference) in references.iter().enumerate() {
                    let raw_index = reference
                        .split('/')
                        .next()
                        .unwrap_or("")
                        .parse::<i64>()
                        .map_err(|_| format!("Índice inválido en la línea {}", index + 1))?;

                    let count = model.vertices.len() as i64;

                    let vertex_index = if raw_index > 0 {
                        raw_index - 1
                    } else {
                        count + raw_index
                    };

                    if raw_index == 0 || vertex_index < 0 || vertex_index >= count {
                        return Err(format!("Índice fuera de rango en la línea {}", index + 1));
                    }

                    face[corner] = vertex_index as usize;
                }

                model.faces.push(face);
            }

            _ => {}
        }
    }

    Ok(model)
}
