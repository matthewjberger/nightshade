use crate::components::{Home, Offset, Reach, Yaw};
use crate::contract::WorldSide;
use crate::data;
use crate::queries;
use crate::resources::MashupResources;
use crate::systems::{hud, menu, terrain, textures};
use nightshade::prelude::*;
use nightshade_camera::commands::scene::spawn_camera;
use nightshade_camera::prelude::*;
use nightshade_graphics::commands::builtins::set_mesh_material;
use nightshade_graphics::prelude::*;
use nightshade_particles::contract::{ColorGradient, EmitterShape, EmitterType, ParticleEmitter};
use nightshade_platform::commands::window::set_pointer_grab;
use nightshade_platform::prelude::Window;
use nightshade_portal::prelude::{Portal, PortalPlane};
use nightshade_portal::queries::portal_model;
use nightshade_render::commands::mesh_cache::{mesh_cache_insert, mesh_cache_protect};
use nightshade_render::commands::mesh_state::mesh_state_mark_entity_added;
use nightshade_render::components::material::MaterialRef;
use nightshade_render::components::mesh::RenderMesh;
use nightshade_render::data::bounding_volume::{BoundingVolume, OrientedBoundingBox};
use nightshade_render::prelude::{MeshCache, MeshRenderState};
use nightshade_scene::prelude::{GlobalTransform, LocalTransform, Name, Visibility};
use nightshade_sprite::prelude::*;

const TERRAIN_MESH: &str = "mashup_quarry_ground";

pub fn build_overlay(
    mut resources: ResMut<MashupResources>,
    style: Res<nightshade_ui_menu::prelude::MenuStyle>,
    mut menus: ResMut<nightshade_ui_menu::prelude::Menus<crate::resources::Screen>>,
    mut build: nightshade_ui::commands::builder::UiBuild,
) {
    let mut tree = nightshade_ui::prelude::UiTreeBuilder::new(build.context());
    resources.hud = hud::build(&mut tree);
    resources.menu = menu::build(&mut tree, &style, &mut menus);
    tree.finish();
}

