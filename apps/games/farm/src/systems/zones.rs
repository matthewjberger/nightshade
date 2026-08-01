use crate::commands::pocket::show_toast;
use crate::queries::terrain::zone_at;
use crate::resources::{Diorama, Toast};
use nalgebra_glm::Vec2;
use nightshade::prelude::*;

pub fn name_the_zone(mut diorama: ResMut<Diorama>, mut toast: ResMut<Toast>) {
    let zone = zone_at(Vec2::new(diorama.feet.x, diorama.feet.z));
    if diorama.zone == Some(zone) {
        return;
    }
    let first = diorama.zone.is_none();
    diorama.zone = Some(zone);
    if !first {
        show_toast(&mut toast, zone.name());
    }
}
