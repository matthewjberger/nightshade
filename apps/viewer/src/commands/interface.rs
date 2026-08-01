use nightshade::prelude::*;
use nightshade_platform::winit;
use nightshade_text::prelude::*;
use nightshade_ui::commands::builder::UiBuild;
use nightshade_ui::prelude::*;

use crate::data::{
    ANTI_ALIASING_LABELS, ATMOSPHERE_LABELS, PBR_LABELS, RESOLUTION_LABELS, SHADING_LABELS,
    TONEMAP_LABELS,
};
use crate::queries::layout::panel_top;
use crate::resources::{Layout, UiHandles, ViewerResources};
use crate::theme::*;

pub fn build_tree(viewer: &mut ViewerResources, layout: Layout, build: &mut UiBuild<'_>) {
    viewer.layout = layout;
    if layout.narrow {
        viewer.view.scene_open = false;
        viewer.view.inspector_open = false;
    }
    let mut handles = UiHandles::default();
    let mut tree = UiTreeBuilder::new(build.context());

    let interface = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .entity();
    handles.interface = interface;

    tree.in_parent(interface, |tree| {
        build_top_bar(tree, &mut handles, layout);
        build_left_panel(tree, &mut handles, layout);
        build_inspector(tree, &mut handles, layout);
        build_animation_bar(tree, &mut handles, layout);
        build_layer_panel(tree, &mut handles, layout);
        build_nav_gizmo(tree, &mut handles, layout);
        build_browser(tree, &mut handles);
        build_add_menu(tree, &mut handles);
        build_recent(tree, &mut handles);
        build_loading_label(tree, &mut handles, layout);
        build_notice(tree, &mut handles);
    });

    handles.show_button = tree
        .add_node()
        .window(
            Rl(vec2(100.0, 100.0)) + Ab(vec2(-16.0, -16.0)),
            Ab(vec2(12.0, 12.0)),
            Anchor::BottomRight,
        )
        .rect(6.0)
        .color_raw::<UiBase>(vec4(1.0, 1.0, 1.0, 0.25))
        .fg_hover(ThemeColor::Text)
        .with_interaction()
        .with_tooltip("Show the interface (H)")
        .with_cursor_icon(winit::window::CursorIcon::Pointer)
        .with_visible(false)
        .entity();
    let show_slot = tree.text_state_mut().cache.add_text("");
    tree.commands().set(
        handles.show_button,
        UiButtonData {
            clicked: false,
            text_slot: Some(show_slot),
        },
    );

    tree.finish();
    viewer.handles = handles;
}

pub fn button(tree: &mut UiTreeBuilder, label: &str, width: f32) -> Entity {
    let text_slot = tree.text_state_mut().cache.add_text(label);
    let entity = tree
        .add_node()
        .flow_child(Ab(vec2(width, 28.0)))
        .rect(6.0)
        .color_raw::<UiBase>(vec4(0.0, 0.0, 0.0, 0.0))
        .fg_hover(ThemeColor::BackgroundHover)
        .fg_pressed(ThemeColor::BackgroundActive)
        .with_transition::<UiHover>(12.0, 6.0)
        .with_transition::<UiPressed>(20.0, 10.0)
        .with_interaction()
        .with_cursor_icon(winit::window::CursorIcon::Pointer)
        .with_children(|inner| {
            inner
                .add_node()
                .window(Rl(vec2(50.0, 50.0)), Ab(vec2(width, 18.0)), Anchor::Center)
                .with_text_slot(text_slot, 13.0)
                .fg(ThemeColor::Text);
        })
        .entity();
    tree.commands().set(
        entity,
        UiButtonData {
            clicked: false,
            text_slot: Some(text_slot),
        },
    );
    entity
}

fn label_node(
    tree: &mut UiTreeBuilder,
    text: &str,
    size: f32,
    width: f32,
    color: ThemeColor,
) -> Entity {
    tree.add_node()
        .flow_child(Ab(vec2(width, 18.0)))
        .with_text(text, size)
        .text_left()
        .fg(color)
        .entity()
}

fn spacer(tree: &mut UiTreeBuilder, width: f32) -> Entity {
    tree.add_node().flow_child(Ab(vec2(width, 1.0))).entity()
}

