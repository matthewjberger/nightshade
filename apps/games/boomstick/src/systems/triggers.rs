use crate::commands::hurt;
use crate::components::{EnemyMood, Group};
use crate::contract::BoomstickEvent;
use crate::data;
use crate::resources::BoomstickResources;
use crate::tags::Warden;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;

pub fn ambush(
    mut resources: ResMut<BoomstickResources>,
    sleepers: Query<(&EnemyMood, &mut Group)>,
) {
    let Some(at) = resources.player.at else {
        return;
    };
    let mut fired: Vec<u32> = Vec::new();
    resources.mission.ambushes.retain(|(center, half, group)| {
        let gap = at - center;
        let inside = gap.x.abs() < half.x && gap.y.abs() < half.y + 2.0 && gap.z.abs() < half.z;
        if inside {
            fired.push(*group);
        }
        !inside
    });
    if fired.is_empty() {
        return;
    }
    sleepers.for_each(|_entity, (_mood, group)| {
        if fired.contains(&group.0) {
            group.0 = 0;
        }
    });
}

pub fn hazards(
    time: Res<Time>,
    mut resources: ResMut<BoomstickResources>,
    mut events: EventWriter<BoomstickEvent>,
) {
    let delta_time = time.delta_time;
    let Some(at) = resources.player.at else {
        return;
    };
    if resources.player.dead {
        return;
    }
    let mut rate = 0.0;
    for (center, half, hurt_rate) in &resources.mission.hazards {
        let gap = at - center;
        if gap.x.abs() < half.x && gap.y.abs() < half.y && gap.z.abs() < half.z {
            rate += hurt_rate;
        }
    }
    if rate <= 0.0 {
        resources.player.hazard_clock = 0.0;
        return;
    }
    resources.player.hazard_clock += delta_time;
    if resources.player.hazard_clock < data::HAZARD_TICK {
        return;
    }
    resources.player.hazard_clock = 0.0;
    let damage = rate * data::HAZARD_TICK;
    hurt(&mut resources.player, damage);
    resources.player.flash = 0.7;
    events.send(BoomstickEvent::PlayerHurt { damage });
}

pub fn exit(
    mut resources: ResMut<BoomstickResources>,
    wardens: Query<(&EnemyMood,), WithTag<Warden>>,
) {
    if resources.mission.done || resources.mission.failed || !resources.mission.populated {
        return;
    }
    let Some(at) = resources.player.at else {
        return;
    };
    let plan = crate::data::levels::plan(resources.mission.level);
    let mut boss_standing = false;
    wardens.for_each(|_entity, (mood,)| {
        if *mood != EnemyMood::Dying {
            boss_standing = true;
        }
    });
    if boss_standing {
        return;
    }
    let gap = at - plan.exit;
    if gap.xz().magnitude() > data::EXIT_REACH || gap.y.abs() > 3.0 {
        return;
    }
    resources.mission.done = true;
    resources.mission.since_end = 0.0;
    resources.mission.banner = plan.outro.to_string();
    resources.mission.banner_left = data::CLEAR_DELAY;
}
