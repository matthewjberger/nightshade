use crate::components::{Footing, Ground, Grounded, Pitch, Velocity};
use crate::contract::TidefallEvent;
use crate::data::mode::Event as ModeEvent;
use crate::data::{Ask, BOARD_REACH, EXIT_OFFSET};
use crate::queries::{Bodies, collect_bodies};
use crate::resources::{Mode, TidefallResources};
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_scene::prelude::*;

pub fn change_the_mode(
    actions: Res<Actions<Ask>>,
    mut resources: ResMut<TidefallResources>,
    mut events: EventWriter<TidefallEvent>,
    bodies: Bodies<'_, '_>,
    mut ships: Query<(&Footing, &Ground, &LocalTransform)>,
    mut walkers: Query<(
        &mut Velocity,
        &mut Ground,
        &mut Grounded,
        &mut Pitch,
        &mut LocalTransform,
    )>,
) {
    if !actions.just_pressed(Ask::Interact) {
        return;
    }
    let (Some(ship), Some(walker)) = (resources.ship, resources.walker) else {
        return;
    };
    let mut parked = None;
    let mut hull = None;
    ships.for_entity(ship, |(footing, ground, transform)| {
        hull = Some(transform.translation);
        if *footing == Footing::Landed {
            parked = Some((ground.0, transform.translation, transform.rotation));
        }
    });

    match resources.mode {
        Mode::Aboard => {
            let Some((ground, translation, rotation)) = parked else {
                return;
            };
            let Some(next) = resources.mode.process_event(ModeEvent::Board) else {
                return;
            };
            resources.mode = next;
            let right = nalgebra_glm::quat_rotate_vec3(&rotation, &Vec3::x());
            let spot = translation + right * EXIT_OFFSET;
            walkers.for_entity(
                walker,
                |(velocity, walker_ground, grounded, pitch, transform)| {
                    transform.translation = spot;
                    transform.rotation = rotation;
                    pitch.0 = 0.0;
                    velocity.0 = Vec3::zeros();
                    walker_ground.0 = ground;
                    grounded.0 = false;
                },
            );
            let views = collect_bodies(bodies);
            let body = views
                .iter()
                .find(|view| Some(view.entity) == ground)
                .map(|view| view.name.clone())
                .unwrap_or_default();
            events.send(TidefallEvent::Disembarked { body });
        }
        Mode::Afoot => {
            let Some(hull) = hull else {
                return;
            };
            let mut near = false;
            walkers.for_entity(walker, |(_, _, _, _, transform)| {
                near = (transform.translation - hull).magnitude() <= BOARD_REACH;
            });
            if !near {
                return;
            }
            let Some(next) = resources.mode.process_event(ModeEvent::Board) else {
                return;
            };
            resources.mode = next;
            events.send(TidefallEvent::Boarded);
        }
    }
}
