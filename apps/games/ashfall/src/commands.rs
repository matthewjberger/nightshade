use crate::components::{ItemKind, SurfaceKind};
use crate::data::level::{HEIGHT, WIDTH};
use crate::data::rules::{HURT_FADE, MAP_REVEAL_RADIUS, MAX_HEALTH, MESSAGE_SECONDS, STACK_LIMIT};
use crate::data::theme;
use crate::queries::tile_index;
use crate::resources::{PlayerState, RunState, Satchel, WorldPhase};
use crate::systems::textures::{surface_material, surface_name};
use nightshade::prelude::*;
use nightshade_platform::prelude::{Window, set_cursor_locked, set_cursor_visible};
use nightshade_render::commands::material_registry::insert_material;
use nightshade_render::commands::mesh_state::mesh_state_mark_material_dirty;
use nightshade_render::components::material::MaterialRef;
use nightshade_render::data::material::Material;
use nightshade_render::resources::material_registry::MaterialRegistry;
use nightshade_render::resources::mesh_state::MeshRenderState;
use nightshade_ui_flash::Flash;
use nightshade_ui_flash::commands::flash;

pub fn attach_material(
    commands: &mut Commands,
    entity: Entity,
    name: impl Into<String>,
    material: Material,
) {
    let name = name.into();
    commands.queue(move |storage, resources| {
        let name = insert_material(resources.res_mut::<MaterialRegistry>(), name, material);
        storage.set(entity, MaterialRef::new(name));
        mesh_state_mark_material_dirty(resources.res_mut::<MeshRenderState>(), entity);
    });
}

pub fn attach_surface(
    commands: &mut Commands,
    entity: Entity,
    kind: SurfaceKind,
    phase: WorldPhase,
) {
    commands.queue(move |storage, resources| {
        let name = insert_material(
            resources.res_mut::<MaterialRegistry>(),
            surface_name(kind),
            surface_material(kind, phase),
        );
        storage.set(entity, MaterialRef::new(name));
        mesh_state_mark_material_dirty(resources.res_mut::<MeshRenderState>(), entity);
    });
}

pub fn post_message(run: &mut RunState, text: impl Into<String>) {
    run.message = text.into();
    run.message_until = run.clock + MESSAGE_SECONDS;
}

pub fn lock_cursor(window: &mut Window, locked: bool) {
    if window.cursor_locked == locked && window.show_cursor != locked {
        return;
    }
    set_cursor_locked(window, locked);
    set_cursor_visible(window, !locked);
    window.show_cursor = !locked;
}

pub fn hurt_player(player: &mut PlayerState, screen: &mut Flash, amount: f32) {
    player.health = (player.health - amount).max(0.0);
    flash(screen, theme::HURT_FLASH, HURT_FADE);
}

pub fn stow(satchel: &mut Satchel, kind: ItemKind) -> bool {
    if stacks(kind) {
        for slot in satchel.slots.iter_mut() {
            if slot.kind == Some(kind) && slot.count < STACK_LIMIT {
                slot.count += 1;
                return true;
            }
        }
    }
    for slot in satchel.slots.iter_mut() {
        if slot.kind.is_none() {
            slot.kind = Some(kind);
            slot.count = 1;
            return true;
        }
    }
    false
}

pub fn take_out(satchel: &mut Satchel, index: usize) {
    let Some(slot) = satchel.slots.get_mut(index) else {
        return;
    };
    if slot.count > 1 {
        slot.count -= 1;
        return;
    }
    slot.kind = None;
    slot.count = 0;
}

fn stacks(kind: ItemKind) -> bool {
    matches!(
        kind,
        ItemKind::HealthDrink | ItemKind::Ammo | ItemKind::Battery
    )
}

pub fn heal_player(player: &mut PlayerState, amount: f32) {
    player.health = (player.health + amount).min(MAX_HEALTH);
}

pub fn unblock_tile(run: &mut RunState, tile: (i32, i32), upper: bool) {
    if let Some(index) = tile_index(tile.0, tile.1) {
        let grid = if upper {
            &mut run.blocked_upper
        } else {
            &mut run.blocked
        };
        grid[index] = false;
    }
}

pub fn block_tile(run: &mut RunState, tile: (i32, i32), upper: bool) {
    if let Some(index) = tile_index(tile.0, tile.1) {
        let grid = if upper {
            &mut run.blocked_upper
        } else {
            &mut run.blocked
        };
        grid[index] = true;
    }
}

pub fn reveal_around(run: &mut RunState, tile: (i32, i32), upper: bool) {
    let radius = MAP_REVEAL_RADIUS;
    for row in (tile.1 - radius)..=(tile.1 + radius) {
        for col in (tile.0 - radius)..=(tile.0 + radius) {
            if col < 0 || row < 0 || col >= WIDTH as i32 || row >= HEIGHT as i32 {
                continue;
            }
            let reach = (col - tile.0).pow(2) + (row - tile.1).pow(2);
            if reach > radius * radius {
                continue;
            }
            let Some(index) = tile_index(col, row) else {
                continue;
            };
            let seen = if upper {
                &mut run.explored_upper
            } else {
                &mut run.explored
            };
            if !seen[index] {
                seen[index] = true;
                run.map_dirty = true;
            }
        }
    }
}

pub fn scatter_seed(col: i32, row: i32) -> u32 {
    (col as u32)
        .wrapping_mul(73_856_093)
        .wrapping_add((row as u32).wrapping_mul(19_349_663))
        | 1
}
