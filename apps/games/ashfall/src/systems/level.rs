use crate::commands::{attach_material, attach_surface, block_tile, scatter_seed};
use crate::components::{
    Alive, Bled, Body, Bolted, Floor, Glow, Health, Home, HuskKind, HuskState, Inscription,
    Interest, ItemKind, Lock, Moving, NextTile, NpcIndex, Opens, Phase, Repath, Rest, Scream,
    Speed, Stride, SurfaceKind, Swing, Tile, Timer,
};
use crate::data::level::*;
use crate::data::people::PEOPLE;
use crate::data::props::{EXAMINE_SITES, LANTERNS};
use crate::data::rules::*;
use crate::data::theme;
use crate::queries::{
    door_runs_along_x, is_floor, is_room, item_look, note_at, stair_run, tile, tile_center,
    upper_tile,
};
use crate::resources::{AshfallArguments, AshfallResources, WorldPhase};
use crate::systems::{effects, props, textures};
use crate::tags::{Eyes, Flashlight, Grabbable, LevelPiece, Player};
use nightshade::prelude::*;
use nightshade_camera::commands::spawn_first_person_camera;
use nightshade_flashlight::{Battery, Beam, Drain, Lit};
use nightshade_flicker::{FlickerBase, Flickering};
use nightshade_graphics::commands::builtins::spawn_mesh_at;
use nightshade_graphics::prelude::*;
use nightshade_phase::PhaseTinted;
use nightshade_physics::commands::joints::{SphericalJoint, create_spherical_joint};
use nightshade_physics::commands::spawn_character_body;
use nightshade_physics::contract::components::{
    CharacterControllerComponent, ColliderComponent, RigidBodyComponent,
};
use nightshade_physics::resources::PhysicsWorld;
use nightshade_senses::Exposure;

const SIDES: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

pub fn build(
    commands: &mut Commands,
    resources: &mut AshfallResources,
    arguments: &AshfallArguments,
) {
    let phase = resources.run.phase;
    resources.run.blocked = vec![false; WIDTH * HEIGHT];
    resources.run.blocked_upper = vec![true; WIDTH * HEIGHT];
    resources.run.explored = vec![false; WIDTH * HEIGHT];
    resources.run.explored_upper = vec![false; WIDTH * HEIGHT];
    resources.run.otherworld_spawns.clear();
    let mut player_start = tile_center(20, 33);
    for row in 0..HEIGHT as i32 {
        for col in 0..WIDTH as i32 {
            if tile(col, row) == PLAYER {
                player_start = tile_center(col, row);
            }
        }
    }

    for row in 0..HEIGHT as i32 {
        for col in 0..WIDTH as i32 {
            let center = tile_center(col, row);
            match tile(col, row) {
                FACADE => {
                    spawn_facade(commands, col, row, center, phase);
                    block_tile(&mut resources.run, (col, row), false);
                }
                PARTITION => {
                    spawn_partition(commands, center, 0.0, phase);
                    block_tile(&mut resources.run, (col, row), false);
                }
                here => {
                    if is_room(col, row) {
                        spawn_floor_and_ceiling(commands, col, row, center, phase);
                    }
                    ground_contents(commands, resources, col, row, center, here, phase);
                }
            }
            upper_contents(commands, resources, col, row, center, phase);
        }
    }

    for site in EXAMINE_SITES {
        let level = if site.upper { STOREY } else { 0.0 };
        let look = commands.spawn((
            Name("Something To Look At".to_string()),
            LocalTransform {
                translation: tile_center(site.col, site.row) + Vec3::new(0.0, level + 1.0, 0.0),
                ..Default::default()
            },
            GlobalTransform::default(),
            Inscription { index: site.index },
        ));
        commands.add_tag_type::<LevelPiece>(look);
    }

    for (index, who) in PEOPLE.iter().enumerate() {
        let level = if who.upper { STOREY } else { 0.0 };
        spawn_person(
            commands,
            tile_center(who.col, who.row) + Vec3::new(0.0, level, 0.0),
            index,
        );
    }

    for (col, row) in LANTERNS {
        spawn_hanging_lamp(commands, tile_center(*col, *row), phase);
    }

    props::spawn_all(commands, phase);
    if let Some(tile) = requested_start(arguments) {
        player_start = tile_center(tile.0, tile.1);
    }
    effects::spawn_ash_cloud(commands, player_start, phase);
    spawn_player(commands, resources, player_start);
}

fn requested_start(arguments: &AshfallArguments) -> Option<(i32, i32)> {
    let text = arguments.start_at.as_ref()?;
    let (col, row) = text.split_once(',')?;
    Some((col.trim().parse().ok()?, row.trim().parse().ok()?))
}

