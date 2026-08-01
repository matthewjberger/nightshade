use crate::commands as game;
use crate::contract::{Cue, Dimension, FourfoldEvent};
use crate::data;
use crate::queries;
use crate::resources::FourfoldResources;
use nightshade::prelude::*;
use nightshade_camera::prelude::ActiveCamera;
use nightshade_graphics::prelude::RenderSettings;
use nightshade_portal::prelude::PortalSettings;
use nightshade_render::resources::viewport::Viewport;
use nightshade_state::prelude::NextState;

pub fn leave_hub(
    mut resources: ResMut<FourfoldResources>,
    mut next: ResMut<NextState<Dimension>>,
    mut events: EventWriter<FourfoldEvent>,
) {
    let Some(gate) = data::gates()
        .into_iter()
        .find(|gate| queries::in_arch(&resources.walker, gate))
    else {
        return;
    };
    resources.fade = 1.0;
    next.pending = Some(gate.dimension);
    events.send(FourfoldEvent::Crossed(gate.dimension));
}

pub fn leave_glass(
    mut resources: ResMut<FourfoldResources>,
    mut next: ResMut<NextState<Dimension>>,
    mut events: EventWriter<FourfoldEvent>,
) {
    if !queries::at_gate(&resources.runner) {
        return;
    }
    go_home(&mut resources, &mut next, &mut events);
}

pub fn leave_loom(
    mut resources: ResMut<FourfoldResources>,
    mut next: ResMut<NextState<Dimension>>,
    mut events: EventWriter<FourfoldEvent>,
) {
    if resources.stepper.cell != data::LOOM_GATE {
        return;
    }
    go_home(&mut resources, &mut next, &mut events);
}

fn go_home(
    resources: &mut FourfoldResources,
    next: &mut NextState<Dimension>,
    events: &mut EventWriter<FourfoldEvent>,
) {
    resources.crossings += 1;
    resources.fade = 1.0;
    next.pending = Some(Dimension::Hub);
    events.send(FourfoldEvent::Crossed(Dimension::Hub));
}

pub fn leave_cairn(
    mut resources: ResMut<FourfoldResources>,
    mut next: ResMut<NextState<Dimension>>,
    mut events: EventWriter<FourfoldEvent>,
) {
    if !resources.lit[queries::tone_index(Dimension::Cairn)] {
        return;
    }
    if !queries::at_spot(
        resources.digger.position,
        data::CAIRN_GATE,
        data::CAIRN_GATE_REACH,
    ) {
        return;
    }
    go_home(&mut resources, &mut next, &mut events);
}

pub fn leave_hearth(
    mut resources: ResMut<FourfoldResources>,
    mut next: ResMut<NextState<Dimension>>,
    mut events: EventWriter<FourfoldEvent>,
) {
    if !resources.lit[queries::tone_index(Dimension::Hearth)] {
        return;
    }
    if !queries::at_spot(
        resources.villager.position,
        data::HEARTH_GATE,
        data::HEARTH_GATE_REACH,
    ) {
        return;
    }
    go_home(&mut resources, &mut next, &mut events);
}

pub fn enter_cairn(
    mut resources: ResMut<FourfoldResources>,
    mut active_camera: ResMut<ActiveCamera>,
    mut portals: ResMut<PortalSettings>,
    mut viewport: ResMut<Viewport>,
    mut settings: ResMut<RenderSettings>,
) {
    resources.digger.position = data::CAIRN_START;
    resources.digger.velocity = Vec3::zeros();
    resources.digger.yaw = 0.0;
    resources.digger.pitch = 0.0;
    game::queue_cue(&mut resources.story, Cue::Entered);
    active_camera.0 = Some(resources.cairn_camera);
    portals.viewer = Entity::default();
    dispatch(&resources, &mut viewport, Dimension::Cairn);
    if let Some(fog) = settings.fog.as_mut() {
        fog.color = data::CAIRN_FOG;
        fog.start = data::CAIRN_FOG_START;
        fog.end = data::CAIRN_FOG_END;
    }
}

