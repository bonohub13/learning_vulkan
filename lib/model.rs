pub fn load_model(
    model_path: &std::path::Path,
) -> Result<(Vec<crate::types::VertexWithTexture3D>, Vec<u32>), String> {
    let load_options = tobj::LoadOptions::default();
    let model_obj = tobj::load_obj(model_path, &load_options);

    if model_obj.is_err() {
        return Err(String::from("failed to load model object!"));
    }

    let model_obj = model_obj.unwrap();
    let mut vertices = vec![];
    let mut indices = vec![];

    for model in model_obj.0.iter() {
        let mesh = &model.mesh;

        if mesh.texcoords.len() == 0 {
            return Err(String::from("Missing texture coordinate for the model!"));
        }

        let total_vertices_count = mesh.positions.len() / 3;

        for i in 0..total_vertices_count {
            let vertex = crate::types::VertexWithTexture3D::new(
                [
                    mesh.positions[i * 3],
                    mesh.positions[i * 3 + 1],
                    mesh.positions[i * 3 + 2],
                    1.0,
                ],
                [1.0, 1.0, 1.0],
                [mesh.texcoords[i * 2], mesh.texcoords[i * 2 + 1]],
            );

            vertices.push(vertex);
        }

        indices = mesh.indices.clone();
    }

    Ok((vertices, indices))
}