fn ground_contents(
    commands: &mut Commands,
    resources: &mut AshfallResources,
    col: i32,
    row: i32,
    center: Vec3,
    here: u8,
    phase: WorldPhase,
) {
    match here {
        DOOR | BRASS_DOOR | WARD_DOOR | BONE_DOOR | BOLTED => {
            spawn_door(
                commands,
                center,
                (col, row),
                lock_of(here),
                here == BOLTED,
                false,
                phase,
            );
            block_tile(&mut resources.run, (col, row), false);
        }
        CASE => spawn_pickup(commands, center, ItemKind::Case),
        BONE_HAFT => spawn_pickup(commands, center, ItemKind::BoneHaft),
        BONE_WARD => spawn_pickup(commands, center, ItemKind::BoneWard),
        HEALTH => spawn_pickup(commands, center, ItemKind::HealthDrink),
        AMMO => spawn_pickup(commands, center, ItemKind::Ammo),
        BATTERY => spawn_pickup(commands, center, ItemKind::Battery),
        TOWN_MAP => spawn_pickup(commands, center, ItemKind::TownMap),
        LOCKBOX => spawn_pickup(commands, center, ItemKind::Lockbox),
        BOARDS => {
            spawn_boards(commands, center, (col, row), phase);
            block_tile(&mut resources.run, (col, row), false);
        }
        WANDERER | CRAWLER | WRETCH | SCREAMER => {
            spawn_husk(commands, center, husk_kind(here), false, true);
        }
        OTHER_WANDERER | OTHER_CRAWLER | OTHER_WRETCH | OTHER_SCREAMER => {
            resources.run.otherworld_spawns.push((col, row, here));
        }
        NOTE => {
            if let Some(index) = note_at(col, row, false) {
                spawn_pickup(commands, center, ItemKind::Note(index));
            }
        }
        PLAYER => {}
        EXIT => resources.run.exit_tile = (col, row),
        LAMP => spawn_lamp(commands, center, phase),
        CRATE => {
            spawn_crate(commands, center, phase);
            block_tile(&mut resources.run, (col, row), false);
        }
        STAIR => {
            block_tile(&mut resources.run, (col, row), false);
            if let Some(run) = stair_run(col, row) {
                spawn_stair(commands, center, run, phase);
            }
        }
        _ => {}
    }
}

fn upper_contents(
    commands: &mut Commands,
    resources: &mut AshfallResources,
    col: i32,
    row: i32,
    center: Vec3,
    phase: WorldPhase,
) {
    let here = upper_tile(col, row);
    let level = Vec3::new(0.0, STOREY, 0.0);
    match here {
        NOTHING | FACADE => {}
        PARTITION => spawn_partition(commands, center, STOREY, phase),
        WALKWAY | LEDGE => {
            spawn_upper_slab(commands, center, phase, false);
            if here == WALKWAY {
                spawn_rails(commands, col, row, center, phase);
            }
            crate::commands::unblock_tile(&mut resources.run, (col, row), true);
        }
        _ => {
            spawn_upper_slab(commands, center, phase, true);
            if here == DOOR {
                spawn_door(
                    commands,
                    center + level,
                    (col, row),
                    None,
                    false,
                    true,
                    phase,
                );
            } else {
                crate::commands::unblock_tile(&mut resources.run, (col, row), true);
            }
            match here {
                HEALTH => spawn_pickup(commands, center + level, ItemKind::HealthDrink),
                AMMO => spawn_pickup(commands, center + level, ItemKind::Ammo),
                BATTERY => spawn_pickup(commands, center + level, ItemKind::Battery),
                NOTE => {
                    if let Some(index) = note_at(col, row, true) {
                        spawn_pickup(commands, center + level, ItemKind::Note(index));
                    }
                }
                _ => {}
            }
        }
    }
}

fn lock_of(here: u8) -> Option<ItemKind> {
    match here {
        BRASS_DOOR => Some(ItemKind::BrassKey),
        WARD_DOOR => Some(ItemKind::WardKey),
        BONE_DOOR => Some(ItemKind::BoneKey),
        _ => None,
    }
}

pub fn husk_kind(glyph: u8) -> HuskKind {
    match glyph {
        CRAWLER | OTHER_CRAWLER => HuskKind::Crawler,
        WRETCH | OTHER_WRETCH => HuskKind::Wretch,
        SCREAMER | OTHER_SCREAMER => HuskKind::Screamer,
        _ => HuskKind::Wanderer,
    }
}

