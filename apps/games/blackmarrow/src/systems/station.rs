use crate::commands::{attach_material, attach_surface};
use crate::components::{Handle, Heft, Item, Locked, Readable};
use crate::contract::{Fixture, ItemKind, Surface};
use crate::data::props;
use crate::data::rooms::{
    DOOR_CLEARANCE, DOOR_HEIGHT, DOOR_THICKNESS, DOOR_WIDTH, DOORWAYS, Doorway, ROOM_HEIGHT, ROOMS,
    Room, SLAB_THICKNESS, WALL_THICKNESS, WALLS, Wall,
};
use crate::data::rules::{DOOR_SWING, DRAWER_TRAVEL};
use crate::data::surfaces::{KEY_TINT, TILE};
use crate::data::theme;
use crate::data::tools::{
    RACK_BOARD_AT, RACK_BOARD_SIZE, RACK_BRACKET_SIZE, RACK_BRACKETS, RACK_HOOK_SIZE, RACK_HOOKS,
    RACK_SHELF_AT, RACK_SHELF_SIZE, TOOLS, Tool,
};
use crate::queries::{plain, surface_material, tile_material, tile_name, tiled};
use crate::resources::BlackmarrowResources;
use crate::tags::{Eyes, Player, Station, Torch};
use nightshade::prelude::*;
use nightshade_camera::commands::scene::spawn_first_person_camera;
use nightshade_flashlight::{Battery, Beam, Drain, Lit};
use nightshade_flicker::{FlickerBase, Flickering};
use nightshade_graphics::commands::builtins::spawn_mesh_at;
use nightshade_physics::commands::joints::{
    JointAxisDirection, JointLimits, PrismaticJoint, RevoluteJoint, SphericalJoint,
    create_prismatic_joint, create_revolute_joint, create_spherical_joint,
};
use nightshade_physics::commands::{
    spawn_character_body, spawn_dynamic_physics_cube_with_material,
    spawn_static_physics_cube_with_material,
};
use nightshade_physics::contract::components::{
    CharacterControllerComponent, ColliderComponent, RigidBodyComponent,
};
use nightshade_physics::resources::PhysicsWorld;
use nightshade_render::components::light::{Light, LightType};
use nightshade_scene::prelude::*;
use nightshade_senses::Exposure;

pub fn build(mut resources: ResMut<BlackmarrowResources>, mut commands: Commands) {
    if resources.run.built {
        return;
    }
    resources.run.built = true;

    for room in ROOMS {
        raise_slabs(&mut commands, room);
    }
    for wall in WALLS {
        raise_wall(&mut commands, wall);
    }
    for doorway in DOORWAYS {
        raise_doorway(&mut commands, doorway);
    }
    raise_cabinet(&mut commands);
    raise_spare_cell(&mut commands);
    for hook_at in props::HANGING_LAMPS {
        hang_lamp(&mut commands, *hook_at);
    }
    raise_rack(&mut commands);
    raise_note(&mut commands);
    raise_player(&mut commands, &mut resources);
}

