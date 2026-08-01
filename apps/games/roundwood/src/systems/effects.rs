use crate::commands::life::spawn_shell;
use crate::components::{Lean, Regrow, Ripe, Rocks, Shaking, Tree};
use crate::data;
use crate::queries::sheet::tile;
use crate::resources::{Beach, Sheet, Toast, Town, Village};
use crate::tags::Bush;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_sprite::prelude::*;

pub fn shake_the_trees(time: Res<Time>, trees: Query<(&mut Shaking, &mut Lean, &mut Sprite)>) {
    let delta_time = time.delta_time;
    trees.for_each(|_entity, (shaking, lean, sprite)| {
        if shaking.0 <= 0.0 {
            return;
        }
        shaking.0 = (shaking.0 - delta_time).max(0.0);
        let share = shaking.0 / data::SHAKE_TIME;
        lean.0 = (shaking.0 * 34.0).sin() * data::SHAKE_LEAN * share;
        sprite.lean = lean.0;
    });
}

pub fn ripen_the_bushes(
    time: Res<Time>,
    sheet: Res<Sheet>,
    bushes: Query<(&mut Ripe, &mut Regrow, &mut Sprite), WithTag<Bush>>,
) {
    let delta_time = time.delta_time;
    let ripe = tile(&sheet, data::BERRY_BUSH);
    bushes.for_each(|_entity, (is_ripe, regrow, sprite)| {
        if is_ripe.0 {
            return;
        }
        regrow.0 -= delta_time;
        if regrow.0 > 0.0 {
            return;
        }
        is_ripe.0 = true;
        sprite.region = ripe;
    });
}

pub fn rest_the_ground(time: Res<Time>, trees: Query<(&mut Tree,)>, rocks: Query<(&mut Rocks,)>) {
    let delta_time = time.delta_time;
    trees.for_each(|_entity, (tree,)| {
        tree.0 = (tree.0 - delta_time).max(0.0);
    });
    rocks.for_each(|_entity, (rocks,)| {
        rocks.0 = (rocks.0 - delta_time).max(0.0);
    });
}

pub fn wash_up_shells(
    time: Res<Time>,
    town: Res<Town>,
    sheet: Res<Sheet>,
    village: Res<Village>,
    mut beach: ResMut<Beach>,
    mut commands: Commands,
) {
    let Some(turntable) = village.turntable else {
        return;
    };
    beach.timer -= time.delta_time;
    if beach.timer > 0.0 {
        return;
    }
    beach.timer = data::SHELL_RETURN;
    spawn_shell(&mut commands, &sheet, &town, turntable, &mut beach.rng);
}

pub fn fade_the_toast(time: Res<Time>, mut toast: ResMut<Toast>) {
    if toast.remaining <= 0.0 {
        return;
    }
    toast.remaining -= time.delta_time;
    if toast.remaining <= 0.0 {
        toast.line.clear();
    }
}
