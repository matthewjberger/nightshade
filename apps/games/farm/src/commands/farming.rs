use crate::commands::effects::sparkle;
use crate::commands::pocket::{add_item, show_toast, take_item};
use crate::components::{PlotIndex, Progress, Upright};
use crate::data::CropKind;
use crate::queries::crops::stage_of;
use crate::queries::sheet::standing;
use crate::queries::terrain::{center, height};
use crate::resources::{Farmland, Inventory, Plot, Sheet, Terrain, Toast};
use crate::theme;
use nalgebra_glm::{Vec2, Vec3};
use nightshade::prelude::*;
use nightshade_graphics::commands::builtins::{set_mesh_material, spawn_mesh_with_material};
use nightshade_sprite::prelude::*;

pub fn till(
    commands: &mut Commands,
    terrain: &Terrain,
    farmland: &mut Farmland,
    column: u32,
    row: u32,
) {
    let middle = center(terrain, column, row);
    let top = height(terrain, column, row) as f32;
    let cap = spawn_mesh_with_material(
        commands,
        "Cube",
        Vec3::new(middle.x, top - theme::CAP_THICKNESS / 2.0 + 0.012, middle.y),
        Vec3::new(1.0, theme::CAP_THICKNESS, 1.0),
        theme::TILLED_MATERIAL,
    );
    farmland.plots.push(Plot {
        column,
        row,
        crop: None,
        cap: Some(cap),
        watered_day: None,
    });
}

pub fn set_wet(commands: &mut Commands, plot: &Plot, wet: bool) {
    if let Some(cap) = plot.cap {
        let material = if wet {
            theme::WET_TILLED_MATERIAL
        } else {
            theme::TILLED_MATERIAL
        };
        set_mesh_material(commands, cap, material);
    }
}

pub fn water(commands: &mut Commands, farmland: &mut Farmland, plot_index: usize, day: u32) {
    let plot = &mut farmland.plots[plot_index];
    if plot.watered_day == Some(day) {
        return;
    }
    plot.watered_day = Some(day);
    set_wet(commands, plot, true);
}

pub fn sow(
    commands: &mut Commands,
    sheet: &Sheet,
    terrain: &Terrain,
    farmland: &mut Farmland,
    plot_index: usize,
    kind: CropKind,
    progress: f32,
) {
    let plot = &farmland.plots[plot_index];
    let middle = center(terrain, plot.column, plot.row);
    let ground = height(terrain, plot.column, plot.row) as f32;
    let (region, tall) = standing(sheet, kind.stages()[stage_of(progress)]);
    let sprite = Sprite::new(theme::ATLAS, region)
        .with_size(1.0, tall)
        .with_anchor(Vec2::new(0.5, 0.0))
        .with_layer(theme::LAYER_STANDING);
    let position = Vec3::new(middle.x, ground, middle.y);
    let crop = spawn_sprite(commands, sprite, position);
    commands.set(crop, Upright::default());
    commands.set(crop, kind);
    commands.set(crop, Progress(progress));
    commands.set(crop, PlotIndex(plot_index));
    crate::commands::life::let_sway(commands, crop, position, 0.8);
    commands.set(
        crop,
        crate::components::Lit {
            base: [1.0, 1.0, 1.0, 1.0],
        },
    );
    farmland.plots[plot_index].crop = Some(crop);
}

pub fn plant(
    commands: &mut Commands,
    sheet: &Sheet,
    terrain: &Terrain,
    farmland: &mut Farmland,
    inventory: &mut Inventory,
    toast: &mut Toast,
    plot_index: usize,
    kind: CropKind,
) {
    if !take_item(inventory, kind.seed(), 1) {
        return;
    }
    sow(commands, sheet, terrain, farmland, plot_index, kind, 0.0);
    show_toast(toast, format!("Planted {}", kind.name()));
}

pub fn harvest(
    commands: &mut Commands,
    terrain: &Terrain,
    farmland: &mut Farmland,
    inventory: &mut Inventory,
    toast: &mut Toast,
    plot_index: usize,
    kind: CropKind,
) {
    let plot = &mut farmland.plots[plot_index];
    let Some(crop) = plot.crop.take() else {
        return;
    };
    commands.despawn(crop);
    let middle = center(terrain, plot.column, plot.row);
    let top = height(terrain, plot.column, plot.row) as f32;
    sparkle(commands, Vec3::new(middle.x, top + 0.4, middle.y));
    add_item(inventory, kind.harvest(), theme::HARVEST_YIELD);
    show_toast(
        toast,
        format!(
            "Harvested {} {}",
            theme::HARVEST_YIELD,
            kind.harvest().name()
        ),
    );
}
