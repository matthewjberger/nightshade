use nightshade::prelude::*;
use nightshade_camera::components::{Camera, Projection};
use nightshade_platform::contract::input::mouse::MouseState;
use nightshade_platform::prelude::*;
use nightshade_scene::contract::GlobalTransform;
use nightshade_ui::components::{
    UiButtonData, UiDropdownData, UiLayoutNode, UiSliderData, UiToggleData,
};
use nightshade_ui::prelude::*;

use crate::commands::figure::ControlKind;
use crate::resources::{DemoInput, DemoRuntime, PageStage, SourceView, StageAim};

pub fn gather(
    mut demo_input: ResMut<DemoInput>,
    mut input: ResMut<Input>,
    view: Res<SourceView>,
    interaction: Res<RetainedUiInteraction>,
    page_stage: Res<PageStage>,
    runtime: Res<DemoRuntime>,
    mut nodes: Query<(&UiLayoutNode,)>,
    mut buttons: Query<(&UiButtonData,)>,
    mut sliders: Query<(&UiSliderData,)>,
    mut toggles: Query<(&UiToggleData,)>,
    mut choices: Query<(&UiDropdownData,)>,
    mut cameras: Query<(&Camera, &GlobalTransform)>,
) {
    demo_input.clicked.clear();
    read_controls(
        &mut demo_input,
        &page_stage,
        &mut buttons,
        &mut sliders,
        &mut toggles,
        &mut choices,
    );

    demo_input
        .stages
        .retain(|demo, _| page_stage.figures.iter().any(|figure| figure.demo == *demo));

    let covered = view.demo.is_some();
    let mut took_the_wheel = false;
    let just_pressed = input.mouse.state.contains(MouseState::LEFT_JUST_PRESSED);
    let mut pressed_figure = None;

    for (index, figure) in page_stage.figures.iter().enumerate() {
        let held = demo_input.stages.entry(figure.demo).or_default();
        let mut aim = *held;
        aim.drag = vec2(0.0, 0.0);
        aim.wheel = 0.0;
        aim.pressed = false;

        let mut stage = None;
        nodes.for_entity(figure.node, |(node,)| {
            if node.visible {
                stage = Some(node.computed_rect);
            }
        });

        let Some(rect) = stage.filter(|_| !covered) else {
            aim.hovered = false;
            aim.dragging = false;
            aim.stage_size = vec2(0.0, 0.0);
            demo_input.stages.insert(figure.demo, aim);
            continue;
        };

        aim.stage_size = vec2(rect.width(), rect.height());

        let pointer = input.mouse.position;
        let hovered = rect.width() > 0.0
            && rect.height() > 0.0
            && pointer.x >= rect.min.x
            && pointer.x <= rect.max.x
            && pointer.y >= rect.min.y
            && pointer.y <= rect.max.y;
        aim.hovered = hovered;
        aim.pointer = vec2(
            (pointer.x - rect.min.x) / rect.width().max(1.0),
            (pointer.y - rect.min.y) / rect.height().max(1.0),
        );

        let over_stage_only = interaction
            .hovered_entity
            .is_none_or(|hovered_entity| hovered_entity == figure.node);

        if just_pressed && hovered && over_stage_only {
            aim.dragging = true;
            aim.pressed = true;
            pressed_figure = Some(figure.demo);
        }
        if !input.mouse.state.contains(MouseState::LEFT_CLICKED) {
            aim.dragging = false;
        }
        if aim.dragging {
            aim.drag = input.mouse.position_delta;
        }
        let in_hand = demo_input.held == Some(figure.demo) || page_stage.expanded == Some(index);
        if hovered && over_stage_only && in_hand {
            aim.wheel = input.mouse.wheel_delta.y;
            took_the_wheel |= aim.wheel != 0.0;
        }

        if let Some(camera) = runtime.cameras.get(figure.demo).copied() {
            read_camera(&mut aim, camera, &mut cameras);
        }

        demo_input.stages.insert(figure.demo, aim);
    }

    if just_pressed {
        demo_input.held = pressed_figure;
    }

    if took_the_wheel {
        input.mouse.wheel_delta = vec2(0.0, 0.0);
    }
}

pub fn aim_at(input: &mut DemoInput, demo: &'static str) {
    let aim = input.stages.get(demo).copied().unwrap_or_default();
    input.demo = Some(demo);
    input.hovered = aim.hovered;
    input.pointer = aim.pointer;
    input.drag = aim.drag;
    input.dragging = aim.dragging;
    input.wheel = aim.wheel;
    input.pressed = aim.pressed;
    input.stage_size = aim.stage_size;
    input.camera_origin = aim.camera_origin;
    input.camera_forward = aim.camera_forward;
    input.camera_right = aim.camera_right;
    input.camera_up = aim.camera_up;
    input.camera_half_height = aim.camera_half_height;
}

fn read_controls(
    demo_input: &mut DemoInput,
    page_stage: &PageStage,
    buttons: &mut Query<(&UiButtonData,)>,
    sliders: &mut Query<(&UiSliderData,)>,
    toggles: &mut Query<(&UiToggleData,)>,
    choices: &mut Query<(&UiDropdownData,)>,
) {
    for figure in &page_stage.figures {
        for (action, entity, kind) in &figure.controls {
            match kind {
                ControlKind::Button => {
                    let mut clicked = false;
                    buttons.for_entity(*entity, |(data,)| clicked = data.clicked);
                    if clicked {
                        demo_input.clicked.push(action);
                    }
                }
                ControlKind::Slider => {
                    let mut value = None;
                    sliders.for_entity(*entity, |(data,)| value = Some(data.value));
                    if let Some(value) = value {
                        demo_input.values.insert(action, value);
                    }
                }
                ControlKind::Toggle => {
                    let mut value = None;
                    toggles.for_entity(*entity, |(data,)| value = Some(data.value));
                    if let Some(value) = value {
                        demo_input
                            .values
                            .insert(action, if value { 1.0 } else { 0.0 });
                    }
                }
                ControlKind::Choice => {
                    let mut value = None;
                    choices.for_entity(*entity, |(data,)| value = Some(data.selected_index));
                    if let Some(value) = value {
                        demo_input.values.insert(action, value as f32);
                    }
                }
            }
        }
    }
}

fn read_camera(
    aim: &mut StageAim,
    entity: Entity,
    cameras: &mut Query<(&Camera, &GlobalTransform)>,
) {
    let mut found = None;
    cameras.for_entity(entity, |(camera, global)| {
        found = Some((*camera, *global));
    });
    let Some((camera, global)) = found else {
        return;
    };

    let matrix = global.0;
    aim.camera_origin = Vec3::new(matrix[(0, 3)], matrix[(1, 3)], matrix[(2, 3)]);
    aim.camera_right = Vec3::new(matrix[(0, 0)], matrix[(1, 0)], matrix[(2, 0)]);
    aim.camera_up = Vec3::new(matrix[(0, 1)], matrix[(1, 1)], matrix[(2, 1)]);
    aim.camera_forward = -Vec3::new(matrix[(0, 2)], matrix[(1, 2)], matrix[(2, 2)]);

    if let Projection::Perspective(perspective) = camera.projection {
        aim.camera_half_height = (perspective.y_fov_rad * 0.5).tan();
    }
}
