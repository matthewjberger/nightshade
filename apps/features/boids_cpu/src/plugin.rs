use crate::resources::BoidsResources;
use crate::systems::{
    debug_lines, hud, input, instances, lifecycle, population, report, stage_marks,
};
use nightshade::prelude::*;

pub struct BoidsPlugin;

impl Plugin for BoidsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BoidsResources::default());
        let starting = app.world.res::<crate::resources::BoidsArguments>().count;
        lifecycle::insert_flock_resources(
            app,
            starting,
            crate::data::regions_for(crate::data::threads()),
        );
        app.add_system(Stage::Startup, lifecycle::initialize);
        app.add_system(Stage::First, stage_marks::after_first);
        app.add_system(
            Stage::Update,
            (
                input::handle_events,
                population::apply_requested_count,
                instances::step_flock,
                instances::sync_instances,
                debug_lines::draw,
                report::report,
                stage_marks::after_update,
            ),
        );
        app.add_system(Stage::Update, hud::update);
        app.add_system(Stage::FrameStart, stage_marks::after_frame_start);
        app.add_system(Stage::FrameUpdate, stage_marks::after_frame_update);
        app.add_system(Stage::FramePostUpdate, stage_marks::after_frame_post_update);
        app.add_system(Stage::Render, stage_marks::after_render);
        app.add_system(Stage::Last, stage_marks::after_last);
    }
}