fn spawn_facade(commands: &mut Commands, col: i32, row: i32, center: Vec3, phase: WorldPhase) {
    let ground = spawn_mesh_at(
        commands,
        "Cube",
        center + Vec3::new(0.0, STOREY * 0.5, 0.0),
        Vec3::new(TILE, STOREY, TILE),
    );
    attach_surface(commands, ground, SurfaceKind::Facade, phase);
    commands.add_tag_type::<LevelPiece>(ground);
    commands.set(
        ground,
        RigidBodyComponent::new_static().with_translation(center.x, STOREY, center.z),
    );
    commands.set(
        ground,
        ColliderComponent::new_cuboid(TILE * 0.5, STOREY, TILE * 0.5).with_friction(0.2),
    );

    let faces: Vec<(i32, i32)> = SIDES
        .iter()
        .copied()
        .filter(|(step_col, step_row)| tile(col + step_col, row + step_row) == STREET)
        .collect();
    let above = upper_tile(col, row);
    if is_floor(above) {
        return;
    }
    let holds_a_floor_up = above == FACADE;
    if faces.is_empty() && !holds_a_floor_up {
        return;
    }

    let upper = spawn_mesh_at(
        commands,
        "Cube",
        center + Vec3::new(0.0, STOREY * 1.5 - 0.02, 0.0),
        Vec3::new(TILE, STOREY + 0.04, TILE),
    );
    attach_surface(commands, upper, SurfaceKind::FacadeUpper, phase);
    commands.add_tag_type::<LevelPiece>(upper);
    commands.set(
        upper,
        RigidBodyComponent::new_static().with_translation(center.x, STOREY * 1.5, center.z),
    );
    commands.set(
        upper,
        ColliderComponent::new_cuboid(TILE * 0.5, STOREY * 0.5, TILE * 0.5).with_friction(0.2),
    );
    if faces.is_empty() {
        return;
    }

    let band = spawn_mesh_at(
        commands,
        "Cube",
        center + Vec3::new(0.0, STOREY + 0.02, 0.0),
        Vec3::new(TILE + 0.16, 0.18, TILE + 0.16),
    );
    attach_surface(commands, band, SurfaceKind::Cornice, phase);
    commands.add_tag_type::<LevelPiece>(band);

    let cornice = spawn_mesh_at(
        commands,
        "Cube",
        center + Vec3::new(0.0, STOREY * 2.0 + 0.1, 0.0),
        Vec3::new(TILE + 0.3, 0.28, TILE + 0.3),
    );
    attach_surface(commands, cornice, SurfaceKind::Cornice, phase);
    commands.add_tag_type::<LevelPiece>(cornice);

    let mut windows = fastrand::Rng::with_seed(scatter_seed(col, row) as u64);
    for (step_col, step_row) in faces {
        for height in [1.7_f32, STOREY + 1.7] {
            let lit = windows.f32() < theme::WINDOW_LIT_CHANCE;
            spawn_window(commands, center, step_col, step_row, height, lit, phase);
        }
    }
}

fn face_scale(step_col: i32, width: f32, height: f32, depth: f32) -> Vec3 {
    if step_col != 0 {
        Vec3::new(depth, height, width)
    } else {
        Vec3::new(width, height, depth)
    }
}

fn spawn_window(
    commands: &mut Commands,
    center: Vec3,
    step_col: i32,
    step_row: i32,
    height: f32,
    lit: bool,
    phase: WorldPhase,
) {
    let toward = Vec3::new(step_col as f32, 0.0, step_row as f32);
    let sill = center + toward * (TILE * 0.5 + 0.02) + Vec3::new(0.0, height, 0.0);
    let pane = center + toward * (TILE * 0.5 + 0.05) + Vec3::new(0.0, height, 0.0);

    let frame = spawn_mesh_at(
        commands,
        "Cube",
        sill,
        face_scale(step_col, 0.98, 1.18, 0.12),
    );
    attach_surface(commands, frame, SurfaceKind::Cornice, phase);
    commands.add_tag_type::<LevelPiece>(frame);

    let glow = lit.then(|| {
        let light = commands.spawn((
            Name("Window Light".to_string()),
            LocalTransform {
                translation: pane + toward * 0.6,
                ..Default::default()
            },
            GlobalTransform::default(),
            Light {
                light_type: LightType::Point,
                color: theme::WINDOW_LIGHT,
                intensity: theme::WINDOW_INTENSITY,
                range: theme::WINDOW_RANGE,
                ..Default::default()
            },
        ));
        commands.add_tag_type::<LevelPiece>(light);
        light
    });

    let entity = spawn_mesh_at(
        commands,
        "Cube",
        pane,
        face_scale(step_col, 0.84, 1.02, 0.1),
    );
    let kind = if lit {
        SurfaceKind::GlassLit
    } else {
        SurfaceKind::Glass
    };
    attach_surface(commands, entity, kind, phase);
    commands.add_tag_type::<LevelPiece>(entity);
    commands.set(entity, Health(GLASS_HEALTH));
    commands.set(entity, Glow(glow));
    commands.set(entity, Tile((0, 0)));
    commands.set(entity, Opens(false));
}

fn spawn_boards(
    commands: &mut Commands,
    center: Vec3,
    tile_position: (i32, i32),
    phase: WorldPhase,
) {
    let along_x = door_runs_along_x(tile_position.0, tile_position.1, false);
    let frame = if along_x {
        Vec3::new(TILE, WALL_HEIGHT, 0.34)
    } else {
        Vec3::new(0.34, WALL_HEIGHT, TILE)
    };
    let position = center + Vec3::new(0.0, WALL_HEIGHT * 0.5, 0.0);
    let jamb = spawn_mesh_at(commands, "Cube", position, frame);
    attach_surface(commands, jamb, SurfaceKind::Concrete, phase);
    commands.add_tag_type::<LevelPiece>(jamb);

    let panel = if along_x {
        Vec3::new(TILE * 0.9, WALL_HEIGHT - 0.6, 0.18)
    } else {
        Vec3::new(0.18, WALL_HEIGHT - 0.6, TILE * 0.9)
    };
    let boards = spawn_mesh_at(commands, "Cube", position, panel);
    attach_surface(commands, boards, SurfaceKind::Door, phase);
    commands.add_tag_type::<LevelPiece>(boards);
    commands.set(boards, Health(BOARD_HEALTH));
    commands.set(boards, Glow(None));
    commands.set(boards, Tile(tile_position));
    commands.set(boards, Opens(true));
    commands.set(
        boards,
        RigidBodyComponent::new_static().with_translation(position.x, position.y, position.z),
    );
    commands.set(
        boards,
        ColliderComponent::new_cuboid(panel.x * 0.5, panel.y * 0.5, panel.z * 0.5),
    );
}

