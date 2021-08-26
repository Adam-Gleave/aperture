use glsl_to_spirv::ShaderType;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=../data/shaders");
    std::fs::create_dir_all("../data/shaders/gen")?;

    for entry in std::fs::read_dir("../data/shaders")? {
        let entry = entry?;

        if entry.file_type()?.is_file() {
            let path = entry.path();

            let shader_type =
                path.extension()
                    .and_then(|ext| match ext.to_string_lossy().as_ref() {
                        "vert" => Some(ShaderType::Vertex),
                        "frag" => Some(ShaderType::Fragment),
                        _ => None,
                    });

            if let Some(shader_type) = shader_type {
                use std::io::Read;

                let source = std::fs::read_to_string(&path)?;
                let mut compiled = glsl_to_spirv::compile(&source, shader_type)?;

                println!("Compiling shader {}", path.to_string_lossy().as_ref());

                let mut bytes = vec![];
                compiled.read_to_end(&mut bytes)?;

                let out_path = format!(
                    "../data/shaders/gen/{}.spv",
                    path.file_name().unwrap().to_string_lossy()
                );

                std::fs::write(&out_path, &bytes)?;
            }
        }
    }

    Ok(())
}
