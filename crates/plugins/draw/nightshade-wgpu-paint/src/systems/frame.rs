use crate::systems::paint::{install_paint_passes_into, paint_image_pass_mut, paint_pass_mut};
use nightshade::ecs::system_param::{Res, ResMut};
use nightshade_wgpu::backend::PassBuild;
use nightshade_wgpu::wgpu;

pub fn install_paint_passes(
    graph: &mut nightshade_wgpu::backend::RenderGraph<nightshade_wgpu::backend::RenderInputs>,
    build: PassBuild,
    format: wgpu::TextureFormat,
) {
    let color = build.targets.swapchain;
    let depth = build.targets.swapchain_depth;
    install_paint_passes_into(graph, build, format, color, depth);
}

pub fn sync_paint_passes(
    mut renderer: nightshade_wgpu::pre_render::Renderer,
    slot_allocator: Res<nightshade_paint::RenderSlotAllocator>,
    mut paint: ResMut<nightshade_paint::PaintFrame>,
    mut text_cache: ResMut<nightshade_paint::PaintTextCache>,
    text_state: Option<ResMut<nightshade_text::contract::resources::TextState>>,
) {
    let render_slots = slot_allocator.clone();

    let rects = std::mem::take(&mut paint.rects);
    let rect_entities = std::mem::take(&mut paint.rect_entities);
    let images = std::mem::take(&mut paint.images);
    let mut text_meshes = std::mem::take(&mut paint.text_meshes);
    let image_uploads = std::mem::take(&mut paint.image_uploads);

    if let Some(mut text_state) = text_state {
        mesh_frame_text(
            &mut renderer,
            &mut text_cache,
            &mut text_state,
            &mut text_meshes,
        );
    }

    let queue = renderer.queue.clone();
    let returned_images = paint_image_pass_mut(&mut renderer).map(|pass| {
        for upload in &image_uploads {
            pass.upload_image_layer(&queue, upload);
        }
        pass.swap_draw_list(nightshade_paint::PaintDrawList {
            images,
            render_slots: render_slots.clone(),
            ..Default::default()
        })
    });

    let returned_draws = paint_pass_mut(&mut renderer).map(|pass| {
        pass.swap_draw_list(nightshade_paint::PaintDrawList {
            rects,
            rect_entities,
            text_meshes,
            render_slots,
            ..Default::default()
        })
    });

    if let Some(returned) = returned_draws {
        paint.rects = returned.rects;
        paint.rect_entities = returned.rect_entities;
        paint.text_meshes = returned.text_meshes;
    }
    if let Some(returned) = returned_images {
        paint.images = returned.images;
    }
}

fn mesh_frame_text(
    renderer: &mut nightshade_wgpu::backend::WgpuRenderer,
    text_cache: &mut nightshade_paint::PaintTextCache,
    text_state: &mut nightshade_text::contract::resources::TextState,
    text_meshes: &mut [nightshade_paint::DrawText],
) {
    use nightshade_text::contract::data::text_data::FontKind;

    let pre_revision = nightshade_wgpu::backend::glyph_atlas_revision(renderer);
    let mut signatures = std::mem::take(&mut text_cache.signatures);
    let mut active: std::collections::HashSet<nightshade::ecs::Entity> =
        std::collections::HashSet::new();

    for instance in text_meshes.iter_mut() {
        let signature = nightshade_paint::text_signature(instance);
        let cached = instance
            .entity
            .and_then(|entity| signatures.get(&entity).copied());
        if cached == Some(signature) {
            if let Some(entity) = instance.entity {
                active.insert(entity);
            }
            continue;
        }
        let params = nightshade_wgpu::backend::TextMeshParams {
            font_size: instance.font_size,
            line_height: 1.2,
            font_kind: match instance.font_kind {
                FontKind::IconsMaterial | FontKind::IconsLucide => instance.font_kind,
                _ if instance.monospace => FontKind::Mono,
                kind => kind,
            },
            alignment: instance.alignment,
            vertical_alignment: instance.vertical_alignment,
            anchor_character: None,
        };
        instance.mesh = nightshade_wgpu::backend::generate_screen_text_mesh(
            renderer,
            &mut text_state.font_engine,
            &instance.text,
            &params,
            instance.wrap_width,
        );
        if let Some(entity) = instance.entity {
            signatures.insert(entity, signature);
            active.insert(entity);
        }
    }

    signatures.retain(|entity, _| active.contains(entity));
    text_cache.signatures = signatures;

    nightshade_wgpu::backend::publish_glyph_atlas_if_grown(renderer, pre_revision);
}
