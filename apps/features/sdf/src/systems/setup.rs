use crate::commands::{
    carry_along, dress_edit, finish_edit, placed, spawn_anchor, spawn_edit, spawn_flying_camera,
};
use crate::components::{Axis, Bob, Drift, Height, Orbit, Phase, Spin};
use crate::data::{Prop, Settings};
use crate::queries::shade;
use crate::resources::{SdfArguments, SdfScene, Waiting};
use crate::systems::hud;
use crate::theme;
use nalgebra_glm::Vec3;
use nightshade::prelude::*;
use nightshade_camera::prelude::*;
use nightshade_config::prelude::Config;
use nightshade_graphics::prelude::*;
use nightshade_sdf::contract::field::terrain_top;
use nightshade_sdf::contract::{SdfOperation, SdfShape};

pub fn initialize(
    mut settings: ResMut<RenderSettings>,
    mut active_camera: ResMut<ActiveCamera>,
    mut scene: ResMut<SdfScene>,
    mut view: ResMut<nightshade_sdf::contract::render_data::SdfDebugView>,
    arguments: Res<SdfArguments>,
    config: Res<Config<Settings>>,
    mut build: nightshade_ui::commands::builder::UiBuild,
    mut commands: Commands,
) {
    settings.atmosphere = Atmosphere::Sky;
    *view = match arguments.view.unwrap_or(0) {
        1 => nightshade_sdf::contract::render_data::SdfDebugView::Bricks,
        2 => nightshade_sdf::contract::render_data::SdfDebugView::Levels,
        3 => nightshade_sdf::contract::render_data::SdfDebugView::Steps,
        4 => nightshade_sdf::contract::render_data::SdfDebugView::Baked,
        _ => nightshade_sdf::contract::render_data::SdfDebugView::Lit,
    };
    spawn_sun(&mut commands);

    let base = config.number(Settings::GroundBase);
    let ground = SdfShape::Terrain {
        base,
        size: config.number(Settings::GroundSize),
        octaves: config.whole(Settings::GroundOctaves).max(1) as u32,
        gain: config.number(Settings::GroundGain),
        clip: config.number(Settings::GroundClip),
        clip_smooth: config.number(Settings::GroundClipSmooth),
        join_smooth: config.number(Settings::GroundJoinSmooth),
        skin: config.number(Settings::GroundSkin),
        dirt: shade(&config, Settings::GroundDirt),
    };
    let floor = terrain_top(&ground, 0.0, 0.0).unwrap_or(base);
    scene.terrain = Some(ground);
    let under_camera =
        terrain_top(&ground, theme::CAMERA_START.x, theme::CAMERA_START.z).unwrap_or(floor);
    let camera_start = Vec3::new(
        theme::CAMERA_START.x,
        (floor + theme::CAMERA_START.y)
            .max(under_camera + config.number(Settings::CameraClearance)),
        theme::CAMERA_START.z,
    );
    let camera = spawn_flying_camera(
        &mut commands,
        camera_start,
        theme::CAMERA_YAW,
        theme::CAMERA_PITCH,
        config.number(Settings::CameraSpeed),
    );
    active_camera.0 = Some(camera);
    scene.camera = Some(camera);
    commands.set(
        camera,
        nightshade_sdf::contract::SdfCollisionFocus(config.number(Settings::CollisionReach)),
    );

    build_ground(&mut commands, &config, ground);
    build_blobs(&mut commands, &config, floor);
    build_cubes(&mut commands, &config, floor);
    build_ring(&mut commands, &config, floor);
    build_props(&mut commands, &mut scene, &config, &ground, base);
    build_stones(&mut commands, &mut scene, &config, &ground, base);
    scene.hole = Some(build_hole(&mut commands, &config, &ground, base));
    scene.tunnel = Some(build_tunnel(&mut commands, &config, floor));

    let mut tree = nightshade_ui::prelude::UiTreeBuilder::new(build.context());
    scene.hud = hud::build(&mut tree);
    tree.finish();
}

fn build_ground(commands: &mut Commands, config: &Config<Settings>, shape: SdfShape) {
    let ground = spawn_edit(commands, "Ground", placed(Vec3::zeros()), shape);
    dress_edit(
        commands,
        ground,
        SdfOperation::Union,
        0.0,
        shade(config, Settings::GroundTint),
        theme::GROUND_ORDER,
    );
    finish_edit(
        commands,
        ground,
        config.number(Settings::GroundRoughness),
        0.0,
    );
}

