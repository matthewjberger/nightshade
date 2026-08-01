use crate::commands::audio::cue_one_of;
use crate::components::{Approach, Cooldowns, Giver, Health, Pickup, Target, Worth};
use crate::data::TARGET_CLIPS;
use crate::queries::{flat_distance, is_alive};
use crate::resources::{Cues, Hero, Hover, Journal};
use crate::theme;
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
    hero: Res<Hero>,
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
                        if Some(current) != hero.player {
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

    let orbiting = input.mouse.state.contains(MouseState::RIGHT_CLICKED);
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
        hover.timer = theme::HOVER_REFRESH_SECONDS;
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
    ),
>;

pub fn click(
    input: Res<Input>,
    capture: Res<InputCapture>,
    hover: Res<Hover>,
    mut hero: ResMut<Hero>,
    mut journal: ResMut<Journal>,
    mut cues: ResMut<Cues>,
    mut players: Query<(&mut Target, &mut Approach, &LocalTransform), With<Cooldowns>>,
    mut marks: Marks<'_, '_>,
) {
    if !input.mouse.state.contains(MouseState::LEFT_JUST_PRESSED) || pointer_captured(&capture) {
        return;
    }
    let Some(entity) = hover.entity else {
        return;
    };
    let mut mark = None;
    marks.for_entity(entity, |(transform, health, worth, giver, pickup)| {
        mark = Some((
            transform.translation,
            health.is_none_or(|health| is_alive(health.0)),
            worth.is_some(),
            giver.map(|giver| giver.0),
            pickup.is_some(),
        ));
    });
    let Some((spot, alive, hostile, giver, pickup)) = mark else {
        return;
    };

    let mut chosen = false;
    let quests = &mut journal.quests;
    players.for_each_mut(|_entity, (target, approach, transform)| {
        if hostile {
            if alive && target.0 != Some(entity) {
                target.0 = Some(entity);
                chosen = true;
            }
            return;
        }
        let distance = flat_distance(transform.translation, spot);
        if let Some(giver) = giver {
            if distance < theme::GIVER_REACH {
                quests.near_giver[giver] = false;
                approach.0 = None;
            } else {
                approach.0 = Some(entity);
            }
            return;
        }
        if pickup {
            approach.0 = if distance < theme::PICKUP_REACH {
                None
            } else {
                Some(entity)
            };
            return;
        }
        approach.0 = Some(entity);
    });
    if chosen {
        cue_one_of(
            &mut cues,
            &mut hero.random,
            &TARGET_CLIPS,
            theme::SOUND_QUIET,
        );
    }
}