fn build_top_bar(tree: &mut UiTreeBuilder, handles: &mut UiHandles, layout: Layout) {
    let border = tree.active_theme().border_color;
    let bar = tree
        .add_node()
        .boundary(
            Rl(vec2(0.0, 0.0)),
            Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, layout.top_bar_height)),
        )
        .with_rect(0.0, 0.0, border)
        .fg(ThemeColor::Panel)
        .with_interaction()
        .with_depth(UiDepthMode::Set(10.0))
        .entity();

    tree.in_parent(bar, |tree| {
        tree.add_node()
            .boundary(
                Rl(vec2(0.0, 100.0)) + Ab(vec2(0.0, -1.0)),
                Rl(vec2(100.0, 100.0)),
            )
            .rect(0.0)
            .fg(ThemeColor::Border);

        if layout.narrow {
            let first = tree
                .add_node()
                .window(
                    Ab(vec2(12.0, 6.0)),
                    Ab(vec2(layout.panel_width, 30.0)),
                    Anchor::TopLeft,
                )
                .flow_horizontal()
                .padding(0.0)
                .gap(4.0)
                .align_cross(FlowAlignment::Center)
                .entity();
            tree.in_parent(first, |tree| {
                panel_buttons(tree, handles);
                handles.browse_button = button(tree, "Browse", 66.0);
                edit_buttons(tree, handles);
            });

            let second = tree
                .add_node()
                .window(
                    Ab(vec2(12.0, 42.0)),
                    Ab(vec2(layout.panel_width, 30.0)),
                    Anchor::TopLeft,
                )
                .flow_horizontal()
                .padding(0.0)
                .gap(4.0)
                .align_cross(FlowAlignment::Center)
                .entity();
            tree.in_parent(second, |tree| {
                view_buttons(tree, handles);
                random_buttons(tree, handles);
                fps_readout(tree, handles);
            });
            return;
        }

        let left = tree
            .add_node()
            .window(
                Ab(vec2(12.0, layout.top_bar_height * 0.5)),
                Ab(vec2(760.0, 30.0)),
                Anchor::CenterLeft,
            )
            .flow_horizontal()
            .padding(0.0)
            .gap(4.0)
            .align_cross(FlowAlignment::Center)
            .entity();

        tree.in_parent(left, |tree| {
            label_node(tree, "Nightshade", 14.0, 92.0, ThemeColor::Text);
            spacer(tree, 6.0);
            panel_buttons(tree, handles);
            spacer(tree, 6.0);
            view_buttons(tree, handles);
            spacer(tree, 6.0);
            edit_buttons(tree, handles);
        });

        let right = tree
            .add_node()
            .window(
                Rl(vec2(100.0, 0.0)) + Ab(vec2(-12.0, layout.top_bar_height * 0.5)),
                Ab(vec2(400.0, 30.0)),
                Anchor::CenterRight,
            )
            .flow_horizontal()
            .padding(0.0)
            .gap(4.0)
            .align_main(FlowAlignment::End)
            .align_cross(FlowAlignment::Center)
            .entity();

        tree.in_parent(right, |tree| {
            handles.browse_button = button(tree, "Browse", 66.0);
            label_node(tree, "Random:", 12.0, 56.0, ThemeColor::TextDisabled);
            random_buttons(tree, handles);
            fps_readout(tree, handles);
        });
    });
}

fn panel_buttons(tree: &mut UiTreeBuilder, handles: &mut UiHandles) {
    handles.scene_button = button(tree, "Scene", 58.0);
    handles.inspect_button = button(tree, "Inspect", 66.0);
    handles.add_button = button(tree, "+ Add", 58.0);
}

fn view_buttons(tree: &mut UiTreeBuilder, handles: &mut UiHandles) {
    handles.grid_button = button(tree, "Grid", 48.0);
    handles.spin_button = button(tree, "Spin", 48.0);
    handles.frame_button = button(tree, "Frame", 58.0);
}

fn edit_buttons(tree: &mut UiTreeBuilder, handles: &mut UiHandles) {
    handles.undo_button = button(tree, "Undo", 54.0);
    handles.redo_button = button(tree, "Redo", 54.0);
    handles.shot_button = button(tree, "Shot", 50.0);
    handles.save_button = button(tree, "Save", 50.0);
    handles.recent_button = button(tree, "Recent", 64.0);
}

fn random_buttons(tree: &mut UiTreeBuilder, handles: &mut UiHandles) {
    handles.random_model_button = button(tree, "Model", 56.0);
    handles.random_sky_button = button(tree, "Sky", 44.0);
}

fn fps_readout(tree: &mut UiTreeBuilder, handles: &mut UiHandles) {
    handles.fps_label = tree
        .add_node()
        .flow_child(Ab(vec2(64.0, 18.0)))
        .with_text("0 fps", 12.0)
        .text_right()
        .fg(ThemeColor::TextDisabled)
        .entity();
}

fn build_left_panel(tree: &mut UiTreeBuilder, handles: &mut UiHandles, layout: Layout) {
    let panel = tree
        .add_node()
        .boundary(
            Ab(vec2(PANEL_MARGIN, panel_top(layout))),
            Rl(vec2(0.0, 100.0)) + Ab(vec2(PANEL_MARGIN + layout.panel_width, -PANEL_MARGIN)),
        )
        .card()
        .with_interaction()
        .with_clip()
        .with_visible(false)
        .entity();
    handles.left_panel = panel;

    tree.in_parent(panel, |tree| {
        let tabs = tree
            .add_node()
            .boundary(
                Rl(vec2(0.0, 0.0)),
                Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 34.0)),
            )
            .flow_horizontal()
            .padding(4.0)
            .gap(2.0)
            .entity();
        tree.in_parent(tabs, |tree| {
            handles.tab_buttons[0] = button(tree, "Scene", 92.0);
            handles.tab_buttons[1] = button(tree, "Render", 92.0);
            handles.tab_buttons[2] = button(tree, "Stats", 92.0);
        });

        tree.add_node()
            .boundary(
                Rl(vec2(0.0, 0.0)) + Ab(vec2(0.0, 34.0)),
                Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 35.0)),
            )
            .rect(0.0)
            .fg(ThemeColor::Border);

        let body = tree
            .add_node()
            .boundary(Ab(vec2(0.0, 35.0)), Rl(vec2(100.0, 100.0)))
            .entity();

        tree.in_parent(body, |tree| {
            handles.tab_roots[0] = build_scene_tab(tree, handles);
            handles.tab_roots[1] = build_render_tab(tree, handles);
            handles.tab_roots[2] = build_stats_tab(tree, handles);
        });
    });
}

