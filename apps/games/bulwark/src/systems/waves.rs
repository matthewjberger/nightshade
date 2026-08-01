use crate::commands::{effects, spawn};
use crate::data::{self, Ask, Settings};
use crate::queries;
use crate::resources::{Bulwark, Screen};
use crate::theme;
use nightshade::prelude::*;
use nightshade_config::contract::Config;
use nightshade_input::Actions;
use nightshade_state::prelude::{NextState, State};
use nightshade_ui::prelude::ui_button_clicks;
use nightshade_ui::resources::RetainedUiFrame;
use nightshade_waves::commands::hurry;
use nightshade_waves::{Arrival, WaveCleared, WavePlan, WaveProgress};

pub fn set_the_pace(bulwark: Res<Bulwark>, mut progress: ResMut<WaveProgress>) {
    progress.pace = queries::pace(&bulwark);
}

pub fn meet_the_arrivals(
    config: Res<Config<Settings>>,
    arrivals: EventReader<Arrival>,
    bulwark: Res<Bulwark>,
    mut commands: Commands,
) {
    let share = data::DIFFICULTY_SHARE[config.choice(Settings::Difficulty).min(2)];
    for arrival in arrivals.iter() {
        let mut wave = queries::wave_of(arrival.wave);
        let scale = queries::wave_scale(arrival.wave);
        wave.ward *= scale;
        let heavy = wave.boss && arrival.last;
        let mut health = wave.health * share * scale;
        if heavy {
            health *= theme::BOSS_HEALTH;
        }
        let walker = data::Walker {
            tier: wave.tier,
            health,
            speed: wave.speed,
            bounty: wave.bounty,
            ward: wave.ward,
            splits: wave.splits,
        };
        spawn::send_a_crystal(
            &mut commands,
            &bulwark.art,
            bulwark.board,
            walker,
            (0, 0.0),
            heavy,
        );
    }
}

pub fn count_the_cleared(
    config: Res<Config<Settings>>,
    cleared: EventReader<WaveCleared>,
    mut bulwark: ResMut<Bulwark>,
    mut plan: ResMut<WavePlan>,
    mut progress: ResMut<WaveProgress>,
    mut next: ResMut<NextState<Screen>>,
    mut commands: Commands,
) {
    let endless = config.toggle(Settings::Endless);
    for wave in cleared.iter() {
        if !endless && wave.wave + 1 >= queries::waves_total() {
            next.pending = Some(Screen::Victory);
            return;
        }
        if wave.wave + 1 >= plan.0.len() {
            let next_wave = queries::planned_wave(plan.0.len());
            plan.0.push(next_wave);
        }
        progress.rest = config.number(Settings::Rest);
        let bounty = queries::wave_bounty(wave.wave);
        bulwark.coin = bulwark.coin.saturating_add(bounty);
        let keep = queries::cell_center(queries::keep_cell(bulwark.board));
        effects::wave_is_cleared(&mut commands, Vec3::new(keep.x, keep.y, 0.0));
    }
}

pub fn hurry_the_march(
    actions: Res<Actions<Ask>>,
    frame: Res<RetainedUiFrame>,
    screen: Res<State<Screen>>,
    mut bulwark: ResMut<Bulwark>,
    mut progress: ResMut<WaveProgress>,
) {
    if screen.current != Screen::Playing || !progress.resting {
        return;
    }
    let send = bulwark.hud.send;
    let asked =
        actions.just_pressed(Ask::Hurry) || ui_button_clicks(&frame).any(|entity| entity == send);
    if !asked {
        return;
    }
    let saved = hurry(&mut progress).ceil() as u32;
    bulwark.coin = bulwark
        .coin
        .saturating_add(saved.saturating_mul(theme::EARLY_BONUS));
}