fn spawn_partition(commands: &mut Commands, center: Vec3, level: f32, phase: WorldPhase) {
    let position = center + Vec3::new(0.0, level + WALL_HEIGHT * 0.5, 0.0);
    let entity = spawn_mesh_at(
        commands,
        "Cube",
        position,
        Vec3::new(TILE, WALL_HEIGHT, TILE),
    );
    attach_surface(commands, entity, SurfaceKind::Partition, phase);
    commands.add_tag_type::<LevelPiece>(entity);
    commands.set(
        entity,
        RigidBodyComponent::new_static().with_translation(position.x, position.y, position.z),
    );
    commands.set(
        entity,
        ColliderComponent::new_cuboid(TILE * 0.5, WALL_HEIGHT * 0.5, TILE * 0.5).with_friction(0.2),
    );
}

fn spawn_floor_and_ceiling(
    commands: &mut Commands,
    col: i32,
    row: i32,
    center: Vec3,
    phase: WorldPhase,
) {
    let floor = spawn_mesh_at(commands, "Cube", center, Vec3::new(TILE, 0.1, TILE));
    attach_surface(commands, floor, SurfaceKind::RoomFloor, phase);
    commands.add_tag_type::<LevelPiece>(floor);

    if is_floor(upper_tile(col, row)) || tile(col, row) == STAIR {
        return;
    }
    let ceiling = spawn_mesh_at(
        commands,
        "Cube",
        center + Vec3::new(0.0, WALL_HEIGHT + 0.05, 0.0),
        Vec3::new(TILE, 0.1, TILE),
    );
    attach_surface(commands, ceiling, SurfaceKind::Ceiling, phase);
    commands.add_tag_type::<LevelPiece>(ceiling);
}

fn spawn_rails(commands: &mut Commands, col: i32, row: i32, center: Vec3, phase: WorldPhase) {
    for (step_col, step_row) in SIDES.iter().copied() {
        if is_floor(upper_tile(col + step_col, row + step_row)) {
            continue;
        }
        let along = Vec3::new(step_col as f32, 0.0, step_row as f32);
        let position = center + along * (TILE * 0.5 - 0.06) + Vec3::new(0.0, STOREY + 0.45, 0.0);
        let scale = face_scale(step_col, TILE, 0.9, 0.12);
        let rail = spawn_mesh_at(commands, "Cube", position, scale);
        attach_surface(commands, rail, SurfaceKind::Grille, phase);
        commands.add_tag_type::<LevelPiece>(rail);
        commands.set(
            rail,
            RigidBodyComponent::new_static().with_translation(position.x, position.y, position.z),
        );
        commands.set(
            rail,
            ColliderComponent::new_cuboid(scale.x * 0.5, scale.y * 0.5, scale.z * 0.5),
        );
    }
}

fn spawn_upper_slab(commands: &mut Commands, center: Vec3, phase: WorldPhase, roofed: bool) {
    let position = center + Vec3::new(0.0, STOREY - 0.1, 0.0);
    let slab = spawn_mesh_at(commands, "Cube", position, Vec3::new(TILE, 0.2, TILE));
    attach_surface(commands, slab, SurfaceKind::Concrete, phase);
    commands.add_tag_type::<LevelPiece>(slab);
    commands.set(
        slab,
        RigidBodyComponent::new_static().with_translation(position.x, position.y, position.z),
    );
    commands.set(
        slab,
        ColliderComponent::new_cuboid(TILE * 0.5, 0.1, TILE * 0.5).with_friction(0.9),
    );

    if !roofed {
        return;
    }
    let ceiling = spawn_mesh_at(
        commands,
        "Cube",
        center + Vec3::new(0.0, STOREY + WALL_HEIGHT + 0.05, 0.0),
        Vec3::new(TILE, 0.1, TILE),
    );
    attach_surface(commands, ceiling, SurfaceKind::Ceiling, phase);
    commands.add_tag_type::<LevelPiece>(ceiling);
}

