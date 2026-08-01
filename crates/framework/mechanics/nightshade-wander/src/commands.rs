use crate::components::{Home, Resting, Wandering};
use crate::resources::WanderSettings;
use nightshade_plane::Plane;
use nightshade_plane::queries::raise;
use nightshade_random::Random;
use nightshade_random::commands::{between, disc};

pub fn pick_a_spot(
    plane: Plane,
    wandering: &mut Wandering,
    home: &Home,
    settings: &WanderSettings,
    random: &mut Random,
) {
    wandering.0 = home.0 + raise(plane, disc(random, settings.reach));
}

pub fn rest_a_while(resting: &mut Resting, settings: &WanderSettings, random: &mut Random) {
    resting.0 = between(random, settings.rest_low, settings.rest_high);
}
