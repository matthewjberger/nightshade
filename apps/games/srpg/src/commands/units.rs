use crate::commands::decor::spawn_standing;
use crate::commands::set_unit;
use crate::components::{Fill, Owner, StatusIcon};
use crate::data::{Bond, Recruit, Team, growth_of, level_of, morale_edge, rapport_growth};
use crate::data::{Fighter, Unit};
use crate::queries::nearest_tile;
use crate::resources::{Sheet, Terrain};
use crate::theme;
use nalgebra_glm::{Vec2, Vec3};
use nightshade::ecs::ChildOf;
use nightshade::prelude::*;
use nightshade_scene::prelude::*;
use nightshade_sprite::prelude::*;

pub fn spawn_body(
    commands: &mut Commands,
    terrain: &Terrain,
    sheet: &Sheet,
    unit: Unit,
    wanted: (u32, u32),
) -> Entity {
    let mobility = unit.mobility;
    let (column, row) = nearest_tile(terrain, wanted, |column, row| {
        crate::queries::terrain::can_stop(terrain, column, row, mobility)
    });
    let hover = mobility.hover();
    let entity = spawn_standing(
        commands,
        sheet,
        crate::queries::sheet::standing(sheet, unit.class.tile(unit.team)),
        crate::queries::terrain::stand(terrain, column, row),
        Vec2::new(0.85, 0.55),
        unit.team.shadow_tint(),
        hover,
        1.0,
    );
    set_unit(
        commands,
        entity,
        &Unit {
            column,
            row,
            ..unit
        },
    );

    let track = Sprite::new(theme::ATLAS, sheet.cell)
        .with_size(theme::HEALTH_BAR_SIZE.x, theme::HEALTH_BAR_SIZE.y)
        .with_color(theme::HEALTH_TRACK)
        .with_layer(theme::LAYER_STANDING + 1);
    let track = spawn_sprite(
        commands,
        track,
        Vec3::new(0.0, theme::HEALTH_BAR_HEIGHT, 0.0),
    );
    commands.set(track, ChildOf(entity));
    commands.set(track, Owner(entity));
    commands.set(track, Fill(false));
    let fill = Sprite::new(theme::ATLAS, sheet.cell)
        .with_size(theme::HEALTH_BAR_SIZE.x, theme::HEALTH_BAR_SIZE.y)
        .with_anchor(Vec2::new(0.0, 0.5))
        .with_color(theme::HEALTH_GOOD)
        .with_layer(theme::LAYER_STANDING + 2);
    let fill = spawn_sprite(
        commands,
        fill,
        Vec3::new(
            -theme::HEALTH_BAR_SIZE.x / 2.0,
            theme::HEALTH_BAR_HEIGHT,
            0.0,
        ),
    );
    commands.set(fill, ChildOf(entity));
    commands.set(fill, Owner(entity));
    commands.set(fill, Fill(true));

    let icon = Sprite::new(
        theme::ATLAS,
        crate::queries::sheet::tile(sheet, theme::LOCK_ICON),
    )
    .with_size(theme::STATUS_ICON_SIZE, theme::STATUS_ICON_SIZE)
    .with_anchor(Vec2::new(0.5, 0.0))
    .with_layer(theme::LAYER_STANDING);
    let icon = spawn_sprite(
        commands,
        icon,
        Vec3::new(0.0, theme::STATUS_ICON_HEIGHT, 0.0),
    );
    commands.set(icon, ChildOf(entity));
    commands.set(icon, Visibility { visible: false });
    commands.set(icon, StatusIcon { owner: entity });
    entity
}

pub fn spawn_fighter(
    commands: &mut Commands,
    terrain: &Terrain,
    sheet: &Sheet,
    entity: Entity,
    fighter: &Fighter,
    home: (u32, u32),
) -> Entity {
    let class = fighter.class.sheet();
    let growth = growth_of(level_of(fighter.experience));
    let earned = rapport_growth(Bond::of(fighter.rapport)) + fighter.boon;
    let edge = morale_edge(fighter.morale);
    let hp = class.hp + growth.hp + earned.hp;
    let unit = Unit {
        fighter: Some(entity),
        bonds: fighter.bonds.clone(),
        name: fighter.name,
        class: fighter.class,
        team: Team::Player,
        mobility: class.mobility,
        hp,
        max_hp: hp,
        mp: class.mp + growth.mp + earned.mp,
        max_mp: class.mp + growth.mp + earned.mp,
        attack: class.attack + growth.attack + earned.attack + edge,
        defense: class.defense + growth.defense + earned.defense,
        magic: class.magic + growth.magic + earned.magic + edge,
        resist: class.resist + growth.resist + earned.resist,
        speed: class.speed + growth.speed + earned.speed,
        move_range: class.move_range,
        jump: class.jump,
        evasion: class.evasion + growth.evasion + earned.evasion,
        facing: fighter.facing,
        flash_tint: [1.0, 1.0, 1.0, 1.0],
        ..Default::default()
    };
    spawn_body(commands, terrain, sheet, unit, home)
}

pub fn spawn_recruit(
    commands: &mut Commands,
    terrain: &Terrain,
    sheet: &Sheet,
    recruit: &Recruit,
    team: Team,
) -> Entity {
    let class = recruit.class.sheet();
    let hp = class.hp + recruit.hp_bonus;
    let unit = Unit {
        name: recruit.name,
        class: recruit.class,
        team,
        mobility: class.mobility,
        hp,
        max_hp: hp,
        mp: class.mp,
        max_mp: class.mp,
        attack: class.attack + recruit.attack_bonus,
        defense: class.defense,
        magic: class.magic + recruit.attack_bonus,
        resist: class.resist,
        speed: class.speed,
        move_range: class.move_range,
        jump: class.jump,
        evasion: class.evasion,
        facing: recruit.facing,
        flash_tint: [1.0, 1.0, 1.0, 1.0],
        ..Default::default()
    };
    spawn_body(
        commands,
        terrain,
        sheet,
        unit,
        (recruit.column, recruit.row),
    )
}