fn spawn_stair(commands: &mut Commands, center: Vec3, run: (i32, i32), phase: WorldPhase) {
    let along = Vec3::new(run.0 as f32, 0.0, run.1 as f32);
    let tread = TILE * 2.0 / STAIR_STEPS as f32;
    for step in 0..STAIR_STEPS {
        let top = (step + 1) as f32 / STAIR_STEPS as f32 * STOREY;
        let travel = -TILE * 0.5 + (step as f32 + 0.5) * tread;
        let position = center + along * travel + Vec3::new(0.0, top * 0.5, 0.0);
        let scale = if run.0 != 0 {
            Vec3::new(tread + 0.04, top, TILE)
        } else {
            Vec3::new(TILE, top, tread + 0.04)
        };
        let entity = spawn_mesh_at(commands, "Cube", position, scale);
        attach_surface(commands, entity, SurfaceKind::Concrete, phase);
        commands.add_tag_type::<LevelPiece>(entity);
        commands.set(
            entity,
            RigidBodyComponent::new_static().with_translation(position.x, position.y, position.z),
        );
        commands.set(
            entity,
            ColliderComponent::new_cuboid(scale.x * 0.5, scale.y * 0.5, scale.z * 0.5)
                .with_friction(0.9),
        );
    }
}

fn spawn_door(
    commands: &mut Commands,
    center: Vec3,
    tile_position: (i32, i32),
    lock: Option<ItemKind>,
    bolted: bool,
    upper: bool,
    phase: WorldPhase,
) {
    let along_x = door_runs_along_x(tile_position.0, tile_position.1, upper);
    let (leaf, frame) = if along_x {
        (
            Vec3::new(TILE * 0.82, WALL_HEIGHT - 0.2, 0.22),
            Vec3::new(TILE, WALL_HEIGHT, 0.34),
        )
    } else {
        (
            Vec3::new(0.22, WALL_HEIGHT - 0.2, TILE * 0.82),
            Vec3::new(0.34, WALL_HEIGHT, TILE),
        )
    };
    let position = center + Vec3::new(0.0, WALL_HEIGHT * 0.5, 0.0);

    let jamb = spawn_mesh_at(commands, "Cube", position, frame);
    attach_surface(commands, jamb, SurfaceKind::Concrete, phase);
    commands.add_tag_type::<LevelPiece>(jamb);

    let entity = spawn_mesh_at(commands, "Cube", position + Vec3::new(0.0, -0.1, 0.0), leaf);
    attach_surface(commands, entity, SurfaceKind::Door, phase);
    commands.set(entity, Lock(lock));
    commands.set(entity, Tile(tile_position));
    commands.set(entity, Bolted(bolted));
    commands.add_tag_type::<LevelPiece>(entity);
    commands.set(
        entity,
        RigidBodyComponent::new_static().with_translation(position.x, position.y, position.z),
    );
    commands.set(
        entity,
        ColliderComponent::new_cuboid(frame.x * 0.5, frame.y * 0.5, frame.z * 0.5),
    );
}

pub fn drop_pickup(commands: &mut Commands, at: Vec3, upstairs: bool, kind: ItemKind) {
    let level = if upstairs { STOREY } else { 0.0 };
    spawn_pickup(commands, Vec3::new(at.x, level, at.z), kind);
}

fn spawn_pickup(commands: &mut Commands, center: Vec3, kind: ItemKind) {
    let home = center + Vec3::new(0.0, 0.9, 0.0);
    let (mesh, scale, color, glow) = item_look(kind);
    let entity = spawn_mesh_at(commands, mesh, home, scale);
    let name = match kind {
        ItemKind::Note(_) => "ashfall_item_Note".to_string(),
        other => format!("ashfall_item_{other:?}"),
    };
    attach_material(commands, entity, name, textures::solid(color, glow));
    commands.set(entity, kind);
    commands.set(entity, Home(home));
    commands.set(entity, Phase(home.x * 0.7 + home.z * 0.3));
    commands.add_tag_type::<LevelPiece>(entity);
}

fn spawn_lamp(commands: &mut Commands, center: Vec3, phase: WorldPhase) {
    let post = spawn_mesh_at(
        commands,
        "Cylinder",
        center + Vec3::new(0.0, 2.1, 0.0),
        Vec3::new(0.18, 4.2, 0.18),
    );
    attach_surface(commands, post, SurfaceKind::Metal, phase);
    commands.add_tag_type::<LevelPiece>(post);
    commands.set(
        post,
        RigidBodyComponent::new_static().with_translation(center.x, 2.1, center.z),
    );
    commands.set(post, ColliderComponent::new_cylinder(2.1, 0.2));

    let arm = spawn_mesh_at(
        commands,
        "Cube",
        center + Vec3::new(0.0, 4.24, 0.0),
        Vec3::new(0.16, 0.16, 0.9),
    );
    attach_surface(commands, arm, SurfaceKind::Metal, phase);
    commands.add_tag_type::<LevelPiece>(arm);

    let hood = spawn_mesh_at(
        commands,
        "Cube",
        center + Vec3::new(0.0, 4.3, 0.42),
        Vec3::new(0.62, 0.22, 0.62),
    );
    attach_surface(commands, hood, SurfaceKind::Metal, phase);
    commands.add_tag_type::<LevelPiece>(hood);

    let bulb = spawn_mesh_at(
        commands,
        "Cube",
        center + Vec3::new(0.0, 4.14, 0.42),
        Vec3::new(0.5, 0.1, 0.5),
    );
    attach_material(
        commands,
        bulb,
        "ashfall_lamp_bulb",
        textures::solid([1.0, 0.88, 0.62, 1.0], theme::LAMP_HEAD_GLOW),
    );
    commands.add_tag_type::<LevelPiece>(bulb);

    let lamp = commands.spawn((
        Name("Street Lamp".to_string()),
        LocalTransform {
            translation: center + Vec3::new(0.0, 4.0, 0.42),
            ..Default::default()
        },
        GlobalTransform::default(),
        Light {
            light_type: LightType::Point,
            color: theme::LAMP_COLOR,
            intensity: theme::LAMP_INTENSITY,
            range: theme::LAMP_RANGE,
            ..Default::default()
        },
    ));
    commands.set(lamp, FlickerBase(theme::LAMP_INTENSITY));
    commands.set(lamp, Flickering(1.0));
    commands.set(lamp, PhaseTinted(1.0));
    commands.add_tag_type::<LevelPiece>(lamp);
}

