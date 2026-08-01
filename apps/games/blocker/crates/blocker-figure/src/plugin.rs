use crate::resources::{Hero, Nameplates, Wardrobe};
use crate::systems;
use nightshade::prelude::*;

pub struct FigurePlugin;

impl Plugin for FigurePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Hero::default());
        app.insert_resource(Wardrobe::default());
        app.insert_resource(Nameplates::default());
        app.add_system(
            Stage::Update,
            (
                systems::wardrobe::demand_the_looks,
                systems::wardrobe::collect_the_looks,
                systems::wardrobe::dress_the_figures,
            ),
        );
        app.add_system(
            Stage::Update,
            (
                systems::figure::follow_the_plates,
                systems::figure::swing_the_limbs,
                systems::figure::turn_the_heads,
            ),
        );
    }
}
