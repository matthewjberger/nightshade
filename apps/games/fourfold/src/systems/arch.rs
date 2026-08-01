use crate::commands as game;
use crate::components::{Dial, Pip};
use crate::contract::{Cue, FourfoldEvent};
use crate::data;
use crate::queries;
use crate::resources::{FourfoldResources, Screen};
use nightshade::prelude::*;
use nightshade_graphics::commands::builtins::set_mesh_material;
use nightshade_platform::prelude::Time;
use nightshade_scene::prelude::Visibility;
use nightshade_state::prelude::NextState;

pub fn turn(
    mut resources: ResMut<FourfoldResources>,
    mut events: EventWriter<FourfoldEvent>,
    pips: Query<(&Dial, &Pip, &mut Visibility)>,
) {
    if resources.arch.open || !resources.walker.near_dials {
        return;
    }
    let Some(dial) = resources.drive.dial else {
        return;
    };

    let glyph = game::turn_dial(&mut resources.arch, dial);
    game::queue_cue(&mut resources.story, Cue::Turned);
    events.send(FourfoldEvent::Turned { dial, glyph });
    pips.for_each(|_entity, (which, pip, visibility)| {
        if which.0 != dial {
            return;
        }
        visibility.visible = data::pip_lit(glyph, pip.0);
    });
}

pub fn open(
    mut resources: ResMut<FourfoldResources>,
    mut events: EventWriter<FourfoldEvent>,
    mut commands: Commands,
) {
    if resources.arch.open || !queries::dials_match(&resources.arch) {
        return;
    }
    game::open_arch(&mut resources.arch);
    game::queue_cue(&mut resources.story, Cue::Opened);
    events.send(FourfoldEvent::Opened);
    if resources.seal != Entity::default() {
        commands.set(resources.seal, Visibility { visible: false });
        commands.set(resources.seal, crate::components::Extent(Vec3::zeros()));
    }
}

pub fn descend(
    time: Res<Time>,
    mut resources: ResMut<FourfoldResources>,
    mut next_screen: ResMut<NextState<Screen>>,
    mut commands: Commands,
) {
    if resources.ending > 0.0 {
        resources.ending -= time.delta_time;
        game::restart_hub(&mut resources.walker);
        if resources.ending > 0.0 {
            return;
        }
        close_out(&mut resources, &mut commands);
        next_screen.pending = Some(Screen::Title);
        return;
    }
    if !resources.arch.open || resources.walker.position.y > data::SHAFT_MOUTH {
        return;
    }
    let flat = Vec3::new(
        resources.walker.position.x,
        0.0,
        resources.walker.position.z,
    );
    if flat.norm() > data::SHAFT_WIDTH * 0.5 {
        return;
    }
    game::queue_cue(&mut resources.story, Cue::Ended);
    resources.ending = data::ENDING_HOLD;
    game::restart_hub(&mut resources.walker);
}

fn close_out(resources: &mut FourfoldResources, commands: &mut Commands) {
    game::close_arch(&mut resources.arch);
    resources.ending = 0.0;
    resources.crossings = 0;
    resources.autoplay.step = 0;
    resources.autoplay.held = 0.0;
    resources.autoplay.jumps = 0;
    resources.autoplay.moves = 0;
    resources.story = crate::resources::Story::default();
    for index in 0..data::DIAL_COUNT {
        resources.lit[index] = false;
        if resources.frames[index] != Entity::default() {
            set_mesh_material(commands, resources.frames[index], data::PLATE);
        }
    }
    if resources.seal != Entity::default() {
        commands.set(resources.seal, Visibility { visible: true });
        commands.set(
            resources.seal,
            crate::components::Extent(Vec3::new(data::SHAFT_WIDTH, 0.8, data::SHAFT_WIDTH)),
        );
    }
}
