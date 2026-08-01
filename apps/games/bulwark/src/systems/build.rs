use crate::commands::effects;
use crate::commands::purse;
use crate::commands::spawn;
use crate::components::{Focus, Kind, Level, Post, Reach};
use crate::contract::Tower;
use crate::data::{Ask, Settings};
use crate::queries;
use crate::resources::{Bulwark, Screen};
use crate::theme;
use nalgebra_glm::{Vec2, Vec3};
use nightshade::prelude::*;
use nightshade_camera::prelude::ActiveCamera;
use nightshade_config::contract::Config;
use nightshade_input::Actions;
use nightshade_picking::queries::screen_ray;
use nightshade_platform::prelude::*;
use nightshade_platform::queries::input_capture::pointer_captured;
use nightshade_render::resources::viewport::Viewport;
use nightshade_scene::prelude::*;
use nightshade_sprite::prelude::Sprite;
use nightshade_state::prelude::State;
use nightshade_ui::prelude::ui_button_clicks;
use nightshade_ui::resources::RetainedUiFrame;

pub fn aim_the_marker(
    input: Res<Input>,
    window: Res<Window>,
    viewport: Res<Viewport>,
    capture: Res<InputCapture>,
    active_camera: Res<ActiveCamera>,
    config: Res<Config<Settings>>,
    mut bulwark: ResMut<Bulwark>,
    storage: &mut nightshade::ecs::Storage,
) {
    let Some(marker) = bulwark.marker else {
        return;
    };

    let cell = if pointer_captured(&capture) {
        None
    } else {
        screen_ray(
            &window,
            &viewport,
            &active_camera,
            storage.query_ref(),
            input.mouse.position,
        )
        .and_then(|ray| ray.intersect_plane(Vec3::z(), 0.0))
        .and_then(|point| queries::cell_under(Vec2::new(point.x, point.y)))
    };

    bulwark.hovered = cell;
    bulwark.buildable = cell.is_some_and(|cell| queries::buildable(&bulwark, cell));

    let marks = config.toggle(Settings::Marks);
    if let Some(visibility) = storage.get_mut::<Visibility>(marker) {
        visibility.visible = marks && cell.is_some();
    }
    let Some(cell) = cell else {
        return;
    };

    let center = queries::cell_center(cell);
    if let Some(transform) = storage.get_mut::<LocalTransform>(marker) {
        transform.translation = Vec3::new(center.x, center.y, 0.0);
    }
    let holds = queries::tower_on(&bulwark, cell).is_some();
    let cost = queries::build_cost(bulwark.chosen);
    let color = if holds {
        theme::MARKER_CHOSEN
    } else if bulwark.buildable && queries::affordable(&bulwark, cost) {
        theme::MARKER_FREE
    } else {
        theme::MARKER_BLOCKED
    };
    if let Some(sprite) = storage.get_mut::<Sprite>(marker) {
        sprite.color = color;
    }
}

pub fn show_the_ring(
    config: Res<Config<Settings>>,
    bulwark: Res<Bulwark>,
    storage: &mut nightshade::ecs::Storage,
) {
    let Some(ring) = bulwark.ring else {
        return;
    };

    let wanted = if !config.toggle(Settings::Rings) {
        None
    } else if let Some(tower) = bulwark.selected {
        let reach = storage.get::<Reach>(tower).map(|reach| reach.0);
        let at = storage
            .get::<LocalTransform>(tower)
            .map(|transform| transform.translation);
        reach.zip(at)
    } else if let Some(cell) = bulwark.hovered {
        if let Some(standing) = queries::tower_on(&bulwark, cell) {
            let reach = storage.get::<Reach>(standing).map(|reach| reach.0);
            let at = storage
                .get::<LocalTransform>(standing)
                .map(|transform| transform.translation);
            reach.zip(at)
        } else if bulwark.buildable {
            let center = queries::cell_center(cell);
            Some((
                queries::grade_of(bulwark.chosen, 1).reach,
                Vec3::new(center.x, center.y, 0.0),
            ))
        } else {
            None
        }
    } else {
        None
    };

    if let Some(visibility) = storage.get_mut::<Visibility>(ring) {
        visibility.visible = wanted.is_some();
    }
    let Some((reach, at)) = wanted else {
        return;
    };
    if let Some(transform) = storage.get_mut::<LocalTransform>(ring) {
        transform.translation = Vec3::new(at.x, at.y, 0.0);
    }
    if let Some(sprite) = storage.get_mut::<Sprite>(ring) {
        sprite.size = Vec2::new(reach * 2.0, reach * 2.0);
    }
}

