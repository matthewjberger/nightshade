use crate::resources::Jarvis;
use nightshade::prelude::*;
use nightshade_openxr::contract::{XrFrame, XrInput, XrPose, XrUiAnchor, XrUiPanel};
use nightshade_ui::prelude::*;
use nightshade_ui::resources::RetainedUiFrame;

pub fn drive_controls(
    mut jarvis: ResMut<Jarvis>,
    frame: Res<RetainedUiFrame>,
    xr_frame: Res<XrFrame>,
    xr_input: Res<XrInput>,
    mut panel: ResMut<XrUiPanel>,
    mut commands: Commands,
) {
    let handles = jarvis.handles;
    if handles.root == Entity::default() {
        return;
    }

    if xr_input.a_pressed() {
        panel.enabled = !panel.enabled;
    }

    let mut pin_clicked = false;
    let mut flight_clicked = false;
    for entity in ui_button_clicks(&frame) {
        if entity == handles.pin_button {
            pin_clicked = true;
        } else if entity == handles.flight_button {
            flight_clicked = true;
        }
    }

    if pin_clicked {
        let label = match panel.anchor {
            XrUiAnchor::Head if xr_frame.tracking_valid => {
                let orientation = nalgebra_glm::quat_normalize(&xr_frame.head_orientation);
                let forward = nalgebra_glm::quat_rotate_vec3(&orientation, &-Vec3::z());
                panel.anchor = XrUiAnchor::Stage(XrPose {
                    position: xr_frame.head_position + forward * panel.distance,
                    orientation,
                });
                Some("FOLLOW HEAD")
            }
            XrUiAnchor::Head => None,
            XrUiAnchor::Stage(_) => {
                panel.anchor = XrUiAnchor::Head;
                Some("PIN VISOR")
            }
        };
        if let Some(label) = label {
            ui_set_text(&mut commands, handles.pin_button, label);
        }
    }

    if flight_clicked {
        jarvis.flight_mode = !jarvis.flight_mode;
        let label = if jarvis.flight_mode {
            "HOVER MODE"
        } else {
            "FLIGHT MODE"
        };
        ui_set_text(&mut commands, handles.flight_button, label);
    }
}
