use crate::commands::audio::{cue, cue_one_of};
use crate::commands::chat::system_line;
use crate::components::{Approach, Cooldowns, Giver, Health, Merchant, Pickup, Target, Worth};
use crate::data::items::{MERCHANT_REACH, MERCHANTS};
use crate::data::{
    GIVER_REACH, HOVER_REFRESH_SECONDS, Objective, PICKUP_REACH, PICKUPS, QUESTS, SOUND_ACCEPT,
    SOUND_QUIET, TARGET_CLIPS,
};
use crate::queries::{flat_distance, is_alive};
use crate::resources::{Cues, Hover, Panels, ThornmereResources};
use nightshade::prelude::*;
use nightshade_graphics::commands::gpu_picking::request_pick_of;
use nightshade_platform::contract::input::MouseState;
use nightshade_platform::prelude::*;
use nightshade_render::components::mesh::RenderMesh;
use nightshade_render::data::config::RenderSettings;
use nightshade_render::resources::gpu_picking::GpuPicking;
use nightshade_render::resources::outline::Outline;
use nightshade_render::resources::viewport::Viewport;
use nightshade_scene::prelude::LocalTransform;

const ROOT_DEPTH: usize = 12;

pub fn hover(
    time: Res<Time>,
    input: Res<Input>,
    capture: Res<InputCapture>,
    window: Res<Window>,
    viewport: Res<Viewport>,
    settings: Res<RenderSettings>,
    hierarchy: Res<nightshade::ecs::HierarchyIndex>,
    resources: Res<ThornmereResources>,
    mut gpu_picking: ResMut<GpuPicking>,
    mut hover: ResMut<Hover>,
    mut outline: ResMut<Outline>,
    meshes: Query<(&RenderMesh,)>,
    mut bodies: Query<(&Health,)>,
    mut pickups: Query<(&Pickup,)>,
) {
    let hover = &mut *hover;
    if let Some(result) = gpu_picking.take_result() {
        hover.entity = None;
        if result.depth > 0.0
            && let Some(picked) = result.entity_id
        {
            let mut mesh = None;
            meshes.for_each(|entity, _| {
                if entity.id == picked {
                    mesh = Some(entity);
                }
            });
            if let Some(mesh) = mesh {
                let mut current = mesh;
                for _ in 0..ROOT_DEPTH {
                    let mut found = false;
                    bodies.for_entity(current, |_| found = true);
                    pickups.for_entity(current, |_| found = true);
                    if found {
                        if Some(current) != resources.player {
                            hover.entity = Some(current);
                        }
                        break;
                    }
                    let Some(parent) = hierarchy.parent_of.get(&current) else {
                        break;
                    };
                    current = *parent;
                }
            }
        }
    }

    let orbiting = window.cursor_locked || input.mouse.state.contains(MouseState::RIGHT_CLICKED);
    let cursor = input.mouse.position;
    let pixel = (cursor.x as u32, cursor.y as u32);
    hover.timer -= time.delta_time;
    if pointer_captured(&capture) || orbiting {
        hover.entity = None;
    } else if pixel != hover.cursor || hover.timer <= 0.0 {
        request_pick_of(
            viewport.active_viewport_rect,
            window.cached_viewport_size,
            settings.render_scale,
            &mut gpu_picking,
            cursor,
        );
        hover.cursor = pixel;
        hover.timer = HOVER_REFRESH_SECONDS;
    }
    outline.entities = hover.entity.into_iter().collect();
}

type Marks<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (
        &'item LocalTransform,
        Option<&'item Health>,
        Option<&'item Worth>,
        Option<&'item Giver>,
        Option<&'item Pickup>,
        Option<&'item Merchant>,
    ),
>;

pub fn click(
    input: Res<Input>,
    capture: Res<InputCapture>,
    window: Res<Window>,
    hover: Res<Hover>,
    mut resources: ResMut<ThornmereResources>,
    mut panels: ResMut<Panels>,
    mut cues: ResMut<Cues>,
    mut players: Query<(&mut Target, &mut Approach, &LocalTransform), With<Cooldowns>>,
    mut marks: Marks<'_, '_>,
) {
    if !input.mouse.state.contains(MouseState::LEFT_JUST_PRESSED)
        || pointer_captured(&capture)
        || window.cursor_locked
    {
        return;
    }
    let resources = &mut *resources;
    let mut mark = None;
    if let Some(entity) = hover.entity {
        marks.for_entity(
            entity,
            |(transform, health, worth, giver, pickup, merchant)| {
                mark = Some((
                    entity,
                    transform.translation,
                    health.is_none_or(|health| is_alive(health.0)),
                    worth.is_some(),
                    giver.map(|giver| giver.0),
                    pickup.map(|pickup| pickup.0),
                    merchant.map(|merchant| merchant.0),
                ));
            },
        );
    }

    let mut chosen = false;
    let mut opened = None;
    let mut line = None;
    let quests = &mut resources.quests;
    players.for_each_mut(|_entity, (target, approach, transform)| {
        let Some((entity, spot, alive, hostile, giver, pickup, merchant)) = mark else {
            target.0 = None;
            approach.0 = None;
            return;
        };
        if hostile {
            if alive && target.0 != Some(entity) {
                target.0 = Some(entity);
                chosen = true;
            }
            return;
        }
        let distance = flat_distance(transform.translation, spot);
        if let Some(merchant) = merchant {
            if distance < MERCHANT_REACH {
                opened = Some(merchant);
                approach.0 = None;
            } else {
                approach.0 = Some(entity);
            }
            return;
        }
        if let Some(giver) = giver {
            if distance < GIVER_REACH {
                quests.near_giver[giver] = false;
                approach.0 = None;
            } else {
                approach.0 = Some(entity);
            }
            return;
        }
        if let Some(pickup) = pickup {
            if distance < PICKUP_REACH {
                let wanted = quests.active.iter().any(|state| {
                    matches!(
                        QUESTS[state.quest].objective,
                        Objective::Gather { pickup: kind, .. } if kind == pickup
                    ) && !state.complete
                });
                if !wanted {
                    line = Some(format!(
                        "You have no use for {} right now.",
                        PICKUPS[pickup].name
                    ));
                }
                approach.0 = None;
            } else {
                approach.0 = Some(entity);
            }
            return;
        }
        approach.0 = Some(entity);
    });
    if chosen {
        cue_one_of(&mut cues, &mut resources.random, &TARGET_CLIPS, SOUND_QUIET);
    }
    if let Some(merchant) = opened
        && panels.shop != Some(merchant)
    {
        panels.shop = Some(merchant);
        panels.note = String::new();
        system_line(
            &mut resources.chat,
            format!(
                "{} lays out the {}.",
                MERCHANTS[merchant].name,
                MERCHANTS[merchant].title.to_lowercase()
            ),
        );
        cue(&mut cues, SOUND_ACCEPT, SOUND_QUIET);
    }
    if let Some(line) = line {
        system_line(&mut resources.chat, line);
    }
}