fn hang_lamp(commands: &mut Commands<'_>, hook_at: Vec3) {
    let hook = commands.spawn((
        Name("Lamp Hook".to_string()),
        LocalTransform {
            translation: hook_at,
            ..Default::default()
        },
        GlobalTransform::default(),
        RigidBodyComponent::new_static().with_translation(hook_at.x, hook_at.y, hook_at.z),
        ColliderComponent::new_cuboid(0.06, 0.06, 0.06),
    ));
    commands.add_tag_type::<Station>(hook);
    let plate = spawn_mesh_at(
        commands,
        "Cube",
        hook_at + Vec3::new(0.0, 0.05, 0.0),
        Vec3::new(0.3, 0.1, 0.3),
    );
    attach_surface(commands, plate, Surface::Metal, 0.3);
    commands.add_tag_type::<Station>(plate);

    let drop = props::HANGING_DROP;
    let lantern_at = hook_at + Vec3::new(props::HANGING_NUDGE, -drop, 0.0);
    let lantern = commands.spawn((
        Name("Hanging Lamp".to_string()),
        LocalTransform {
            translation: lantern_at,
            ..Default::default()
        },
        GlobalTransform::default(),
        RigidBodyComponent::new_dynamic()
            .with_translation(lantern_at.x, lantern_at.y, lantern_at.z)
            .with_linear_damping(0.05)
            .with_angular_damping(0.08),
        ColliderComponent::new_cuboid(0.16, 0.22, 0.16).with_density(1.6),
    ));
    commands.set(lantern, Handle(Fixture::Loose));
    commands.set(lantern, Heft(2.0));
    commands.add_tag_type::<Station>(lantern);

    let cage = [
        (
            "Cylinder",
            Vec3::new(0.0, drop * 0.5, 0.0),
            Vec3::new(0.04, drop, 0.04),
        ),
        (
            "Cube",
            Vec3::new(0.0, 0.24, 0.0),
            Vec3::new(0.36, 0.06, 0.36),
        ),
        (
            "Cube",
            Vec3::new(0.0, -0.24, 0.0),
            Vec3::new(0.28, 0.05, 0.28),
        ),
        (
            "Cube",
            Vec3::new(-0.14, 0.0, -0.14),
            Vec3::new(0.03, 0.48, 0.03),
        ),
        (
            "Cube",
            Vec3::new(-0.14, 0.0, 0.14),
            Vec3::new(0.03, 0.48, 0.03),
        ),
        (
            "Cube",
            Vec3::new(0.14, 0.0, -0.14),
            Vec3::new(0.03, 0.48, 0.03),
        ),
        (
            "Cube",
            Vec3::new(0.14, 0.0, 0.14),
            Vec3::new(0.03, 0.48, 0.03),
        ),
    ];
    for (mesh, offset, scale) in cage {
        let piece = spawn_mesh_at(commands, mesh, offset, scale);
        attach_surface(commands, piece, Surface::Metal, scale.y.max(scale.x));
        commands.set(piece, ChildOf(lantern));
        commands.add_tag_type::<Station>(piece);
    }
    let bulb = spawn_mesh_at(
        commands,
        "Cube",
        Vec3::new(0.0, 0.06, 0.0),
        Vec3::new(0.12, 0.15, 0.12),
    );
    attach_material(
        commands,
        bulb,
        "blackmarrow_bulb",
        plain(theme::BULB_TINT, theme::BULB_GLOW, 0.4),
    );
    commands.set(bulb, ChildOf(lantern));
    commands.add_tag_type::<Station>(bulb);

    let glow = commands.spawn((
        Name("Hanging Light".to_string()),
        LocalTransform {
            translation: Vec3::new(0.0, -0.1, 0.0),
            ..Default::default()
        },
        GlobalTransform::default(),
        Light {
            light_type: LightType::Point,
            color: theme::LAMP_COLOR,
            intensity: theme::HANGING_INTENSITY,
            range: theme::HANGING_RANGE,
            cast_shadows: true,
            shadow_bias: theme::LAMP_SHADOW_BIAS,
            shadow_softness: theme::LAMP_SHADOW_SOFTNESS,
            ..Default::default()
        },
    ));
    commands.set(glow, ChildOf(lantern));
    commands.set(glow, FlickerBase(theme::HANGING_INTENSITY));
    commands.set(glow, Flickering(1.0));
    commands.add_tag_type::<Station>(glow);

    commands.queue(move |storage, resources| {
        create_spherical_joint(
            storage,
            resources.res_mut::<PhysicsWorld>(),
            hook,
            lantern,
            SphericalJoint::new()
                .with_local_anchor1(Vec3::zeros())
                .with_local_anchor2(Vec3::new(0.0, drop, 0.0)),
        );
    });
}

fn raise_rack(commands: &mut Commands<'_>) {
    let mut fixtures = vec![
        (RACK_BOARD_AT, RACK_BOARD_SIZE, Surface::Rust),
        (RACK_SHELF_AT, RACK_SHELF_SIZE, Surface::Metal),
    ];
    for z in RACK_BRACKETS {
        fixtures.push((
            Vec3::new(RACK_SHELF_AT.x, RACK_SHELF_AT.y - RACK_SHELF_SIZE.y, *z),
            RACK_BRACKET_SIZE,
            Surface::Metal,
        ));
    }
    for at in RACK_HOOKS {
        fixtures.push((*at, RACK_HOOK_SIZE, Surface::Metal));
    }
    for (center, size, surface) in fixtures {
        let span = size.x.max(size.y).max(size.z);
        let entity = spawn_static_physics_cube_with_material(
            commands,
            center,
            size,
            surface_material(surface, span),
        );
        commands.add_tag_type::<Station>(entity);
    }
    for tool in TOOLS {
        lay_tool(commands, tool);
    }
    lay_glowstick(commands);
}

