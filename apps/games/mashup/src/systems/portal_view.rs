use crate::contract::WorldSide;
use crate::data;
use crate::queries;
use crate::resources::{MashupResources, Phase};
use nightshade::prelude::*;
use nightshade_camera::prelude::RenderTarget;
use nightshade_portal::prelude::{Portal, PortalPlane};
use nightshade_portal::queries::{
    portal_model, portal_projection, portal_target, viewport_size, virtual_camera,
};
use nightshade_render::resources::viewport::Viewport;
use nightshade_scene::prelude::LocalTransform;

pub fn update(
    mut resources: ResMut<MashupResources>,
    viewport: Res<Viewport>,
    mut commands: Commands,
    portals: Query<(&Portal, &WorldSide)>,
    eyes: Query<(&WorldSide, &RenderTarget)>,
) {
    let mut placements: [Option<PortalPlane>; 2] = [None; 2];
    portals.for_each(|_entity, (portal, side)| {
        placements[side.index()] = Some(portal.0);
    });
    let (Some(meadow_gate), Some(quarry_gate)) = (placements[0], placements[1]) else {
        return;
    };

    let mut sides: Vec<(Entity, WorldSide, (u32, u32))> = Vec::new();
    eyes.for_each(|entity, (side, target)| sides.push((entity, *side, target.size())));

    let viewer = queries::pose_matrix(resources.view.eye, resources.view.look);
    let field_of_view = resources.view.field_of_view;
    let size = viewport_size(&viewport);
    let aspect = size.0 as f32 / size.1.max(1) as f32;
    let target = portal_target(size, data::PORTAL_TARGET_MAX);
    let fade = rim_fade(&resources);

    for (entity, side, held) in sides {
        let (source, destination) = match side {
            WorldSide::Meadow => (meadow_gate, quarry_gate),
            WorldSide::Quarry => (quarry_gate, meadow_gate),
        };
        if held != target {
            commands.set(entity, RenderTarget::renderer_owned(target.0, target.1));
        }
        let world = virtual_camera(&source, &destination, &viewer);
        commands.set(entity, LocalTransform::from_matrix(&world));
        commands.set(
            entity,
            portal_projection(
                &world,
                &destination,
                aspect,
                field_of_view,
                data::NEAR_PLANE,
                data::FAR_PLANE,
                data::PLANE_BIAS,
            ),
        );
        let shown = if side == resources.here { fade } else { 1.0 };
        crate::commands::remember_frame(&mut resources, side, portal_model(&source), entity, shown);
    }
}

fn rim_fade(resources: &MashupResources) -> f32 {
    match resources.crossing.phase {
        Phase::Diving => {
            let share = (resources.crossing.elapsed / data::DIVE_SECONDS).clamp(0.0, 1.0);
            1.0 - queries::eased(share)
        }
        _ => 1.0,
    }
}