fn spawn_hanging_lamp(commands: &mut Commands, center: Vec3, phase: WorldPhase) {
    let hook_at = center + Vec3::new(0.0, WALL_HEIGHT - 0.14, 0.0);
    let hook = commands.spawn((
        Name("Lantern Hook".to_string()),
        LocalTransform {
            translation: hook_at,
            ..Default::default()
        },
        GlobalTransform::default(),
        RigidBodyComponent::new_static().with_translation(hook_at.x, hook_at.y, hook_at.z),
        ColliderComponent::new_cuboid(0.07, 0.07, 0.07),
    ));
    commands.add_tag_type::<LevelPiece>(hook);
    let plate = spawn_mesh_at(commands, "Cube", hook_at, Vec3::new(0.3, 0.12, 0.3));
    attach_surface(commands, plate, SurfaceKind::Metal, phase);
    commands.add_tag_type::<LevelPiece>(plate);

    let lantern_at = hook_at + Vec3::new(LANTERN_NUDGE * 0.28, -LANTERN_DROP, 0.0);
    let lantern = commands.spawn((
        Name("Hanging Lantern".to_string()),
        LocalTransform {
            translation: lantern_at,
            ..Default::default()
        },
        GlobalTransform::default(),
        RigidBodyComponent::new_dynamic()
            .with_translation(lantern_at.x, lantern_at.y, lantern_at.z)
            .with_linear_damping(0.03)
            .with_angular_damping(0.05),
        ColliderComponent::new_cuboid(0.17, 0.22, 0.17).with_density(1.6),
    ));
    commands.add_tag_type::<Grabbable>(lantern);
    commands.add_tag_type::<LevelPiece>(lantern);

    let chain = spawn_mesh_at(
        commands,
        "Cylinder",
        Vec3::new(0.0, LANTERN_DROP * 0.5, 0.0),
        Vec3::new(0.05, LANTERN_DROP, 0.05),
    );
    attach_surface(commands, chain, SurfaceKind::Metal, phase);
    commands.set(chain, ChildOf(lantern));

    let cap = spawn_mesh_at(
        commands,
        "Cube",
        Vec3::new(0.0, 0.24, 0.0),
        Vec3::new(0.38, 0.07, 0.38),
    );
    attach_surface(commands, cap, SurfaceKind::Metal, phase);
    commands.set(cap, ChildOf(lantern));

    let base = spawn_mesh_at(
        commands,
        "Cube",
        Vec3::new(0.0, -0.24, 0.0),
        Vec3::new(0.3, 0.06, 0.3),
    );
    attach_surface(commands, base, SurfaceKind::Metal, phase);
    commands.set(base, ChildOf(lantern));

    for corner in [(-1.0_f32, -1.0_f32), (-1.0, 1.0), (1.0, -1.0), (1.0, 1.0)] {
        let bar = spawn_mesh_at(
            commands,
            "Cube",
            Vec3::new(corner.0 * 0.15, 0.0, corner.1 * 0.15),
            Vec3::new(0.035, 0.48, 0.035),
        );
        attach_surface(commands, bar, SurfaceKind::Metal, phase);
        commands.set(bar, ChildOf(lantern));
    }

    let bulb = spawn_mesh_at(
        commands,
        "Cube",
        Vec3::new(0.0, 0.07, 0.0),
        Vec3::new(0.13, 0.16, 0.13),
    );
    attach_material(
        commands,
        bulb,
        "ashfall_lantern_bulb",
        textures::solid([1.0, 0.9, 0.66, 1.0], theme::LAMP_HEAD_GLOW),
    );
    commands.set(bulb, ChildOf(lantern));

    let light = commands.spawn((
        Name("Lantern Light".to_string()),
        LocalTransform {
            translation: Vec3::new(0.0, -0.11, 0.0),
            ..Default::default()
        },
        GlobalTransform::default(),
        Light {
            light_type: LightType::Point,
            color: theme::LAMP_COLOR,
            intensity: theme::LANTERN_INTENSITY,
            range: theme::LANTERN_RANGE,
            cast_shadows: true,
            shadow_bias: theme::LANTERN_SHADOW_BIAS,
            shadow_softness: theme::LANTERN_SHADOW_SOFTNESS,
            ..Default::default()
        },
    ));
    commands.set(light, ChildOf(lantern));

    commands.queue(move |storage, resources| {
        create_spherical_joint(
            storage,
            resources.res_mut::<PhysicsWorld>(),
            hook,
            lantern,
            SphericalJoint::new()
                .with_local_anchor1(Vec3::zeros())
                .with_local_anchor2(Vec3::new(0.0, LANTERN_DROP, 0.0)),
        );
    });
}

