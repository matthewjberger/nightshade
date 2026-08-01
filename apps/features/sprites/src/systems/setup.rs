use crate::components::Spin;
use crate::resources::Farm;
use crate::systems::farm;
use crate::theme;
use nightshade::prelude::*;
use nightshade_camera::prelude::*;
use nightshade_graphics::commands::builtins::spawn_mesh_at;
use nightshade_graphics::prelude::*;
use nightshade_render::commands::material_registry::insert_material;
use nightshade_render::components::material::MaterialRef;
use nightshade_render::data::decoded_image::decode_to_rgba8;
use nightshade_render::data::material::Material;
use nightshade_sprite::prelude::*;

pub fn initialize(
    mut settings: ResMut<RenderSettings>,
    mut registry: ResMut<MaterialRegistry>,
    mut atlases: ResMut<SpriteAtlases>,
    mut active_camera: ResMut<ActiveCamera>,
    mut farm_handles: ResMut<Farm>,
    mut commands: Commands,
) {
    settings.atmosphere = Atmosphere::None;
    spawn_sun_without_shadows(&mut commands);

    let sheet = pack_the_sheet(&mut atlases);
    farm_handles.farmer = Some(farm::build(&mut commands, &sheet));
    farm_handles.camera = Some(nightshade_camera::commands::scene::spawn_flat_camera(
        &mut commands,
        "Flat Camera".to_string(),
        Vec3::new(0.0, 0.0, 20.0),
        8.0,
        0.1,
        100.0,
    ));
    active_camera.0 = farm_handles.camera;
    spawn_buried_cube(&mut registry, &mut commands);
}

pub struct Sheet {
    pub tiles: SpriteRegion,
    pub grass: SpriteRegion,
}

impl Sheet {
    pub fn tile(&self, index: u32) -> SpriteRegion {
        self.tiles
            .cell(theme::SHEET_COLUMNS, theme::SHEET_ROWS, index)
    }
}

fn pack_the_sheet(atlases: &mut SpriteAtlases) -> Sheet {
    sprite_atlas_create(
        atlases,
        theme::ATLAS,
        theme::ATLAS_SIZE,
        SpriteFilter::Nearest,
    );

    let decoded = decode_to_rgba8(theme::SHEET_BYTES).expect("the tiny farm sheet decodes");
    let (width, height) = (decoded.width, decoded.height);
    let tiles = sprite_atlas_insert(
        atlases,
        theme::ATLAS,
        theme::SHEET,
        &decoded.rgba,
        width,
        height,
    )
    .expect("the sheet fits the atlas");

    let grass = sprite_atlas_insert(
        atlases,
        theme::ATLAS,
        theme::GRASS,
        &grass_tile(),
        theme::TILE_TEXELS,
        theme::TILE_TEXELS,
    )
    .expect("the grass tile fits the atlas");

    Sheet { tiles, grass }
}

fn grass_tile() -> Vec<u8> {
    const BLADES: [(u32, u32); 7] = [(3, 4), (4, 4), (11, 2), (7, 9), (8, 9), (13, 12), (2, 13)];
    nightshade_texture::queries::paint(theme::TILE_TEXELS, |x, y| {
        if BLADES.contains(&(x, y)) {
            theme::GRASS_BLADE_COLOR
        } else {
            theme::GRASS_COLOR
        }
    })
}

fn spawn_buried_cube(registry: &mut MaterialRegistry, commands: &mut Commands) {
    let cube = spawn_mesh_at(
        commands,
        "Cube",
        Vec3::new(6.0, -4.0, 0.0),
        Vec3::new(2.0, 2.0, 2.0),
    );
    let material = insert_material(
        registry,
        "buried_cube",
        Material {
            base_color: [0.85, 0.32, 0.22, 1.0],
            roughness: 0.55,
            metallic: 0.0,
            ..Default::default()
        },
    );
    commands.set(cube, MaterialRef::new(material));
    commands.set(cube, Spin(theme::CUBE_SPIN_SPEED));
}
