pub fn sample_model(name: &str) -> std::path::PathBuf {
    sample_model_root()
        .join("Models")
        .join(name)
        .join("glTF-Binary")
        .join(format!("{name}.glb"))
}

pub fn sample_model_parts(name: &str) -> std::path::PathBuf {
    sample_model_root()
        .join("Models")
        .join(name)
        .join("glTF")
        .join(format!("{name}.gltf"))
}

pub fn sample_model_url(name: &str) -> String {
    format!(
        "https://raw.githubusercontent.com/KhronosGroup/glTF-Sample-Assets/main/Models/{name}/glTF-Binary/{name}.glb"
    )
}

pub fn sample_model_url_compressed(name: &str) -> String {
    format!(
        "https://raw.githubusercontent.com/KhronosGroup/glTF-Sample-Assets/main/Models/{name}/glTF-KTX-BasisU/{name}.gltf"
    )
}

fn sample_model_root() -> std::path::PathBuf {
    std::path::PathBuf::from(
        std::env::var("GLTF_SAMPLE_ASSETS").unwrap_or_else(|_| "../glTF-Sample-Assets".to_string()),
    )
}
