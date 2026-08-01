use crate::commands::art::sprite_of;
use crate::components::{
    Aim, Backing, Bar, Bob, Bounty, Chill, Chills, Cooldown, Cut, Damage, Flash, Focus, Health,
    Kind, Leg, Level, MaxHealth, Owner, Pace, Pierce, Post, Progress, Reach, Reload, Rising, Shade,
    Speed, Splash, Splits, Tally, Tint, Ward,
};
use crate::contract::{Cell, Tower};
use crate::data::Walker;
use crate::queries;
use crate::resources::Art;
use crate::theme;
use nalgebra_glm::{Vec2, Vec3};
use nightshade::prelude::*;
use nightshade_sprite::prelude::spawn_sprite;

pub fn raise_a_tower(commands: &mut Commands, art: &Art, cell: Cell, kind: Tower) -> Entity {
    let center = queries::cell_center(cell);
    let grade = queries::grade_of(kind, 1);
    let tower = spawn_sprite(
        commands,
        sprite_of(art.towers[kind.slot()][0], theme::LAYER_STANDING),
        Vec3::new(center.x, center.y, 0.0),
    );
    commands.set(tower, Kind(kind));
    commands.set(tower, Level(1));
    commands.set(tower, Post(queries::cell_index(cell) as u32));
    commands.set(tower, Reach(grade.reach));
    commands.set(tower, Damage(grade.damage));
    commands.set(tower, Reload(grade.reload));
    commands.set(tower, Cooldown(0.0));
    commands.set(tower, Focus(0));
    commands.set(tower, Rising(theme::RISE_SECONDS));
    commands.set(tower, Tally(0));
    tower
}

pub fn raise_the_level(commands: &mut Commands, art: &Art, tower: Entity, kind: Tower, level: u8) {
    let grade = queries::grade_of(kind, level);
    let piece = art.towers[kind.slot()][(level as usize - 1).min(2)];
    commands.set(tower, Level(level));
    commands.set(tower, Reach(grade.reach));
    commands.set(tower, Damage(grade.damage));
    commands.set(tower, Reload(grade.reload));
    commands.set(tower, sprite_of(piece, theme::LAYER_STANDING));
    commands.set(tower, Rising(theme::RISE_SECONDS));
}

pub fn send_a_crystal(
    commands: &mut Commands,
    art: &Art,
    board: usize,
    walker: Walker,
    step: (usize, f32),
    heavy: bool,
) {
    let start = queries::leg_center(board, step.0, step.1);
    let tier = walker.tier.slot();
    let shade = spawn_sprite(
        commands,
        sprite_of(art.shade, theme::LAYER_MARKER)
            .with_size(theme::SHADE_WIDTH, theme::SHADE_HEIGHT)
            .with_color(theme::SHADE_COLOR),
        Vec3::new(start.x, start.y, 0.0),
    );
    let backing = spawn_sprite(
        commands,
        sprite_of(art.bar, theme::LAYER_BEAD).with_color(theme::BAR_BACK_COLOR),
        Vec3::new(start.x, start.y, 0.0),
    );
    let bar = spawn_sprite(
        commands,
        sprite_of(art.bar, theme::LAYER_BEAD).with_color(theme::BAR_COLOR),
        Vec3::new(start.x, start.y, 0.0),
    );
    let piece = art.shards[tier];
    let swell = if heavy { theme::BOSS_SCALE } else { 1.0 };
    let tint = if walker.ward > 0.0 {
        theme::WARD_COLOR
    } else {
        theme::SHARD_COLOR
    };
    let crystal = spawn_sprite(
        commands,
        sprite_of(piece, theme::LAYER_STANDING)
            .with_size(piece.size.x * swell, piece.size.y * swell)
            .with_color(tint),
        Vec3::new(start.x, start.y, 0.0),
    );
    commands.set(crystal, Health(walker.health));
    commands.set(crystal, MaxHealth(walker.health));
    commands.set(crystal, Leg(step.0));
    commands.set(crystal, Progress(step.1));
    let pace = if heavy { theme::BOSS_SPEED } else { 1.0 };
    let purse = if heavy { theme::BOSS_BOUNTY } else { 1 };
    commands.set(crystal, Speed(walker.speed * pace));
    commands.set(crystal, Pace(walker.speed * pace));
    commands.set(crystal, Chill(0.0));
    commands.set(crystal, Ward(walker.ward));
    let breaks = if heavy { 0 } else { walker.splits };
    commands.set(crystal, Splits(breaks));
    commands.set(crystal, Cut(walker.tier));
    commands.set(crystal, Tint(tint));
    commands.set(crystal, Bounty(walker.bounty * purse));
    commands.set(crystal, Bar(bar));
    commands.set(crystal, Backing(backing));
    commands.set(crystal, Shade(shade));
    commands.set(crystal, Bob(start.x * 3.1 + start.y));
    commands.set(crystal, Flash(0.0));
}

pub fn loose_a_bead(
    commands: &mut Commands,
    art: &Art,
    post: Entity,
    from: Vec2,
    at: Entity,
    damage: f32,
    chills: f32,
    splash: f32,
    pierce: f32,
) {
    let bead = spawn_sprite(
        commands,
        sprite_of(art.bead, theme::LAYER_BEAD),
        Vec3::new(from.x, from.y + theme::BEAD_LIFT, 0.0),
    );
    commands.set(bead, Aim(at));
    commands.set(bead, Owner(post));
    commands.set(bead, Damage(damage));
    commands.set(bead, Speed(theme::BEAD_SPEED));
    commands.set(bead, Chills(chills));
    commands.set(bead, Splash(splash));
    commands.set(bead, Pierce(pierce));
}
