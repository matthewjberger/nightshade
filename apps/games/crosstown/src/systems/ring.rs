use crate::commands::fighters::spawn_thug;
use crate::commands::interface::say;
use crate::components::{Chief, Health, Piece};
use crate::contract::CrosstownEvent;
use crate::data;
use crate::queries::{at_once, chief_wave, noise};
use crate::resources::{Mode, Ring, Sheets, Talk};
use nalgebra_glm::Vec3;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;

pub fn keep(
    time: Res<Time>,
    sheets: Res<Sheets>,
    mut ring: ResMut<Ring>,
    mut talk: ResMut<Talk>,
    mut events: EventWriter<CrosstownEvent>,
    mut commands: Commands,
    gates: Query<(&mut Visibility, &Piece)>,
) {
    if !ring.built || !ring.opened || ring.dead {
        return;
    }
    let delta_time = time.delta_time;
    let waiting: u32 = ring.left.iter().sum();
    if !ring.cleared && waiting > 0 {
        ring.delay -= delta_time;
        if ring.alive < at_once(&ring) && ring.delay <= 0.0 {
            let kind = pick(&ring.left, ring.score + waiting, chief_wave(&ring));
            let seed = waiting.wrapping_mul(37).wrapping_add(ring.round * 7);
            let side = if ring.from_right { 1.0 } else { -1.0 };
            let at = Vec3::new(
                ring.camera_x + side * data::SPAWN_MARGIN,
                0.0,
                (noise(seed) - 0.5) * 2.0 * data::LANE_HALF,
            );
            let slot = (noise(seed.wrapping_add(5)) - 0.5) * 2.0;
            spawn_thug(&mut commands, &sheets, kind, at, -side, slot, ring.piece);
            ring.from_right = !ring.from_right;
            ring.left[kind as usize] -= 1;
            ring.alive += 1;
            ring.delay = data::SPAWN_DELAY;
            let pick = talk.next_taunt as usize % data::TAUNTS.len();
            talk.next_taunt += 1;
            say(&mut talk, data::TAUNTS[pick]);
        }
    }

    if !ring.cleared && waiting == 0 && ring.alive == 0 {
        ring.cleared = true;
        ring.flash = data::GO_TIME;
        ring.score += if chief_wave(&ring) {
            data::CHIEF_SCORE
        } else {
            data::CLEAR_SCORE
        };
        if ring.last_wave {
            say(&mut talk, data::CLEARED_LINE);
        }
        events.send(CrosstownEvent::Cleared);
        let open = std::mem::take(&mut ring.gate);
        gates.for_each(|entity, (visibility, _)| {
            if open.contains(&entity) {
                visibility.visible = false;
            }
        });
    }

    if ring.flash > 0.0 {
        ring.flash = (ring.flash - delta_time).max(0.0);
    }
    if ring.cleared && ring.feet.x > ring.gate_x + 1.5 {
        ring.stepped = true;
    }
    if ring.mode == Mode::Arcade && ring.cleared && ring.flash <= 0.0 {
        ring.stepped = true;
    }
}

pub fn watch(mut ring: ResMut<Ring>, chiefs: Query<(&Health, &Chief)>) {
    let mut found = None;
    chiefs.for_each(|entity, (health, chief)| {
        if health.0 <= 0.0 {
            return;
        }
        let plan = &data::THUG_KINDS[chief.0 as usize];
        found = Some((entity, health.0 / plan.health, plan.name));
    });
    match found {
        Some((entity, share, name)) => {
            ring.chief = entity;
            ring.chief_share = share;
            ring.chief_name = name;
        }
        None => {
            ring.chief = Entity::default();
            ring.chief_share = 0.0;
        }
    }
}

fn pick(left: &[u32; data::KIND_COUNT], seed: u32, chief: bool) -> u32 {
    let total: u32 = left.iter().sum();
    if total == 0 {
        return data::PUNK;
    }
    if chief && let Some(index) = left.iter().rposition(|count| *count > 0) {
        return index as u32;
    }
    let mut step = (noise(seed.wrapping_mul(13)) * total as f32) as u32;
    for (index, count) in left.iter().enumerate() {
        if step < *count {
            return index as u32;
        }
        step -= count;
    }
    left.iter()
        .position(|count| *count > 0)
        .unwrap_or(data::PUNK as usize) as u32
}
