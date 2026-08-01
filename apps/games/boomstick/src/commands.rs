use crate::components::{EnemyMood, Group, Health, PickupKind, Timer};
use crate::contract::{Blast, FireQueue, Lob, Shot};
use crate::data;
use crate::data::arsenal::Ammo;
use crate::data::bestiary::Beast;
use crate::data::shape::Part;
use crate::queries::ceiling;
use crate::resources::{PlayerState, slot_of};
use nightshade::prelude::*;
use nightshade_graphics::prelude::spawn_mesh_at;
use nightshade_physics::contract::components::{ColliderComponent, RigidBodyComponent};
use nightshade_render::commands::mesh_state::mesh_state_mark_material_dirty;
use nightshade_render::components::material::MaterialRef;
use nightshade_render::data::material::Material;
use nightshade_render::resources::material_registry::MaterialRegistry;
use nightshade_render::resources::mesh_state::MeshRenderState;
use nightshade_scene::prelude::*;

pub fn send_shot(queue: &mut FireQueue, gun: usize, origin: Vec3, direction: Vec3) {
    queue.shots.push(Shot {
        gun,
        origin: [origin.x, origin.y, origin.z],
        direction: [direction.x, direction.y, direction.z],
    });
}

pub fn send_bolt(queue: &mut FireQueue, beast: &Beast, origin: Vec3, direction: Vec3) {
    queue.lobs.push(Lob {
        origin: [origin.x, origin.y, origin.z],
        direction: [direction.x, direction.y, direction.z],
        speed: beast.bolt_speed,
        damage: beast.bolt_damage,
        splash: beast.bolt_splash,
        fall: beast.bolt_fall,
    });
}

pub fn send_blast(queue: &mut FireQueue, at: Vec3, radius: f32, damage: f32, from_player: bool) {
    queue.blasts.push(Blast {
        at: [at.x, at.y, at.z],
        radius,
        damage,
        from_player,
    });
}

pub fn paint(commands: &mut Commands, entity: Entity, name: &str, material: Material) {
    let name = name.to_string();
    commands.queue(move |storage, resources| {
        let registered = nightshade_graphics::prelude::insert_material(
            resources.res_mut::<MaterialRegistry>(),
            name,
            material,
        );
        storage.set(entity, MaterialRef::new(registered));
        mesh_state_mark_material_dirty(resources.res_mut::<MeshRenderState>(), entity);
    });
}

pub fn spawn_plate(
    commands: &mut Commands,
    name: &str,
    position: Vec3,
    size: Vec3,
    material: Material,
) -> Entity {
    let entity = spawn_mesh_at(commands, "Cube", position, size);
    paint(commands, entity, name, material);
    entity
}

pub fn spawn_block(
    commands: &mut Commands,
    name: &str,
    position: Vec3,
    size: Vec3,
    material: Material,
) -> Entity {
    let entity = spawn_plate(commands, name, position, size, material);
    commands.set(
        entity,
        RigidBodyComponent::new_static().with_translation(position.x, position.y, position.z),
    );
    commands.set(
        entity,
        ColliderComponent::new_cuboid(size.x * 0.5, size.y * 0.5, size.z * 0.5)
            .with_friction(0.9)
            .with_restitution(0.0),
    );
    entity
}

pub fn build_parts(
    commands: &mut Commands,
    label: &str,
    parts: &[Part],
    at: Vec3,
    scale: f32,
) -> Entity {
    let root = commands.spawn((
        Name(label.to_string()),
        LocalTransform {
            translation: at,
            scale: Vec3::new(scale, scale, scale),
            ..Default::default()
        },
        GlobalTransform::default(),
    ));
    for (index, part) in parts.iter().enumerate() {
        let piece = spawn_mesh_at(commands, "Cube", part.at, part.size);
        paint(
            commands,
            piece,
            &format!("{label}_{index}"),
            Material {
                base_color: part.tint,
                emissive_factor: lit(part.tint, part.glow),
                roughness: 0.55,
                metallic: 0.12,
                ..Default::default()
            },
        );
        commands.set(
            piece,
            LocalTransform {
                translation: part.at,
                scale: part.size,
                ..Default::default()
            },
        );
        commands.set(piece, nightshade::ecs::ChildOf(root));
    }
    root
}

