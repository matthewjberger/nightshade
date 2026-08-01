use crate::commands::lights::{spawn_lamp, spawn_neon};
use crate::commands::models::place_model;
use crate::commands::paving::spawn_pavement;
use crate::components::Cell;
use crate::data::scale::{AHEAD_KEEP, AHEAD_LOAD, CELL, CELLS_IN_FLIGHT, TILE, VIEW_BEHIND};
use crate::data::settings::Settings;
use crate::queries::grid::{cell_of, in_keep, in_ring, tile_center, view_rank, view_sector};
use crate::queries::view::{anchor, forward};
use crate::resources::{CellJob, City, GridlockResources, Indoors, Plan, Player, Streaming};
use nightshade::ecs::Storage;
use nightshade::prelude::*;
use nightshade_camera::prelude::ActiveCamera;
use nightshade_config::contract::Config;
use nightshade_worker::prelude::*;
use std::collections::HashSet;

pub fn feed(
    player: Res<Player>,
    active_camera: Res<ActiveCamera>,
    resources: Res<GridlockResources>,
    config: Res<Config<Settings>>,
    city: Res<City>,
    indoors: Res<Indoors>,
    mut streaming: ResMut<Streaming>,
    mut crew: ResMut<Workers<CellJob>>,
    storage: &mut Storage,
) {
    if indoors.visit.is_some() {
        return;
    }
    let Some(anchor) = anchor(&player, storage) else {
        return;
    };
    let view = config.whole(Settings::ViewCells) as i32;
    let keep = (config.whole(Settings::KeepCells) as i32).max(view + 1);
    let ring = (config.whole(Settings::KeepBehind) as i32).min(view);
    streaming.life = (config.whole(Settings::LifeCells) as i32).min(ring);
    let plan = Plan {
        neon_chance: config.number(Settings::NeonChance),
        parking: config.number(Settings::Parking),
        trees: config.whole(Settings::Trees).max(0) as usize,
        tower_chance: config.number(Settings::TowerChance),
        props: config.number(Settings::Props),
        site_chance: config.number(Settings::SiteChance),
    };
    let facing = forward(&active_camera, storage);
    let deadzone = config.number(Settings::Deadzone);
    let center = match streaming.center {
        Some(current) if within_deadzone(anchor, current, deadzone) => current,
        _ => cell_of(anchor),
    };
    let sector = view_sector(facing);
    let moved = streaming.center != Some(center);
    let turned = streaming.facing != Some(sector);
    if moved || turned || streaming.view != view {
        streaming.center = Some(center);
        streaming.facing = Some(sector);
        streaming.view = view;
        streaming.pending = wanted_cells(center, facing, view, ring);
    }
    streaming.keep = keep;
    streaming.ring = ring;
    streaming.forward = facing;

    while workers_in_flight(&crew) < CELLS_IN_FLIGHT {
        let Some(key) = streaming.pending.pop() else {
            break;
        };
        if city.cells.contains_key(&key) || workers_holds(&crew, key) {
            continue;
        }
        let rank = view_rank(key[0] - center[0], key[1] - center[1], facing, VIEW_BEHIND);
        workers_submit(
            &mut crew,
            CellJob {
                key,
                seed: resources.seed,
                plan,
            },
            rank,
        );
    }

    if moved || turned {
        let gone: HashSet<[i32; 2]> = city
            .cells
            .keys()
            .copied()
            .filter(|key| !in_keep(*key, center, facing, keep, ring, AHEAD_KEEP))
            .collect();
        streaming.leaving.clear();
        streaming.leaving.extend(gone);
    }
}

