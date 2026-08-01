use nalgebra_glm::{vec2, vec4};
use nightshade::prelude::nalgebra_glm;
use nightshade_ui::prelude::*;
use nightshade_ui::{default_ui_systems, insert_ui_resources};

fn main() {
    let mut ecs = nightshade::ecs::World::new();
    insert_ui_resources(&mut ecs.resources);
    ecs.resources
        .insert(nightshade_platform::contract::window::Window::default());
    ecs.resources
        .insert(nightshade_platform::contract::input::Input::default());
    ecs.resources
        .insert(nightshade_text::contract::resources::TextState::default());
    ecs.resources
        .insert(nightshade::ecs::HierarchyIndex::default());
    ecs.resources
        .insert(nightshade_platform::contract::time::Time::default());

    {
        let window = ecs.res_mut::<nightshade_platform::contract::window::Window>();
        window.cached_viewport_size = Some((1280, 720));
        window.cached_scale_factor = 1.0;
    }
    ecs.res_mut::<RetainedUiRuntime>().visible = true;

    ecs.with_commands(|ecs, commands| {
        let mut tree = UiTreeBuilder::new(nightshade_ui::commands::builder::ui_build_context(
            &mut ecs.resources,
            commands,
        ));
        let root = tree
            .add_node()
            .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
            .entity();
        tree.in_parent(root, |tree| {
            let panel = tree
                .add_node()
                .window(
                    Ab(vec2(20.0, 20.0)),
                    Ab(vec2(260.0, 120.0)),
                    Anchor::TopLeft,
                )
                .with_rect(8.0, 1.5, vec4(0.35, 0.35, 0.4, 0.9))
                .flow(FlowDirection::Vertical, 12.0, 8.0)
                .entity();
            tree.in_parent(panel, |tree| {
                tree.add_label("nightshade-ui, hosted standalone");
                tree.add_button("Click me");
            });
        });
        tree.finish();
    });

    ecs.resources
        .res_mut::<nightshade::ecs::HierarchyIndex>()
        .sync(&mut ecs.storage);
    for mut system in default_ui_systems() {
        system(&mut ecs);
    }

    let frame = &ecs.res::<RetainedUiFrame>();
    println!(
        "retained UI produced {} rects and {} text instances",
        frame.rects.len(),
        frame.text_meshes.len()
    );
}
