use crate::data;
use crate::data::Ask;
use crate::resources::FourfoldResources;
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;

pub fn from_hands(
    actions: Res<Actions<Ask>>,
    input: Res<Input>,
    window: Res<Window>,
    mut resources: ResMut<FourfoldResources>,
) {
    let wish = actions.axis2(Ask::Walk);

    let look = if window.cursor_locked {
        input.mouse.raw_mouse_delta * data::MOUSE_SENSITIVITY
    } else {
        Vec2::zeros()
    };

    let dial = [Ask::DialOne, Ask::DialTwo, Ask::DialThree]
        .iter()
        .position(|ask| actions.just_pressed(*ask));

    resources.drive = crate::resources::Drive {
        wish,
        look,
        jump: actions.just_pressed(Ask::Jump),
        dial,
    };
}