pub fn settle(
    resources: Res<GridlockResources>,
    config: Res<Config<Settings>>,
    mut streaming: ResMut<Streaming>,
    mut city: ResMut<City>,
    mut crew: ResMut<Workers<CellJob>>,
    mut commands: Commands,
    storage: &mut Storage,
) {
    let Some(center) = streaming.center else {
        return;
    };
    let keep = streaming.keep;
    let ring = streaming.ring;
    let forward = streaming.forward;
    for layout in workers_take(&mut crew, usize::MAX) {
        if !in_keep(layout.key, center, forward, keep, ring, AHEAD_KEEP)
            || city.cells.contains_key(&layout.key)
        {
            continue;
        }
        city.cells.insert(layout.key, layout.clone());
        streaming.settling.push_back((layout, 0));
    }
    let strength = config.number(Settings::LampLight);
    let reach = config.number(Settings::LampReach);
    let light_cells = config.whole(Settings::LightCells) as i32;
    let budget = config.number(Settings::SettleMillis) / 1000.0;
    let started = Instant::now();
    loop {
        let Some((layout, cursor)) = streaming.settling.front_mut() else {
            return;
        };
        if !city.cells.contains_key(&layout.key) {
            streaming.settling.pop_front();
            continue;
        }
        while *cursor < layout.placements.len() {
            let placement = &layout.placements[*cursor];
            place_model(
                storage,
                placement.model,
                placement.position,
                placement.yaw,
                placement.scale,
                placement.prop,
                layout.key,
            );
            *cursor += 1;
            if started.elapsed().as_secs_f32() >= budget {
                return;
            }
        }
        let lit = in_ring(layout.key, center, light_cells);
        for lamp in &layout.lamps {
            spawn_lamp(&mut commands, lamp, strength, reach, lit, layout.key);
        }
        for neon in &layout.neons {
            let material = resources.neon_materials[neon.color % resources.neon_materials.len()];
            spawn_neon(&mut commands, neon, material, layout.key);
        }
        if let Some(material) = resources.pavement_material {
            for pavement in &layout.pavements {
                spawn_pavement(&mut commands, pavement, material, layout.key);
            }
        }
        streaming.settling.pop_front();
        if started.elapsed().as_secs_f32() >= budget {
            return;
        }
    }
}

pub fn unload(
    config: Res<Config<Settings>>,
    mut streaming: ResMut<Streaming>,
    mut city: ResMut<City>,
    mut crew: ResMut<Workers<CellJob>>,
    mut commands: Commands,
    cells: Query<(&Cell,)>,
) {
    let Some(center) = streaming.center else {
        return;
    };
    let keep = streaming.keep;
    let ring = streaming.ring;
    let forward = streaming.forward;
    let leaving: Vec<[i32; 2]> = streaming
        .leaving
        .drain(..)
        .filter(|key| !in_keep(*key, center, forward, keep, ring, AHEAD_KEEP))
        .collect();
    if !leaving.is_empty() {
        for key in &leaving {
            workers_cancel(&mut crew, *key);
            city.cells.remove(key);
        }
        let mut gone = Vec::new();
        cells.for_each(|entity, (cell,)| {
            if leaving.contains(&cell.0) {
                gone.push(entity);
            }
        });
        streaming.dying.extend(gone);
    }
    let budget = config.whole(Settings::UnloadRoots).max(1) as usize;
    let taken = budget.min(streaming.dying.len());
    for entity in streaming.dying.drain(..taken) {
        commands.queue(move |storage, _| {
            storage.despawn_recursive(entity);
        });
    }
}

fn within_deadzone(anchor: Vec3, center: [i32; 2], deadzone: f32) -> bool {
    let middle = tile_center(center, 0, 0)
        + Vec3::new(CELL * 0.5 - TILE * 0.5, 0.0, CELL * 0.5 - TILE * 0.5);
    let reach = CELL * (0.5 + deadzone);
    (anchor.x - middle.x).abs() <= reach && (anchor.z - middle.z).abs() <= reach
}

fn wanted_cells(center: [i32; 2], facing: Vec3, view: i32, ring: i32) -> Vec<[i32; 2]> {
    let mut cells = Vec::new();
    for dz in -view..=view {
        for dx in -view..=view {
            if dx * dx + dz * dz > view * view + view {
                continue;
            }
            let key = [center[0] + dx, center[1] + dz];
            if !in_keep(key, center, facing, view, ring, AHEAD_LOAD) {
                continue;
            }
            cells.push(key);
        }
    }
    cells.sort_by_cached_key(|key| {
        -view_rank(key[0] - center[0], key[1] - center[1], facing, VIEW_BEHIND)
    });
    cells
}
