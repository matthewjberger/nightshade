use crate::components::{Anchor, Found, Inside, StandingOn};
use crate::contract::OrreryEvent;
use crate::data;
use crate::queries;
use crate::resources::{Mode, OrreryResources, Signal};
use nightshade::prelude::*;
use nightshade_graphics::commands::builtins::set_mesh_material;
use nightshade_graphics::prelude::*;

pub fn touch(
    mut resources: ResMut<OrreryResources>,
    mut events: EventWriter<OrreryEvent>,
    mut commands: Commands,
    mut beacons: Query<(
        &StandingOn,
        &Anchor,
        &Inside,
        &mut Found,
        &mut LocalTransform,
    )>,
) {
    let elapsed = resources.flight.elapsed;
    let on_foot = resources.flight.mode == Mode::OnFoot;
    let walker = resources.pose.walker_position;
    let ship = resources.pose.ship_position;
    let mut reached = Vec::new();
    let mut sweep: Option<Signal> = None;

    beacons.for_each_mut(|entity, (standing_on, anchor, inside, found, transform)| {
        let up = queries::normalized_or(anchor.0, Vec3::y());
        let up = if inside.0 { -up } else { up };
        let (forward, _) = queries::tangent_frame(up, 0.0);
        transform.translation = queries::surface_position(
            standing_on.0,
            anchor.0,
            data::BEACON_HEIGHT * 0.5,
            inside.0,
            elapsed,
        );
        transform.rotation = queries::basis_rotation(forward, up);

        if !found.0 {
            let distance = (transform.translation - ship).magnitude();
            if sweep.is_none_or(|current| distance < current.distance) {
                sweep = Some(Signal {
                    body: standing_on.0,
                    distance,
                });
            }
        }

        if !found.0 && on_foot && (transform.translation - walker).magnitude() < data::BEACON_REACH
        {
            found.0 = true;
            reached.push((entity, standing_on.0));
        }
    });

    resources.sweep = sweep;
    for (entity, body) in reached {
        set_mesh_material(&mut commands, entity, "orrery_beacon_found");
        crate::commands::record_signal(&mut resources);
        resources.logged.push(body);
        events.send(OrreryEvent::SignalFound { body });
    }
}
