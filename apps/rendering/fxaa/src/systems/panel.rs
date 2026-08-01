use crate::resources::{Opening, Panel};
use crate::theme;
use nightshade::prelude::*;
use nightshade_graphics::prelude::{AntiAliasing, RenderSettings};
use nightshade_ui::prelude::*;

pub fn build(context: UiBuildContext<'_, '_>, smoothing_on: bool) -> Panel {
    let mut panel = Panel::default();
    let mut tree = UiTreeBuilder::new(context);

    let docked = tree.add_docked_panel_left("fxaa", theme::TITLE, theme::PANEL_WIDTH);
    let content = tree
        .component::<UiPanelData>(docked)
        .map(|data| data.content_entity)
        .expect("the panel has a content node");
    tree.edit::<UiLayoutNode>(content, |node| {
        node.flow_layout = Some(FlowLayout {
            direction: FlowDirection::Vertical,
            padding: theme::PANEL_PADDING,
            spacing: theme::PANEL_SPACING,
            alignment: FlowAlignment::Start,
            cross_alignment: FlowAlignment::Start,
            wrap: false,
        });
    });

    tree.in_parent(content, |tree| {
        tree.add_heading("Smoothing");
        panel.smoothing = tree.add_checkbox("FXAA", smoothing_on);
        tree.add_label("Turn it off and the edges of");
        tree.add_label("the star break into steps.");
        tree.add_separator();
        tree.add_heading("Pixels for each step");
        tree.add_label("How many pixels of the screen");
        tree.add_label("one pixel of the picture takes.");
        panel.pixels = tree.add_slider(
            theme::PIXELS_LEAST,
            theme::PIXELS_MOST,
            theme::PIXELS_PER_STEP,
        );
        tree.add_separator();
        tree.add_heading("Where to look");
        tree.add_label("The bars nearest to flat and");
        tree.add_label("nearest to upright step worst,");
        tree.add_label("because the run of such an edge");
        tree.add_label("lands in one row of pixels and");
        tree.add_label("then jumps to the next.");
    });

    let (accessibility, commands) = tree.accessibility_and_commands();
    ui_set_test_id(
        accessibility,
        commands,
        panel.smoothing,
        theme::SWITCH_TEST_ID,
    );

    tree.runtime_mut().visible = true;
    tree.finish();
    panel
}

pub fn drive(
    panel: Res<Panel>,
    mut settings: ResMut<RenderSettings>,
    mut checkboxes: Query<(&UiCheckboxData,)>,
    mut sliders: Query<(&UiSliderData,)>,
) {
    let mut on = None;
    checkboxes.for_entity(panel.smoothing, |(checkbox,)| {
        on = Some(checkbox.value);
    });
    if let Some(on) = on {
        let wanted = if on {
            AntiAliasing::Fxaa
        } else {
            AntiAliasing::Off
        };
        if settings.anti_aliasing != wanted {
            settings.anti_aliasing = wanted;
        }
    }

    let mut pixels = None;
    sliders.for_entity(panel.pixels, |(slider,)| {
        pixels = Some(slider.value);
    });
    if let Some(pixels) = pixels {
        let wanted = 1.0 / pixels.max(theme::PIXELS_LEAST);
        if (settings.render_scale - wanted).abs() > 0.001 {
            settings.render_scale = wanted;
        }
    }
}

pub fn rehearse(
    mut opening: ResMut<Opening>,
    mut stage: nightshade_ui_test::TestStage,
    storage: &mut nightshade::ecs::Storage,
) {
    let Some(mut driver) = opening.driver.take() else {
        return;
    };
    driver.step(storage, &mut stage);
    opening.driver = Some(driver);
}
