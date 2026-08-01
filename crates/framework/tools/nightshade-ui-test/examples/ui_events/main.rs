use nalgebra_glm::vec2;
use nightshade::prelude::nalgebra_glm;
use nightshade_ui::prelude::*;
use nightshade_ui::resources::UiEvent;
use nightshade_ui::{default_ui_systems, insert_ui_resources};
use nightshade_ui_test::UiTestDriver;

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

#[derive(Debug, Default, PartialEq)]
struct Intent {
    start_pressed: bool,
    volume: Option<f32>,
}

fn read_intent(resources: &nightshade::ecs::ResourceMap, start: Entity, volume: Entity) -> Intent {
    let mut intent = Intent::default();
    for event in &resources.res::<RetainedUiFrame>().events {
        match event {
            UiEvent::ButtonClicked(entity) if *entity == start => intent.start_pressed = true,
            UiEvent::SliderChanged { entity, value } if *entity == volume => {
                intent.volume = Some(*value)
            }
            _ => {}
        }
    }
    intent
}

fn main() {
    let mut ecs = host(1280, 720);

    let mut start = Entity::default();
    let mut volume = Entity::default();

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
                        Ab(vec2(300.0, 200.0)),
                        Anchor::TopLeft,
                    )
                    .card()
                    .flow(FlowDirection::Vertical, 16.0, 10.0)
                    .entity()
            });
            tree.in_parent(panel, |tree| {
                start = tree.add_button("Start");
                volume = tree.add_slider(0.0, 1.0, 0.5);
            });
            tree.finish();
        });
    }
    ecs.with_commands(|ecs, commands| {
        ui_set_test_id(
            ecs.res_mut::<nightshade_ui::resources::RetainedUiAccessibility>(),
            commands,
            start,
            "start",
        );
    });

    ecs.resources
        .res_mut::<nightshade::ecs::HierarchyIndex>()
        .sync(&mut ecs.storage);
    for mut system in default_ui_systems() {
        system(&mut ecs);
    }

    println!(
        "quiet frame  -> {:?}",
        read_intent(&ecs.resources, start, volume)
    );

    let mut driver = UiTestDriver::new("events");
    driver.set_exit_on_complete(false);
    driver.start();
    driver.at_frame(1).click_entity("start");

    let mut clicked = Intent::default();
    for _ in 0..6 {
        driver.step(
            &mut ecs.storage,
            &mut nightshade_ui_test::test_stage(&mut ecs.resources),
        );
        ecs.resources
            .res_mut::<nightshade::ecs::HierarchyIndex>()
            .sync(&mut ecs.storage);
        for mut system in default_ui_systems() {
            system(&mut ecs);
        }
        let intent = read_intent(&ecs.resources, start, volume);
        if intent != Intent::default() {
            clicked = intent;
        }
    }

    println!("scripted click -> {:?}", clicked);
    println!("note: setting a value from code changes the widget but raises nothing");
    ecs.with_commands(|_ecs, commands| ui_slider_set_value(commands, volume, 0.3));
    for mut system in default_ui_systems() {
        system(&mut ecs);
    }
    println!(
        "  slider value {:?}, events {:?}",
        ui_slider_value(&ecs, volume),
        read_intent(&ecs.resources, start, volume)
    );
}
