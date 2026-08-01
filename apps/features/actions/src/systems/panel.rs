use crate::data::{Ask, LISTENING_HINT, PANEL_PADDING, PANEL_SPACING, PANEL_WIDTH};
use crate::queries::{describe_all, held_names};
use crate::resources::ActionsResources;
use nightshade::prelude::*;
use nightshade_input::prelude::{Action, Actions, Bindings, bindings_of, worn};
use nightshade_ui::prelude::*;

pub fn build(
    context: UiBuildContext<'_, '_>,
    bindings: &Bindings<Ask>,
    resources: &mut ActionsResources,
) {
    let mut tree = UiTreeBuilder::new(context);
    let panel = tree.add_docked_panel_left("actions", "Actions", PANEL_WIDTH);
    let content = tree.panel_content(panel).expect("panel content");
    tree.edit::<UiLayoutNode>(content, |node| {
        node.flow_layout = Some(FlowLayout {
            direction: FlowDirection::Vertical,
            padding: PANEL_PADDING,
            spacing: PANEL_SPACING,
            alignment: FlowAlignment::Start,
            cross_alignment: FlowAlignment::Start,
            wrap: false,
        });
    });
    tree.in_parent(content, |tree| {
        resources.profile_label = tree.add_heading(&profile_line(bindings));
        resources.action_labels = Ask::all()
            .iter()
            .map(|ask| tree.add_label(&action_line(bindings, *ask)))
            .collect();
        resources.held_label = tree.add_label("held: nothing");
        resources.hint_label = tree.add_label("");
    });
    tree.finish();
    resources.shown_revision = Some(bindings.revision);
}

pub fn refresh_the_panel(
    bindings: Res<Bindings<Ask>>,
    actions: Res<Actions<Ask>>,
    mut resources: ResMut<ActionsResources>,
    mut commands: Commands,
) {
    if resources.shown_revision != Some(bindings.revision) {
        resources.shown_revision = Some(bindings.revision);
        ui_set_text(
            &mut commands,
            resources.profile_label,
            &profile_line(&bindings),
        );
        for (ask, label) in Ask::all().iter().zip(resources.action_labels.iter()) {
            ui_set_text(&mut commands, *label, &action_line(&bindings, *ask));
        }
    }
    let held = held_names(&actions);
    if resources.shown_held != held {
        ui_set_text(&mut commands, resources.held_label, &held);
        resources.shown_held = held;
    }
    let hint = if resources.listening {
        LISTENING_HINT
    } else {
        ""
    };
    if resources.shown_hint != hint {
        ui_set_text(&mut commands, resources.hint_label, hint);
        resources.shown_hint = hint.to_string();
    }
}

fn profile_line(bindings: &Bindings<Ask>) -> String {
    format!("profile: {}", worn(bindings))
}

fn action_line(bindings: &Bindings<Ask>, ask: Ask) -> String {
    format!(
        "{}: {}",
        ask.name(),
        describe_all(bindings_of(bindings, ask))
    )
}