fn build_scene_tab(tree: &mut UiTreeBuilder, handles: &mut UiHandles) -> Entity {
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .entity();
    tree.in_parent(root, |tree| {
        let header = tree
            .add_node()
            .boundary(
                Ab(vec2(6.0, 6.0)),
                Rl(vec2(100.0, 0.0)) + Ab(vec2(-6.0, 34.0)),
            )
            .entity();
        handles.tree_search = tree.in_parent(header, |tree| tree.add_text_input("Filter"));

        let body = tree
            .add_node()
            .boundary(Ab(vec2(0.0, 38.0)), Rl(vec2(100.0, 100.0)))
            .entity();
        tree.in_parent(body, |tree| {
            handles.tree_view = tree.add_tree_view(true);
            handles.tree_container = tree
                .component::<UiTreeViewData>(handles.tree_view)
                .map(|data| data.content_entity)
                .unwrap_or(handles.tree_view);
            handles.tree_empty = tree
                .add_node()
                .window(
                    Ab(vec2(8.0, 8.0)),
                    Rl(vec2(100.0, 0.0)) + Ab(vec2(-16.0, 24.0)),
                    Anchor::TopLeft,
                )
                .with_text("No model is loaded.", 12.0)
                .text_left()
                .fg(ThemeColor::TextDisabled)
                .entity();
        });
    });
    root
}

fn build_render_tab(tree: &mut UiTreeBuilder, handles: &mut UiHandles) -> Entity {
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .with_visible(false)
        .entity();
    tree.in_parent(root, |tree| {
        let scroll = tree.add_scroll_area_fill(8.0, 6.0);
        let content = tree
            .component::<UiScrollAreaData>(scroll)
            .map(|data| data.content_entity)
            .unwrap_or(scroll);
        tree.in_parent(content, |tree| {
            let grid = tree.add_property_grid(LABEL_WIDTH);
            let shading = tree.add_property_section(grid, "Shading");
            handles.shading_dropdown =
                tree.add_property_dropdown(grid, shading, "Mode", &SHADING_LABELS, 0);
            handles.pbr_dropdown =
                tree.add_property_dropdown(grid, shading, "Channel", &PBR_LABELS, 0);
            handles.normals_toggle = property_toggle(tree, grid, shading, "Normals", false);
            handles.bounds_toggle = property_toggle(tree, grid, shading, "Bounds", false);
            handles.skeleton_toggle = property_toggle(tree, grid, shading, "Skeleton", false);

            let environment = tree.add_property_section(grid, "Environment");
            handles.atmosphere_dropdown =
                tree.add_property_dropdown(grid, environment, "Sky", &ATMOSPHERE_LABELS, 1);
            handles.sky_toggle = property_toggle(tree, grid, environment, "Show sky", true);
            handles.exposure_slider =
                property_slider(tree, grid, environment, "Exposure", 0.1, 5.0, 1.0);
            handles.tonemap_dropdown =
                tree.add_property_dropdown(grid, environment, "Tone map", &TONEMAP_LABELS, 0);

            let sun = tree.add_property_section(grid, "Sun");
            handles.sun_intensity = property_slider(tree, grid, sun, "Intensity", 0.0, 20.0, 3.5);
            handles.sun_yaw = property_drag(tree, grid, sun, "Heading", -360.0, 360.0, 45.0, 0.5);
            handles.sun_pitch = property_drag(tree, grid, sun, "Height", -89.0, 0.0, -30.0, 0.25);
            handles.sun_shadows = property_toggle(tree, grid, sun, "Shadows", true);

            let post = tree.add_property_section(grid, "Post");
            handles.anti_aliasing_dropdown =
                tree.add_property_dropdown(grid, post, "Edges", &ANTI_ALIASING_LABELS, 1);
            handles.render_scale_slider =
                property_slider(tree, grid, post, "Scale", 0.25, 2.0, 1.0);
            handles.ssao_toggle = property_toggle(tree, grid, post, "Occlusion", true);
            handles.bloom_toggle = property_toggle(tree, grid, post, "Bloom", true);
            handles.ssr_toggle = property_toggle(tree, grid, post, "Reflections", false);
            handles.ssgi_toggle = property_toggle(tree, grid, post, "Bounce light", false);
            let fog = tree.add_property_section(grid, "Fog");
            handles.fog_toggle = property_toggle(tree, grid, fog, "Fog", false);
            handles.fog_start = property_drag(tree, grid, fog, "Start", 0.0, 1000.0, 2.0, 0.05);
            handles.fog_end = property_drag(tree, grid, fog, "End", 0.1, 1000.0, 15.0, 0.1);
            handles.fog_color = {
                let row = tree.add_property_row(grid, fog, "Color");
                tree.in_parent(row, |tree| tree.add_color_picker(vec4(0.5, 0.5, 0.55, 1.0)))
            };

            let focus = tree.add_property_section(grid, "Focus");
            handles.depth_of_field_toggle = property_toggle(tree, grid, focus, "Focus", false);
            handles.focus_distance =
                property_drag(tree, grid, focus, "Distance", 0.01, 1000.0, 10.0, 0.05);
            handles.focus_range = property_drag(tree, grid, focus, "Range", 0.01, 500.0, 5.0, 0.05);
            tree.add_node()
                .fill_width()
                .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 28.0)))
                .with_text("A click in the viewport sets the distance.", 10.0)
                .text_left()
                .with_text_wrap()
                .fg(ThemeColor::TextDisabled);

            handles.camera_container = tree
                .add_node()
                .fill_width()
                .auto_height()
                .flow_vertical()
                .padding(0.0)
                .gap(4.0)
                .entity();
            handles.variant_container = tree
                .add_node()
                .fill_width()
                .auto_height()
                .flow_vertical()
                .padding(0.0)
                .gap(4.0)
                .entity();
            handles.morph_container = tree
                .add_node()
                .fill_width()
                .auto_height()
                .flow_vertical()
                .padding(0.0)
                .gap(4.0)
                .entity();
        });
    });
    root
}

