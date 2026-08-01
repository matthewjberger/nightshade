use crate::components::{HuskKind, SurfaceKind};
use crate::data::theme;
use crate::resources::WorldPhase;
use nightshade_graphics::commands::loading::load_texture_pack_from_image_bytes;
use nightshade_graphics::resources::loading::TextureLoad;
use nightshade_render::data::material::{Material, TextureTransform};
use nightshade_render::data::texture_data::{
    SamplerFilter, SamplerSettings, SamplerWrap, TextureUsage,
};

const STREET: &str = "ashfall_stone_11";
const ROOM_FLOOR: &str = "ashfall_floor_12";
const FACADE: &str = "ashfall_brick_01";
const FACADE_UPPER: &str = "ashfall_brick_11";
const CORNICE: &str = "ashfall_stone_07";
const PARTITION: &str = "ashfall_wall_03";
const CEILING: &str = "ashfall_wall_09";
const DOOR: &str = "ashfall_metal_08";
const CRATE: &str = "ashfall_metal_12";
const METAL: &str = "ashfall_metal_06";
const RUST: &str = "ashfall_metal_01";
const WOOD: &str = "ashfall_wall_04";
const CONCRETE: &str = "ashfall_floor_05";
const GRILLE: &str = "ashfall_misc_07";
const OTHER_STREET: &str = "ashfall_floor_09";
const OTHER_ROOM_FLOOR: &str = "ashfall_misc_08";
const OTHER_FACADE: &str = "ashfall_metal_13";
const OTHER_PARTITION: &str = "ashfall_misc_10";
const OTHER_CEILING: &str = "ashfall_metal_04";
const OTHER_DOOR: &str = "ashfall_metal_07";
const HUSK_BODY: &str = "ashfall_stone_02";
const HUSK_HEAD: &str = "ashfall_misc_05";
const STAIN: &str = theme::STAIN_TEXTURE;

const TEXTURES: &[(&str, &[u8])] = &[
    (STREET, include_bytes!("../../assets/textures/stone_11.png")),
    (
        ROOM_FLOOR,
        include_bytes!("../../assets/textures/floor_12.png"),
    ),
    (FACADE, include_bytes!("../../assets/textures/brick_01.png")),
    (
        FACADE_UPPER,
        include_bytes!("../../assets/textures/brick_11.png"),
    ),
    (
        CORNICE,
        include_bytes!("../../assets/textures/stone_07.png"),
    ),
    (
        PARTITION,
        include_bytes!("../../assets/textures/wall_03.png"),
    ),
    (CEILING, include_bytes!("../../assets/textures/wall_09.png")),
    (DOOR, include_bytes!("../../assets/textures/metal_08.png")),
    (CRATE, include_bytes!("../../assets/textures/metal_12.png")),
    (METAL, include_bytes!("../../assets/textures/metal_06.png")),
    (RUST, include_bytes!("../../assets/textures/metal_01.png")),
    (WOOD, include_bytes!("../../assets/textures/wall_04.png")),
    (
        CONCRETE,
        include_bytes!("../../assets/textures/floor_05.png"),
    ),
    (GRILLE, include_bytes!("../../assets/textures/misc_07.png")),
    (
        OTHER_STREET,
        include_bytes!("../../assets/textures/floor_09.png"),
    ),
    (
        OTHER_ROOM_FLOOR,
        include_bytes!("../../assets/textures/misc_08.png"),
    ),
    (
        OTHER_FACADE,
        include_bytes!("../../assets/textures/metal_13.png"),
    ),
    (
        OTHER_PARTITION,
        include_bytes!("../../assets/textures/misc_10.png"),
    ),
    (
        OTHER_CEILING,
        include_bytes!("../../assets/textures/metal_04.png"),
    ),
    (
        OTHER_DOOR,
        include_bytes!("../../assets/textures/metal_07.png"),
    ),
    (
        HUSK_BODY,
        include_bytes!("../../assets/textures/stone_02.png"),
    ),
    (
        HUSK_HEAD,
        include_bytes!("../../assets/textures/misc_05.png"),
    ),
    (STAIN, include_bytes!("../../assets/textures/stain_01.png")),
];

pub fn load_textures(load: &mut TextureLoad<'_>) {
    let sampler = SamplerSettings {
        wrap_u: SamplerWrap::Repeat,
        wrap_v: SamplerWrap::Repeat,
        mag_filter: SamplerFilter::Nearest,
        min_filter: SamplerFilter::Nearest,
        mipmap_filter: SamplerFilter::Nearest,
    };
    load_texture_pack_from_image_bytes(load, TEXTURES, TextureUsage::Color, sampler);
}

pub fn tiled(texture: &str, tiling: [f32; 2], tint: [f32; 4]) -> Material {
    Material {
        base_color: tint,
        base_texture: Some(texture.to_string()),
        base_texture_transform: TextureTransform {
            scale: tiling,
            ..Default::default()
        },
        roughness: 0.92,
        metallic: 0.0,
        ..Default::default()
    }
}

pub fn solid(color: [f32; 4], emissive: [f32; 3]) -> Material {
    Material {
        base_color: color,
        emissive_factor: emissive,
        roughness: 0.6,
        metallic: 0.0,
        ..Default::default()
    }
}

pub fn surface_name(kind: SurfaceKind) -> String {
    format!("ashfall_surface_{kind:?}")
}