pub fn enter_hearth(
    mut resources: ResMut<FourfoldResources>,
    mut active_camera: ResMut<ActiveCamera>,
    mut portals: ResMut<PortalSettings>,
    mut viewport: ResMut<Viewport>,
    mut settings: ResMut<RenderSettings>,
) {
    resources.villager.position = data::HEARTH_START;
    resources.villager.velocity = Vec3::zeros();
    resources.villager.focus = data::HEARTH_START;
    game::queue_cue(&mut resources.story, Cue::Entered);
    active_camera.0 = Some(resources.hearth_camera);
    portals.viewer = Entity::default();
    dispatch(&resources, &mut viewport, Dimension::Hearth);
    if let Some(fog) = settings.fog.as_mut() {
        fog.color = data::HEARTH_FOG;
        fog.start = data::HEARTH_FOG_START;
        fog.end = data::HEARTH_FOG_END;
    }
}

pub fn enter_hub(
    mut resources: ResMut<FourfoldResources>,
    mut active_camera: ResMut<ActiveCamera>,
    mut portals: ResMut<PortalSettings>,
    mut viewport: ResMut<Viewport>,
    mut settings: ResMut<RenderSettings>,
) {
    if resources.crossings > 0 {
        game::place_in_hub(&mut resources.walker);
        game::queue_cue(&mut resources.story, Cue::Returned);
    }
    active_camera.0 = Some(resources.hub_camera);
    portals.viewer = resources.hub_camera;
    dispatch(&resources, &mut viewport, Dimension::Hub);
    if let Some(fog) = settings.fog.as_mut() {
        fog.color = data::HUB_FOG;
        fog.start = data::FOG_START;
        fog.end = data::FOG_END;
    }
}

pub fn enter_glass(
    mut resources: ResMut<FourfoldResources>,
    mut active_camera: ResMut<ActiveCamera>,
    mut portals: ResMut<PortalSettings>,
    mut viewport: ResMut<Viewport>,
    mut settings: ResMut<RenderSettings>,
) {
    game::place_in_glass(&mut resources.runner);
    game::queue_cue(&mut resources.story, Cue::Entered);
    active_camera.0 = Some(resources.glass_camera);
    portals.viewer = Entity::default();
    dispatch(&resources, &mut viewport, Dimension::Glass);
    if let Some(fog) = settings.fog.as_mut() {
        fog.color = data::GLASS_FOG;
        fog.start = data::FOG_START;
        fog.end = data::FOG_END;
    }
}

pub fn enter_loom(
    mut resources: ResMut<FourfoldResources>,
    mut active_camera: ResMut<ActiveCamera>,
    mut portals: ResMut<PortalSettings>,
    mut viewport: ResMut<Viewport>,
    mut settings: ResMut<RenderSettings>,
) {
    resources.stepper.cell = data::LOOM_START;
    resources.stepper.rest = data::LOOM_STEP_SECONDS;
    game::queue_cue(&mut resources.story, Cue::Entered);
    active_camera.0 = Some(resources.loom_camera);
    portals.viewer = Entity::default();
    dispatch(&resources, &mut viewport, Dimension::Loom);
    if let Some(fog) = settings.fog.as_mut() {
        fog.color = data::LOOM_FOG;
        fog.start = data::LOOM_FOG_START;
        fog.end = data::LOOM_FOG_END;
    }
}

fn dispatch(resources: &FourfoldResources, viewport: &mut Viewport, here: Dimension) {
    let cameras = [
        resources.hub_camera,
        resources.glass_camera,
        resources.loom_camera,
        resources.cairn_camera,
        resources.hearth_camera,
    ];
    let active = match here {
        Dimension::Hub => resources.hub_camera,
        Dimension::Glass => resources.glass_camera,
        Dimension::Loom => resources.loom_camera,
        Dimension::Cairn => resources.cairn_camera,
        Dimension::Hearth => resources.hearth_camera,
    };
    game::dispatch_last(viewport, cameras, active);
}
