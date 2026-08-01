use crate::resources::Meadow;
use crate::systems;
use crate::theme;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;
use nightshade_ui::prelude::UiTreeBuilder;

pub struct ComposedPlugin;

impl Plugin for ComposedPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Meadow::default());

        app.add_plugin(nightshade_readout::ReadoutPlugin::titled(theme::TITLE));
        app.add_plugin(nightshade_daynight::DayNightPlugin {
            settings: nightshade_daynight::DaySettings {
                hours_a_second: theme::HOURS_A_SECOND,
                ..Default::default()
            },
            hour: theme::START_HOUR,
        });
        app.add_plugin(nightshade_weather::WeatherPlugin::default());
        app.add_plugin(nightshade_wander::WanderPlugin {
            settings: nightshade_wander::WanderSettings {
                reach: theme::WANDER_REACH,
                speed: theme::WANDER_SPEED,
                ..Default::default()
            },
        });
        app.add_plugin(nightshade_grounded::GroundedPlugin);
        app.add_plugin(nightshade_health::HealthPlugin);
        app.add_plugin(nightshade_cooldown::CooldownPlugin);
        app.add_plugin(nightshade_shake::ShakePlugin::default());
        app.add_plugin(nightshade_follow::FollowPlugin {
            settings: nightshade_follow::FollowSettings {
                rate: theme::CAMERA_RATE,
                snap: None,
                aim: true,
            },
        });
        app.add_plugin(nightshade_toast::ToastPlugin::default());
        app.add_plugin(nightshade_ui_toast::UiToastPlugin::default());

        app.add_system(Stage::Startup, systems::setup::stand_the_meadow);
        app.add_system(Stage::Startup, build_the_interface);
        app.add_system(Stage::Update, systems::play::strike_the_nearest);
        app.add_system(Stage::Update, systems::play::say_the_weather);
        app.add_system(Stage::Update, systems::play::show_the_hour);
        app.add_system(Stage::FrameUpdate, systems::play::carry_the_shake);
    }
}

fn build_the_interface(
    style: nightshade::ecs::system_param::Res<nightshade_ui_toast::ToastStyle>,
    mut card: nightshade::ecs::system_param::ResMut<nightshade_ui_toast::ToastCard>,
    mut build: nightshade_ui::commands::builder::UiBuild,
) {
    let mut tree = UiTreeBuilder::new(build.context());
    nightshade_ui_toast::commands::build(&mut tree, &style, &mut card);
    tree.finish();
}