pub fn pick_the_kind(
    actions: Res<Actions<Ask>>,
    frame: Res<RetainedUiFrame>,
    mut bulwark: ResMut<Bulwark>,
) {
    let slots = bulwark.hud.slots;
    let pace_button = bulwark.hud.pace;
    let mut stepped = false;
    for entity in ui_button_clicks(&frame) {
        for (index, tower) in Tower::ALL.into_iter().enumerate() {
            if entity == slots[index].cost {
                bulwark.chosen = tower;
            }
        }
        if entity == pace_button {
            stepped = true;
        }
    }
    for (index, choice) in Ask::CHOICES.into_iter().enumerate() {
        if actions.just_pressed(choice) {
            bulwark.chosen = Tower::ALL[index];
        }
    }
    if stepped || actions.just_pressed(Ask::Pace) {
        bulwark.pace_step = (bulwark.pace_step + 1) % theme::PACE_STEPS.len();
    }
}

pub fn raise_the_tower(
    actions: Res<Actions<Ask>>,
    screen: Res<State<Screen>>,
    mut bulwark: ResMut<Bulwark>,
    mut commands: Commands,
) {
    if screen.current != Screen::Playing || !actions.just_pressed(Ask::Build) {
        return;
    }
    let Some(cell) = bulwark.hovered else {
        return;
    };

    if let Some(tower) = queries::tower_on(&bulwark, cell) {
        bulwark.selected = Some(tower);
        return;
    }
    bulwark.selected = None;

    let kind = bulwark.chosen;
    let cost = queries::build_cost(kind);
    if !bulwark.buildable || !queries::affordable(&bulwark, cost) {
        return;
    }
    let art = bulwark.art;
    let tower = spawn::raise_a_tower(&mut commands, &art, cell, kind);
    purse::take_the_cell(&mut bulwark, cell, tower, cost);
    let center = queries::cell_center(cell);
    effects::stone_settles(&mut commands, Vec3::new(center.x, center.y, 0.0));
}

pub fn work_the_tower(
    actions: Res<Actions<Ask>>,
    frame: Res<RetainedUiFrame>,
    mut bulwark: ResMut<Bulwark>,
    mut commands: Commands,
    towers: Query<(&Kind, &Level, &Post, &Focus, &LocalTransform)>,
) {
    let inspect = bulwark.hud.inspect;
    let Some(selected) = bulwark.selected else {
        return;
    };
    let mut raise = actions.just_pressed(Ask::Upgrade);
    let mut pull = actions.just_pressed(Ask::Sell);
    let mut turn = actions.just_pressed(Ask::Aim);
    for entity in ui_button_clicks(&frame) {
        if entity == inspect.upgrade {
            raise = true;
        } else if entity == inspect.sell {
            pull = true;
        } else if entity == inspect.aim {
            turn = true;
        }
    }
    if !raise && !pull && !turn {
        return;
    }

    let mut found = None;
    towers.for_each(|entity, (kind, level, post, focus, transform)| {
        if entity == selected {
            found = Some((kind.0, level.0, post.0, focus.0, transform.translation));
        }
    });
    let Some((kind, level, post, focus, at)) = found else {
        return;
    };

    if turn {
        commands.set(selected, Focus((focus + 1) % 3));
        return;
    }

    if pull {
        purse::free_the_cell(&mut bulwark, post, queries::sell_value(kind, level));
        commands.despawn(selected);
        bulwark.selected = None;
        effects::stone_settles(&mut commands, at);
        return;
    }

    let Some(cost) = queries::upgrade_cost(kind, level) else {
        return;
    };
    if !queries::affordable(&bulwark, cost) {
        return;
    }
    purse::pay_the_cost(&mut bulwark, cost);
    let art = bulwark.art;
    spawn::raise_the_level(&mut commands, &art, selected, kind, level + 1);
    effects::tower_rises(&mut commands, at);
}

pub fn show_the_ghost(bulwark: Res<Bulwark>, storage: &mut nightshade::ecs::Storage) {
    let Some(ghost) = bulwark.ghost else {
        return;
    };
    let cost = queries::build_cost(bulwark.chosen);
    let wanted = bulwark
        .hovered
        .filter(|_| bulwark.buildable && queries::affordable(&bulwark, cost));

    if let Some(visibility) = storage.get_mut::<Visibility>(ghost) {
        visibility.visible = wanted.is_some();
    }
    let Some(cell) = wanted else {
        return;
    };
    let center = queries::cell_center(cell);
    if let Some(transform) = storage.get_mut::<LocalTransform>(ghost) {
        transform.translation = Vec3::new(center.x, center.y, 0.0);
    }
    let piece = bulwark.art.towers[bulwark.chosen.slot()][0];
    if let Some(sprite) = storage.get_mut::<Sprite>(ghost) {
        sprite.region = piece.region;
        sprite.size = piece.size;
        sprite.anchor = piece.anchor;
        sprite.color = theme::GHOST_COLOR;
    }
}
