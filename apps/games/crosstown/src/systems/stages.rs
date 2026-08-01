use crate::commands::block;
use crate::commands::interface::say;
use crate::commands::pickups::{scatter_breakables, scatter_pickups, spawn_pickup};
use crate::components::{Combo, Fling, Health, Holding, Lift, Piece, Pose, Rise, Stun};
use crate::data;
use crate::data::Look;
use crate::queries::{chief_wave, far_end, gate, kinds, look, wave_count, wave_line};
use crate::resources::{Mode, Ring, Screen, Sheets, Talk};
use nightshade::prelude::*;
use nightshade_graphics::prelude::{Atmosphere, RenderSettings};
use nightshade_render::data::config::{Fog, FogMode};
use nightshade_scene::prelude::*;
use nightshade_state::resources::NextState;

pub fn raise(
    sheets: Res<Sheets>,
    mut ring: ResMut<Ring>,
    mut settings: ResMut<RenderSettings>,
    mut commands: Commands,
    pieces: Query<(&Piece,)>,
    heroes: Rested,
) {
    if ring.built {
        return;
    }
    let current = ring.piece;
    pieces.for_each(|entity, (piece,)| {
        if piece.0 != current && piece.0 != u32::MAX {
            commands.despawn_recursive(entity);
        }
    });

    let shape = look(&ring);
    let end = far_end(&ring);
    ring.end_x = end;
    dress(&mut settings, shape);
    block::build(&mut commands, shape, end, current);
    scatter_breakables(&mut commands, shape, end, current);
    scatter_pickups(&mut commands, &sheets, end, current);

    ring.wave = ring.wave.min(wave_count(&ring).saturating_sub(1));
    ring.waves = wave_count(&ring);
    ring.camera_x = data::STREET_START + 9.0;
    ring.built = true;
    ring.opened = false;

    let start = nalgebra_glm::Vec3::new(data::STREET_START + 9.0, 0.0, 0.6);
    ring.feet = start;
    heroes.for_each(
        |_entity, (transform, holding, stun, lift, rise, fling, _)| {
            transform.translation = start;
            holding.0 = 0;
            stun.0 = 0.0;
            lift.0 = 0.0;
            rise.0 = 0.0;
            fling.0 = 0.0;
        },
    );
}

pub type Rested<'world, 'item> = Query<
    'world,
    (
        &'item mut LocalTransform,
        &'item mut Holding,
        &'item mut Stun,
        &'item mut Lift,
        &'item mut Rise,
        &'item mut Fling,
        &'item Combo,
    ),
>;

pub fn open(
    sheets: Res<Sheets>,
    mut ring: ResMut<Ring>,
    mut talk: ResMut<Talk>,
    mut commands: Commands,
) {
    if !ring.built || ring.opened {
        return;
    }
    ring.opened = true;
    for entity in std::mem::take(&mut ring.gate) {
        commands.despawn_recursive(entity);
    }
    ring.gate_x = gate(&ring);
    ring.last_wave = ring.wave + 1 >= ring.waves.max(1);
    ring.gate = block::gate_posts(&mut commands, look(&ring), ring.gate_x, ring.piece);
    ring.left = kinds(&ring);
    ring.alive = 0;
    ring.delay = data::SPAWN_DELAY;
    ring.cleared = false;
    ring.stepped = false;
    ring.flash = 0.0;
    ring.mark = 1.0;
    ring.chief_share = 0.0;

    say(&mut talk, &wave_line(&ring));
    if !chief_wave(&ring) {
        return;
    }
    say(&mut talk, data::CHIEF_LINE);
    let rest = nalgebra_glm::Vec3::new(ring.feet.x + 2.5, 0.0, ring.feet.z);
    spawn_pickup(&mut commands, &sheets, data::ROAST, rest, ring.piece);
}

pub fn advance(
    mut ring: ResMut<Ring>,
    mut talk: ResMut<Talk>,
    mut next: ResMut<NextState<Screen>>,
    heroes: Query<(&Health, &mut Pose, &Combo)>,
) {
    let mut health = ring.hero_health;
    let cheering = ring.cleared && ring.flash > 0.0;
    heroes.for_each(|_entity, (value, pose, _)| {
        health = value.0;
        if cheering && pose.0 == data::POSE_IDLE {
            pose.0 = data::POSE_CHEER;
        }
    });
    ring.hero_health = health;
    if health <= 0.0 && !ring.dead {
        ring.dead = true;
        ring.won = false;
        let line = if ring.mode == Mode::Arcade {
            data::ARCADE_OVER
        } else {
            data::DEAD_LINE
        };
        say(&mut talk, line);
        next.pending = Some(Screen::Over);
        return;
    }
    if !ring.stepped {
        return;
    }
    ring.stepped = false;
    ring.opened = false;
    if !ring.last_wave {
        ring.wave += 1;
        return;
    }
    ring.wave = 0;
    ring.piece += 1;
    ring.built = false;
    match ring.mode {
        Mode::Story => {
            if ring.stage + 1 >= data::STAGE_COUNT {
                ring.won = true;
                say(&mut talk, data::VICTORY_LINE);
                next.pending = Some(Screen::Over);
                return;
            }
            ring.stage += 1;
        }
        Mode::Arcade => ring.round += 1,
    }
}

fn dress(settings: &mut RenderSettings, shape: Look) {
    let (color, start, end, sky) = match shape {
        Look::Street => (
            data::FOG_COLOR,
            data::FOG_START,
            data::FOG_END,
            Atmosphere::Nebula,
        ),
        Look::Alley => (data::ALLEY_FOG, 10.0, 34.0, Atmosphere::None),
        Look::Subway => (data::SUBWAY_FOG, 8.0, 30.0, Atmosphere::None),
        Look::Rooftop => (data::ROOFTOP_FOG, 18.0, 70.0, Atmosphere::Nebula),
        Look::Yard => (data::YARD_FOG, 12.0, 42.0, Atmosphere::Nebula),
    };
    settings.atmosphere = sky;
    settings.fog = Some(Fog {
        color,
        start,
        end,
        mode: FogMode::Linear,
        ..Default::default()
    });
}
