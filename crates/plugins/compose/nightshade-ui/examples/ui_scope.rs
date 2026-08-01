use nalgebra_glm::vec2;
use nightshade::prelude::nalgebra_glm;
use nightshade_ui::prelude::*;
use nightshade_ui::{default_ui_systems, insert_ui_resources};

fn host(width: u32, height: u32) -> nightshade::ecs::World {
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
    let window = ecs.res_mut::<nightshade_platform::contract::window::Window>();
    window.cached_viewport_size = Some((width, height));
    window.cached_scale_factor = 1.0;
    ecs.res_mut::<RetainedUiRuntime>().visible = true;
    ecs
}

fn main() {
    let mut ecs = host(1280, 720);

    let mut confirm = None;
    let mut cancel = None;

    {
        ecs.with_commands(|ecs, commands| {
            let mut tree = UiTreeBuilder::new(nightshade_ui::commands::builder::ui_build_context(
                &mut ecs.resources,
                commands,
            ));
            let root = tree
                .add_node()
                .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
                .entity();

            let panel = tree.in_parent(root, |tree| {
                tree.add_node()
                    .window(
                        Ab(vec2(24.0, 24.0)),
                        Ab(vec2(360.0, 260.0)),
                        Anchor::TopLeft,
                    )
                    .card()
                    .flow(FlowDirection::Vertical, 16.0, 8.0)
                    .entity()
            });

            tree.build_ui(panel, |ui| {
                ui.heading("Delete save?");
                ui.separator();
                ui.label("This cannot be undone.");

                ui.spacing(8.0);

                ui.row(|ui| {
                    cancel = Some(ui.button("Cancel"));
                    confirm = Some(ui.button("Delete"));
                });

                ui.spacing(8.0);

                ui.collapsing_header("Details", true, |ui: &mut Ui| {
                    ui.label("Slot 3");
                    ui.label("4 hours played");
                });

                ui.tree()
                    .add_node()
                    .fill_width()
                    .size_px(0.0, 2.0)
                    .rect(0.0);
            });

            tree.finish();
        });
    }

    ecs.resources
        .res_mut::<nightshade::ecs::HierarchyIndex>()
        .sync(&mut ecs.storage);
    for mut system in default_ui_systems() {
        system(&mut ecs);
    }

    println!("cancel  {:?}", cancel);
    println!("confirm {:?}", confirm);
    println!(
        "{} rects laid out",
        ecs.res::<RetainedUiFrame>().rects.len()
    );
}
