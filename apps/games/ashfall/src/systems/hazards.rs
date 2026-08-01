use crate::commands::{hurt_player, post_message, unblock_tile};
use crate::components::{Explosive, Glow, Health, Opens, Tile};
use crate::data::rules::*;
use crate::data::theme;
use crate::resources::{AshfallResources, Screen, Senses};
use crate::systems::{effects, husks};
use crate::tags::Player;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;
use nightshade_senses::Noises;
use nightshade_senses::commands::make_a_noise;
use nightshade_ui_flash::Flash;
use nightshade_ui_flash::commands::flash;

pub fn update(
    time: Res<Time>,
    mut resources: ResMut<AshfallResources>,
    mut senses: ResMut<Senses>,
    mut noises: ResMut<Noises>,
    mut screen: ResMut<Flash>,
    mut commands: Commands,
    mut panes: Query<(&mut Health, &Glow, &Tile, &Opens, &LocalTransform)>,
    mut barrels: Query<(&mut Explosive, &LocalTransform)>,
    mut husks: husks::Wounded<'_, '_>,
    mut players: Query<(&LocalTransform,), WithTag<Player>>,
) {
    let resources = &mut *resources;
    let screen = &mut *screen;
    if resources.screen != Screen::Playing {
        return;
    }
    let impacts = std::mem::take(&mut senses.impacts);
    let delta = time.delta_time;

    let mut shattered = Vec::new();
    panes.for_each_mut(|entity, (health, glow, tile, opens, transform)| {
        let here = transform.translation;
        if !impacts
            .iter()
            .any(|impact| impact.force >= GLASS_MINIMUM && (impact.at - here).magnitude() < 0.9)
        {
            return;
        }
        health.0 -= 1.0;
        if health.0 <= 0.0 {
            shattered.push((entity, here, glow.0, tile.0, opens.0));
        }
    });
    let mut broke_glass = false;
    let mut broke_boards = false;
    for (entity, here, light, tile, opens) in shattered {
        commands.despawn_recursive(entity);
        if let Some(light) = light {
            commands.despawn_recursive(light);
        }
        if opens {
            unblock_tile(&mut resources.run, tile, false);
            broke_boards = true;
        } else {
            broke_glass = true;
        }
        make_a_noise(&mut noises, here, NOISE_GLASS);
    }
    if broke_glass {
        post_message(
            &mut resources.run,
            "The pane goes in. The street keeps the sound for a long time.",
        );
    }
    if broke_boards {
        post_message(
            &mut resources.run,
            "The board splits and drops. There is a way through here now.",
        );
    }

    let mut blasts = Vec::new();
    barrels.for_each_mut(|entity, (barrel, transform)| {
        let here = transform.translation;
        match barrel.fuse {
            Some(remaining) if remaining - delta <= 0.0 => blasts.push((entity, here)),
            Some(remaining) => barrel.fuse = Some(remaining - delta),
            None => {
                let drum = here + Vec3::new(0.0, 0.5, 0.0);
                if impacts.iter().any(|impact| {
                    impact.force >= BARREL_TRIGGER && (impact.at - drum).magnitude() < 1.4
                }) {
                    barrel.fuse = Some(BARREL_FUSE);
                }
            }
        }
    });
    if blasts.is_empty() {
        return;
    }

    let mut feet = None;
    if let Some(body) = resources.player.body {
        players.for_entity(body, |(transform,)| {
            feet = Some(transform.translation - Vec3::new(0.0, 1.0, 0.0));
        });
    }

    let mut kills = 0;
    for (entity, here) in &blasts {
        commands.despawn_recursive(*entity);
        make_a_noise(&mut noises, *here, NOISE_BLAST);
        husks.for_each_mut(|_entity, (state, health, timer, moving, transform)| {
            let reach = (transform.translation - here).magnitude();
            if reach > BLAST_RADIUS {
                return;
            }
            let falloff = 1.0 - reach / BLAST_RADIUS;
            if husks::wound(state, health, timer, moving, BLAST_DAMAGE * falloff) {
                kills += 1;
            }
        });
        if let Some(feet) = feet {
            let reach = (feet - here).magnitude();
            if reach < BLAST_RADIUS {
                let falloff = 1.0 - reach / BLAST_RADIUS;
                hurt_player(&mut resources.player, screen, BLAST_PLAYER_DAMAGE * falloff);
            }
        }
    }
    resources.run.kills += kills;
    flash(screen, theme::BLAST_FLASH, BLAST_FADE);
    for (_, at) in &blasts {
        effects::spawn_stain(
            &mut commands,
            *at + Vec3::new(0.0, 1.0, 0.0),
            theme::SCORCH_SIZE,
            theme::SCORCH_COLOR,
        );
    }
    post_message(
        &mut resources.run,
        "The drum goes up. Heat, then a ringing that will not stop.",
    );
}