fn lay_tool(commands: &mut Commands<'_>, tool: &Tool) {
    let lying = nalgebra_glm::quat_angle_axis(std::f32::consts::FRAC_PI_2, &Vec3::x());
    let center = Vec3::new(
        tool.at.x,
        RACK_SHELF_AT.y + RACK_SHELF_SIZE.y * 0.5 + tool.half.z + 0.01,
        tool.at.z,
    );
    let body = commands.spawn((
        Name(tool.name.to_string()),
        LocalTransform {
            translation: center,
            rotation: lying,
            ..Default::default()
        },
        GlobalTransform::default(),
        RigidBodyComponent::new_dynamic()
            .with_translation(center.x, center.y, center.z)
            .with_rotation(lying.i, lying.j, lying.k, lying.w)
            .with_mass(tool.mass)
            .with_linear_damping(0.2)
            .with_angular_damping(0.5),
        ColliderComponent::new_cuboid(tool.half.x, tool.half.y, tool.half.z).with_friction(0.8),
    ));
    commands.set(body, Handle(Fixture::Loose));
    commands.set(body, Heft(tool.mass));
    commands.add_tag_type::<Station>(body);
    for part in tool.parts {
        let piece = spawn_mesh_at(commands, part.mesh, part.offset, part.scale);
        attach_surface(
            commands,
            piece,
            part.surface,
            part.scale.y.max(part.scale.x),
        );
        commands.set(piece, ChildOf(body));
        commands.add_tag_type::<Station>(piece);
    }
}

fn lay_glowstick(commands: &mut Commands<'_>) {
    let lying = nalgebra_glm::quat_angle_axis(std::f32::consts::FRAC_PI_2, &Vec3::x());
    let at = props::GLOWSTICK_AT;
    let stick = commands.spawn((
        Name("Glowstick".to_string()),
        LocalTransform {
            translation: at,
            rotation: lying,
            ..Default::default()
        },
        GlobalTransform::default(),
        RigidBodyComponent::new_dynamic()
            .with_translation(at.x, at.y, at.z)
            .with_rotation(lying.i, lying.j, lying.k, lying.w)
            .with_mass(0.12)
            .with_linear_damping(0.2)
            .with_angular_damping(0.6),
        ColliderComponent::new_cylinder(props::GLOWSTICK_HALF_LENGTH, props::GLOWSTICK_RADIUS)
            .with_friction(0.9),
    ));
    commands.set(stick, Handle(Fixture::Loose));
    commands.set(stick, Heft(0.12));
    commands.add_tag_type::<Station>(stick);

    let tube = spawn_mesh_at(
        commands,
        "Cylinder",
        Vec3::zeros(),
        Vec3::new(
            props::GLOWSTICK_RADIUS * 2.0,
            props::GLOWSTICK_HALF_LENGTH * 2.0,
            props::GLOWSTICK_RADIUS * 2.0,
        ),
    );
    attach_material(
        commands,
        tube,
        "blackmarrow_glowstick",
        plain(theme::GLOWSTICK_TINT, theme::GLOWSTICK_GLOW, 0.5),
    );
    commands.set(tube, ChildOf(stick));
    commands.add_tag_type::<Station>(tube);

    let shine = commands.spawn((
        Name("Glowstick Light".to_string()),
        LocalTransform::default(),
        GlobalTransform::default(),
        Light {
            light_type: LightType::Point,
            color: theme::GLOWSTICK_COLOR,
            intensity: theme::GLOWSTICK_INTENSITY,
            range: theme::GLOWSTICK_RANGE,
            cast_shadows: false,
            ..Default::default()
        },
    ));
    commands.set(shine, ChildOf(stick));
    commands.add_tag_type::<Station>(shine);
}

fn raise_note(commands: &mut Commands<'_>) {
    let page = spawn_dynamic_physics_cube_with_material(
        commands,
        props::NOTE_AT,
        props::NOTE_SIZE,
        0.05,
        plain(theme::PAPER_TINT, [0.0, 0.0, 0.0], 0.95),
    );
    commands.set(page, Handle(Fixture::Loose));
    commands.set(page, Heft(0.05));
    commands.set(page, Readable(0));
    commands.add_tag_type::<Station>(page);
}

