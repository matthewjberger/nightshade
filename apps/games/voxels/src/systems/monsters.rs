use crate::commands::toast_show;
use crate::components::{Bob, Health, Hurt, Wait};
use crate::data::{Ask, Item, MonsterKind, ToolKind};
use crate::queries::{dungeon_standing, footing, ground_height, selected_stack};
use crate::resources::{Generation, Hand, Horde, Inventory, Player, Sheet, Toast, VoxelWorld};
use crate::theme;
use fastrand::Rng;
use nalgebra_glm::{IVec3, Vec2, Vec3};
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::LocalTransform;
use nightshade_sprite::prelude::*;

pub fn raise_the_monsters(
    time: Res<Time>,
    world: Res<VoxelWorld>,
    generation: Res<Generation>,
    player: Res<Player>,
    sheet: Res<Sheet>,
    mut horde: ResMut<Horde>,
    mut commands: Commands,
    mut monsters: Query<(&MonsterKind,)>,
) {
    horde.wait -= time.delta_time.min(0.05);
    if !generation.plan.monsters {
        return;
    }
    if horde.wait > 0.0 || player.watching {
        return;
    }
    horde.wait = theme::MONSTER_WAIT;

    let mut alive = 0usize;
    monsters.for_each_mut(|_, (_,)| alive += 1);
    if alive >= theme::MONSTER_CAP {
        return;
    }

    let head = IVec3::new(
        player.position.x.floor() as i32,
        player.position.y.floor() as i32,
        player.position.z.floor() as i32,
    );
    if !under_a_roof(&world, head.x, head.z, head.y) {
        return;
    }

    for _ in 0..14 {
        let angle = horde.rng.f32() * std::f32::consts::TAU;
        let spread = theme::MONSTER_NEAR
            + (horde.rng.f32() * (theme::MONSTER_FAR - theme::MONSTER_NEAR) as f32) as i32;
        let x = head.x + (angle.cos() * spread as f32) as i32;
        let z = head.z + (angle.sin() * spread as f32) as i32;
        let lift = (horde.rng.f32() * 9.0) as i32 - 4;
        let Some(floor) = footing(
            &world,
            x,
            z,
            (head.y + lift) as f32,
            theme::MONSTER_RISE,
            theme::MONSTER_FALL,
        ) else {
            continue;
        };
        if !under_a_roof(&world, x, z, floor + 1) {
            continue;
        }
        let kind = MonsterKind::ALL[(horde.rng.u32(..) as usize) % MonsterKind::ALL.len()];
        let info = kind.info();
        let rest = if info.floats {
            theme::MONSTER_FLOAT
        } else {
            0.0
        };
        let place = Vec3::new(x as f32 + 0.5, floor as f32 + 1.0 + rest, z as f32 + 0.5);
        spawn_monster(&mut commands, &sheet, kind, place, &mut horde.rng);
        return;
    }
}

pub fn hunt_the_player(
    time: Res<Time>,
    world: Res<VoxelWorld>,
    mut player: ResMut<Player>,
    mut commands: Commands,
    mut monsters: Query<(
        &MonsterKind,
        &mut Hurt,
        &mut Wait,
        &mut Bob,
        &mut LocalTransform,
        &mut Sprite,
    )>,
) {
    let delta_time = time.delta_time.min(0.05);
    let watching = player.watching;
    let mut bites = 0.0f32;
    let eye = player.position + Vec3::new(0.0, theme::EYE_HEIGHT * 0.5, 0.0);

    monsters.for_each_mut(|entity, (kind, hurt, wait, bob, transform, sprite)| {
        let info = kind.info();
        hurt.0 = (hurt.0 - delta_time).max(0.0);
        wait.0 = (wait.0 - delta_time).max(0.0);
        sprite.color = if hurt.0 > 0.0 {
            theme::MONSTER_HURT_TINT
        } else {
            [1.0, 1.0, 1.0, 1.0]
        };

        let gap = (eye - transform.translation).magnitude();
        if gap > theme::MONSTER_FORGET {
            commands.despawn(entity);
            return;
        }
        if watching {
            return;
        }

        let here = Vec2::new(transform.translation.x, transform.translation.z);
        let there = Vec2::new(eye.x, eye.z);
        let to_player = there - here;
        let flat = to_player.magnitude();
        if gap < theme::MONSTER_CHASE && flat > 0.3 {
            let direction = to_player / flat;
            let step = (info.speed * delta_time).min(flat);
            let wanted = here + direction * step;
            let column = (wanted.x.floor() as i32, wanted.y.floor() as i32);
            if let Some(floor) = footing(
                &world,
                column.0,
                column.1,
                transform.translation.y,
                theme::MONSTER_RISE,
                theme::MONSTER_FALL,
            ) {
                transform.translation.x = wanted.x;
                transform.translation.z = wanted.y;
                let rest = if info.floats {
                    theme::MONSTER_FLOAT
                } else {
                    0.0
                };
                let stand = floor as f32 + 1.0 + rest;
                let share = (theme::STEP_SMOOTHING * delta_time).clamp(0.0, 1.0);
                transform.translation.y += (stand - transform.translation.y) * share;
            }
            if direction.x.abs() > 0.01 {
                sprite.flip_x = direction.x < 0.0;
            }
        }

        if info.floats {
            bob.0 += delta_time * theme::MONSTER_BOB_RATE;
            transform.translation.y += bob.0.sin() * theme::MONSTER_BOB * delta_time;
        }

        if gap < theme::MONSTER_TOUCH + info.width * 0.5 && wait.0 <= 0.0 {
            wait.0 = theme::MONSTER_BITE_WAIT;
            bites += info.bite;
        }
    });

    if bites > 0.0 {
        player.health -= bites;
        player.hurt = theme::HURT_SHAKE;
    }
}

