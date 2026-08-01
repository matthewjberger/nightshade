use crate::queries::loading::loading_pipeline_is_idle;
use crate::resources::loading::LoadingState;
use nightshade::ecs::system_param::ResMut;

pub fn count_quiet_loading_frames(mut loading: ResMut<LoadingState>) {
    if loading_pipeline_is_idle(&loading.pipeline) {
        loading.pipeline.quiet = loading.pipeline.quiet.saturating_add(1);
    } else {
        loading.pipeline.quiet = 0;
    }
}