fn raise_slabs(commands: &mut Commands<'_>, room: &Room) {
    let width = room.x1 - room.x0;
    let depth = room.z1 - room.z0;
    let center_x = (room.x0 + room.x1) * 0.5;
    let center_z = (room.z0 + room.z1) * 0.5;

    let floor_center = Vec3::new(center_x, -SLAB_THICKNESS * 0.5, center_z);
    let ceiling_center = Vec3::new(center_x, ROOM_HEIGHT + SLAB_THICKNESS * 0.5, center_z);
    let size = Vec3::new(width, SLAB_THICKNESS, depth);
    raise_collider(commands, floor_center, size);
    raise_facet(commands, floor_center, size, room.floor, RIGHT, BACK, &[]);
    raise_collider(commands, ceiling_center, size);
    raise_facet(
        commands,
        ceiling_center,
        size,
        room.ceiling,
        RIGHT,
        AHEAD,
        &[],
    );
}

const RIGHT: Vec3 = Vec3::new(1.0, 0.0, 0.0);
const LEFT: Vec3 = Vec3::new(-1.0, 0.0, 0.0);
const UP: Vec3 = Vec3::new(0.0, 1.0, 0.0);
const AHEAD: Vec3 = Vec3::new(0.0, 0.0, 1.0);
const BACK: Vec3 = Vec3::new(0.0, 0.0, -1.0);

fn raise_collider(commands: &mut Commands<'_>, center: Vec3, size: Vec3) {
    let body = commands.spawn((
        Name("Slab".to_string()),
        LocalTransform {
            translation: center,
            ..Default::default()
        },
        GlobalTransform::default(),
        RigidBodyComponent::new_static().with_translation(center.x, center.y, center.z),
        ColliderComponent::new_cuboid(size.x * 0.5, size.y * 0.5, size.z * 0.5),
    ));
    commands.add_tag_type::<Station>(body);
}

fn raise_facet(
    commands: &mut Commands<'_>,
    center: Vec3,
    size: Vec3,
    face: Surface,
    across: Vec3,
    up: Vec3,
    breaks: &[f32],
) {
    let normal = nalgebra_glm::cross(&across, &up);
    let reach_total = nalgebra_glm::dot(&size, &across.abs());
    let lift_total = nalgebra_glm::dot(&size, &up.abs());
    let depth = nalgebra_glm::dot(&size, &normal.abs()) * 0.5;
    let plane = center + normal * depth;
    let along_center = nalgebra_glm::dot(&plane, &across);
    let rise_center = nalgebra_glm::dot(&plane, &up);
    let along_cuts = cuts(
        along_center - reach_total * 0.5,
        along_center + reach_total * 0.5,
        &[],
    );
    let rise_cuts = cuts(
        rise_center - lift_total * 0.5,
        rise_center + lift_total * 0.5,
        breaks,
    );
    let rotation =
        nalgebra_glm::mat3_to_quat(&nalgebra_glm::Mat3::from_columns(&[across, up, normal]));

    for along in along_cuts.windows(2) {
        let reach = along[1] - along[0];
        for rise in rise_cuts.windows(2) {
            let lift = rise[1] - rise[0];
            let tile_center = plane
                + across * ((along[0] + along[1]) * 0.5 - along_center)
                + up * ((rise[0] + rise[1]) * 0.5 - rise_center);
            let offset = [
                along[0].rem_euclid(TILE),
                (-rise[0] - lift).rem_euclid(TILE),
            ];
            let tile = spawn_mesh_at(commands, "Quad", tile_center, Vec3::new(reach, lift, 1.0));
            commands.queue(move |storage, _| {
                if let Some(transform) = storage.get_mut::<LocalTransform>(tile) {
                    transform.rotation = rotation;
                }
            });
            attach_material(
                commands,
                tile,
                tile_name(face, reach, lift, offset),
                tile_material(face, reach, lift, offset),
            );
            commands.add_tag_type::<Station>(tile);
        }
    }
}

fn cuts(from: f32, to: f32, extra: &[f32]) -> Vec<f32> {
    let mut marks = vec![from];
    let mut mark = (from / TILE).floor() * TILE + TILE;
    while mark < to - 0.001 {
        if mark > from + 0.001 {
            marks.push(mark);
        }
        mark += TILE;
    }
    for &value in extra {
        if value > from + 0.001 && value < to - 0.001 {
            marks.push(value);
        }
    }
    marks.push(to);
    marks.sort_by(|left, right| left.total_cmp(right));
    marks.dedup_by(|left, right| (*left - *right).abs() < 0.001);
    marks
}

