use crate::resources::{Player, VoxelViewer};
use nightshade::prelude::*;

pub fn aim_the_viewer(player: Res<Player>, mut viewer: ResMut<VoxelViewer>) {
    viewer.eye = player.eye;
    viewer.forward = player.forward;
    viewer.whole = player.watching;
}