fn spawn_crate(commands: &mut Commands, center: Vec3, phase: WorldPhase) {
    let position = center + Vec3::new(0.0, 0.6, 0.0);
    let entity = spawn_mesh_at(commands, "Cube", position, Vec3::new(1.2, 1.2, 1.2));
    attach_surface(commands, entity, SurfaceKind::Crate, phase);
    commands.add_tag_type::<LevelPiece>(entity);
    commands.set(
        entity,
        RigidBodyComponent::new_static().with_translation(position.x, position.y, position.z),
    );
    commands.set(entity, ColliderComponent::new_cuboid(0.6, 0.6, 0.6));

    let lid = spawn_mesh_at(
        commands,
        "Cube",
        position + Vec3::new(0.0, 0.63, 0.0),
        Vec3::new(1.3, 0.1, 1.3),
    );
    attach_surface(commands, lid, SurfaceKind::Wood, phase);
    commands.add_tag_type::<LevelPiece>(lid);
}

pub fn spawn_husk(
    commands: &mut Commands,
    center: Vec3,
    kind: HuskKind,
    otherworld: bool,
    slumped: bool,
) -> Entity {
    let profile = stats(kind);
    let position = center + Vec3::new(0.0, profile.height, 0.0);
    let root = commands.spawn((
        Name(format!("Husk {kind:?}")),
        LocalTransform {
            translation: position,
            ..Default::default()
        },
        GlobalTransform::default(),
    ));
    commands.add_tag_type::<LevelPiece>(root);
    commands.set(root, kind);
    commands.set(root, Health(profile.health));
    commands.set(
        root,
        if slumped {
            HuskState::Slumped
        } else {
            HuskState::Dormant
        },
    );
    commands.set(root, Timer(0.0));
    commands.set(root, Repath(0.0));
    commands.set(root, Scream(0.0));
    commands.set(root, NextTile(None));
    commands.set(root, Interest(None));
    commands.set(root, Stride(0.0));
    commands.set(
        root,
        Speed(profile.speed * if otherworld { 1.25 } else { 1.0 }),
    );
    commands.set(root, Floor(profile.height));
    commands.set(root, Moving(false));
    commands.set(root, Bled(false));
    commands.set(
        root,
        RigidBodyComponent::new_kinematic().with_translation(position.x, position.y, position.z),
    );
    commands.set(
        root,
        ColliderComponent::new_capsule(profile.height * 0.55, 0.35 * profile.width),
    );

    let (body_name, body_material) = textures::husk_body_material(kind, otherworld);
    let (head_name, head_material) = textures::husk_head_material(otherworld);
    let wide = profile.width;
    let tall = profile.height;
    let parts: [(&str, Vec3, Vec3, f32, f32); 6] = [
        (
            "Torso",
            Vec3::new(0.0, 0.15 * tall, 0.0),
            Vec3::new(0.55 * wide, 0.75 * tall, 0.3 * wide),
            0.0,
            0.0,
        ),
        (
            "Left Leg",
            Vec3::new(-0.15 * wide, -0.6 * tall, 0.0),
            Vec3::new(0.2 * wide, 0.8 * tall, 0.2 * wide),
            0.55,
            0.0,
        ),
        (
            "Right Leg",
            Vec3::new(0.15 * wide, -0.6 * tall, 0.0),
            Vec3::new(0.2 * wide, 0.8 * tall, 0.2 * wide),
            0.55,
            std::f32::consts::PI,
        ),
        (
            "Left Arm",
            Vec3::new(-0.4 * wide, 0.1 * tall, 0.05),
            Vec3::new(0.15 * wide, 0.7 * tall, 0.15 * wide),
            0.7,
            std::f32::consts::PI,
        ),
        (
            "Right Arm",
            Vec3::new(0.4 * wide, 0.1 * tall, 0.05),
            Vec3::new(0.15 * wide, 0.7 * tall, 0.15 * wide),
            0.7,
            0.0,
        ),
        (
            "Head",
            Vec3::new(0.0, 0.72 * tall, -0.06),
            Vec3::new(0.28 * wide, 0.32 * tall, 0.3 * wide),
            0.12,
            1.3,
        ),
    ];
    for (label, rest, scale, swing, phase) in parts {
        let part = spawn_mesh_at(commands, "Cube", rest, scale);
        commands.set(part, Name(label.to_string()));
        commands.set(part, ChildOf(root));
        commands.set(part, Body(root));
        commands.set(part, Rest(rest));
        commands.set(part, Swing(swing));
        commands.set(part, Phase(phase));
        if label == "Head" {
            attach_material(commands, part, head_name.clone(), head_material.clone());
        } else {
            attach_material(commands, part, body_name.clone(), body_material.clone());
        }
    }
    root
}