fn raise_wall(commands: &mut Commands<'_>, wall: &Wall) {
    let half = DOOR_WIDTH * 0.5 + DOOR_CLEARANCE;
    let mut door_cuts: Vec<f32> = DOORWAYS
        .iter()
        .filter(|doorway| {
            doorway.along_x == wall.along_x
                && (doorway.fixed - wall.fixed).abs() < 0.01
                && doorway.at > wall.from
                && doorway.at < wall.to
        })
        .map(|doorway| doorway.at)
        .collect();
    door_cuts.sort_by(|left, right| left.total_cmp(right));

    let mut cursor = wall.from;
    let mut jamb_behind = false;
    for cut in &door_cuts {
        raise_segment(
            commands,
            wall,
            cursor,
            cut - half,
            0.0,
            ROOM_HEIGHT,
            (jamb_behind, true),
        );
        raise_segment(
            commands,
            wall,
            cut - half,
            cut + half,
            DOOR_HEIGHT,
            ROOM_HEIGHT,
            (false, false),
        );
        cursor = cut + half;
        jamb_behind = true;
    }
    raise_segment(
        commands,
        wall,
        cursor,
        wall.to,
        0.0,
        ROOM_HEIGHT,
        (jamb_behind, false),
    );
}

fn raise_segment(
    commands: &mut Commands<'_>,
    wall: &Wall,
    from: f32,
    to: f32,
    low: f32,
    high: f32,
    jambs: (bool, bool),
) {
    let length = to - from;
    let height = high - low;
    if length <= 0.01 || height <= 0.01 {
        return;
    }
    let middle = (from + to) * 0.5;
    let center_y = (low + high) * 0.5;
    let (center, size) = if wall.along_x {
        (
            Vec3::new(middle, center_y, wall.fixed),
            Vec3::new(length, height, WALL_THICKNESS),
        )
    } else {
        (
            Vec3::new(wall.fixed, center_y, middle),
            Vec3::new(WALL_THICKNESS, height, length),
        )
    };
    raise_collider(commands, center, size);
    let breaks = [DOOR_HEIGHT];
    let (front, back, start_end, stop_end, underside) = if wall.along_x {
        (RIGHT, LEFT, AHEAD, BACK, (RIGHT, AHEAD))
    } else {
        (BACK, AHEAD, RIGHT, LEFT, (AHEAD, LEFT))
    };
    raise_facet(commands, center, size, wall.face, front, UP, &breaks);
    raise_facet(commands, center, size, wall.face, back, UP, &breaks);
    if jambs.0 {
        raise_facet(commands, center, size, wall.face, start_end, UP, &breaks);
    }
    if jambs.1 {
        raise_facet(commands, center, size, wall.face, stop_end, UP, &breaks);
    }
    if low > 0.0 {
        raise_facet(
            commands,
            center,
            size,
            wall.face,
            underside.0,
            underside.1,
            &[],
        );
    }
}

fn raise_doorway(commands: &mut Commands<'_>, doorway: &Doorway) {
    let center = leaf_center(doorway);
    if let Some(lock) = doorway.lock {
        let size = leaf_size(doorway);
        let entity = spawn_static_physics_cube_with_material(
            commands,
            center,
            size,
            surface_material(Surface::Door, DOOR_HEIGHT),
        );
        commands.set(entity, Locked(Some(lock)));
        commands.set(entity, Handle(Fixture::Door));
        commands.add_tag_type::<Station>(entity);
        return;
    }
    hang_leaf(commands, doorway);
}

