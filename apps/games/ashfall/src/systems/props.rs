use crate::commands::attach_surface;
use crate::components::{Explosive, Heft, SurfaceKind};
use crate::data::level::{STOREY, TILE};
use crate::data::props::{PLACEMENTS, Placement, PropKind};
use crate::data::rules::LOOSE_SPIN_DAMPING;
use crate::resources::WorldPhase;
use crate::tags::{Fragile, Grabbable, LevelPiece};
use nightshade::prelude::*;
use nightshade_graphics::commands::builtins::spawn_mesh_at;
use nightshade_physics::contract::components::{ColliderComponent, RigidBodyComponent};
use nightshade_scene::prelude::*;

struct Kit<'a, 'c> {
    commands: &'a mut Commands<'c>,
    root: Entity,
    phase: WorldPhase,
    lift: f32,
}

impl Kit<'_, '_> {
    fn box_part(&mut self, offset: Vec3, scale: Vec3, kind: SurfaceKind) {
        let offset = self.settle(offset);
        let entity = spawn_mesh_at(self.commands, "Cube", offset, scale);
        self.commands.set(entity, ChildOf(self.root));
        attach_surface(self.commands, entity, kind, self.phase);
    }

    fn settle(&self, offset: Vec3) -> Vec3 {
        Vec3::new(offset.x, offset.y - self.lift, offset.z)
    }

    fn tube_part(&mut self, offset: Vec3, scale: Vec3, kind: SurfaceKind) {
        let offset = self.settle(offset);
        let entity = spawn_mesh_at(self.commands, "Cylinder", offset, scale);
        self.commands.set(entity, ChildOf(self.root));
        attach_surface(self.commands, entity, kind, self.phase);
    }

    fn wheel(&mut self, offset: Vec3) {
        let offset = self.settle(offset);
        let entity = spawn_mesh_at(
            self.commands,
            "Cylinder",
            offset,
            Vec3::new(0.66, 0.3, 0.66),
        );
        self.commands.set(entity, ChildOf(self.root));
        self.commands.queue(move |storage, _| {
            if let Some(transform) = storage.get_mut::<LocalTransform>(entity) {
                transform.rotation =
                    nalgebra_glm::quat_angle_axis(std::f32::consts::FRAC_PI_2, &Vec3::z());
            }
        });
        attach_surface(self.commands, entity, SurfaceKind::Grille, self.phase);
    }

    fn tilted_part(&mut self, offset: Vec3, scale: Vec3, yaw: f32, kind: SurfaceKind) {
        let offset = self.settle(offset);
        let entity = spawn_mesh_at(self.commands, "Cube", offset, scale);
        self.commands.set(entity, ChildOf(self.root));
        self.commands.queue(move |storage, _| {
            if let Some(transform) = storage.get_mut::<LocalTransform>(entity) {
                transform.rotation = nalgebra_glm::quat_angle_axis(yaw, &Vec3::y());
            }
        });
        attach_surface(self.commands, entity, kind, self.phase);
    }
}

pub fn spawn_all(commands: &mut Commands, phase: WorldPhase) {
    for placement in PLACEMENTS {
        spawn_one(commands, placement, phase);
    }
}

