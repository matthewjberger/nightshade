use crate::commands::net::open_wire;
use crate::commands::server::keep_serving;
use crate::resources::{Arguments, Models, Role, Screen, Session, UiHandles};
use crate::systems;
use crate::theme;
use nightshade::prelude::*;
use nightshade_camera_controller::systems::controllers::{
    third_person_gamepad_input, third_person_mouse_input, third_person_update_transform,
};
use nightshade_platform::prelude::Window;

pub struct SandboxPlugin;

impl Plugin for SandboxPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = theme::TITLE.to_string();

        let arguments = app.world.res::<Arguments>();
        let peer = fastrand::u32(1..u32::MAX);
        let name = arguments
            .name
            .clone()
            .unwrap_or_else(|| format!("player {peer}"));
        let (role, bind, connect) = match (&arguments.join, arguments.host) {
            (Some(address), _) => {
                let connect = if address.contains(':') {
                    address.clone()
                } else {
                    format!("{address}:{}", theme::DEFAULT_PORT)
                };
                (Role::Guest, String::new(), connect)
            }
            (None, port) => {
                let port = port.unwrap_or(theme::DEFAULT_PORT);
                (
                    Role::Host,
                    format!("0.0.0.0:{port}"),
                    format!("{}:{port}", theme::LOOPBACK),
                )
            }
        };

        let seats = if role == Role::Host {
            vec![crate::data::wire::Seat {
                peer,
                name: name.clone(),
                character: 0,
            }]
        } else {
            Vec::new()
        };
        let screen = if arguments.play {
            Screen::Playing
        } else {
            Screen::Title
        };
        let session = Session {
            name,
            seats,
            screen,
            ..Default::default()
        };

        app.insert_resource(open_wire(role, peer, bind, connect));
        app.insert_resource(session);
        app.insert_resource(Models::default());
        app.insert_resource(UiHandles::default());
        app.add_plugin(nightshade_input::InputPlugin::<crate::data::Ask>::new());

        app.add_system(Stage::Startup, systems::setup::initialize);
        app.add_system(Stage::Startup, systems::hud::build);

        app.add_system(
            Stage::Update,
            (
                systems::drive::advance_clock,
                systems::net::pump,
                systems::screens::watch_the_keys,
                systems::screens::watch_for_the_exit,
                systems::screens::buttons,
                systems::hud::switches,
                systems::hud::chat,
                systems::drive::latch_input,
                systems::net::report,
                systems::drive::remember,
                systems::drive::tick,
                systems::net::serve,
                systems::net::reconcile,
                systems::net::file,
                systems::net::follow,
                systems::drive::render,
            ),
        );
        app.add_system(
            Stage::Update,
            (
                third_person_mouse_input,
                third_person_gamepad_input,
                third_person_update_transform,
            ),
        );
        app.add_system(
            Stage::Update,
            (
                systems::net::aim,
                systems::net::expire,
                systems::models::collect,
                systems::models::dress,
                systems::animation::pose,
                systems::animation::amble,
                systems::screens::show,
                systems::hud::refresh,
                systems::animals::walk,
            ),
        );
    }
}

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        let arguments = app.world.res::<Arguments>();
        let port = arguments.host.unwrap_or(theme::DEFAULT_PORT);
        let name = arguments
            .name
            .clone()
            .unwrap_or_else(|| "server".to_string());
        let peer = fastrand::u32(1..u32::MAX);
        let bind = format!("0.0.0.0:{port}");
        let connect = format!("{}:{port}", theme::LOOPBACK);

        let session = Session {
            name,
            screen: Screen::Playing,
            ..Default::default()
        };

        app.insert_resource(open_wire(Role::Host, peer, bind, connect));
        app.insert_resource(session);

        app.add_system(Stage::Startup, systems::setup::prepare);
        app.add_system(
            Stage::Update,
            (
                systems::drive::advance_clock,
                systems::net::pump,
                systems::drive::remember,
                systems::drive::tick,
                systems::net::serve,
                systems::net::expire,
            ),
        );
        app.take_over_runner("the sandbox server", keep_serving);
    }
}
