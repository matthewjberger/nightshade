use crate::commands::insert_beast_materials;
use nightshade::prelude::*;
use nightshade_render::resources::material_registry::MaterialRegistry;

pub fn paint_the_beasts(mut registry: ResMut<MaterialRegistry>) {
    insert_beast_materials(&mut registry);
}
