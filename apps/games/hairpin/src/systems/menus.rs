use crate::commands::{reset_lap, set_screen};
use crate::components::{Steer, Velocity, Yaw};
use crate::data::{Ask, CAR_START, CAR_YAW, PAUSE_AREA, TOUCH_CONTROLS};
use crate::queries::{pad_pressed, pointer_starts, touch_starts, viewport_size};
use crate::resources::{HairpinResources, Screen};
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::LocalTransform;
use nightshade_ui::components::UiLayoutNode;
use nightshade_ui::prelude::ui_button_clicks;
use nightshade_ui::resources::{RetainedUiDirty, RetainedUiFrame};

pub fn update(
    input: Res<Input>,
    actions: Res<Actions<Ask>>,
    frame: Res<RetainedUiFrame>,
    mut resources: ResMut<HairpinResources>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut window: ResMut<Window>,
    mut commands: Commands,
    mut nodes: Query<(&UiLayoutNode,)>,
    cars: Query<(&mut Velocity, &mut Yaw, &mut Steer, &mut LocalTransform)>,
) {
    let ui = resources.ui;
    let viewport = viewport_size(&window);
    let mut wanted = None;
    let mut restart = false;

    let pause_pressed = TOUCH_CONTROLS
        && resources.screen == Screen::Playing
        && pointer_starts(&input).any(|point| pad_pressed(PAUSE_AREA, point, viewport));

    if actions.just_pressed(Ask::Pause) || pause_pressed {
        wanted = match resources.screen {
            Screen::Playing => Some(Screen::Paused),
            Screen::Paused => Some(Screen::Playing),
            Screen::Title => None,
        };
    }

    let mut pressed: Vec<Entity> = ui_button_clicks(&frame).collect();
    let buttons = [
        ui.start_button,
        ui.exit_button,
        ui.resume_button,
        ui.restart_button,
        ui.quit_button,
    ];
    for point in touch_starts(&input) {
        for button in buttons.into_iter().flatten() {
            if over_node(&mut nodes, button, point) {
                pressed.push(button);
            }
        }
    }

    for entity in pressed {
        let clicked = |button: Option<Entity>| button == Some(entity);
        if clicked(ui.start_button) || clicked(ui.restart_button) {
            restart = true;
            wanted = Some(Screen::Playing);
        } else if clicked(ui.resume_button) {
            wanted = Some(Screen::Playing);
        } else if clicked(ui.quit_button) {
            restart = true;
            wanted = Some(Screen::Title);
        } else if clicked(ui.exit_button) {
            window.should_exit = true;
        }
    }

    if restart {
        reset_lap(&mut resources);
        cars.for_each(|_entity, (velocity, yaw, steer, transform)| {
            *velocity = Velocity::default();
            *yaw = Yaw(CAR_YAW);
            *steer = Steer::default();
            transform.translation = CAR_START;
            transform.rotation = nalgebra_glm::quat_angle_axis(CAR_YAW, &Vec3::y());
        });
    }

    if let Some(screen) = wanted {
        set_screen(&mut resources, &mut dirty, &mut commands, screen);
    }
}

fn over_node(nodes: &mut Query<(&UiLayoutNode,)>, entity: Entity, point: Vec2) -> bool {
    let mut over = false;
    nodes.for_entity(entity, |(node,)| {
        over = node.visible && node.computed_rect.contains(point);
    });
    over
}
