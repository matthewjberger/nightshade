use nightshade_ecs::Entity;

#[derive(Debug, Clone)]
pub enum EcsCommand {
    DespawnRecursive {
        entity: Entity,
    },
    ReloadMaterial {
        name: String,
        material: Box<nightshade_render::data::material::Material>,
    },
}

#[derive(Debug, Clone)]
pub enum RenderCommand {
    LoadHdrSkybox {
        hdr_data: Vec<u8>,
    },
    LoadHdrSkyboxFromPath {
        path: std::path::PathBuf,
    },
    CaptureScreenshot {
        path: Option<std::path::PathBuf>,
        max_dimension: Option<u32>,
    },
    ReloadTexture {
        name: String,
        rgba_data: Vec<u8>,
        width: u32,
        height: u32,
    },
    SetColorLut {
        data: Vec<u8>,
    },
    RemakeRenderer,
}
