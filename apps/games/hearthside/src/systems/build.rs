use crate::components::{Cell, Feeds, FillRate, Preview, Seat};
use crate::data;
use crate::data::Ask;
use crate::queries;
use crate::resources::{Aim, Grid, Lot, Models};
use nightshade::ecs::Storage;
use nightshade::prelude::*;
use nightshade_camera::resources::ActiveCamera;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_platform::queries::input_capture::pointer_captured;
use nightshade_prefab::{PrefabSpawn, spawn_prefab};
use nightshade_render::components::material::MaterialRef;
use nightshade_render::resources::viewport::Viewport;
use nightshade_scene::prelude::LocalTransform;
use nightshade_ui::prelude::ui_button_clicks;
use nightshade_ui::resources::RetainedUiFrame;

pub fn aim(
    models: Res<Models>,
    input_capture: Res<InputCapture>,
    input: Res<Input>,
    mut lot: ResMut<Lot>,
    window: Res<Window>,
    viewport: Res<Viewport>,
    active_camera: Res<ActiveCamera>,
    grid: Res<Grid>,
    materials: Res<nightshade_render::resources::material_registry::MaterialRegistry>,
    storage: &mut Storage,
) {
    if !models.ready {
        return;
    }

    let blocked_by_interface = pointer_captured(&input_capture);
    let mouse_position = input.mouse.position;
    let selected = lot.selected;
    let placement = data::CATALOG[selected].placement;

    let landing = if blocked_by_interface {
        None
    } else {
        nightshade_picking::screen_ray(
            &window,
            &viewport,
            &active_camera,
            storage.query_ref(),
            mouse_position,
        )
        .and_then(|ray| ray.intersect_ground_plane(0.0))
    };

    let target = landing
        .and_then(|point| match placement {
            data::Placement::Edge => queries::point_to_edge(point),
            _ => queries::point_to_cell(point).map(Aim::Cell),
        })
        .unwrap_or(Aim::Nowhere);

    let clear = {
        match (placement, target) {
            (data::Placement::Floor, Aim::Cell(index)) => !grid.floored[index],
            (data::Placement::Cell, Aim::Cell(index)) => {
                grid.floored[index] && queries::cell_is_free(&grid, index)
            }
            (data::Placement::Edge, edge) => queries::edge_is_free(&grid, edge),
            _ => false,
        }
    };

    {
        lot.aim = target;
        lot.clear = clear;
    }

    let pad = lot.pad;
    let spot = queries::aim_position(target);
    let yaw = queries::piece_yaw(target, placement, lot.yaw_steps);

    let mut previews: Vec<(Entity, usize)> = Vec::new();
    for (entity, (preview,)) in storage.query_ref::<(&Preview,)>().iter() {
        previews.push((entity, preview.0));
    }

    match spot {
        Some(at) => {
            if let Some(transform) = storage.get_mut::<LocalTransform>(pad) {
                transform.translation = Vec3::new(at.x, data::PAD_LIFT, at.z);
                transform.scale = match target {
                    Aim::North(_) => Vec3::new(data::GRID * 0.5, 1.0, data::PAD_THICKNESS),
                    Aim::West(_) => Vec3::new(data::PAD_THICKNESS, 1.0, data::GRID * 0.5),
                    _ => Vec3::new(data::GRID * 0.5, 1.0, data::GRID * 0.5),
                };
            }
            let material = if clear {
                data::GHOST_CLEAR_MATERIAL
            } else {
                data::GHOST_BLOCKED_MATERIAL
            };
            let material_id =
                nightshade_render::queries::material_registry::material_registry_resolve(
                    &materials, material,
                )
                .unwrap_or_default();
            storage.set(pad, MaterialRef::new(material_id));

            for (entity, piece) in previews {
                park(storage, entity, (piece == selected).then_some((at, yaw)));
            }
        }
        None => {
            park(storage, pad, None);
            for (entity, _) in previews {
                park(storage, entity, None);
            }
        }
    }
}

pub fn choose(mut lot: ResMut<Lot>, actions: Res<Actions<Ask>>, frame: Res<RetainedUiFrame>) {
    for entity in ui_button_clicks(&frame) {
        if let Some(tab) = lot.interface.tabs.iter().position(|slot| *slot == entity) {
            if let Some(category) = data::CATEGORIES.get(tab).copied() {
                crate::commands::select_category(&mut lot, category);
            }
            continue;
        }
        if let Some(slot) = lot
            .interface
            .palette_slots
            .iter()
            .position(|held| *held == entity)
        {
            crate::commands::select_slot(&mut lot, slot);
        }
    }

    for (slot, ask) in Ask::SLOTS.into_iter().enumerate() {
        if actions.just_pressed(ask) {
            crate::commands::select_slot(&mut lot, slot);
        }
    }

    if actions.just_pressed(Ask::SwitchCategory) {
        let next = match lot.category {
            data::Category::Build => data::Category::Buy,
            data::Category::Buy => data::Category::Build,
        };
        crate::commands::select_category(&mut lot, next);
    }

    if actions.just_pressed(Ask::Rotate) {
        crate::commands::rotate_piece(&mut lot);
    }
}

pub fn place(
    models: Res<Models>,
    mut lot: ResMut<Lot>,
    mut grid: ResMut<Grid>,
    actions: Res<Actions<Ask>>,
    mut parts: nightshade_prefab::PrefabParts,
    storage: &mut Storage,
) {
    if !models.ready || !actions.just_pressed(Ask::Place) {
        return;
    }
    if !lot.clear {
        return;
    }

    let target = lot.aim;
    let Some(at) = queries::aim_position(target) else {
        return;
    };
    let selected = lot.selected;
    let piece = &data::CATALOG[selected];
    let Some(prefab) = models.prefabs[selected].clone() else {
        return;
    };
    if !crate::commands::spend(&mut lot, piece.price) {
        return;
    }

    let drop = if piece.placement == data::Placement::Floor {
        data::FLOOR_DROP
    } else {
        0.0
    };
    let yaw = queries::piece_yaw(target, piece.placement, lot.yaw_steps);

    let entity = {
        let mut spawn = PrefabSpawn {
            storage,
            parts: parts.reborrow(),
        };
        spawn_prefab(&mut spawn, &prefab, Vec3::new(at.x, drop, at.z))
    };

    if let Some(transform) = storage.get_mut::<LocalTransform>(entity) {
        transform.rotation = nalgebra_glm::quat_angle_axis(yaw, &Vec3::y());
    }

    if piece.placement == data::Placement::Cell
        && let Aim::Cell(index) = target
    {
        storage.set(entity, Cell(index));
        storage.set(entity, Seat(piece.sits));
        if let Some(need) = piece.need {
            storage.set(entity, Feeds(need));
            storage.set(entity, FillRate(piece.fill_rate));
        }
    }

    crate::commands::mark_placed(&mut grid, target, selected);
}

fn park(storage: &mut Storage, entity: Entity, showing: Option<(Vec3, f32)>) {
    if let Some(transform) = storage.get_mut::<LocalTransform>(entity) {
        match showing {
            Some((at, yaw)) => {
                transform.translation = at;
                transform.rotation = nalgebra_glm::quat_angle_axis(yaw, &Vec3::y());
            }
            None => transform.translation = data::PARKED,
        }
    }
}