pub fn hang_leaf(commands: &mut Commands<'_>, doorway: &Doorway) -> Entity {
    let center = leaf_center(doorway);
    let size = leaf_size(doorway);
    let leaf = spawn_dynamic_physics_cube_with_material(
        commands,
        center,
        size,
        34.0,
        surface_material(Surface::Door, DOOR_HEIGHT),
    );
    commands.set(leaf, Handle(Fixture::Door));
    commands.set(leaf, Heft(34.0));
    commands.set(leaf, Locked(None));
    commands.add_tag_type::<Station>(leaf);

    let reach = DOOR_WIDTH * 0.5;
    let toward = if doorway.hinge_low { -reach } else { reach };
    let hinge = if doorway.along_x {
        Vec3::new(doorway.at + toward, center.y, doorway.fixed)
    } else {
        Vec3::new(doorway.fixed, center.y, doorway.at + toward)
    };
    let local = if doorway.along_x {
        Vec3::new(toward, 0.0, 0.0)
    } else {
        Vec3::new(0.0, 0.0, toward)
    };

    let post = commands.spawn((
        Name("Hinge".to_string()),
        LocalTransform {
            translation: hinge,
            ..Default::default()
        },
        GlobalTransform::default(),
        nightshade_physics::contract::components::RigidBodyComponent::new_static()
            .with_translation(hinge.x, hinge.y, hinge.z),
    ));
    commands.add_tag_type::<Station>(post);

    commands.queue(move |storage, resources| {
        create_revolute_joint(
            storage,
            resources.res_mut::<PhysicsWorld>(),
            post,
            leaf,
            RevoluteJoint::new(JointAxisDirection::Y)
                .with_local_anchor1(Vec3::zeros())
                .with_local_anchor2(local)
                .with_limits(JointLimits::symmetric(DOOR_SWING)),
        );
        if let Some(body) =
            storage.get_mut::<nightshade_physics::contract::components::RigidBodyComponent>(leaf)
        {
            body.angular_damping = 2.6;
            body.linear_damping = 1.4;
        }
    });
    leaf
}

fn leaf_center(doorway: &Doorway) -> Vec3 {
    if doorway.along_x {
        Vec3::new(doorway.at, DOOR_HEIGHT * 0.5, doorway.fixed)
    } else {
        Vec3::new(doorway.fixed, DOOR_HEIGHT * 0.5, doorway.at)
    }
}

fn leaf_size(doorway: &Doorway) -> Vec3 {
    if doorway.along_x {
        Vec3::new(DOOR_WIDTH, DOOR_HEIGHT, DOOR_THICKNESS)
    } else {
        Vec3::new(DOOR_THICKNESS, DOOR_HEIGHT, DOOR_WIDTH)
    }
}

fn raise_cabinet(commands: &mut Commands<'_>) {
    let base = props::CABINET_AT;
    let width = props::CABINET_WIDTH;
    let height = props::CABINET_HEIGHT;
    let depth = props::CABINET_DEPTH;
    let shell = props::CABINET_SHELL;

    let panels = [
        (
            Vec3::new(base.x, shell * 0.5, base.z),
            Vec3::new(width, shell, depth),
        ),
        (
            Vec3::new(base.x, height - shell * 0.5, base.z),
            Vec3::new(width, shell, depth),
        ),
        (
            Vec3::new(base.x - width * 0.5 + shell * 0.5, height * 0.5, base.z),
            Vec3::new(shell, height, depth),
        ),
        (
            Vec3::new(base.x + width * 0.5 - shell * 0.5, height * 0.5, base.z),
            Vec3::new(shell, height, depth),
        ),
        (
            Vec3::new(base.x, height * 0.5, base.z + depth * 0.5 - shell * 0.5),
            Vec3::new(width, height, shell),
        ),
    ];
    for (center, size) in panels {
        let span = size.x.max(size.y).max(size.z);
        let entity = spawn_static_physics_cube_with_material(
            commands,
            center,
            size,
            surface_material(Surface::Metal, span),
        );
        commands.add_tag_type::<Station>(entity);
    }

    let tray_size = Vec3::new(width - shell * 2.2, props::TRAY_HEIGHT, depth - shell * 1.4);
    let tray_center = Vec3::new(
        base.x,
        shell + props::TRAY_HEIGHT * 0.5,
        base.z - shell * 0.2,
    );
    let tray = spawn_dynamic_physics_cube_with_material(
        commands,
        tray_center,
        tray_size,
        9.0,
        surface_material(Surface::Drawer, tray_size.x),
    );
    commands.set(tray, Handle(Fixture::Drawer));
    commands.set(tray, Heft(9.0));
    commands.add_tag_type::<Station>(tray);

    let slide = commands.spawn((
        Name("Drawer Slide".to_string()),
        LocalTransform {
            translation: tray_center,
            ..Default::default()
        },
        GlobalTransform::default(),
        nightshade_physics::contract::components::RigidBodyComponent::new_static()
            .with_translation(tray_center.x, tray_center.y, tray_center.z),
    ));
    commands.add_tag_type::<Station>(slide);

    commands.queue(move |storage, resources| {
        create_prismatic_joint(
            storage,
            resources.res_mut::<PhysicsWorld>(),
            slide,
            tray,
            PrismaticJoint::new(JointAxisDirection::Z)
                .with_local_anchor1(Vec3::zeros())
                .with_local_anchor2(Vec3::zeros())
                .with_limits(JointLimits::new(-DRAWER_TRAVEL, 0.0)),
        );
        if let Some(body) =
            storage.get_mut::<nightshade_physics::contract::components::RigidBodyComponent>(tray)
        {
            body.linear_damping = 3.2;
            body.angular_damping = 4.0;
        }
    });

    let key_at = Vec3::new(
        base.x,
        shell + props::TRAY_HEIGHT + props::KEY_SIZE.y,
        base.z - shell * 0.2,
    );
    let key = spawn_dynamic_physics_cube_with_material(
        commands,
        key_at,
        props::KEY_SIZE,
        0.3,
        tiled(crate::data::surfaces::METAL, KEY_TINT, 1.0),
    );
    commands.set(key, Handle(Fixture::Loose));
    commands.set(key, Heft(0.3));
    commands.set(key, Item(ItemKind::ShaftKey));
    commands.add_tag_type::<Station>(key);
}