fn spawn_person(commands: &mut Commands, center: Vec3, index: usize) {
    let position = center + Vec3::new(0.0, 1.0, 0.0);
    let root = commands.spawn((
        Name(format!("Person {index}")),
        LocalTransform {
            translation: position,
            ..Default::default()
        },
        GlobalTransform::default(),
        NpcIndex(index),
        Health(PERSON_HEALTH),
        Alive(true),
        RigidBodyComponent::new_kinematic().with_translation(position.x, position.y, position.z),
        ColliderComponent::new_capsule(0.5, 0.32),
    ));
    commands.add_tag_type::<LevelPiece>(root);

    let (coat_name, coat) = textures::person_body_material();
    let (skin_name, skin) = textures::person_head_material();
    let parts: [(&str, Vec3, Vec3, bool); 6] = [
        (
            "Coat",
            Vec3::new(0.0, 0.12, 0.0),
            Vec3::new(0.5, 0.86, 0.3),
            false,
        ),
        (
            "Left Leg",
            Vec3::new(-0.13, -0.62, 0.0),
            Vec3::new(0.18, 0.78, 0.18),
            false,
        ),
        (
            "Right Leg",
            Vec3::new(0.13, -0.62, 0.0),
            Vec3::new(0.18, 0.78, 0.18),
            false,
        ),
        (
            "Left Arm",
            Vec3::new(-0.33, 0.08, 0.02),
            Vec3::new(0.14, 0.66, 0.16),
            false,
        ),
        (
            "Right Arm",
            Vec3::new(0.33, 0.08, 0.02),
            Vec3::new(0.14, 0.66, 0.16),
            false,
        ),
        (
            "Head",
            Vec3::new(0.0, 0.74, 0.0),
            Vec3::new(0.26, 0.3, 0.28),
            true,
        ),
    ];
    for (label, rest, scale, is_head) in parts {
        let part = spawn_mesh_at(commands, "Cube", rest, scale);
        commands.set(part, Name(label.to_string()));
        commands.set(part, ChildOf(root));
        if is_head {
            attach_material(commands, part, skin_name.clone(), skin.clone());
        } else {
            attach_material(commands, part, coat_name.clone(), coat.clone());
        }
    }
}

fn spawn_player(commands: &mut Commands, resources: &mut AshfallResources, start: Vec3) {
    let body = spawn_character_body(commands, start + Vec3::new(0.0, 1.05, 0.0), "Player");
    let eyes = spawn_first_person_camera(
        commands,
        body,
        Vec3::new(0.0, EYE_HEIGHT - 1.0, 0.0),
        FIELD_OF_VIEW,
    );
    commands.add_tag_type::<Eyes>(eyes);
    commands.queue(move |storage, _| {
        if let Some(transform) = storage.get_mut::<LocalTransform>(eyes) {
            transform.rotation = nalgebra_glm::quat_angle_axis(theme::START_FACING, &Vec3::y());
        }
    });
    let light = commands.spawn((
        Name("Flashlight".to_string()),
        LocalTransform {
            translation: Vec3::new(0.14, -0.14, 0.0),
            ..Default::default()
        },
        GlobalTransform::default(),
        Light {
            light_type: LightType::Spot,
            color: theme::FLASHLIGHT_COLOR,
            intensity: theme::FLASHLIGHT_INTENSITY,
            range: theme::FLASHLIGHT_RANGE,
            inner_cone_angle: theme::FLASHLIGHT_INNER,
            outer_cone_angle: theme::FLASHLIGHT_OUTER,
            cast_shadows: true,
            shadow_bias: theme::FLASHLIGHT_SHADOW_BIAS,
            shadow_softness: theme::FLASHLIGHT_SHADOW_SOFTNESS,
            ..Default::default()
        },
    ));
    commands.set(light, Battery(MAX_BATTERY));
    commands.set(light, Drain(BATTERY_DRAIN));
    commands.set(light, Lit(true));
    commands.set(light, Beam(theme::FLASHLIGHT_INTENSITY));
    commands.add_tag_type::<Flashlight>(light);
    commands.set(light, ChildOf(eyes));
    resources.player.flashlight = Some(light);
    commands.queue(move |storage, _| {
        if let Some(controller) = storage.get_mut::<CharacterControllerComponent>(body) {
            controller.max_speed = PLAYER_SPEED;
            controller.sprint_speed_multiplier = PLAYER_SPRINT;
            controller.jump_impulse = 0.0;
            controller.crouch_enabled = true;
            controller.crouch_speed_multiplier = CROUCH_SPEED_SCALE;
        }
        storage.set(body, Exposure(1.0));
        storage.add_tag_type::<Player>(body);
        storage.add_tag_type::<LevelPiece>(body);
    });
    resources.player.hand = nalgebra_glm::quat_angle_axis(theme::START_FACING, &Vec3::y());
    resources.player.head = resources.player.hand;
    resources.player.body = Some(body);
    resources.player.eyes = Some(eyes);
}
