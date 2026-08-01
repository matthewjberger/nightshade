use crate::components::Cell;
use crate::resources::Pavement;
use nightshade::prelude::*;
use nightshade_graphics::commands::builtins::spawn_mesh_at;
use nightshade_render::components::material::MaterialRef;
use nightshade_render::data::asset_id::MaterialId;

pub fn spawn_pavement(
    commands: &mut Commands,
    pavement: &Pavement,
    material: MaterialId,
    cell: [i32; 2],
) -> Entity {
    let slab = spawn_mesh_at(commands, "Cube", pavement.position, pavement.size);
    commands.set(slab, MaterialRef::new(material));
    commands.set(slab, Cell(cell));
    slab
}