fn spawn_one(commands: &mut Commands, placement: &Placement, phase: WorldPhase) {
    let level = if placement.upper { STOREY } else { 0.0 };
    let loose = loose(placement.kind);
    let lift = loose.map(|(_, height, _)| height).unwrap_or(0.0);
    let position = Vec3::new(placement.col * TILE, level + lift, placement.row * TILE);
    let yaw = placement.yaw_degrees.to_radians();
    let rotation = nalgebra_glm::quat_angle_axis(yaw, &Vec3::y());
    let root = commands.spawn((
        Name(format!("{:?}", placement.kind)),
        LocalTransform {
            translation: position,
            rotation,
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
        GlobalTransform::default(),
    ));
    commands.add_tag_type::<LevelPiece>(root);

    if let Some((half, _, density)) = loose {
        commands.set(
            root,
            RigidBodyComponent::new_dynamic()
                .with_translation(position.x, position.y, position.z)
                .with_rotation(rotation.i, rotation.j, rotation.k, rotation.w)
                .with_linear_damping(0.25)
                .with_angular_damping(LOOSE_SPIN_DAMPING),
        );
        commands.set(
            root,
            ColliderComponent::new_cuboid(half.x, half.y, half.z)
                .with_friction(0.75)
                .with_density(density),
        );
        commands.add_tag_type::<Grabbable>(root);
        commands.set(
            root,
            Heft {
                mass: 8.0 * half.x * half.y * half.z * density,
            },
        );
    } else if let Some((half, center_height)) = footprint(placement.kind) {
        let center = Vec3::new(position.x, level + center_height, position.z);
        let collider = commands.spawn((
            Name("Prop Collider".to_string()),
            LocalTransform {
                translation: center,
                rotation,
                scale: Vec3::new(1.0, 1.0, 1.0),
            },
            GlobalTransform::default(),
            RigidBodyComponent::new_static()
                .with_translation(center.x, center.y, center.z)
                .with_rotation(rotation.i, rotation.j, rotation.k, rotation.w),
            ColliderComponent::new_cuboid(half.x, half.y, half.z).with_friction(0.6),
        ));
        commands.add_tag_type::<LevelPiece>(collider);
    }

    let mut kit = Kit {
        commands,
        root,
        phase,
        lift,
    };
    match placement.kind {
        PropKind::Car => car(&mut kit),
        PropKind::Dumpster => dumpster(&mut kit),
        PropKind::Barrel => barrel(&mut kit),
        PropKind::Bench => bench(&mut kit),
        PropKind::Fence => fence(&mut kit),
        PropKind::Sign => sign(&mut kit),
        PropKind::Rubble => rubble(&mut kit),
        PropKind::Pillar => pillar(&mut kit),
        PropKind::Desk => desk(&mut kit),
        PropKind::Locker => locker(&mut kit),
        PropKind::Shelf => shelf(&mut kit),
        PropKind::Pew => pew(&mut kit),
        PropKind::Gurney => gurney(&mut kit),
        PropKind::Bottle => bottle(&mut kit),
        PropKind::Brick => brick(&mut kit),
        PropKind::Corpse => corpse(&mut kit),
        PropKind::Gravestone => gravestone(&mut kit),
        PropKind::Cross => cross(&mut kit),
        PropKind::Chalkboard => chalkboard(&mut kit),
        PropKind::Wheelchair => wheelchair(&mut kit),
        PropKind::Mattress => mattress(&mut kit),
        PropKind::Television => television(&mut kit),
        PropKind::Crib => crib(&mut kit),
        PropKind::Bathtub => bathtub(&mut kit),
        PropKind::Shrine => shrine(&mut kit),
        PropKind::Mannequin => mannequin(&mut kit),
        PropKind::Piano => piano(&mut kit),
        PropKind::Bicycle => bicycle(&mut kit),
    }

    if placement.kind == PropKind::Barrel {
        commands.set(root, Explosive { fuse: None });
    }
    if placement.kind == PropKind::Bottle {
        commands.add_tag_type::<Fragile>(root);
    }
}

fn loose(kind: PropKind) -> Option<(Vec3, f32, f32)> {
    match kind {
        PropKind::Bottle => Some((Vec3::new(0.12, 0.22, 0.12), 0.22, 24.0)),
        PropKind::Brick => Some((Vec3::new(0.21, 0.09, 0.11), 0.09, 150.0)),
        PropKind::Barrel => Some((Vec3::new(0.38, 0.52, 0.38), 0.52, 42.0)),
        PropKind::Wheelchair => Some((Vec3::new(0.42, 0.46, 0.44), 0.46, 18.0)),
        PropKind::Television => Some((Vec3::new(0.43, 0.36, 0.36), 0.36, 45.0)),
        PropKind::Bench => Some((Vec3::new(0.98, 0.46, 0.32), 0.46, 26.0)),
        _ => None,
    }
}

fn footprint(kind: PropKind) -> Option<(Vec3, f32)> {
    match kind {
        PropKind::Car => Some((Vec3::new(0.9, 0.75, 2.05), 0.75)),
        PropKind::Dumpster => Some((Vec3::new(0.78, 0.62, 1.16), 0.62)),
        PropKind::Barrel => Some((Vec3::new(0.38, 0.5, 0.38), 0.5)),
        PropKind::Bench => Some((Vec3::new(0.98, 0.45, 0.32), 0.45)),
        PropKind::Fence => Some((Vec3::new(1.02, 0.95, 0.12), 0.95)),
        PropKind::Sign => Some((Vec3::new(0.16, 1.2, 0.16), 1.2)),
        PropKind::Rubble => None,
        PropKind::Pillar => Some((Vec3::new(0.42, 1.5, 0.42), 1.5)),
        PropKind::Desk => Some((Vec3::new(0.68, 0.4, 0.58), 0.4)),
        PropKind::Locker => Some((Vec3::new(0.4, 0.95, 0.28), 0.95)),
        PropKind::Shelf => Some((Vec3::new(0.6, 0.9, 0.28), 0.9)),
        PropKind::Pew => Some((Vec3::new(1.62, 0.45, 0.32), 0.45)),
        PropKind::Gurney => Some((Vec3::new(0.44, 0.5, 1.02), 0.5)),
        PropKind::Bottle | PropKind::Brick => None,
        PropKind::Corpse => None,
        PropKind::Gravestone => Some((Vec3::new(0.42, 0.6, 0.16), 0.6)),
        PropKind::Cross => Some((Vec3::new(0.18, 1.4, 0.18), 1.4)),
        PropKind::Chalkboard => Some((Vec3::new(1.1, 0.7, 0.1), 1.5)),
        PropKind::Wheelchair => Some((Vec3::new(0.42, 0.45, 0.44), 0.45)),
        PropKind::Mattress => Some((Vec3::new(0.72, 0.14, 1.06), 0.14)),
        PropKind::Television => Some((Vec3::new(0.42, 0.34, 0.36), 0.34)),
        PropKind::Crib => Some((Vec3::new(0.46, 0.5, 0.86), 0.5)),
        PropKind::Bathtub => Some((Vec3::new(0.5, 0.36, 1.0), 0.36)),
        PropKind::Shrine => None,
        PropKind::Mannequin => Some((Vec3::new(0.26, 0.9, 0.22), 0.9)),
        PropKind::Piano => Some((Vec3::new(0.78, 0.6, 0.52), 0.6)),
        PropKind::Bicycle => None,
    }
}

fn bottle(kit: &mut Kit<'_, '_>) {
    kit.tube_part(
        Vec3::new(0.0, 0.14, 0.0),
        Vec3::new(0.19, 0.28, 0.19),
        SurfaceKind::Glass,
    );
    kit.tube_part(
        Vec3::new(0.0, 0.33, 0.0),
        Vec3::new(0.09, 0.16, 0.09),
        SurfaceKind::Glass,
    );
    kit.tube_part(
        Vec3::new(0.0, 0.41, 0.0),
        Vec3::new(0.11, 0.05, 0.11),
        SurfaceKind::Rust,
    );
}

fn brick(kit: &mut Kit<'_, '_>) {
    kit.box_part(
        Vec3::new(0.0, 0.08, 0.0),
        Vec3::new(0.4, 0.16, 0.2),
        SurfaceKind::Facade,
    );
}

fn car(kit: &mut Kit<'_, '_>) {
    kit.box_part(
        Vec3::new(0.0, 0.62, 0.0),
        Vec3::new(1.72, 0.64, 4.0),
        SurfaceKind::Rust,
    );
    kit.box_part(
        Vec3::new(0.0, 1.16, -0.25),
        Vec3::new(1.5, 0.6, 1.9),
        SurfaceKind::Rust,
    );
    kit.box_part(
        Vec3::new(0.0, 1.16, 0.72),
        Vec3::new(1.36, 0.5, 0.08),
        SurfaceKind::Glass,
    );
    kit.box_part(
        Vec3::new(0.0, 1.16, -1.22),
        Vec3::new(1.36, 0.5, 0.08),
        SurfaceKind::Glass,
    );
    kit.box_part(
        Vec3::new(0.0, 0.78, 2.02),
        Vec3::new(1.5, 0.28, 0.1),
        SurfaceKind::Metal,
    );
    for side in [-1.0_f32, 1.0] {
        for end in [-1.38_f32, 1.38] {
            kit.wheel(Vec3::new(side * 0.9, 0.33, end));
        }
    }
}

fn dumpster(kit: &mut Kit<'_, '_>) {
    kit.box_part(
        Vec3::new(0.0, 0.55, 0.0),
        Vec3::new(1.52, 1.1, 2.24),
        SurfaceKind::Metal,
    );
    kit.box_part(
        Vec3::new(0.0, 1.14, 0.0),
        Vec3::new(1.62, 0.18, 2.34),
        SurfaceKind::Rust,
    );
    for side in [-1.0_f32, 1.0] {
        kit.tube_part(
            Vec3::new(side * 0.7, 0.13, 0.9),
            Vec3::new(0.26, 0.24, 0.26),
            SurfaceKind::Metal,
        );
    }
}

fn barrel(kit: &mut Kit<'_, '_>) {
    kit.tube_part(
        Vec3::new(0.0, 0.5, 0.0),
        Vec3::new(0.74, 1.0, 0.74),
        SurfaceKind::Rust,
    );
    kit.tube_part(
        Vec3::new(0.0, 0.99, 0.0),
        Vec3::new(0.8, 0.12, 0.8),
        SurfaceKind::Metal,
    );
    kit.tube_part(
        Vec3::new(0.0, 0.5, 0.0),
        Vec3::new(0.79, 0.1, 0.79),
        SurfaceKind::Metal,
    );
}

fn bench(kit: &mut Kit<'_, '_>) {
    kit.box_part(
        Vec3::new(0.0, 0.45, 0.0),
        Vec3::new(1.9, 0.1, 0.56),
        SurfaceKind::Wood,
    );
    kit.box_part(
        Vec3::new(0.0, 0.78, -0.24),
        Vec3::new(1.9, 0.56, 0.09),
        SurfaceKind::Wood,
    );
    for side in [-1.0_f32, 1.0] {
        kit.box_part(
            Vec3::new(side * 0.8, 0.22, 0.0),
            Vec3::new(0.12, 0.45, 0.5),
            SurfaceKind::Metal,
        );
    }
}

fn fence(kit: &mut Kit<'_, '_>) {
    kit.box_part(
        Vec3::new(0.0, 0.95, 0.0),
        Vec3::new(2.0, 1.7, 0.06),
        SurfaceKind::Grille,
    );
    kit.box_part(
        Vec3::new(0.0, 1.85, 0.0),
        Vec3::new(2.04, 0.09, 0.12),
        SurfaceKind::Metal,
    );
    for side in [-1.0_f32, 1.0] {
        kit.box_part(
            Vec3::new(side * 0.99, 0.95, 0.0),
            Vec3::new(0.11, 1.9, 0.11),
            SurfaceKind::Metal,
        );
    }
}

fn sign(kit: &mut Kit<'_, '_>) {
    kit.tube_part(
        Vec3::new(0.0, 1.2, 0.0),
        Vec3::new(0.14, 2.4, 0.14),
        SurfaceKind::Metal,
    );
    kit.box_part(
        Vec3::new(0.0, 2.2, 0.06),
        Vec3::new(1.1, 0.72, 0.07),
        SurfaceKind::Rust,
    );
}

fn rubble(kit: &mut Kit<'_, '_>) {
    let chunks = [
        (Vec3::new(0.3, 0.14, 0.1), Vec3::new(0.7, 0.28, 0.6), 0.4),
        (Vec3::new(-0.35, 0.1, 0.35), Vec3::new(0.55, 0.2, 0.5), 1.1),
        (
            Vec3::new(0.05, 0.09, -0.4),
            Vec3::new(0.62, 0.18, 0.45),
            2.3,
        ),
        (Vec3::new(-0.5, 0.07, -0.2), Vec3::new(0.4, 0.14, 0.4), 3.0),
    ];
    for (offset, scale, yaw) in chunks {
        kit.tilted_part(offset, scale, yaw, SurfaceKind::Concrete);
    }
}

fn pillar(kit: &mut Kit<'_, '_>) {
    kit.tube_part(
        Vec3::new(0.0, 1.5, 0.0),
        Vec3::new(0.72, 3.0, 0.72),
        SurfaceKind::Concrete,
    );
    kit.box_part(
        Vec3::new(0.0, 0.12, 0.0),
        Vec3::new(0.98, 0.24, 0.98),
        SurfaceKind::Concrete,
    );
    kit.box_part(
        Vec3::new(0.0, 2.9, 0.0),
        Vec3::new(0.98, 0.22, 0.98),
        SurfaceKind::Concrete,
    );
}

fn desk(kit: &mut Kit<'_, '_>) {
    kit.box_part(
        Vec3::new(0.0, 0.72, 0.0),
        Vec3::new(1.3, 0.08, 0.72),
        SurfaceKind::Wood,
    );
    kit.box_part(
        Vec3::new(0.0, 0.38, -0.3),
        Vec3::new(1.2, 0.62, 0.07),
        SurfaceKind::Wood,
    );
    for side in [-1.0_f32, 1.0] {
        kit.box_part(
            Vec3::new(side * 0.6, 0.36, 0.0),
            Vec3::new(0.09, 0.72, 0.68),
            SurfaceKind::Wood,
        );
    }
    kit.box_part(
        Vec3::new(0.0, 0.44, 0.66),
        Vec3::new(0.46, 0.08, 0.46),
        SurfaceKind::Wood,
    );
    kit.box_part(
        Vec3::new(0.0, 0.76, 0.86),
        Vec3::new(0.46, 0.56, 0.07),
        SurfaceKind::Wood,
    );
}

fn locker(kit: &mut Kit<'_, '_>) {
    kit.box_part(
        Vec3::new(0.0, 0.95, 0.0),
        Vec3::new(0.78, 1.9, 0.54),
        SurfaceKind::Metal,
    );
    kit.box_part(
        Vec3::new(0.0, 0.95, 0.28),
        Vec3::new(0.7, 1.8, 0.04),
        SurfaceKind::Rust,
    );
    kit.box_part(
        Vec3::new(0.0, 1.6, 0.31),
        Vec3::new(0.42, 0.2, 0.04),
        SurfaceKind::Grille,
    );
}

fn shelf(kit: &mut Kit<'_, '_>) {
    for side in [-1.0_f32, 1.0] {
        kit.box_part(
            Vec3::new(side * 0.56, 0.9, 0.0),
            Vec3::new(0.08, 1.8, 0.52),
            SurfaceKind::Wood,
        );
    }
    for height in [0.3_f32, 0.8, 1.3, 1.76] {
        kit.box_part(
            Vec3::new(0.0, height, 0.0),
            Vec3::new(1.12, 0.06, 0.5),
            SurfaceKind::Wood,
        );
    }
    kit.box_part(
        Vec3::new(0.0, 0.9, -0.26),
        Vec3::new(1.12, 1.8, 0.04),
        SurfaceKind::Wood,
    );
}

fn pew(kit: &mut Kit<'_, '_>) {
    kit.box_part(
        Vec3::new(0.0, 0.45, 0.0),
        Vec3::new(3.2, 0.1, 0.52),
        SurfaceKind::Wood,
    );
    kit.box_part(
        Vec3::new(0.0, 0.78, -0.22),
        Vec3::new(3.2, 0.62, 0.08),
        SurfaceKind::Wood,
    );
    for side in [-1.0_f32, 1.0] {
        kit.box_part(
            Vec3::new(side * 1.55, 0.25, 0.0),
            Vec3::new(0.1, 0.5, 0.52),
            SurfaceKind::Wood,
        );
    }
}

fn gurney(kit: &mut Kit<'_, '_>) {
    kit.box_part(
        Vec3::new(0.0, 0.78, 0.0),
        Vec3::new(0.82, 0.1, 2.0),
        SurfaceKind::Metal,
    );
    kit.box_part(
        Vec3::new(0.0, 0.88, 0.0),
        Vec3::new(0.74, 0.11, 1.86),
        SurfaceKind::Rust,
    );
    for side in [-1.0_f32, 1.0] {
        for end in [-0.86_f32, 0.86] {
            kit.box_part(
                Vec3::new(side * 0.33, 0.39, end),
                Vec3::new(0.07, 0.78, 0.07),
                SurfaceKind::Metal,
            );
        }
        kit.box_part(
            Vec3::new(side * 0.43, 1.02, 0.0),
            Vec3::new(0.05, 0.36, 1.4),
            SurfaceKind::Metal,
        );
    }
}

fn corpse(kit: &mut Kit<'_, '_>) {
    kit.box_part(
        Vec3::new(0.0, 0.13, 0.0),
        Vec3::new(0.62, 0.26, 1.72),
        SurfaceKind::Concrete,
    );
    kit.box_part(
        Vec3::new(0.0, 0.3, -0.62),
        Vec3::new(0.44, 0.34, 0.4),
        SurfaceKind::Concrete,
    );
    kit.box_part(
        Vec3::new(0.0, 0.05, 0.9),
        Vec3::new(0.5, 0.1, 0.3),
        SurfaceKind::Wood,
    );
}

fn gravestone(kit: &mut Kit<'_, '_>) {
    kit.box_part(
        Vec3::new(0.0, 0.1, 0.0),
        Vec3::new(1.0, 0.2, 0.5),
        SurfaceKind::Concrete,
    );
    kit.box_part(
        Vec3::new(0.0, 0.7, 0.0),
        Vec3::new(0.78, 1.2, 0.26),
        SurfaceKind::Cornice,
    );
    kit.box_part(
        Vec3::new(0.0, 1.26, 0.0),
        Vec3::new(0.62, 0.2, 0.3),
        SurfaceKind::Cornice,
    );
}

fn cross(kit: &mut Kit<'_, '_>) {
    kit.box_part(
        Vec3::new(0.0, 1.4, 0.0),
        Vec3::new(0.24, 2.8, 0.24),
        SurfaceKind::Wood,
    );
    kit.box_part(
        Vec3::new(0.0, 2.1, 0.0),
        Vec3::new(1.3, 0.24, 0.24),
        SurfaceKind::Wood,
    );
}

fn chalkboard(kit: &mut Kit<'_, '_>) {
    kit.box_part(
        Vec3::new(0.0, 1.5, 0.0),
        Vec3::new(2.3, 1.5, 0.14),
        SurfaceKind::Grille,
    );
    kit.box_part(
        Vec3::new(0.0, 0.72, 0.06),
        Vec3::new(2.4, 0.1, 0.24),
        SurfaceKind::Wood,
    );
}

fn wheelchair(kit: &mut Kit<'_, '_>) {
    kit.box_part(
        Vec3::new(0.0, 0.5, 0.0),
        Vec3::new(0.62, 0.1, 0.62),
        SurfaceKind::Grille,
    );
    kit.box_part(
        Vec3::new(0.0, 0.86, -0.3),
        Vec3::new(0.62, 0.72, 0.08),
        SurfaceKind::Grille,
    );
    for side in [-1.0_f32, 1.0] {
        kit.wheel(Vec3::new(side * 0.4, 0.44, 0.0));
        kit.tube_part(
            Vec3::new(side * 0.3, 0.14, 0.42),
            Vec3::new(0.26, 0.1, 0.26),
            SurfaceKind::Metal,
        );
    }
}

fn mattress(kit: &mut Kit<'_, '_>) {
    kit.box_part(
        Vec3::new(0.0, 0.14, 0.0),
        Vec3::new(1.42, 0.28, 2.1),
        SurfaceKind::Concrete,
    );
    kit.box_part(
        Vec3::new(0.0, 0.32, -0.7),
        Vec3::new(0.8, 0.16, 0.5),
        SurfaceKind::Wood,
    );
}

fn television(kit: &mut Kit<'_, '_>) {
    kit.box_part(
        Vec3::new(0.0, 0.42, 0.0),
        Vec3::new(0.82, 0.68, 0.7),
        SurfaceKind::Wood,
    );
    kit.box_part(
        Vec3::new(0.0, 0.46, 0.36),
        Vec3::new(0.62, 0.48, 0.06),
        SurfaceKind::GlassLit,
    );
    for side in [-1.0_f32, 1.0] {
        kit.tube_part(
            Vec3::new(side * 0.2, 0.92, -0.2),
            Vec3::new(0.05, 0.5, 0.05),
            SurfaceKind::Metal,
        );
    }
}

fn crib(kit: &mut Kit<'_, '_>) {
    kit.box_part(
        Vec3::new(0.0, 0.44, 0.0),
        Vec3::new(0.86, 0.12, 1.66),
        SurfaceKind::Wood,
    );
    for side in [-1.0_f32, 1.0] {
        kit.box_part(
            Vec3::new(side * 0.44, 0.76, 0.0),
            Vec3::new(0.08, 0.68, 1.72),
            SurfaceKind::Grille,
        );
        kit.box_part(
            Vec3::new(0.0, 0.76, side * 0.84),
            Vec3::new(0.9, 0.68, 0.08),
            SurfaceKind::Grille,
        );
    }
}

fn bathtub(kit: &mut Kit<'_, '_>) {
    kit.box_part(
        Vec3::new(0.0, 0.12, 0.0),
        Vec3::new(0.96, 0.24, 1.96),
        SurfaceKind::Cornice,
    );
    for side in [-1.0_f32, 1.0] {
        kit.box_part(
            Vec3::new(side * 0.46, 0.4, 0.0),
            Vec3::new(0.1, 0.6, 1.96),
            SurfaceKind::Cornice,
        );
        kit.box_part(
            Vec3::new(0.0, 0.4, side * 0.94),
            Vec3::new(0.96, 0.6, 0.1),
            SurfaceKind::Cornice,
        );
    }
    kit.box_part(
        Vec3::new(0.0, 0.36, 0.0),
        Vec3::new(0.8, 0.04, 1.78),
        SurfaceKind::Glass,
    );
}

fn shrine(kit: &mut Kit<'_, '_>) {
    let shoes: [(f32, f32, f32); 7] = [
        (-0.4, 0.06, -0.3),
        (-0.2, 0.06, -0.34),
        (0.1, 0.06, -0.2),
        (0.36, 0.06, -0.3),
        (-0.3, 0.2, 0.1),
        (0.05, 0.2, 0.14),
        (0.3, 0.2, 0.05),
    ];
    for (x, y, z) in shoes {
        kit.box_part(
            Vec3::new(x, y, z),
            Vec3::new(0.22, 0.12, 0.4),
            SurfaceKind::Wood,
        );
    }
    kit.tube_part(
        Vec3::new(0.0, 0.36, 0.4),
        Vec3::new(0.12, 0.32, 0.12),
        SurfaceKind::Cornice,
    );
    kit.tube_part(
        Vec3::new(0.0, 0.54, 0.4),
        Vec3::new(0.07, 0.08, 0.07),
        SurfaceKind::GlassLit,
    );
}

fn mannequin(kit: &mut Kit<'_, '_>) {
    kit.box_part(
        Vec3::new(0.0, 1.2, 0.0),
        Vec3::new(0.46, 0.72, 0.3),
        SurfaceKind::Cornice,
    );
    kit.box_part(
        Vec3::new(0.0, 1.68, 0.0),
        Vec3::new(0.24, 0.3, 0.26),
        SurfaceKind::Cornice,
    );
    for side in [-1.0_f32, 1.0] {
        kit.box_part(
            Vec3::new(side * 0.14, 0.42, 0.0),
            Vec3::new(0.18, 0.86, 0.2),
            SurfaceKind::Cornice,
        );
        kit.box_part(
            Vec3::new(side * 0.3, 1.2, 0.06),
            Vec3::new(0.12, 0.7, 0.14),
            SurfaceKind::Cornice,
        );
    }
    kit.tube_part(
        Vec3::new(0.0, 0.02, 0.0),
        Vec3::new(0.5, 0.06, 0.5),
        SurfaceKind::Metal,
    );
}

fn piano(kit: &mut Kit<'_, '_>) {
    kit.box_part(
        Vec3::new(0.0, 0.7, 0.0),
        Vec3::new(1.5, 0.8, 0.92),
        SurfaceKind::Wood,
    );
    kit.box_part(
        Vec3::new(0.0, 1.14, 0.16),
        Vec3::new(1.56, 0.1, 1.0),
        SurfaceKind::Wood,
    );
    kit.box_part(
        Vec3::new(0.0, 0.86, 0.5),
        Vec3::new(1.36, 0.08, 0.28),
        SurfaceKind::Cornice,
    );
    for side in [-1.0_f32, 1.0] {
        kit.box_part(
            Vec3::new(side * 0.66, 0.3, 0.3),
            Vec3::new(0.14, 0.6, 0.14),
            SurfaceKind::Wood,
        );
    }
}

fn bicycle(kit: &mut Kit<'_, '_>) {
    for end in [-0.52_f32, 0.52] {
        kit.wheel(Vec3::new(0.0, 0.34, end));
    }
    kit.box_part(
        Vec3::new(0.0, 0.52, 0.0),
        Vec3::new(0.08, 0.08, 1.0),
        SurfaceKind::Metal,
    );
    kit.box_part(
        Vec3::new(0.0, 0.72, -0.4),
        Vec3::new(0.5, 0.06, 0.06),
        SurfaceKind::Metal,
    );
    kit.box_part(
        Vec3::new(0.0, 0.68, 0.3),
        Vec3::new(0.14, 0.08, 0.3),
        SurfaceKind::Rust,
    );
}
