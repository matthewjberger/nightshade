use nightshade::prelude::*;
use nightshade_platform::prelude::*;

use crate::resources::{ViewerArguments, ViewerResources};
use crate::systems;

pub struct ViewerPlugin;

impl Plugin for ViewerPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = "Nightshade Viewer".to_string();
        if app.world.res::<ViewerArguments>().preview {
            app.world.res_mut::<Window>().start_hidden = true;
        }
        app.insert_resource(ViewerResources::default());
        #[cfg(not(target_arch = "wasm32"))]
        compose_agent(app);
        app.add_system(Stage::Startup, systems::setup::initialize);
        app.add_system(Stage::Startup, systems::settings::load_settings);
        app.add_system(Stage::Startup, systems::interface::build_ui);
        app.add_system(
            Stage::Update,
            (
                systems::input::handle_events,
                systems::interface::relayout,
                systems::poll::poll_ui,
                systems::gizmo::update_gizmo,
                systems::picking::request_pick,
                systems::picking::apply_pick,
                systems::actions::apply_actions,
                systems::browsers::ensure_indices,
                systems::browsers::watch_library,
                systems::browsers::poll_browsers,
                systems::load::poll_load,
                systems::load::place_the_model,
            ),
        );
        app.add_system(
            Stage::Update,
            (
                systems::load::flush_report,
                systems::camera::update_camera,
                systems::animation::mirror_animation,
                systems::selection::refresh_selection,
                systems::selection::sync_outline,
                systems::picking::draw_hover,
                systems::markers::draw_markers,
                systems::textures::apply_texture_drop,
                systems::skeleton::update_skeleton,
                systems::previews::upload_previews,
            ),
        );
        app.add_system(Stage::Update, systems::library::draw_previews);
        app.add_system(Stage::Update, systems::library::hold_preview_capture);
        app.add_system(Stage::Update, systems::lists::rebuild_lists);
        app.add_system(Stage::Update, systems::refresh::refresh_ui);
        app.add_system(Stage::Update, systems::nav_gizmo::draw_nav_gizmo);
        app.add_system(Stage::Update, systems::settings::save_settings);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn compose_agent(app: &mut App) {
    use crate::commands::agent::start_agent;
    use crate::commands::notice::report;
    use crate::data::DEFAULT_PORT;
    use crate::resources::AgentChannel;

    let arguments = app.world.res::<ViewerArguments>();
    if arguments.no_agent {
        app.insert_resource(AgentChannel::default());
        return;
    }
    let port = arguments.agent_port.unwrap_or(DEFAULT_PORT);
    let mut crew = app.world.res::<ViewerResources>().incoming.crew.clone();
    let (channel, failure) = start_agent(&mut crew, port);
    app.insert_resource(channel);
    if let Some(message) = failure {
        report(&app.world.res::<ViewerResources>().incoming.notice, message);
    }
    app.add_system(Stage::Update, systems::agent::serve);
}