pub fn swing_at_the_monsters(
    actions: Res<Actions<Ask>>,
    player: Res<Player>,
    inventory: Res<Inventory>,
    mut hand: ResMut<Hand>,
    mut toast: ResMut<Toast>,
    mut commands: Commands,
    mut monsters: Query<(&MonsterKind, &mut Health, &mut Hurt, &LocalTransform)>,
) {
    if !actions.just_pressed(Ask::Swing) || player.watching {
        return;
    }

    let eye = player.eye;
    let mut best: Option<(Entity, f32)> = None;
    monsters.for_each_mut(|entity, (kind, _, _, transform)| {
        let middle = transform.translation + Vec3::new(0.0, kind.info().width * 0.5, 0.0);
        let to_monster = middle - eye;
        let gap = to_monster.magnitude();
        if gap > theme::SWING_REACH || gap < 0.001 {
            return;
        }
        if to_monster.dot(&player.forward) / gap < theme::SWING_ARC {
            return;
        }
        if best.is_none_or(|(_, nearest)| gap < nearest) {
            best = Some((entity, gap));
        }
    });

    let Some((target, _)) = best else {
        return;
    };
    hand.swing = hand.swing.max(0.01);
    let held = selected_stack(&inventory).map(|stack| stack.item);
    let force = if held == Some(Item::Tool(ToolKind::Sword)) {
        theme::SWORD_HIT
    } else {
        theme::FIST_HIT
    };
    let mut fallen = None;
    monsters.for_each_mut(|entity, (kind, health, hurt, _)| {
        if entity != target {
            return;
        }
        health.0 -= force;
        hurt.0 = theme::MONSTER_HURT_FLASH;
        if health.0 <= 0.0 {
            fallen = Some(*kind);
            commands.despawn(entity);
        }
    });
    if let Some(kind) = fallen {
        toast_show(&mut toast, format!("The {} falls", kind.info().name));
    }
}

fn spawn_monster(
    commands: &mut Commands,
    sheet: &Sheet,
    kind: MonsterKind,
    position: Vec3,
    rng: &mut Rng,
) -> Entity {
    let info = kind.info();
    let (region, share) = dungeon_standing(sheet, info.tile);
    let sprite = Sprite::new(theme::SPRITE_ATLAS, region)
        .with_size(info.width, share * info.width)
        .with_anchor(Vec2::new(0.5, 0.0))
        .with_layer(theme::SPRITE_LAYER)
        .as_billboard();
    let entity = spawn_sprite(commands, sprite, position);
    commands.set(entity, kind);
    commands.set(entity, Health(info.health));
    commands.set(entity, Hurt(0.0));
    commands.set(entity, Wait(rng.f32() * theme::MONSTER_BITE_WAIT));
    commands.set(entity, Bob(rng.f32() * std::f32::consts::TAU));
    entity
}

fn under_a_roof(world: &VoxelWorld, x: i32, z: i32, y: i32) -> bool {
    ground_height(world, x, z).is_some_and(|top| top > y + theme::MONSTER_ROOF)
}
