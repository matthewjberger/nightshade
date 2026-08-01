use crate::commands::post_message;
use crate::components::SurfaceKind;
use crate::data::theme;
use crate::queries::tile_center;
use crate::resources::{AshfallResources, WorldPhase};
use crate::systems::{level, textures};
use crate::tags::Moon;
use nightshade::ecs::system_param::EventReader;
use nightshade::prelude::*;
use nightshade_graphics::commands::queues::queue_ecs_command;
use nightshade_graphics::data::queues::EcsCommand;
use nightshade_graphics::prelude::*;
use nightshade_phase::commands::turn_to;
use nightshade_phase::{Phase, PhaseTurned};
use nightshade_ui_flash::Flash;
use nightshade_ui_flash::commands::flash;

pub fn update(resources: Res<AshfallResources>, mut phase: ResMut<Phase>) {
    turn_to(&mut phase, face_of(resources.run.phase));
}

pub fn settle(
    turned: EventReader<PhaseTurned>,
    mut resources: ResMut<AshfallResources>,
    mut screen: ResMut<Flash>,
    mut commands: Commands,
    mut moons: Query<(&mut Light,), WithTag<Moon>>,
) {
    let Some(event) = turned.iter().last() else {
        return;
    };
    let resources = &mut *resources;
    let phase = phase_of(event.at);
    let moon_color = match phase {
        WorldPhase::Fog => theme::MOON_COLOR,
        WorldPhase::Otherworld => theme::OTHERWORLD_MOON_COLOR,
    };
    moons.for_each_mut(|_entity, (light,)| light.color = moon_color);

    for kind in SurfaceKind::ALL {
        queue_ecs_command(
            &mut commands,
            EcsCommand::ReloadMaterial {
                name: textures::surface_name(kind),
                material: Box::new(textures::surface_material(kind, phase)),
            },
        );
    }

    if phase != WorldPhase::Otherworld {
        return;
    }
    for (col, row, glyph) in resources.run.otherworld_spawns.clone() {
        level::spawn_husk(
            &mut commands,
            tile_center(col, row),
            level::husk_kind(glyph),
            true,
            false,
        );
    }
    flash(
        &mut screen,
        theme::SHIFT_FLASH,
        crate::data::rules::SHIFT_FADE,
    );
    post_message(
        &mut resources.run,
        "A siren, far off. The walls turn to rust and the fog runs red.",
    );
}

fn face_of(phase: WorldPhase) -> usize {
    match phase {
        WorldPhase::Fog => 0,
        WorldPhase::Otherworld => 1,
    }
}

fn phase_of(face: usize) -> WorldPhase {
    match face {
        1 => WorldPhase::Otherworld,
        _ => WorldPhase::Fog,
    }
}
