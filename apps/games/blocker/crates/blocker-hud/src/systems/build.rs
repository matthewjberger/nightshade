use crate::queries::{art_sheet, decode_frame_border};
use crate::resources::{ArtLayers, HudBuild};
use crate::systems::hud::chat::build_chat;
use crate::systems::hud::compass::build_compass;
use crate::systems::hud::crosshair::build_crosshair;
use crate::systems::hud::hotbar::build_hotbar;
use crate::systems::hud::prompt::build_prompt;
use crate::systems::hud::quests::build_quests;
use crate::systems::hud::rate::{build_rate, build_readouts};
use crate::systems::hud::target::build_target;
use crate::systems::hud::toast::build_toast;
use crate::systems::hud::vitals::build_vitals;
use crate::theme;
use blocker_blocks::data::blocks::Palette;
use blocker_blocks::queries::decode_atlas;
use blocker_screen::data::screen::Screen;
use nalgebra_glm::{vec2, vec4};
use nightshade::prelude::*;
use nightshade_ui::commands::builder::{UiBuild, UiTreeBuilder};
use nightshade_ui::prelude::*;
use nightshade_ui_menu::prelude::menu_show_on;

pub fn build(mut hud: HudBuild<'_>, mut build: UiBuild) {
    let mut tree = UiTreeBuilder::new(build.context());
    let (atlas_pixels, atlas_width, atlas_height) = decode_atlas(Palette::Kenney);
    let (tool_pixels, tool_width, tool_height) = art_sheet();
    let border = decode_frame_border();
    let (atlas_upload, tool_upload, frame_upload) = {
        let (images, runtime) = tree.images_and_runtime();
        let atlas = ui_upload_image(images, runtime, &atlas_pixels, atlas_width, atlas_height);
        let tools = ui_upload_image(images, runtime, &tool_pixels, tool_width, tool_height);
        let frame = border.as_ref().and_then(|(pixels, width, height)| {
            ui_upload_image(images, runtime, pixels, *width, *height)
        });
        (atlas, tools, frame)
    };
    *hud.art = ArtLayers {
        atlas: atlas_upload.as_ref().map(|upload| upload.layer),
        tools: tool_upload.as_ref().map(|upload| upload.layer),
        frame: frame_upload.as_ref().map(|upload| upload.layer),
    };

    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .entity();

    tree.in_parent(root, |tree| {
        let backdrop = tree
            .add_node()
            .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
            .with_rect(0.0, 0.0, vec4(0.0, 0.0, 0.0, 0.0))
            .color_raw::<UiBase>(theme::BACKDROP_COLOR)
            .with_visible(false)
            .entity();
        hud.shell.backdrop = backdrop;
        let hud_root = tree
            .add_node()
            .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
            .entity();
        hud.hotbar.hud = hud_root;
        tree.in_parent(hud_root, |tree| {
            let gear = tree
                .add_node()
                .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
                .entity();
            tree.in_parent(gear, |tree| {
                build_crosshair(tree);
                build_hotbar(tree, &mut hud, &atlas_upload);
                build_vitals(tree, &mut hud);
                build_target(tree, &mut hud);
            });
            hud.hotbar.gear = gear;
            build_readouts(tree, &mut hud);
            build_chat(tree, &mut hud);
            build_quests(tree, &mut hud);
            build_compass(tree, &mut hud);
        });
        build_rate(tree, &mut hud);
        build_prompt(tree, &mut hud);
        build_toast(tree, &mut hud);
    });

    hud.shell.root = root;
    menu_show_on(hud.menus, Screen::Playing, hud.hotbar.hud);
    tree.finish();
    hud.icons.layers[Palette::Kenney.code() as usize] = hud.art.atlas;
    hud.icons.painted = Some(Palette::Kenney);
}