fn build_stats_tab(tree: &mut UiTreeBuilder, handles: &mut UiHandles) -> Entity {
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .with_visible(false)
        .entity();
    const STATS: [&str; 6] = [
        "Meshes",
        "Vertices",
        "Triangles",
        "Materials",
        "Textures",
        "Size",
    ];
    tree.in_parent(root, |tree| {
        let scroll = tree.add_scroll_area_fill(8.0, 4.0);
        let content = tree
            .component::<UiScrollAreaData>(scroll)
            .map(|data| data.content_entity)
            .unwrap_or(scroll);
        tree.in_parent(content, |tree| {
            for (index, name) in STATS.iter().enumerate() {
                let row = tree
                    .add_node()
                    .fill_width()
                    .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 22.0)))
                    .flow_horizontal()
                    .padding(0.0)
                    .gap(8.0)
                    .align_cross(FlowAlignment::Center)
                    .entity();
                tree.in_parent(row, |tree| {
                    label_node(tree, name, 12.0, 100.0, ThemeColor::TextDisabled);
                    handles.stat_values[index] = tree
                        .add_node()
                        .flow_child(Ab(vec2(0.0, 18.0)))
                        .flex_grow(1.0)
                        .with_text("-", 12.0)
                        .text_right()
                        .fg(ThemeColor::Text)
                        .entity();
                });
            }
        });
    });
    root
}

fn property_toggle(
    tree: &mut UiTreeBuilder,
    grid: Entity,
    section: Entity,
    label: &str,
    initial: bool,
) -> Entity {
    let area = tree.add_property_row(grid, section, label);
    tree.in_parent(area, |tree| tree.add_toggle(initial))
}

fn property_slider(
    tree: &mut UiTreeBuilder,
    grid: Entity,
    section: Entity,
    label: &str,
    min: f32,
    max: f32,
    initial: f32,
) -> Entity {
    let area = tree.add_property_row(grid, section, label);
    tree.in_parent(area, |tree| {
        let holder = tree
            .add_node()
            .flow_child(Ab(vec2(0.0, 22.0)))
            .flex_grow(1.0)
            .entity();
        tree.in_parent(holder, |tree| {
            tree.add_slider_configured(SliderConfig::new(min, max, initial).precision(2))
        })
    })
}

fn property_drag(
    tree: &mut UiTreeBuilder,
    grid: Entity,
    section: Entity,
    label: &str,
    min: f32,
    max: f32,
    initial: f32,
    speed: f32,
) -> Entity {
    let area = tree.add_property_row(grid, section, label);
    tree.in_parent(area, |tree| {
        let holder = tree
            .add_node()
            .flow_child(Ab(vec2(0.0, 22.0)))
            .flex_grow(1.0)
            .entity();
        tree.in_parent(holder, |tree| {
            tree.add_drag_value_configured(
                DragValueConfig::new(min, max, initial)
                    .speed(speed)
                    .precision(1),
            )
        })
    })
}

