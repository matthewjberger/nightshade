use crate::contract::Surface;
use crate::data::rules::{MAX_HEALTH, MESSAGE_SECONDS};
use crate::data::surfaces::TEXTURES;
use crate::data::theme::{HIDE_ROUGHNESS, HIDE_TINT};
use crate::queries::{descends_from, surface_material, surface_name};
use crate::resources::{PlayerState, RunState};
use nightshade::ecs::Storage;
use nightshade::prelude::{Commands, Entity};
use nightshade_graphics::commands::loading::load_texture_pack_from_image_bytes;
use nightshade_graphics::resources::loading::TextureLoad;
use nightshade_platform::prelude::{Window, set_cursor_locked, set_cursor_visible};
use nightshade_render::commands::material_registry::insert_material;
use nightshade_render::commands::material_registry::material_registry_mutate_id;
use nightshade_render::commands::mesh_state::mesh_state_mark_material_dirty;
use nightshade_render::components::material::MaterialRef;
use nightshade_render::data::asset_id::MaterialId;
use nightshade_render::data::material::Material;
use nightshade_render::data::texture_data::{
    SamplerFilter, SamplerSettings, SamplerWrap, TextureUsage,
};
use nightshade_render::resources::material_registry::MaterialRegistry;
use nightshade_render::resources::mesh_state::MeshRenderState;

pub fn load_textures(load: &mut TextureLoad<'_>) {
    let sampler = SamplerSettings {
        wrap_u: SamplerWrap::Repeat,
        wrap_v: SamplerWrap::Repeat,
        mag_filter: SamplerFilter::Linear,
        min_filter: SamplerFilter::Linear,
        mipmap_filter: SamplerFilter::Linear,
    };
    load_texture_pack_from_image_bytes(load, TEXTURES, TextureUsage::Color, sampler);
}

pub fn attach_material(
    commands: &mut Commands<'_>,
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

pub fn attach_surface(commands: &mut Commands<'_>, entity: Entity, kind: Surface, span: f32) {
    attach_material(
        commands,
        entity,
        surface_name(kind, span),
        surface_material(kind, span),
    );
}

pub fn darken_the_hide(storage: &Storage, registry: &mut MaterialRegistry, root: Entity) {
    let mut ids: Vec<MaterialId> = storage
        .query_ref::<(&MaterialRef,)>()
        .iter()
        .filter(|(entity, _)| descends_from(storage, *entity, root))
        .map(|(_, (material,))| material.id)
        .collect();
    ids.sort_by_key(|id| id.index);
    ids.dedup();
    for id in ids {
        material_registry_mutate_id(registry, id, |material| {
            for (channel, tint) in material.base_color.iter_mut().zip(HIDE_TINT) {
                *channel *= tint;
            }
            material.roughness = HIDE_ROUGHNESS;
            material.metallic = 0.0;
        });
    }
}

pub fn post_message(run: &mut RunState, text: impl Into<String>) {
    run.message = text.into();
    run.message_until = run.clock + MESSAGE_SECONDS;
}

pub fn wound(player: &mut PlayerState, run: &mut RunState, amount: f32) {
    player.health = (player.health - amount).clamp(0.0, MAX_HEALTH);
    run.hurt_until = run.clock + 0.35;
}

pub fn lock_cursor(window: &mut Window, locked: bool) {
    if window.cursor_locked == locked && window.show_cursor != locked {
        return;
    }
    set_cursor_locked(window, locked);
    set_cursor_visible(window, !locked);
    window.show_cursor = !locked;
}
