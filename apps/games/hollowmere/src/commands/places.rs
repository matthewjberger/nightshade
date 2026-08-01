use crate::queries::region_of;
use crate::resources::{Generation, Village};

pub fn town_village(generation: &mut Generation, town: (i32, i32)) -> Option<&mut Village> {
    let region = region_of(town.0, town.1);
    let mut found = None;
    for dx in -1..=1 {
        for dz in -1..=1 {
            let key = (region.0 + dx, region.1 + dz);
            if generation.villages.get(&key).is_some_and(|village| {
                village
                    .as_ref()
                    .is_some_and(|village| village.square == town)
            }) {
                found = Some(key);
            }
        }
    }
    let key = found?;
    generation.villages.get_mut(&key)?.as_mut()
}