pub fn initialize(
    mut resources: ResMut<MashupResources>,
    mut settings: ResMut<RenderSettings>,
    mut registry: ResMut<MaterialRegistry>,
    mut atlases: ResMut<SpriteAtlases>,
    mut mesh_cache: ResMut<MeshCache>,
    mut active_camera: ResMut<ActiveCamera>,
    mut viewport: ResMut<nightshade_render::resources::viewport::Viewport>,
    mut window: ResMut<Window>,
    mut textures_load: nightshade_graphics::resources::loading::TextureLoad,
    mut commands: Commands,
) {
    settings.atmosphere = Atmosphere::Sky;
    settings.color_grading.tonemap_algorithm =
        nightshade_render::data::config::TonemapAlgorithm::Reinhard;
    settings.fog = Some(nightshade_render::data::config::Fog {
        color: data::FOG_COLOR,
        start: data::FOG_START,
        end: data::FOG_END,
        ..Default::default()
    });
    spawn_sun(&mut commands);

    textures::load_block_atlas(&mut textures_load);
    textures::insert_materials(&mut registry);

    let sheet = pack_the_sheet(&mut atlases);
    build_meadow(&mut resources, &sheet, &mut registry, &mut commands);
    build_quarry(&mut mesh_cache, &mut registry, &mut commands);

    let mut dispatched = Vec::new();
    for (side, plane) in placements() {
        let index = side.index();
        commands.spawn((Portal(plane), side));

        let eye = spawn_camera(&mut commands, Vec3::zeros(), format!("Portal Eye {index}"));
        commands.set(eye, side);
        commands.set(
            eye,
            RenderTarget::renderer_owned(data::PORTAL_TARGET_WIDTH, data::PORTAL_TARGET_HEIGHT),
        );
        commands.set(eye, ProjectionOverride::default());
        crate::commands::remember_frame(&mut resources, side, portal_model(&plane), eye, 1.0);
        dispatched.push(eye);
        spawn_sparks(side, &mut commands);
    }

    let (eye, look) = queries::meadow_eye(&resources.meadow);
    let camera = spawn_camera(&mut commands, eye, "Player Camera".to_string());
    commands.set(
        camera,
        LocalTransform {
            translation: eye,
            rotation: look,
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    );
    commands.set(
        camera,
        Camera {
            projection: Projection::Perspective(PerspectiveCamera {
                aspect_ratio: None,
                y_fov_rad: data::MEADOW_FIELD_OF_VIEW,
                z_far: Some(data::FAR_PLANE),
                z_near: data::NEAR_PLANE,
            }),
            smoothing: None,
        },
    );
    resources.camera = camera;
    active_camera.0 = Some(camera);

    dispatched.push(camera);
    viewport.required_cameras = dispatched;

    set_pointer_grab(&mut window, true);
}

struct Sheet {
    tree: SpriteRegion,
    villager: SpriteRegion,
    farmer: SpriteRegion,
    flower: SpriteRegion,
    shadow: SpriteRegion,
}

fn pack_the_sheet(atlases: &mut SpriteAtlases) -> Sheet {
    sprite_atlas_create(
        atlases,
        data::SPRITE_ATLAS,
        data::SPRITE_ATLAS_SIZE,
        SpriteFilter::Nearest,
    );
    Sheet {
        tree: insert_art(atlases, "tree", &data::TREE_ART),
        villager: insert_art(atlases, "villager", &data::VILLAGER_ART),
        farmer: insert_art(atlases, "farmer", &data::FARMER_ART),
        flower: insert_art(atlases, "flower", &data::FLOWER_ART),
        shadow: insert_shadow(atlases),
    }
}

fn insert_art(atlases: &mut SpriteAtlases, name: &str, art: &[&str]) -> SpriteRegion {
    let (pixels, width, height) = textures::paint_art(art);
    sprite_atlas_insert(atlases, data::SPRITE_ATLAS, name, &pixels, width, height)
        .expect("the picture fits the atlas")
}

fn insert_shadow(atlases: &mut SpriteAtlases) -> SpriteRegion {
    let (pixels, width, height) = textures::shadow_pixels();
    sprite_atlas_insert(
        atlases,
        data::SPRITE_ATLAS,
        "shadow",
        &pixels,
        width,
        height,
    )
    .expect("the shadow fits the atlas")
}

fn build_meadow(
    resources: &mut MashupResources,
    sheet: &Sheet,
    registry: &mut MaterialRegistry,
    commands: &mut Commands,
) {
    for slab in data::meadow_blocks() {
        let entity = spawn_mesh_at(
            commands,
            "Cube",
            Vec3::new(slab.center[0], slab.center[1], slab.center[2]),
            Vec3::new(slab.size[0], slab.size[1], slab.size[2]),
        );
        set_material(
            registry,
            commands,
            entity,
            format!("mashup_meadow_{}", entity.id),
            Material {
                base_color: slab.color,
                roughness: 0.9,
                ..Default::default()
            },
        );
    }

    for spot in data::meadow_trees() {
        let feet = Vec3::new(spot[0], data::MEADOW_GROUND, spot[1]);
        spawn_standing(commands, sheet, sheet.tree, feet, 2.8, Vec2::new(1.3, 0.8));
    }
    for spot in data::meadow_flowers() {
        let feet = Vec3::new(spot[0], data::MEADOW_GROUND, spot[1]);
        spawn_standing(
            commands,
            sheet,
            sheet.flower,
            feet,
            0.5,
            Vec2::new(0.34, 0.2),
        );
    }
    for spot in data::meadow_villagers() {
        let feet = Vec3::new(spot[0], data::MEADOW_GROUND, spot[1]);
        let (body, _) = spawn_standing(
            commands,
            sheet,
            sheet.villager,
            feet,
            1.25,
            Vec2::new(0.66, 0.4),
        );
        commands.set(body, Home(feet));
        commands.set(
            body,
            Offset(data::hash_unit(spot[0] as i32, spot[1] as i32) * std::f32::consts::TAU),
        );
        commands.set(body, Reach(data::VILLAGER_REACH));
    }

    let (body, shadow) = spawn_standing(
        commands,
        sheet,
        sheet.farmer,
        resources.meadow.feet,
        1.35,
        Vec2::new(0.7, 0.44),
    );
    resources.meadow.body = body;
    resources.meadow.shadow = shadow;
}

fn spawn_standing(
    commands: &mut Commands,
    sheet: &Sheet,
    region: SpriteRegion,
    feet: Vec3,
    height: f32,
    shadow: Vec2,
) -> (Entity, Entity) {
    let sprite = Sprite::new(data::SPRITE_ATLAS, region)
        .with_size(height * 0.66, height)
        .with_anchor(Vec2::new(0.5, 0.0));
    let body = spawn_sprite(commands, sprite, feet);
    commands.set(body, Yaw::default());

    let blob = Sprite::new(data::SPRITE_ATLAS, sheet.shadow)
        .with_size(shadow.x, shadow.y)
        .with_color([0.0, 0.0, 0.0, data::SHADOW_ALPHA])
        .with_anchor(Vec2::new(0.5, 0.5));
    let mark = spawn_sprite(commands, blob, feet);
    commands.set(
        mark,
        LocalTransform {
            translation: feet + Vec3::new(0.0, data::SHADOW_LIFT, shadow.y * 0.35),
            rotation: nalgebra_glm::quat_angle_axis(-std::f32::consts::FRAC_PI_2, &Vec3::x()),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    );
    (body, mark)
}

fn build_quarry(
    mesh_cache: &mut MeshCache,
    registry: &mut MaterialRegistry,
    commands: &mut Commands,
) {
    mesh_cache_insert(mesh_cache, TERRAIN_MESH.to_string(), terrain::build());
    mesh_cache_protect(mesh_cache, TERRAIN_MESH.to_string());

    let span = data::QUARRY_SPAN as f32 * 2.0 + 2.0;
    let extent = Vec3::new(span, (data::QUARRY_BASE + 10) as f32, span);
    let transform = LocalTransform {
        translation: data::QUARRY_ORIGIN,
        rotation: Quat::identity(),
        scale: Vec3::new(1.0, 1.0, 1.0),
    };
    let ground = commands.spawn((
        Name(TERRAIN_MESH.to_string()),
        transform,
        GlobalTransform(transform.as_matrix()),
        RenderMesh::new(TERRAIN_MESH),
        MaterialRef::default(),
        Visibility { visible: true },
        BoundingVolume::new(
            OrientedBoundingBox::from_aabb(Vec3::zeros(), extent),
            extent.norm() * 0.5,
        ),
    ));
    set_mesh_material(commands, ground, textures::BLOCK_MATERIAL);
    commands.queue(move |_, resources| {
        mesh_state_mark_entity_added(resources.res_mut::<MeshRenderState>(), ground);
    });

    for spot in data::quarry_trees() {
        let top = data::column_height(spot[0], spot[1]) as f32;
        let base = data::QUARRY_ORIGIN + Vec3::new(spot[0] as f32 + 0.5, top, spot[1] as f32 + 0.5);
        let trunk = spawn_mesh_at(
            commands,
            "Cube",
            base + Vec3::new(0.0, 1.6, 0.0),
            Vec3::new(0.55, 3.2, 0.55),
        );
        set_material(
            registry,
            commands,
            trunk,
            format!("mashup_trunk_{}", trunk.id),
            Material {
                base_color: [0.36, 0.26, 0.18, 1.0],
                roughness: 1.0,
                ..Default::default()
            },
        );
        let crown = spawn_mesh_at(
            commands,
            "Cube",
            base + Vec3::new(0.0, 3.9, 0.0),
            Vec3::new(2.6, 2.2, 2.6),
        );
        set_material(
            registry,
            commands,
            crown,
            format!("mashup_crown_{}", crown.id),
            Material {
                base_color: [0.26, 0.52, 0.24, 1.0],
                roughness: 1.0,
                ..Default::default()
            },
        );
    }
}

fn placements() -> [(WorldSide, PortalPlane); 2] {
    let quarry_top = data::column_height(0, -10) as f32;
    [
        (
            WorldSide::Meadow,
            PortalPlane {
                center: Vec3::new(0.0, data::PORTAL_LIFT, -10.0),
                normal: Vec3::z(),
                up: Vec3::y(),
                width: data::PORTAL_WIDTH,
                height: data::PORTAL_HEIGHT,
            },
        ),
        (
            WorldSide::Quarry,
            PortalPlane {
                center: data::QUARRY_ORIGIN
                    + Vec3::new(0.5, quarry_top + data::PORTAL_LIFT, -10.0 + 0.5),
                normal: Vec3::z(),
                up: Vec3::y(),
                width: data::PORTAL_WIDTH,
                height: data::PORTAL_HEIGHT,
            },
        ),
    ]
}

fn spark_gradient(side: WorldSide) -> ColorGradient {
    let rim = match side {
        WorldSide::Meadow => data::MEADOW_RIM,
        WorldSide::Quarry => data::QUARRY_RIM,
    };
    let lifted = |channel: f32| channel + (1.0 - channel) * 0.45;
    ColorGradient {
        colors: vec![
            (0.0, Vec4::new(1.0, 1.0, 1.0, 1.0)),
            (
                0.12,
                Vec4::new(lifted(rim[0]), lifted(rim[1]), lifted(rim[2]), 1.0),
            ),
            (0.5, Vec4::new(rim[0], rim[1], rim[2], 0.9)),
            (
                1.0,
                Vec4::new(rim[0] * 0.3, rim[1] * 0.3, rim[2] * 0.3, 0.0),
            ),
        ],
    }
}

fn spawn_sparks(side: WorldSide, commands: &mut Commands) {
    let gradient = spark_gradient(side);
    for index in 0..data::SPARKS_PER_RIM {
        let emitter = ParticleEmitter {
            emitter_type: EmitterType::Sparks,
            shape: EmitterShape::Point,
            spawn_rate: data::SPARK_RATE,
            particle_lifetime_min: data::SPARK_LIFE[0],
            particle_lifetime_max: data::SPARK_LIFE[1],
            initial_velocity_min: data::SPARK_SPEED[0],
            initial_velocity_max: data::SPARK_SPEED[1],
            velocity_spread: data::SPARK_SPREAD,
            gravity: Vec3::new(0.0, data::SPARK_GRAVITY, 0.0),
            drag: data::SPARK_DRAG,
            size_start: data::SPARK_SIZE[0],
            size_end: data::SPARK_SIZE[1],
            color_gradient: gradient.clone(),
            emissive_strength: data::SPARK_GLOW,
            ..Default::default()
        };
        commands.spawn((
            side,
            Offset(index as f32 / data::SPARKS_PER_RIM as f32),
            emitter,
        ));
    }
}