fn build_inspector(tree: &mut UiTreeBuilder, handles: &mut UiHandles, layout: Layout) {
    let panel = tree
        .add_node()
        .boundary(
            Rl(vec2(100.0, 0.0))
                + Ab(vec2(
                    -(PANEL_MARGIN + layout.panel_width),
                    panel_top(layout),
                )),
            Rl(vec2(100.0, 100.0)) + Ab(vec2(-PANEL_MARGIN, -PANEL_MARGIN)),
        )
        .card()
        .with_interaction()
        .with_clip()
        .with_visible(false)
        .entity();
    handles.inspector_panel = panel;

    tree.in_parent(panel, |tree| {
        let header = tree
            .add_node()
            .boundary(
                Ab(vec2(8.0, 6.0)),
                Rl(vec2(100.0, 0.0)) + Ab(vec2(-8.0, 52.0)),
            )
            .flow_vertical()
            .padding(0.0)
            .gap(2.0)
            .entity();
        tree.in_parent(header, |tree| {
            let holder = tree
                .add_node()
                .fill_width()
                .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 26.0)))
                .entity();
            handles.inspector_name = tree.in_parent(holder, |tree| tree.add_text_input("Name"));
            handles.inspector_mesh = tree
                .add_node()
                .fill_width()
                .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 14.0)))
                .with_text("", 11.0)
                .text_left()
                .fg(ThemeColor::TextAccent)
                .entity();
        });

        let modes = tree
            .add_node()
            .boundary(
                Ab(vec2(8.0, 60.0)),
                Rl(vec2(100.0, 0.0)) + Ab(vec2(-8.0, 92.0)),
            )
            .flow_horizontal()
            .padding(0.0)
            .gap(4.0)
            .align_cross(FlowAlignment::Center)
            .entity();
        tree.in_parent(modes, |tree| {
            handles.gizmo_buttons[0] = button(tree, "Move", 58.0);
            handles.gizmo_buttons[1] = button(tree, "Rotate", 64.0);
            handles.gizmo_buttons[2] = button(tree, "Scale", 58.0);
            handles.snap_button = button(tree, "Snap", 54.0);
        });

        let visibility = tree
            .add_node()
            .boundary(
                Ab(vec2(8.0, 94.0)),
                Rl(vec2(100.0, 0.0)) + Ab(vec2(-8.0, 126.0)),
            )
            .flow_horizontal()
            .padding(0.0)
            .gap(4.0)
            .align_cross(FlowAlignment::Center)
            .entity();
        tree.in_parent(visibility, |tree| {
            handles.visible_button = button(tree, "Hide", 58.0);
            handles.isolate_button = button(tree, "Isolate", 78.0);
        });

        tree.add_node()
            .boundary(
                Rl(vec2(0.0, 0.0)) + Ab(vec2(0.0, 128.0)),
                Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 129.0)),
            )
            .rect(0.0)
            .fg(ThemeColor::Border);

        handles.inspector_empty = tree
            .add_node()
            .boundary(
                Ab(vec2(10.0, 140.0)),
                Rl(vec2(100.0, 0.0)) + Ab(vec2(-10.0, 176.0)),
            )
            .with_text(
                "Click an object in the viewport, or pick one in the scene tree.",
                12.0,
            )
            .with_text_wrap()
            .with_text_alignment(TextAlignment::Left, VerticalAlignment::Top)
            .fg(ThemeColor::TextDisabled)
            .entity();

        let body = tree
            .add_node()
            .boundary(Ab(vec2(0.0, 129.0)), Rl(vec2(100.0, 100.0)))
            .with_visible(false)
            .entity();
        handles.inspector_body = body;

        tree.in_parent(body, |tree| {
            let scroll = tree.add_scroll_area_fill(8.0, 6.0);
            let content = tree
                .component::<UiScrollAreaData>(scroll)
                .map(|data| data.content_entity)
                .unwrap_or(scroll);
            tree.in_parent(content, |tree| {
                let grid = tree.add_property_grid(64.0);
                let transform = tree.add_property_section(grid, "Transform");
                handles.translation = vector_row(tree, grid, transform, "Position", 0.01);
                handles.rotation = vector_row(tree, grid, transform, "Rotation", 0.5);
                handles.scale = vector_row(tree, grid, transform, "Scale", 0.01);
                handles.light_container = section_container(tree);
                handles.camera_section = section_container(tree);
                handles.material_container = section_container(tree);
            });
        });
    });
}

fn section_container(tree: &mut UiTreeBuilder) -> Entity {
    tree.add_node()
        .fill_width()
        .auto_height()
        .flow_vertical()
        .padding(0.0)
        .gap(4.0)
        .entity()
}

fn vector_row(
    tree: &mut UiTreeBuilder,
    grid: Entity,
    section: Entity,
    label: &str,
    speed: f32,
) -> [Entity; 3] {
    let area = tree.add_property_row(grid, section, label);
    let mut fields = [Entity::default(); 3];
    tree.in_parent(area, |tree| {
        for field in fields.iter_mut() {
            let holder = tree
                .add_node()
                .flow_child(Ab(vec2(0.0, 22.0)))
                .flex_grow(1.0)
                .entity();
            *field = tree.in_parent(holder, |tree| {
                tree.add_drag_value_configured(
                    DragValueConfig::new(-1.0e6, 1.0e6, 0.0)
                        .speed(speed)
                        .precision(3),
                )
            });
        }
    });
    fields
}

