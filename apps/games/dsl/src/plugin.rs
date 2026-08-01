use crate::data;
use crate::resources::{
    Clock, Land, Machine, Screen, Script, Store, Studio, Switcher, Tape, Transport, Vault,
};
use crate::systems;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;

pub struct DescriptionPlugin;

impl Plugin for DescriptionPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = data::TITLE.to_string();
        app.add_plugin(nightshade_audio::AudioPlugin);
        app.add_plugin(nightshade_particles::ParticlePlugin);
        app.add_plugin(nightshade_physics::PhysicsPlugin);
        app.add_plugin(nightshade_sprite::SpritePlugin);
        app.add_plugin(nightshade_decals::DecalsPlugin);
        app.add_plugin(nightshade_navmesh::NavmeshPlugin);
        app.add_plugin(nightshade_sdf::SdfPlugin);
        app.insert_resource(Land::default());
        app.add_plugin(nightshade_voxels::VoxelsPlugin::<systems::land::Digger>::default());
        app.insert_resource(Script::default());
        app.insert_resource(Store::default());
        app.insert_resource(Machine::default());
        app.insert_resource(Switcher::default());
        app.insert_resource(Studio::default());
        app.insert_resource(Clock::default());
        app.insert_resource(Tape::default());
        app.insert_resource(Transport::default());
        app.insert_resource(Screen::default());
        app.insert_resource(Vault::default());
        app.add_plugin(nightshade_input::InputPlugin::<crate::data::actions::Ask>::new());
        app.add_system(Stage::Startup, systems::setup::initialize);
        app.add_system(Stage::Startup, systems::spark::dress);
        app.add_system(Stage::Startup, systems::skin::dress);
        app.add_system(Stage::Startup, systems::land::dress);
        app.add_system(Stage::Startup, systems::sheet::dress);
        app.add_system(Stage::Startup, systems::panel::build);
        app.add_system(Stage::Startup, systems::transport::build);
        app.add_system(Stage::FrameStart, systems::run::frame);
        app.add_system(Stage::Update, systems::studio::listen);
        app.add_system(Stage::Update, systems::choose::listen);
        app.add_system(Stage::Update, systems::studio::build);
        app.add_system(Stage::Update, systems::studio::freshen);
        app.add_system(Stage::Update, systems::model::fetch);
        app.add_system(Stage::Update, systems::sound::fetch);
        app.add_system(Stage::Update, systems::run::sight);
        app.add_system(Stage::Update, systems::run::tick);
        app.add_system(Stage::Update, systems::run::aim);
        app.add_system(Stage::Update, systems::point::reach);
        app.add_system(Stage::Update, systems::path::bake);
        app.add_system(Stage::Update, systems::path::walk);
        app.add_system(Stage::Update, systems::field::carve);
        app.add_system(Stage::Update, systems::land::settle);
        app.add_system(Stage::Update, systems::model::raise);
        app.add_system(Stage::Update, systems::look::settle);
        app.add_system(
            Stage::FrameUpdate,
            nightshade_camera_controller::systems::controllers::pan_orbit_mouse_input,
        );
        app.add_system(
            Stage::FrameUpdate,
            nightshade_camera_controller::systems::controllers::pan_orbit_touch_input,
        );
        app.add_system(
            Stage::FrameUpdate,
            nightshade_camera_controller::systems::controllers::pan_orbit_update_transform,
        );
        app.add_system(Stage::Update, systems::screen::paint);
        app.add_system(Stage::Update, systems::transport::follow);
    }
}