fn raise_spare_cell(commands: &mut Commands<'_>) {
    let cell = spawn_dynamic_physics_cube_with_material(
        commands,
        props::CELL_AT,
        Vec3::new(0.09, 0.15, 0.09),
        0.25,
        tiled(crate::data::surfaces::RUST, [0.9, 0.78, 0.4, 1.0], 1.0),
    );
    commands.set(cell, Handle(Fixture::Loose));
    commands.set(cell, Heft(0.25));
    commands.set(cell, Item(ItemKind::Cell));
    commands.add_tag_type::<Station>(cell);
}

fn raise_player(commands: &mut Commands<'_>, resources: &mut BlackmarrowResources) {
    let start = theme::PLAYER_START + Vec3::new(0.0, 1.05, 0.0);
    let body = spawn_character_body(commands, start, "Diver");
    let eyes = spawn_first_person_camera(
        commands,
        body,
        Vec3::new(0.0, crate::data::rules::EYE_HEIGHT - 1.0, 0.0),
        crate::data::rules::FIELD_OF_VIEW,
    );
    commands.add_tag_type::<Eyes>(eyes);

    let facing = nalgebra_glm::quat_angle_axis(theme::PLAYER_FACING, &Vec3::y());
    commands.queue(move |storage, _| {
        if let Some(transform) = storage.get_mut::<LocalTransform>(eyes) {
            transform.rotation = facing;
        }
    });

    let torch = commands.spawn((
        Name("Torch".to_string()),
        LocalTransform {
            translation: theme::TORCH_OFFSET,
            ..Default::default()
        },
        GlobalTransform::default(),
        Light {
            light_type: LightType::Spot,
            color: theme::TORCH_COLOR,
            intensity: theme::TORCH_INTENSITY,
            range: theme::TORCH_RANGE,
            inner_cone_angle: theme::TORCH_INNER,
            outer_cone_angle: theme::TORCH_OUTER,
            cast_shadows: true,
            shadow_bias: theme::TORCH_SHADOW_BIAS,
            shadow_softness: theme::TORCH_SHADOW_SOFTNESS,
            ..Default::default()
        },
    ));
    commands.set(torch, Battery(crate::data::rules::CELL_FULL));
    commands.set(torch, Drain(crate::data::rules::CELL_DRAIN));
    commands.set(torch, Lit(true));
    commands.set(torch, Beam(theme::TORCH_INTENSITY));
    commands.set(torch, ChildOf(eyes));
    commands.add_tag_type::<Torch>(torch);

    commands.queue(move |storage, _| {
        if let Some(controller) = storage.get_mut::<CharacterControllerComponent>(body) {
            controller.max_speed = crate::data::rules::WALK_SPEED;
            controller.sprint_speed_multiplier = crate::data::rules::SPRINT_SCALE;
            controller.crouch_speed_multiplier = crate::data::rules::CROUCH_SCALE;
            controller.crouch_enabled = true;
            controller.jump_impulse = 0.0;
        }
        storage.set(body, Exposure(1.0));
        storage.add_tag_type::<Player>(body);
    });

    resources.player.body = Some(body);
    resources.player.eyes = Some(eyes);
    resources.player.torch = Some(torch);
    resources.player.head = facing;
}