fn build_animation_bar(tree: &mut UiTreeBuilder, handles: &mut UiHandles, layout: Layout) {
    let bar = tree
        .add_node()
        .window(
            Rl(vec2(50.0, 100.0)) + Ab(vec2(0.0, -16.0)),
            Ab(vec2(layout.animation_bar_width, ANIMATION_BAR_HEIGHT)),
            Anchor::BottomCenter,
        )
        .card()
        .with_interaction()
        .with_visible(false)
        .entity();
    handles.animation_bar = bar;

    tree.in_parent(bar, |tree| {
        let row = tree
            .add_node()
            .boundary(
                Ab(vec2(10.0, 0.0)),
                Rl(vec2(100.0, 100.0)) + Ab(vec2(-10.0, 0.0)),
            )
            .flow_horizontal()
            .padding(0.0)
            .gap(8.0)
            .align_cross(FlowAlignment::Center)
            .entity();
        tree.in_parent(row, |tree| {
            handles.play_button = button(tree, "Play", 50.0);
            handles.stop_button = button(tree, "Stop", 50.0);
            handles.clip_container = tree.add_node().flow_child(Ab(vec2(140.0, 26.0))).entity();
            let seek_holder = tree.add_node().flow_child(Ab(vec2(150.0, 22.0))).entity();
            handles.seek_slider = tree.in_parent(seek_holder, |tree| {
                tree.add_slider_configured(SliderConfig::new(0.0, 1.0, 0.0).precision(2))
            });
            handles.time_label = tree
                .add_node()
                .flow_child(Ab(vec2(84.0, 18.0)))
                .with_text("0.0 / 0.0 s", 12.0)
                .text_right()
                .fg(ThemeColor::TextDisabled)
                .entity();
            let speed_holder = tree.add_node().flow_child(Ab(vec2(56.0, 22.0))).entity();
            handles.speed_drag = tree.in_parent(speed_holder, |tree| {
                tree.add_drag_value_configured(
                    DragValueConfig::new(0.1, 4.0, 1.0).speed(0.01).precision(2),
                )
            });
            let blend_holder = tree.add_node().flow_child(Ab(vec2(56.0, 22.0))).entity();
            handles.blend_drag = tree.in_parent(blend_holder, |tree| {
                tree.add_drag_value_configured(
                    DragValueConfig::new(0.0, 4.0, 0.25)
                        .speed(0.01)
                        .precision(2),
                )
            });
            handles.loop_button = button(tree, "Loop", 50.0);
            handles.add_layer_button = button(tree, "+ Layer", 64.0);
        });
    });
}

fn build_layer_panel(tree: &mut UiTreeBuilder, handles: &mut UiHandles, layout: Layout) {
    let panel = tree
        .add_node()
        .window(
            Rl(vec2(50.0, 100.0)) + Ab(vec2(0.0, -(24.0 + ANIMATION_BAR_HEIGHT))),
            Ab(vec2(layout.animation_bar_width, 0.0)),
            Anchor::BottomCenter,
        )
        .card()
        .with_interaction()
        .with_visible(false)
        .entity();
    handles.layer_panel = panel;
    tree.in_parent(panel, |tree| {
        handles.layer_container = tree
            .add_node()
            .fill_width()
            .auto_height()
            .flow_vertical()
            .padding(8.0)
            .gap(4.0)
            .entity();
    });
}

fn build_nav_gizmo(tree: &mut UiTreeBuilder, handles: &mut UiHandles, layout: Layout) {
    let holder = tree
        .add_node()
        .window(
            Rl(vec2(100.0, 0.0)) + Ab(vec2(-PANEL_MARGIN, panel_top(layout))),
            Ab(vec2(NAV_GIZMO_SIZE, NAV_GIZMO_SIZE)),
            Anchor::TopRight,
        )
        .rect(NAV_GIZMO_SIZE * 0.5)
        .color_raw::<UiBase>(vec4(0.0, 0.0, 0.0, 0.35))
        .with_depth(UiDepthMode::Set(20.0))
        .entity();
    handles.nav_holder = holder;
    handles.nav_gizmo = tree.in_parent(holder, |tree| {
        tree.add_canvas_interactive(vec2(NAV_GIZMO_SIZE, NAV_GIZMO_SIZE))
    });
}

