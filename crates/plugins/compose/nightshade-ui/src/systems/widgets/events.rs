pub fn ui_event_bubble_system(
    runtime: nightshade::ecs::system_param::Res<crate::resources::RetainedUiRuntime>,
    mut frame: nightshade::ecs::system_param::ResMut<crate::resources::RetainedUiFrame>,
    mut hierarchy: nightshade::ecs::system_param::ResMut<nightshade::ecs::HierarchyIndex>,
    storage: &mut nightshade::ecs::Storage,
) {
    if !runtime.visible {
        return;
    }

    frame.bubbled_events.clear();

    let events: Vec<crate::resources::UiEvent> = frame.events.to_vec();

    hierarchy.sync(storage);

    for event in &events {
        if !event.bubbles() {
            continue;
        }
        let target = event.target_entity();
        let mut current = target;

        loop {
            let parent_opt = storage
                .get::<nightshade::ecs::ChildOf>(current)
                .map(|child_of| child_of.0);

            let Some(parent) = parent_opt else {
                break;
            };

            frame.bubbled_events.push(crate::resources::BubbledUiEvent {
                event: event.clone(),
                target,
                ancestor: parent,
                stopped: false,
            });

            current = parent;
        }
    }
}

pub fn ui_emit_click_events_system(
    runtime: nightshade::ecs::system_param::Res<crate::resources::RetainedUiRuntime>,
    mut frame: nightshade::ecs::system_param::ResMut<crate::resources::RetainedUiFrame>,
    storage: &mut nightshade::ecs::Storage,
) {
    if !runtime.visible {
        return;
    }

    let already_emitted: std::collections::HashSet<nightshade::ecs::Entity> = frame
        .events
        .iter()
        .map(|event| event.target_entity())
        .collect();

    let candidates: Vec<nightshade::ecs::Entity> = storage
        .query_entities::<(crate::components::UiNodeInteraction,)>()
        .collect();

    for entity in candidates {
        if already_emitted.contains(&entity) {
            continue;
        }
        let clicked = storage
            .get::<crate::components::UiNodeInteraction>(entity)
            .is_some_and(|interaction| interaction.clicked);
        if clicked {
            frame
                .events
                .push(crate::resources::UiEvent::ButtonClicked(entity));
        }
    }
}
