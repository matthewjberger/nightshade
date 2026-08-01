use crate::resources::Rehearsal;
use crate::resources::{
    BookArguments, BookState, DemoInput, DemoRuntime, Library, PageStage, Reading, Search, Shell,
};
use crate::systems;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;

pub struct WebsitePlugin;

impl Plugin for WebsitePlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = "The Nightshade Book".to_string();
        app.world.res_mut::<Window>().initial_size = Some((1440, 900));

        app.insert_resource(Library::default());
        app.insert_resource(Shell::default());
        app.insert_resource(BookState::default());
        app.insert_resource(DemoRuntime::default());
        app.insert_resource(PageStage::default());
        app.insert_resource(crate::commands::picture::Pictures::default());
        app.insert_resource(DemoInput::default());
        app.insert_resource(crate::resources::SourceView::default());
        app.insert_resource(Reading::default());
        app.insert_resource(Search::default());
        let rehearse = app.world.res::<BookArguments>().rehearse.clone();
        app.insert_resource(Rehearsal {
            driver: rehearse.as_ref().map(|_| systems::rehearsal::script()),
            directory: rehearse.unwrap_or_default(),
            frame: 0,
        });
        app.insert_resource(crate::resources::Appearance::default());

        app.add_system(Stage::Startup, systems::setup::initialize);
        app.add_system(Stage::Startup, systems::shell::build_shell);

        app.add_system(Stage::Update, systems::rehearsal::step);
        app.add_system(Stage::Update, systems::search::collect);
        app.add_system(Stage::Update, systems::appearance::drive);
        app.add_system(Stage::Update, systems::navigation::poll);
        app.add_system(Stage::Update, systems::search::drive);
        app.add_system(Stage::Update, systems::page::advance);
        app.add_system(Stage::Update, systems::page::fit);
        app.add_system(Stage::Update, systems::figures::expand);
        app.add_system(Stage::Update, systems::demos::apply);
        app.add_system(Stage::Update, systems::input::gather);
        app.add_system(Stage::Update, systems::demos::advance);
        app.add_system(Stage::Update, systems::camera::orbit);
        app.add_system(Stage::Update, systems::reading::track);
        app.add_system(Stage::Update, systems::shell::fold_tree);
        app.add_system(Stage::Update, systems::source::poll);
        app.add_system(Stage::Update, systems::source::refresh);
        app.add_system(Stage::Update, systems::ink::apply);

        app.add_system(Stage::Render, systems::viewport::track_stage);

        order_startup(app);
        order_update(app);
        order_render(app);
    }
}

fn order_startup(app: &mut App) {
    app.startup.order(
        std::any::type_name_of_val(&systems::setup::initialize),
        std::any::type_name_of_val(&systems::shell::build_shell),
    );
}

fn order_update(app: &mut App) {
    let steps = [
        std::any::type_name_of_val(&systems::rehearsal::step),
        std::any::type_name_of_val(&systems::search::collect),
        std::any::type_name_of_val(&systems::appearance::drive),
        std::any::type_name_of_val(&systems::navigation::poll),
        std::any::type_name_of_val(&systems::search::drive),
        std::any::type_name_of_val(&systems::page::advance),
        std::any::type_name_of_val(&systems::page::fit),
        std::any::type_name_of_val(&systems::figures::expand),
        std::any::type_name_of_val(&systems::demos::apply),
        std::any::type_name_of_val(&systems::input::gather),
        std::any::type_name_of_val(&systems::demos::advance),
        std::any::type_name_of_val(&systems::camera::orbit),
        std::any::type_name_of_val(&systems::reading::track),
        std::any::type_name_of_val(&systems::shell::fold_tree),
    ];
    for pair in steps.windows(2) {
        app.stages
            .stage_mut(Stage::Update.name())
            .order(pair[0], pair[1]);
    }
}

fn order_render(app: &mut App) {
    app.stages.stage_mut(Stage::Render.name()).order(
        std::any::type_name_of_val(&systems::viewport::track_stage),
        std::any::type_name_of_val(&nightshade_camera::systems::views::publish_render_views),
    );
}