fn build_browser(tree: &mut UiTreeBuilder, handles: &mut UiHandles) {
    let backdrop = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .rect(0.0)
        .color_raw::<UiBase>(vec4(0.0, 0.0, 0.0, 0.6))
        .with_interaction()
        .with_depth(UiDepthMode::Set(40.0))
        .with_visible(false)
        .entity();
    handles.browser_panel = backdrop;

    tree.in_parent(backdrop, |tree| {
        let panel = tree
            .add_node()
            .window(Rl(vec2(50.0, 50.0)), Rl(vec2(84.0, 82.0)), Anchor::Center)
            .card()
            .with_interaction()
            .with_clip()
            .entity();

        tree.in_parent(panel, |tree| {
            let header = tree
                .add_node()
                .boundary(
                    Ab(vec2(10.0, 0.0)),
                    Rl(vec2(100.0, 0.0)) + Ab(vec2(-10.0, 46.0)),
                )
                .flow_horizontal()
                .padding(0.0)
                .gap(6.0)
                .align_cross(FlowAlignment::Center)
                .entity();
            tree.in_parent(header, |tree| {
                handles.browser_tabs[0] = button(tree, "Library", 70.0);
                handles.browser_tabs[1] = button(tree, "Khronos", 76.0);
                handles.browser_tabs[2] = button(tree, "Models", 70.0);
                handles.browser_tabs[3] = button(tree, "Skies", 62.0);
                let search_holder = tree
                    .add_node()
                    .flow_child(Ab(vec2(0.0, 26.0)))
                    .flex_grow(1.0)
                    .entity();
                handles.browser_search =
                    tree.in_parent(search_holder, |tree| tree.add_text_input("Search"));
                handles.browser_previous = button(tree, "<", 30.0);
                handles.browser_page_label = tree
                    .add_node()
                    .flow_child(Ab(vec2(52.0, 18.0)))
                    .with_text("1 / 1", 12.0)
                    .text_center()
                    .fg(ThemeColor::TextDisabled)
                    .entity();
                handles.browser_next = button(tree, ">", 30.0);
                let resolution_holder = tree.add_node().flow_child(Ab(vec2(76.0, 26.0))).entity();
                handles.browser_resolution = tree.in_parent(resolution_holder, |tree| {
                    tree.add_dropdown(&RESOLUTION_LABELS, 1)
                });
                handles.browser_refresh = button(tree, "Refresh", 70.0);
                handles.browser_close = button(tree, "Close", 62.0);
            });

            tree.add_node()
                .boundary(
                    Rl(vec2(0.0, 0.0)) + Ab(vec2(0.0, 46.0)),
                    Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 47.0)),
                )
                .rect(0.0)
                .fg(ThemeColor::Border);

            let body = tree
                .add_node()
                .boundary(Ab(vec2(0.0, 47.0)), Rl(vec2(100.0, 100.0)))
                .entity();
            tree.in_parent(body, |tree| {
                let scroll = tree.add_scroll_area_fill(10.0, 10.0);
                let content = tree
                    .component::<UiScrollAreaData>(scroll)
                    .map(|data| data.content_entity)
                    .unwrap_or(scroll);
                tree.in_parent(content, |tree| {
                    build_library_strip(tree, handles);
                    handles.browser_grid = tree
                        .add_node()
                        .fill_width()
                        .auto_height()
                        .flow_horizontal()
                        .flow_wrap()
                        .padding(0.0)
                        .gap(10.0)
                        .entity();
                });
            });
        });
    });
}

fn build_library_strip(tree: &mut UiTreeBuilder, handles: &mut UiHandles) {
    let strip = tree
        .add_node()
        .fill_width()
        .auto_height()
        .flow_vertical()
        .padding(0.0)
        .gap(6.0)
        .with_visible(false)
        .entity();
    handles.library_strip = strip;

    tree.in_parent(strip, |tree| {
        let row = tree
            .add_node()
            .fill_width()
            .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 30.0)))
            .flow_horizontal()
            .padding(0.0)
            .gap(6.0)
            .align_cross(FlowAlignment::Center)
            .entity();
        tree.in_parent(row, |tree| {
            handles.library_add_folder = button(tree, "Add folder", 96.0);
            handles.library_add_files = button(tree, "Add files", 90.0);
            handles.library_save = button(tree, "Save glTF", 90.0);
        });
        handles.library_chips = tree
            .add_node()
            .fill_width()
            .auto_height()
            .flow_horizontal()
            .flow_wrap()
            .padding(0.0)
            .gap(6.0)
            .entity();
        tree.add_node()
            .fill_width()
            .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 1.0)))
            .rect(0.0)
            .fg(ThemeColor::Border);
    });
}

fn build_add_menu(tree: &mut UiTreeBuilder, handles: &mut UiHandles) {
    let backdrop = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .rect(0.0)
        .color_raw::<UiBase>(vec4(0.0, 0.0, 0.0, 0.6))
        .with_interaction()
        .with_depth(UiDepthMode::Set(50.0))
        .with_visible(false)
        .entity();
    handles.add_panel = backdrop;

    const PRIMITIVES: [&str; 6] = ["Cube", "Sphere", "Cylinder", "Cone", "Torus", "Plane"];
    const LIGHTS: [&str; 3] = ["Directional", "Point", "Spot"];

    tree.in_parent(backdrop, |tree| {
        let panel = tree
            .add_node()
            .window(Rl(vec2(50.0, 50.0)), Ab(vec2(380.0, 300.0)), Anchor::Center)
            .card()
            .with_interaction()
            .with_clip()
            .entity();
        tree.in_parent(panel, |tree| {
            let body = tree
                .add_node()
                .boundary(
                    Ab(vec2(12.0, 12.0)),
                    Rl(vec2(100.0, 100.0)) + Ab(vec2(-12.0, -12.0)),
                )
                .flow_vertical()
                .padding(0.0)
                .gap(8.0)
                .entity();
            tree.in_parent(body, |tree| {
                tree.add_node()
                    .fill_width()
                    .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 22.0)))
                    .with_text("Add", 16.0)
                    .text_left()
                    .fg(ThemeColor::Text);
                tree.add_node()
                    .fill_width()
                    .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 16.0)))
                    .with_text("Geometry", 11.0)
                    .text_left()
                    .fg(ThemeColor::TextDisabled);
                let geometry = tree
                    .add_node()
                    .fill_width()
                    .auto_height()
                    .flow_horizontal()
                    .flow_wrap()
                    .padding(0.0)
                    .gap(6.0)
                    .entity();
                tree.in_parent(geometry, |tree| {
                    for (index, name) in PRIMITIVES.iter().enumerate() {
                        handles.primitive_buttons[index] = button(tree, name, 108.0);
                    }
                });
                tree.add_node()
                    .fill_width()
                    .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 16.0)))
                    .with_text("Lights", 11.0)
                    .text_left()
                    .fg(ThemeColor::TextDisabled);
                let lights = tree
                    .add_node()
                    .fill_width()
                    .auto_height()
                    .flow_horizontal()
                    .flow_wrap()
                    .padding(0.0)
                    .gap(6.0)
                    .entity();
                tree.in_parent(lights, |tree| {
                    for (index, name) in LIGHTS.iter().enumerate() {
                        handles.light_buttons[index] = button(tree, name, 108.0);
                    }
                });
                tree.add_node().fill_width().flow_child(Ab(vec2(0.0, 8.0)));
                let footer = tree
                    .add_node()
                    .fill_width()
                    .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 28.0)))
                    .flow_horizontal()
                    .padding(0.0)
                    .gap(6.0)
                    .align_main(FlowAlignment::End)
                    .entity();
                tree.in_parent(footer, |tree| {
                    handles.clips_button = button(tree, "Take clips", 96.0);
                    handles.add_import = button(tree, "Import file", 108.0);
                    handles.add_close = button(tree, "Close", 68.0);
                });
            });
        });
    });
}