fn lit(tint: [f32; 4], glow: [f32; 3]) -> [f32; 3] {
    if glow[0] + glow[1] + glow[2] > 0.0 {
        return glow;
    }
    [
        tint[0] * data::VIEWMODEL_GLOW,
        tint[1] * data::VIEWMODEL_GLOW,
        tint[2] * data::VIEWMODEL_GLOW,
    ]
}

pub fn wound(commands: &mut Commands, entity: Entity, health: f32, heavy: bool) {
    commands.queue(move |storage, _| {
        if !storage.has::<Health>(entity) {
            return;
        }
        storage.set(entity, Health(health));
        storage.set(entity, Group(0));
        if health <= 0.0 {
            storage.set(entity, EnemyMood::Dying);
            storage.set(entity, Timer(data::CORPSE_TIME));
            return;
        }
        let asleep = storage
            .get::<EnemyMood>(entity)
            .map(|mood| *mood == EnemyMood::Asleep)
            .unwrap_or(false);
        if heavy {
            storage.set(entity, EnemyMood::Reeling);
            storage.set(entity, Timer(data::REEL_TIME));
        } else if asleep {
            storage.set(entity, EnemyMood::Waking);
            storage.set(entity, Timer(0.12));
        }
    });
}

pub fn heal(player: &mut PlayerState, amount: f32, ceiling: f32) -> bool {
    if player.health >= ceiling {
        return false;
    }
    player.health = (player.health + amount).min(ceiling);
    true
}

pub fn add_armor(player: &mut PlayerState, amount: f32) -> bool {
    if player.armor >= data::MAX_ARMOR {
        return false;
    }
    player.armor = (player.armor + amount).min(data::MAX_ARMOR);
    true
}

pub fn add_ammo(player: &mut PlayerState, ammo: Ammo, amount: u32) -> bool {
    let slot = slot_of(ammo);
    let cap = ceiling(ammo);
    if player.ammo[slot] >= cap {
        return false;
    }
    player.ammo[slot] = (player.ammo[slot] + amount).min(cap);
    true
}

pub fn take_gun(player: &mut PlayerState, slot: usize, ammo: Ammo, amount: u32) -> bool {
    let fresh = !player.owned[slot];
    let stocked = add_ammo(player, ammo, amount);
    if fresh {
        player.owned[slot] = true;
        player.gun = slot;
    }
    fresh || stocked
}

pub fn collect(player: &mut PlayerState, kind: PickupKind) -> bool {
    match kind {
        PickupKind::Medkit => heal(player, data::MEDKIT, data::MAX_HEALTH),
        PickupKind::Stimpack => heal(player, data::STIMPACK, data::HEALTH_CEILING),
        PickupKind::Plate => add_armor(player, data::PLATE),
        PickupKind::Shells => add_ammo(player, Ammo::Shells, data::SHELL_BOX),
        PickupKind::Bullets => add_ammo(player, Ammo::Bullets, data::BULLET_BOX),
        PickupKind::Rockets => add_ammo(player, Ammo::Rockets, data::ROCKET_BOX),
        PickupKind::Cells => add_ammo(player, Ammo::Cells, data::CELL_BOX),
        PickupKind::RedKey => {
            let fresh = !player.red_key;
            player.red_key = true;
            fresh
        }
        PickupKind::BlueKey => {
            let fresh = !player.blue_key;
            player.blue_key = true;
            fresh
        }
        PickupKind::Chaingun => take_gun(
            player,
            crate::data::arsenal::CHAINGUN,
            Ammo::Bullets,
            data::BULLET_BOX,
        ),
        PickupKind::Launcher => take_gun(
            player,
            crate::data::arsenal::LAUNCHER,
            Ammo::Rockets,
            data::ROCKET_BOX,
        ),
        PickupKind::Plasma => take_gun(
            player,
            crate::data::arsenal::PLASMA,
            Ammo::Cells,
            data::CELL_BOX,
        ),
    }
}

pub fn hurt(player: &mut PlayerState, damage: f32) {
    let soaked = (damage * data::ARMOR_SHARE).min(player.armor);
    player.armor -= soaked;
    player.health -= damage - soaked;
    if player.health <= 0.0 {
        player.health = 0.0;
        player.dead = true;
    }
}
