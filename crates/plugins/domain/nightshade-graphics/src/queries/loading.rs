use crate::resources::loading::{LoadingPipeline, SourceImageId, TextureRecipe};

fn loading_pipeline_source_ready(pipeline: &LoadingPipeline, id: SourceImageId) -> bool {
    let slot = id.0 as usize;
    if pipeline.source_ready.get(slot).copied().unwrap_or(false) {
        return true;
    }
    !pipeline.source_pending.get(slot).copied().unwrap_or(false)
}

pub fn loading_pipeline_recipe_ready(pipeline: &LoadingPipeline, recipe: &TextureRecipe) -> bool {
    let mut ready = true;
    recipe.for_each_source(|id| {
        if !loading_pipeline_source_ready(pipeline, id) {
            ready = false;
        }
    });
    ready
}

pub fn loading_pipeline_pending_len(pipeline: &LoadingPipeline) -> usize {
    pipeline.pending.len()
}

pub fn loading_pipeline_completed(pipeline: &LoadingPipeline) -> usize {
    pipeline.completed
}

pub fn loading_pipeline_total(pipeline: &LoadingPipeline) -> usize {
    pipeline.total
}

pub fn loading_pipeline_fraction(pipeline: &LoadingPipeline) -> f32 {
    if pipeline.total == 0 {
        return 1.0;
    }
    pipeline.completed as f32 / pipeline.total as f32
}

pub fn loading_pipeline_is_active(pipeline: &LoadingPipeline) -> bool {
    !pipeline.pending.is_empty()
}

pub fn loading_pipeline_is_idle(pipeline: &LoadingPipeline) -> bool {
    pipeline.pending.is_empty() && !pipeline.source_pending.iter().any(|waiting| *waiting)
}

pub fn loading_pipeline_quiet_frames(pipeline: &LoadingPipeline) -> u32 {
    pipeline.quiet
}

pub fn loading_pipeline_has_settled(pipeline: &LoadingPipeline, frames: u32) -> bool {
    pipeline.quiet >= frames
}

pub fn loading_pipeline_last_label(pipeline: &LoadingPipeline) -> Option<&str> {
    pipeline.last_label.as_deref()
}

pub fn loading_pipeline_last_category(pipeline: &LoadingPipeline) -> Option<&'static str> {
    pipeline.last_category
}