fn build_recent(tree: &mut UiTreeBuilder, handles: &mut UiHandles) {
    let backdrop = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .rect(0.0)
        .color_raw::<UiBase>(vec4(0.0, 0.0, 0.0, 0.6))
        .with_interaction()
        .with_depth(UiDepthMode::Set(50.0))
        .with_visible(false)
        .entity();
    handles.recent_panel = backdrop;

    tree.in_parent(backdrop, |tree| {
        let panel = tree
            .add_node()
            .window(Rl(vec2(50.0, 50.0)), Ab(vec2(520.0, 420.0)), Anchor::Center)
            .card()
            .with_interaction()
            .with_clip()
            .entity();
        tree.in_parent(panel, |tree| {
            let body = tree
                .add_node()
                .boundary(
                    Ab(vec2(12.0, 12.0)),
                    Rl(vec2(100.0, 100.0)) + Ab(vec2(-12.0, -12.0)),
                )
                .flow_vertical()
                .padding(0.0)
                .gap(8.0)
                .entity();
            tree.in_parent(body, |tree| {
                tree.add_node()
                    .fill_width()
                    .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 22.0)))
                    .with_text("Recent", 16.0)
                    .text_left()
                    .fg(ThemeColor::Text);
                handles.recent_container = tree
                    .add_node()
                    .fill_width()
                    .flow_child(Ab(vec2(0.0, 0.0)))
                    .flex_grow(1.0)
                    .flow_vertical()
                    .padding(0.0)
                    .gap(4.0)
                    .entity();
                let footer = tree
                    .add_node()
                    .fill_width()
                    .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 28.0)))
                    .flow_horizontal()
                    .padding(0.0)
                    .gap(6.0)
                    .align_main(FlowAlignment::End)
                    .entity();
                tree.in_parent(footer, |tree| {
                    handles.recent_close = button(tree, "Close", 68.0);
                });
            });
        });
    });
}

fn build_notice(tree: &mut UiTreeBuilder, handles: &mut UiHandles) {
    let dialog = tree.add_modal_dialog("Notice", 420.0, 200.0);
    handles.notice_dialog = dialog;
    handles.notice_backdrop = tree
        .component::<UiModalDialogData>(dialog)
        .map(|data| data.backdrop_entity)
        .unwrap_or(dialog);
    let content = tree
        .component::<UiModalDialogData>(dialog)
        .map(|data| data.content_entity)
        .unwrap_or(dialog);

    let mut close = Entity::default();
    tree.in_parent(content, |tree| {
        handles.notice_text = tree
            .add_node()
            .fill_width()
            .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 96.0)))
            .with_text("", 12.0)
            .with_text_wrap()
            .with_text_alignment(TextAlignment::Left, VerticalAlignment::Top)
            .fg(ThemeColor::Text)
            .entity();
        let footer = tree
            .add_node()
            .fill_width()
            .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 30.0)))
            .flow_horizontal()
            .padding(0.0)
            .gap(6.0)
            .align_main(FlowAlignment::End)
            .entity();
        tree.in_parent(footer, |tree| {
            close = button(tree, "Close", 76.0);
        });
    });
    tree.edit::<UiModalDialogData>(dialog, move |data| data.ok_button = Some(close));
}

fn build_loading_label(tree: &mut UiTreeBuilder, handles: &mut UiHandles, layout: Layout) {
    handles.loading_label = tree
        .add_node()
        .window(
            Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, layout.top_bar_height + 10.0)),
            Ab(vec2(320.0, 26.0)),
            Anchor::TopCenter,
        )
        .card()
        .with_text("", 12.0)
        .with_depth(UiDepthMode::Set(30.0))
        .with_visible(false)
        .entity();
}
