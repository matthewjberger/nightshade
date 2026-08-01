use crate::commands::{npc_say, system_line};
use crate::data::chat::{
    DAWN_LINES, DUSK_LINES, GATE_CALLER, GATE_DAWN, GATE_DUSK, VILLAGE_WELCOME,
};
use crate::resources::{Chat, Noted};
use crate::theme;
use blocker_figure::components::Eye;
use blocker_figure::queries::read_the_hero;
use blocker_figure::resources::Hero;
use blocker_quests::commands::reward;
use blocker_quests::resources::Journal;
use blocker_screen::data::notice::Notice;
use blocker_weather::resources::Sky;
use blocker_world::data::world::places::{CAPITALS, POIS};
use blocker_world::queries::{biome_for, surface_height};
use blocker_world::resources::{Bench, Generation};
use nightshade::ecs::system_param::EventWriter;
use nightshade::prelude::*;
use nightshade_random::Random;
use nightshade_random::commands::index;
use nightshade_render::data::config::RendererState;

pub fn announce_the_world(
    renderer_state: Res<RendererState>,
    sky: Res<Sky>,
    generation: Res<Generation>,
    bench: Res<Bench>,
    hero: Res<Hero>,
    mut journal: ResMut<Journal>,
    mut notices: EventWriter<Notice>,
    mut chat: ResMut<Chat>,
    mut noted: ResMut<Noted>,
    mut random: ResMut<Random>,
    mut bodies: Query<(&Eye,)>,
) {
    let Some(eye) = read_the_hero(&hero, &mut bodies, |(sight,)| sight.0) else {
        return;
    };
    let hour = renderer_state.day_night.hour;
    let night = !(blocker_weather::data::weather::DAWN_HOUR
        ..blocker_weather::data::weather::DUSK_HOUR)
        .contains(&hour);
    if noted.night != Some(night) {
        let first = noted.night.is_none();
        noted.night = Some(night);
        if !first {
            let line = if night {
                DUSK_LINES[index(&mut random, DUSK_LINES.len())]
            } else {
                DAWN_LINES[index(&mut random, DAWN_LINES.len())]
            };
            system_line(&mut chat, line.to_string());
            let inside = bench.seats.iter().any(|seat| {
                (seat.city[0] as f32 - eye.x)
                    .abs()
                    .max((seat.city[1] as f32 - eye.z).abs())
                    <= blocker_world::data::world::CITY_WALL as f32
            });
            if inside {
                let call = if night {
                    GATE_DUSK[index(&mut random, GATE_DUSK.len())]
                } else {
                    GATE_DAWN[index(&mut random, GATE_DAWN.len())]
                };
                npc_say(&mut chat, GATE_CALLER, call);
            }
        }
    }
    if noted.weather != Some(sky.kind) {
        let first = noted.weather.is_none();
        noted.weather = Some(sky.kind);
        if !first {
            system_line(&mut chat, format!("{}.", sky.kind.word()));
        }
    }
    let across = eye.x.round() as i32;
    let along = eye.z.round() as i32;
    let biome = biome_for(&bench, across, along, surface_height(&bench, across, along));
    if noted.biome != Some(biome) {
        let first = noted.biome.is_none();
        noted.biome = Some(biome);
        if !first {
            system_line(&mut chat, format!("You come into the {}.", biome.name()));
        }
    }
    let mut found: Option<([i32; 3], String)> = None;
    for (sign, name) in generation.signs.iter() {
        if journal.discovered.contains(sign) {
            continue;
        }
        let across = sign[0] as f32 - eye.x;
        let along = sign[2] as f32 - eye.z;
        if across * across + along * along < theme::DISCOVER_RANGE * theme::DISCOVER_RANGE {
            found = Some((*sign, name.clone()));
            break;
        }
    }
    if let Some((sign, name)) = found {
        journal.discovered.insert(sign);
        system_line(&mut chat, format!("You come upon {name}."));
        let blurb = VILLAGE_WELCOME[index(&mut random, VILLAGE_WELCOME.len())];
        system_line(&mut chat, blurb.to_string());
        let landmark = CAPITALS.iter().any(|capital| capital.name == name)
            || POIS.iter().any(|poi| poi.name == name);
        let (xp, coin) = if landmark {
            (theme::LANDMARK_XP, theme::LANDMARK_COIN)
        } else {
            (theme::DISCOVER_XP, theme::DISCOVER_COIN)
        };
        reward(
            &mut journal,
            &mut notices,
            0,
            coin,
            xp,
            format!("Discovered {name} (+{xp} xp, +{coin} coin)"),
        );
    }
    let mut read: Option<([i32; 3], String)> = None;
    for (plaque, text) in generation.plaques.iter() {
        if journal.discovered.contains(plaque) {
            continue;
        }
        let across = plaque[0] as f32 - eye.x;
        let along = plaque[2] as f32 - eye.z;
        let lift = plaque[1] as f32 - eye.y;
        if across * across + along * along < theme::PLAQUE_RANGE * theme::PLAQUE_RANGE
            && lift.abs() < theme::PLAQUE_LIFT
        {
            read = Some((*plaque, text.clone()));
            break;
        }
    }
    if let Some((plaque, text)) = read {
        journal.discovered.insert(plaque);
        system_line(&mut chat, text);
    }
}