fn build_blobs(commands: &mut Commands, config: &Config<Settings>, floor: f32) {
    let count = config.whole(Settings::BlobCount).max(0) as u32;
    let orbit = config.number(Settings::BlobOrbit);
    let height = floor + config.number(Settings::BlobHeight);
    let turn = std::f32::consts::TAU / count.max(1) as f32;
    for index in 0..count {
        let phase = turn * index as f32;
        let blob = spawn_edit(
            commands,
            "Blob",
            placed(Vec3::new(orbit * phase.cos(), height, orbit * phase.sin())),
            SdfShape::Sphere {
                radius: config.number(Settings::BlobRadius),
            },
        );
        dress_edit(
            commands,
            blob,
            SdfOperation::Union,
            config.number(Settings::BlobBlend),
            shade(config, Settings::BlobTint),
            theme::BLOB_ORDER,
        );
        finish_edit(commands, blob, config.number(Settings::BlobRoughness), 0.0);
        carry_along(
            commands,
            blob,
            Vec3::new(orbit * phase.cos(), height, orbit * phase.sin()),
            nightshade_physics::prelude::ColliderComponent::new_ball(
                config.number(Settings::BlobRadius),
            ),
        );
        commands.set(blob, Orbit(orbit));
        commands.set(blob, Phase(phase));
        commands.set(blob, Drift(config.number(Settings::BlobDrift)));
        commands.set(blob, Bob(config.number(Settings::BlobBob)));
        commands.set(blob, Height(height));
    }
}

fn build_cubes(commands: &mut Commands, config: &Config<Settings>, floor: f32) {
    let count = config.whole(Settings::CubeCount).max(0) as u32;
    let orbit = config.number(Settings::CubeOrbit);
    let height = floor + config.number(Settings::CubeHeight);
    let half = config.number(Settings::CubeHalf);
    let rounding = config.number(Settings::CubeRounding);
    let turn = std::f32::consts::TAU / count.max(1) as f32;
    for index in 0..count {
        let phase = turn * index as f32;
        let cube = spawn_edit(
            commands,
            "Cube",
            placed(Vec3::new(orbit * phase.cos(), height, orbit * phase.sin())),
            SdfShape::Cube {
                half_extents: Vec3::repeat(half),
                rounding,
            },
        );
        dress_edit(
            commands,
            cube,
            SdfOperation::Union,
            config.number(Settings::CubeBlend),
            shade(config, Settings::CubeTint),
            theme::CUBE_ORDER,
        );
        finish_edit(
            commands,
            cube,
            config.number(Settings::CubeRoughness),
            config.number(Settings::CubeMetal),
        );
        carry_along(
            commands,
            cube,
            Vec3::new(orbit * phase.cos(), height, orbit * phase.sin()),
            nightshade_physics::prelude::ColliderComponent::new_cuboid(
                half + rounding,
                half + rounding,
                half + rounding,
            ),
        );
        commands.set(cube, Orbit(orbit));
        commands.set(cube, Phase(phase));
        commands.set(cube, Drift(config.number(Settings::CubeDrift)));
        commands.set(cube, Bob(config.number(Settings::CubeBob)));
        commands.set(cube, Height(height));
        commands.set(cube, Spin(config.number(Settings::CubeSpin)));
        commands.set(cube, Axis(Vec3::new(0.4, 1.0, 0.2).normalize()));
    }
}

fn build_ring(commands: &mut Commands, config: &Config<Settings>, floor: f32) {
    let ring = spawn_edit(
        commands,
        "Ring",
        placed(Vec3::new(
            0.0,
            floor + config.number(Settings::RingHeight),
            0.0,
        )),
        SdfShape::Torus {
            major_radius: config.number(Settings::RingMajor),
            minor_radius: config.number(Settings::RingMinor),
        },
    );
    dress_edit(
        commands,
        ring,
        SdfOperation::Union,
        config.number(Settings::BlobBlend),
        shade(config, Settings::RingTint),
        theme::RING_ORDER,
    );
    finish_edit(
        commands,
        ring,
        config.number(Settings::RingRoughness),
        config.number(Settings::RingMetal),
    );
    let torus = nightshade_render::data::procedural_meshes::create_torus_mesh(
        config.number(Settings::RingMajor),
        config.number(Settings::RingMinor),
        theme::RING_SEGMENTS,
        theme::RING_SIDES,
    );
    carry_along(
        commands,
        ring,
        Vec3::new(0.0, floor + config.number(Settings::RingHeight), 0.0),
        nightshade_physics::prelude::ColliderComponent {
            shape: nightshade_physics::prelude::ColliderShape::TriMesh {
                vertices: torus
                    .vertices
                    .iter()
                    .map(|vertex| vertex.position)
                    .collect(),
                indices: torus.indices.as_chunks::<3>().0.to_vec(),
            },
            ..Default::default()
        },
    );
    commands.set(ring, Spin(config.number(Settings::RingSpin)));
    commands.set(ring, Axis(Vec3::new(1.0, 0.3, 0.0).normalize()));
}

