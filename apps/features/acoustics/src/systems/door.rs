use crate::commands::lay_out_the_building;
use crate::resources::AcousticsDemo;
use nightshade::prelude::*;
use nightshade_acoustics::contract::AcousticsGrid;
use nightshade_platform::prelude::*;
use nightshade_scene::contract::primitives::Visibility;

pub fn swing_the_door(
    input: Res<Input>,
    mut demo: ResMut<AcousticsDemo>,
    mut grid: ResMut<AcousticsGrid>,
    mut commands: Commands,
) {
    if !demo.built || !input.keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }
    demo.shut = !demo.shut;
    lay_out_the_building(&mut grid, demo.shut);
    commands.set(demo.door, Visibility { visible: demo.shut });
}