pub fn surface_material(kind: SurfaceKind, phase: WorldPhase) -> Material {
    let plain = [1.0, 1.0, 1.0, 1.0];
    let rusted = [0.75, 0.6, 0.55, 1.0];
    let weathered = [0.62, 0.62, 0.64, 1.0];
    let otherworld = phase == WorldPhase::Otherworld;
    match kind {
        SurfaceKind::Street if otherworld => tiled(OTHER_STREET, [50.0, 40.0], rusted),
        SurfaceKind::Street => tiled(STREET, [50.0, 40.0], plain),
        SurfaceKind::Facade if otherworld => tiled(OTHER_FACADE, [1.0, 1.5], rusted),
        SurfaceKind::Facade => tiled(FACADE, [1.0, 1.5], weathered),
        SurfaceKind::FacadeUpper if otherworld => {
            tiled(OTHER_FACADE, [1.0, 1.5], [0.5, 0.3, 0.28, 1.0])
        }
        SurfaceKind::FacadeUpper => tiled(FACADE_UPPER, [1.0, 1.5], weathered),
        SurfaceKind::Cornice if otherworld => tiled(OTHER_CEILING, [1.0, 1.0], rusted),
        SurfaceKind::Cornice => tiled(CORNICE, [1.0, 1.0], [0.42, 0.42, 0.43, 1.0]),
        SurfaceKind::Partition if otherworld => tiled(OTHER_PARTITION, [1.0, 1.5], plain),
        SurfaceKind::Partition => tiled(PARTITION, [1.0, 1.5], plain),
        SurfaceKind::RoomFloor if otherworld => tiled(OTHER_ROOM_FLOOR, [1.0, 1.0], plain),
        SurfaceKind::RoomFloor => tiled(ROOM_FLOOR, [1.0, 1.0], plain),
        SurfaceKind::Ceiling if otherworld => tiled(OTHER_CEILING, [1.0, 1.0], rusted),
        SurfaceKind::Ceiling => tiled(CEILING, [1.0, 1.0], plain),
        SurfaceKind::Door if otherworld => tiled(OTHER_DOOR, [1.0, 1.5], rusted),
        SurfaceKind::Door => tiled(DOOR, [1.0, 1.5], plain),
        SurfaceKind::Crate if otherworld => tiled(CRATE, [1.0, 1.0], rusted),
        SurfaceKind::Crate => tiled(CRATE, [1.0, 1.0], plain),
        SurfaceKind::Metal if otherworld => tiled(RUST, [1.0, 1.0], rusted),
        SurfaceKind::Metal => tiled(METAL, [1.0, 1.0], [0.55, 0.57, 0.6, 1.0]),
        SurfaceKind::Rust => tiled(
            RUST,
            [1.0, 1.0],
            if otherworld {
                rusted
            } else {
                [0.52, 0.48, 0.46, 1.0]
            },
        ),
        SurfaceKind::Wood if otherworld => tiled(WOOD, [1.0, 1.0], [0.4, 0.24, 0.2, 1.0]),
        SurfaceKind::Wood => tiled(WOOD, [1.0, 1.0], [0.66, 0.52, 0.36, 1.0]),
        SurfaceKind::Concrete if otherworld => tiled(CONCRETE, [1.0, 1.0], rusted),
        SurfaceKind::Concrete => tiled(CONCRETE, [1.0, 1.0], [0.6, 0.6, 0.6, 1.0]),
        SurfaceKind::Grille => tiled(GRILLE, [1.0, 1.0], [0.42, 0.4, 0.38, 1.0]),
        SurfaceKind::Glass if otherworld => solid([0.05, 0.01, 0.01, 1.0], [0.02, 0.0, 0.0]),
        SurfaceKind::Glass => solid([0.03, 0.035, 0.045, 1.0], [0.0, 0.0, 0.0]),
        SurfaceKind::GlassLit if otherworld => solid([0.2, 0.02, 0.0, 1.0], theme::WINDOW_RED),
        SurfaceKind::GlassLit => solid([0.5, 0.4, 0.25, 1.0], theme::WINDOW_GLOW),
    }
}

pub fn husk_body_material(kind: HuskKind, otherworld: bool) -> (String, Material) {
    let tint = if otherworld {
        theme::OTHERWORLD_HUSK_TINT
    } else {
        match kind {
            HuskKind::Wanderer => theme::HUSK_TINT,
            HuskKind::Crawler => theme::CRAWLER_TINT,
            HuskKind::Wretch => theme::WRETCH_TINT,
            HuskKind::Screamer => theme::SCREAMER_TINT,
        }
    };
    (
        format!("ashfall_husk_body_{kind:?}_{otherworld}"),
        tiled(HUSK_BODY, [1.0, 1.0], tint),
    )
}

pub fn person_body_material() -> (String, Material) {
    (
        "ashfall_person_body".to_string(),
        tiled(HUSK_BODY, [1.0, 1.0], theme::PERSON_COAT),
    )
}

pub fn person_head_material() -> (String, Material) {
    (
        "ashfall_person_head".to_string(),
        tiled(HUSK_HEAD, [1.0, 1.0], theme::PERSON_SKIN),
    )
}

pub fn husk_head_material(otherworld: bool) -> (String, Material) {
    let tint = if otherworld {
        [0.35, 0.08, 0.05, 1.0]
    } else {
        [0.6, 0.5, 0.45, 1.0]
    };
    (
        format!("ashfall_husk_head_{otherworld}"),
        tiled(HUSK_HEAD, [1.0, 1.0], tint),
    )
}