fn prop_spot(
    ground: &SdfShape,
    base: f32,
    config: &Config<Settings>,
    index: u32,
    count: u32,
) -> Vec3 {
    let center = config.number(Settings::PropCenter);
    let spread = config.number(Settings::PropSpread);
    let angle = std::f32::consts::TAU * index as f32 / count.max(1) as f32;
    let reach = spread * (0.45 + 0.55 * ((index * 7 % 5) as f32 / 4.0));
    let x = reach * angle.cos();
    let z = center + reach * angle.sin();
    let top = terrain_top(ground, x, z).unwrap_or(base);
    Vec3::new(x, top + config.number(Settings::PropDrop), z)
}

fn build_props(
    commands: &mut Commands,
    scene: &mut SdfScene,
    config: &Config<Settings>,
    ground: &SdfShape,
    base: f32,
) {
    let bars = config.whole(Settings::BarCount).max(0) as u32;
    let blocks = config.whole(Settings::BlockCount).max(0) as u32;
    let spikes = config.whole(Settings::SpikeCount).max(0) as u32;
    let total = bars + blocks + spikes;
    let up = Vec3::new(0.0, 1.0, 0.0);
    let drop = config.number(Settings::PropDrop);
    for index in 0..total {
        let spot = prop_spot(ground, base, config, index, total);
        let prop = if index < bars {
            Prop::Bar
        } else if index < bars + blocks {
            Prop::Block
        } else {
            Prop::Spike
        };
        wait_for_ground(
            commands,
            scene,
            config,
            prop,
            spot,
            spot - up * drop,
            nalgebra_glm::quat_angle_axis(index as f32 * 2.399_963, &up),
        );
    }
}

fn build_stones(
    commands: &mut Commands,
    scene: &mut SdfScene,
    config: &Config<Settings>,
    ground: &SdfShape,
    base: f32,
) {
    let count = config.whole(Settings::StoneCount).max(0) as u32;
    let spread = config.number(Settings::StoneSpread);
    let turn = std::f32::consts::TAU / count.max(1) as f32;
    for index in 0..count {
        let phase = turn * index as f32;
        let x = theme::STONE_START.x + spread * phase.cos();
        let z = theme::STONE_START.z + spread * phase.sin();
        let top = terrain_top(ground, x, z).unwrap_or(base);
        let spot = Vec3::new(
            x,
            top + theme::STONE_START.y + spread * 0.4 * index as f32,
            z,
        );
        wait_for_ground(
            commands,
            scene,
            config,
            Prop::Stone,
            spot,
            Vec3::new(x, top, z),
            nalgebra_glm::Quat::identity(),
        );
    }
}

fn wait_for_ground(
    commands: &mut Commands,
    scene: &mut SdfScene,
    config: &Config<Settings>,
    prop: Prop,
    spot: Vec3,
    ground: Vec3,
    rotation: nalgebra_glm::Quat,
) {
    let anchor = spawn_anchor(commands, ground, config.number(Settings::PropFocus));
    scene.waiting.push(Waiting {
        prop,
        spot,
        ground,
        rotation,
        anchor,
    });
}

fn build_hole(
    commands: &mut Commands,
    config: &Config<Settings>,
    ground: &SdfShape,
    base: f32,
) -> Entity {
    let x = 0.0;
    let z = config.number(Settings::PropCenter);
    let top = terrain_top(ground, x, z).unwrap_or(base);
    let depth = config.number(Settings::HoleDepth);
    let hole = spawn_edit(
        commands,
        "Hole",
        placed(Vec3::new(x, top - depth * 0.5 + 1.0, z)),
        SdfShape::Cylinder {
            half_height: depth * 0.5 + 1.0,
            radius: config.number(Settings::HoleRadius),
        },
    );
    dress_edit(
        commands,
        hole,
        SdfOperation::Subtract,
        config.number(Settings::HoleBlend),
        shade(config, Settings::GroundTint),
        theme::HOLE_ORDER,
    );
    hole
}

fn build_tunnel(commands: &mut Commands, config: &Config<Settings>, floor: f32) -> Entity {
    let tunnel = spawn_edit(
        commands,
        "Tunnel",
        placed(Vec3::new(0.0, floor, 0.0)),
        SdfShape::Capsule {
            half_height: config.number(Settings::TunnelHalfHeight),
            radius: config.number(Settings::TunnelRadius),
        },
    );
    dress_edit(
        commands,
        tunnel,
        SdfOperation::Subtract,
        config.number(Settings::TunnelBlend),
        shade(config, Settings::GroundTint),
        theme::TUNNEL_ORDER,
    );
    commands.set(tunnel, Visibility { visible: false });
    tunnel
}
