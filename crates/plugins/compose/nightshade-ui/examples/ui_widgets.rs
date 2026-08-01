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

struct Form {
    nickname: Entity,
    volume: Entity,
    fullscreen: Entity,
    difficulty: Entity,
    apply: Entity,
}

fn build(
    storage: &mut nightshade::ecs::Storage,
    resources: &mut nightshade::ecs::ResourceMap,
) -> Form {
    nightshade::ecs::with_commands_over(storage, resources, |_storage, resources, commands| {
        let mut tree = UiTreeBuilder::new(nightshade_ui::commands::builder::ui_build_context(
            resources, commands,
        ));

        let root = tree
            .add_node()
            .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
            .entity();

        let form = tree.in_parent(root, |tree| {
            let panel = tree
                .add_node()
                .window(
                    Ab(vec2(24.0, 24.0)),
                    Ab(vec2(320.0, 340.0)),
                    Anchor::TopLeft,
                )
                .card()
                .flow(FlowDirection::Vertical, 16.0, 10.0)
                .entity();

            tree.in_parent(panel, |tree| {
                tree.add_heading("Settings");
                tree.add_separator();

                tree.add_label("Nickname");
                let nickname = tree.add_text_input_with_value("nickname", "player one");

                tree.add_label("Volume");
                let volume = tree.add_slider(0.0, 1.0, 0.8);

                let fullscreen = tree.add_checkbox("Fullscreen", false);

                tree.add_label("Difficulty");
                let difficulty = tree.add_dropdown(&["Easy", "Normal", "Hard"], 1);

                tree.add_separator();
                let apply = tree.add_button("Apply");

                Form {
                    nickname,
                    volume,
                    fullscreen,
                    difficulty,
                    apply,
                }
            })
        });

        tree.finish();
        form
    })
}

fn frame(ecs: &mut nightshade::ecs::World) {
    ecs.resources
        .res_mut::<nightshade::ecs::HierarchyIndex>()
        .sync(&mut ecs.storage);
    for mut system in default_ui_systems() {
        system(ecs);
    }
}

fn main() {
    let mut ecs = host(1280, 720);
    let form = build(&mut ecs.storage, &mut ecs.resources);
    frame(&mut ecs);

    println!("nickname   {:?}", ui_text_input_text(&ecs, form.nickname));
    println!("volume     {:?}", ui_slider_value(&ecs, form.volume));
    println!("fullscreen {:?}", ui_checkbox_value(&ecs, form.fullscreen));
    println!(
        "difficulty {:?}",
        ui_dropdown_selected(&ecs, form.difficulty)
    );
    let apply_held = ecs
        .get::<nightshade_ui::components::UiButtonData>(form.apply)
        .is_some_and(|data| data.clicked);
    println!("apply held {apply_held}");

    ecs.with_commands(|_ecs, commands| {
        ui_slider_set_value(commands, form.volume, 0.25);
        ui_text_input_set_value(commands, form.nickname, "restored");
    });
    frame(&mut ecs);

    println!("after restore:");
    println!("  volume   {:?}", ui_slider_value(&ecs, form.volume));
    println!("  nickname {:?}", ui_text_input_text(&ecs, form.nickname));

    let frame_data = ecs.res::<RetainedUiFrame>();
    println!(
        "draw list: {} rects, {} text instances",
        frame_data.rects.len(),
        frame_data.text_meshes.len()
    );
}
