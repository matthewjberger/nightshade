use crate::resources::{DecalType, DecalsResources, UiHandles};
use crate::theme::{
    BLOOM_INTENSITY_RANGE, DECAL_SIZE_RANGE, EMISSIVE_STRENGTH_RANGE, PANEL_PADDING, PANEL_SPACING,
    PANEL_WIDTH,
};
use nightshade::prelude::*;
use nightshade_decals::contract::Decal;
use nightshade_graphics::prelude::RenderSettings;
use nightshade_ui::prelude::*;
use nightshade_ui::resources::{RetainedUiDirty, RetainedUiFrame};

pub struct PanelSetup {
    pub decal_type: DecalType,
    pub decal_size: f32,
    pub emissive_strength: f32,
    pub show_controls: bool,
    pub show_preview: bool,
    pub use_awesomeface: bool,
    pub bloom_enabled: bool,
    pub bloom_intensity: f32,
}

pub fn build(context: UiBuildContext<'_, '_>, setup: PanelSetup) -> UiHandles {
    let mut tree = UiTreeBuilder::new(context);
    let panel = tree.add_docked_panel_left("decals", "Decals", PANEL_WIDTH);
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

    let mut type_label = Entity::default();
    let mut decal_size = Entity::default();
    let mut emissive_label = Entity::default();
    let mut emissive_strength = Entity::default();
    let mut show_preview = Entity::default();
    let mut use_awesomeface = Entity::default();
    let mut active_label = Entity::default();
    let mut clear_button = Entity::default();
    let mut bloom_enabled = Entity::default();
    let mut bloom_intensity = Entity::default();
    tree.in_parent(content, |tree| {
        tree.add_label("Current Decal Type");
        type_label = tree.add_label(setup.decal_type.name());

        tree.add_label("Decal Size");
        decal_size = tree.add_slider(DECAL_SIZE_RANGE.0, DECAL_SIZE_RANGE.1, setup.decal_size);

        emissive_label = tree.add_label("Emissive Strength");
        emissive_strength = tree.add_slider(
            EMISSIVE_STRENGTH_RANGE.0,
            EMISSIVE_STRENGTH_RANGE.1,
            setup.emissive_strength,
        );

        show_preview = tree.add_checkbox("Show Preview", setup.show_preview);
        use_awesomeface = tree.add_checkbox("Use Awesomeface Texture", setup.use_awesomeface);

        active_label = tree.add_label("Active Decals: 0");
        clear_button = tree.add_button("Clear All Decals");

        tree.add_label("");
        tree.add_label("Bloom");
        bloom_enabled = tree.add_checkbox("Enable Bloom", setup.bloom_enabled);
        tree.add_label("Intensity");
        bloom_intensity = tree.add_slider(
            BLOOM_INTENSITY_RANGE.0,
            BLOOM_INTENSITY_RANGE.1,
            setup.bloom_intensity,
        );
    });

    let emissive_visible = setup.decal_type.is_emissive();
    tree.edit::<UiLayoutNode>(panel, move |node| node.visible = setup.show_controls);
    tree.edit::<UiLayoutNode>(emissive_label, move |node| node.visible = emissive_visible);
    tree.edit::<UiLayoutNode>(emissive_strength, move |node| {
        node.visible = emissive_visible
    });
    tree.finish();

    UiHandles {
        panel,
        type_label,
        decal_size,
        emissive_label,
        emissive_strength,
        show_preview,
        use_awesomeface,
        active_label,
        clear_button,
        bloom_enabled,
        bloom_intensity,
    }
}

pub fn poll(
    mut decals: ResMut<DecalsResources>,
    frame: Res<RetainedUiFrame>,
    mut settings: ResMut<RenderSettings>,
    mut widgets: Query<(Option<&UiSliderData>, Option<&UiCheckboxData>)>,
) {
    let decals = &mut *decals;

    if let Some(value) = slider_value(&mut widgets, decals.handles.decal_size) {
        decals.decal_size = value;
    }
    if let Some(value) = slider_value(&mut widgets, decals.handles.emissive_strength) {
        decals.emissive_strength = value;
    }
    if let Some(value) = checkbox_value(&mut widgets, decals.handles.show_preview) {
        decals.show_preview = value;
    }
    if let Some(value) = checkbox_value(&mut widgets, decals.handles.use_awesomeface) {
        decals.use_awesomeface = value;
    }
    if let Some(value) = checkbox_value(&mut widgets, decals.handles.bloom_enabled) {
        settings.bloom_enabled = value;
    }
    if let Some(value) = slider_value(&mut widgets, decals.handles.bloom_intensity) {
        settings.bloom_intensity = value;
    }

    let clear_clicked = ui_events(&frame).iter().any(|event| {
        matches!(event, UiEvent::ButtonClicked(entity) if *entity == decals.handles.clear_button)
    });
    if clear_clicked {
        decals.clear_requested = true;
    }
}

pub fn refresh_text(
    mut decals: ResMut<DecalsResources>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
    existing: Query<(&Decal,)>,
) {
    let decals = &mut *decals;

    if decals.last_decal_type != Some(decals.current_decal_type) {
        ui_set_text(
            &mut commands,
            decals.handles.type_label,
            decals.current_decal_type.name(),
        );
        let emissive_visible = decals.current_decal_type.is_emissive();
        ui_set_visible(
            &mut dirty,
            &mut commands,
            decals.handles.emissive_label,
            emissive_visible,
        );
        ui_set_visible(
            &mut dirty,
            &mut commands,
            decals.handles.emissive_strength,
            emissive_visible,
        );
        decals.last_decal_type = Some(decals.current_decal_type);
    }

    let mut active = 0;
    existing.for_each(|_entity, _| active += 1);
    if decals.last_active_count != active {
        ui_set_text(
            &mut commands,
            decals.handles.active_label,
            &format!("Active Decals: {active}"),
        );
        decals.last_active_count = active;
    }
}

fn slider_value(
    widgets: &mut Query<'_, (Option<&UiSliderData>, Option<&UiCheckboxData>)>,
    entity: Entity,
) -> Option<f32> {
    let mut value = None;
    widgets.for_entity(entity, |(slider, _)| {
        value = slider.map(|slider| slider.value);
    });
    value
}

fn checkbox_value(
    widgets: &mut Query<'_, (Option<&UiSliderData>, Option<&UiCheckboxData>)>,
    entity: Entity,
) -> Option<bool> {
    let mut value = None;
    widgets.for_entity(entity, |(_, checkbox)| {
        value = checkbox.map(|checkbox| checkbox.value);
    });
    value
}
