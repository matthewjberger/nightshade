use crate::data::Ask;
use crate::resources::{Chosen, Compiled, PaletteRows, Pictures, Preview, Tapping};
use crate::systems;
use crate::theme;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;
use nightshade_input::InputPlugin;
use nightshade_ui_nodegraph::prelude::{NodeGraphPlugin, NodeGraphSettings};
use nightshade_wgpu::RenderAppExt;

pub struct ShaderGraphPlugin;

impl Plugin for ShaderGraphPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Chosen::default());
        app.insert_resource(Compiled::default());
        app.insert_resource(PaletteRows::default());
        app.insert_resource(Preview::default());
        app.insert_resource(Pictures::default());
        app.insert_resource(Tapping::default());
        app.add_plugin(nightshade_ui::UiPlugin::new());
        app.add_plugin(InputPlugin::<Ask>::new());
        app.add_plugin(NodeGraphPlugin {
            settings: NodeGraphSettings {
                grid_tint: theme::GRID_TINT,
                body_tint: theme::BODY_TINT,
                edge_tint: theme::EDGE_TINT,
                wire_tint: theme::WIRE_TINT,
                ..Default::default()
            },
        });
        app.add_render_graph_config(systems::pass::install_graph_pass);
        app.on_pre_render(systems::pass::feed_the_pass);
        app.add_system(Stage::Startup, systems::setup::stand_the_board);
        app.add_system(Stage::Startup, systems::setup::stand_it_up);
        app.add_system(Stage::Startup, systems::setup::spread_the_board);
        app.add_system(Stage::Update, systems::palette::tap_the_field);
        app.add_system(Stage::Update, systems::pluck::pluck_a_file);
        app.add_system(Stage::Update, systems::pluck::write_the_source);
        app.add_system(Stage::Update, systems::palette::follow_the_presets);
        app.add_system(Stage::Update, systems::palette::follow_the_palette);
        app.add_system(Stage::Update, systems::palette::follow_the_choice);
        app.add_system(Stage::Update, systems::palette::drag_the_value);
        app.add_system(Stage::Update, systems::palette::step_the_picture);
        app.add_system(Stage::Update, systems::edit::cut_what_is_under);
        app.add_system(Stage::Update, systems::compile::compile_the_shader);
        app.add_system(Stage::Update, systems::compile::name_the_window);
        app.add_system(Stage::LateUpdate, systems::view::measure_the_preview);
    }
}
