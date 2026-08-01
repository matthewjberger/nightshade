use crate::components::Velocity;
use crate::contract::HairpinEvent;
use crate::data::{LINE, LINE_FORWARD, MINIMUM_LAP_SECONDS};
use crate::queries::lap_readout;
use crate::resources::{HairpinResources, Screen};
use nightshade::prelude::*;
use nightshade_platform::prelude::Time;
use nightshade_scene::prelude::LocalTransform;
use nightshade_ui::prelude::ui_set_text;

pub fn update(
    time: Res<Time>,
    mut resources: ResMut<HairpinResources>,
    mut events: EventWriter<HairpinEvent>,
    mut commands: Commands,
    cars: Query<(&Velocity, &LocalTransform)>,
) {
    if resources.screen != Screen::Playing {
        return;
    }
    resources.lap.current += time.delta_time;

    let mut sample = None;
    cars.for_each(|_entity, (velocity, transform)| {
        sample = Some((
            (transform.translation - LINE).dot(&LINE_FORWARD),
            velocity.0.norm(),
        ));
    });
    let Some((ahead, speed)) = sample else {
        return;
    };

    let crossed = ahead >= 0.0 && !resources.lap.ahead_of_line;
    resources.lap.ahead_of_line = ahead >= 0.0;
    if crossed && resources.lap.current >= MINIMUM_LAP_SECONDS {
        let seconds = resources.lap.current;
        let best = resources.lap.best.is_none_or(|previous| seconds < previous);
        if best {
            resources.lap.best = Some(seconds);
        }
        resources.lap.count += 1;
        events.send(HairpinEvent::LapFinished {
            lap: resources.lap.count,
            seconds,
            best,
        });
        resources.lap.current = 0.0;
    }

    if let Some(hud) = resources.ui.hud {
        ui_set_text(&mut commands, hud, &lap_readout(&resources.lap, speed));
    }
}
