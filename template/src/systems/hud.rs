use crate::resources::{HudHandles, TemplateResources};
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_render::resources::viewport::{Viewport, ViewportExtent};
use nightshade_ui::components::layout_types::FlowDirection;
use nightshade_ui::data::icons::material;
use nightshade_ui::prelude::*;
use nightshade_ui::resources::RetainedUiFrame;

const RATE_BANDS: [(f32, Vec4); 3] = [
    (50.0, Vec4::new(0.45, 0.92, 0.50, 1.0)),
    (25.0, Vec4::new(1.0, 0.68, 0.25, 1.0)),
    (0.0, Vec4::new(1.0, 0.36, 0.36, 1.0)),
];

pub(crate) const PANEL_BACKGROUND: Vec4 = Vec4::new(0.05, 0.05, 0.08, 0.78);
pub(crate) const PANEL_BORDER: Vec4 = Vec4::new(1.0, 1.0, 1.0, 0.12);
pub(crate) const TEXT_DIM: Vec4 = Vec4::new(0.78, 0.80, 0.86, 1.0);

const RATE_WIDTH: f32 = 232.0;
const READOUT_WIDTH: f32 = 620.0;
const RATE_HEIGHT: f32 = 46.0;
const RATE_TEXT_SIZE: f32 = 24.0;
const MENU_BUTTON_WIDTH: f32 = 52.0;
const MENU_BUTTON_HEIGHT: f32 = 34.0;
const MENU_WIDTH: f32 = 340.0;
const MENU_HEIGHT: f32 = 104.0;
pub(crate) const MARGIN: f32 = 16.0;

pub fn build(tree: &mut UiTreeBuilder) -> HudHandles {
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .entity();

    let mut handles = HudHandles::default();

    tree.in_parent(root, |tree| {
        let panel = tree
            .add_node()
            .window(
                Ab(vec2(MARGIN, MARGIN)),
                Ab(vec2(RATE_WIDTH, RATE_HEIGHT)),
                Anchor::TopLeft,
            )
            .with_rect(8.0, 1.0, PANEL_BORDER)
            .color_raw::<UiBase>(PANEL_BACKGROUND)
            .entity();

        tree.in_parent(panel, |tree| {
            handles.rate_readout = tree
                .add_node()
                .window(
                    Ab(vec2(12.0, 8.0)),
                    Ab(vec2(RATE_WIDTH - 24.0, 30.0)),
                    Anchor::TopLeft,
                )
                .with_text("", RATE_TEXT_SIZE)
                .text_left()
                .color_raw::<UiBase>(RATE_BANDS[0].1)
                .entity();
        });

        let bottom_panel = tree
            .add_node()
            .window(
                Rl(vec2(0.0, 100.0)) + Ab(vec2(MARGIN, -MARGIN)),
                Ab(vec2(READOUT_WIDTH, RATE_HEIGHT)),
                Anchor::BottomLeft,
            )
            .with_rect(8.0, 1.0, PANEL_BORDER)
            .color_raw::<UiBase>(PANEL_BACKGROUND)
            .entity();

        tree.in_parent(bottom_panel, |tree| {
            handles.viewport_readout = tree
                .add_node()
                .window(
                    Ab(vec2(12.0, 8.0)),
                    Ab(vec2(READOUT_WIDTH - 24.0, 30.0)),
                    Anchor::TopLeft,
                )
                .with_text("", 18.0)
                .text_left()
                .color_raw::<UiBase>(TEXT_DIM)
                .entity();
        });

        let menu_holder = tree
            .add_node()
            .window(
                Rl(vec2(100.0, 0.0)) + Ab(vec2(-MARGIN, MARGIN)),
                Ab(vec2(MENU_BUTTON_WIDTH, MENU_BUTTON_HEIGHT)),
                Anchor::TopRight,
            )
            .flow(FlowDirection::Vertical, 0.0, 0.0)
            .entity();

        tree.in_parent(menu_holder, |tree| {
            handles.menu_button = tree.add_icon_button(material::MENU, 20.0);
        });

        handles.menu_panel = tree
            .add_node()
            .window(
                Rl(vec2(100.0, 0.0)) + Ab(vec2(-MARGIN, MARGIN + MENU_BUTTON_HEIGHT + 8.0)),
                Ab(vec2(MENU_WIDTH, MENU_HEIGHT)),
                Anchor::TopRight,
            )
            .with_rect(8.0, 1.0, PANEL_BORDER)
            .color_raw::<UiBase>(PANEL_BACKGROUND)
            .flow(FlowDirection::Vertical, 14.0, 10.0)
            .with_visible(false)
            .entity();

        tree.in_parent(handles.menu_panel, |tree| {
            tree.add_node()
                .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 18.0)))
                .with_text("DISPLAY", 12.0)
                .text_left()
                .color_raw::<UiBase>(TEXT_DIM);

            handles.extent_button = tree.add_button(&extent_label(ViewportExtent::default()));
        });
    });

    handles.root = root;
    handles
}

pub fn show_fps(
    mut resources: ResMut<TemplateResources>,
    state: Res<nightshade_render::data::config::RendererState>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let readout = resources.hud.rate_readout;
    if readout == Entity::default() {
        return;
    }

    let rate = time.frames_per_second.round().max(0.0) as u32;
    let gpu_tenths = (state.gpu_frame_time_ms * 10.0).round().max(0.0) as u32;
    if rate == resources.last_rate && gpu_tenths == resources.last_gpu_tenths {
        return;
    }
    resources.last_rate = rate;
    resources.last_gpu_tenths = gpu_tenths;

    let (_, color) = RATE_BANDS
        .iter()
        .find(|(floor, _)| rate as f32 >= *floor)
        .copied()
        .unwrap_or(RATE_BANDS[RATE_BANDS.len() - 1]);

    let gpu = gpu_tenths as f32 / 10.0;
    ui_set_text(&mut commands, readout, &format!("{rate} fps  {gpu:.1} ms"));
    ui_set_color::<UiBase>(&mut commands, readout, color);
}

pub fn show_viewport(
    mut resources: ResMut<TemplateResources>,
    viewport: Res<Viewport>,
    window: Res<Window>,
    mut commands: Commands,
) {
    let readout = resources.hud.viewport_readout;
    if readout == Entity::default() {
        return;
    }

    let rect = viewport.active_viewport_rect;
    let bars = window.content_rect;
    if rect == resources.last_viewport_rect && bars == resources.last_content_rect {
        return;
    }
    resources.last_viewport_rect = rect;
    resources.last_content_rect = bars;

    let picture = match rect {
        Some(rect) => format!(
            "{} {} {} {}",
            rect.x.round() as i32,
            rect.y.round() as i32,
            rect.width.round() as i32,
            rect.height.round() as i32,
        ),
        None => "whole window".to_string(),
    };
    let interface = match bars {
        Some(rect) => format!(
            "{} {} {} {}",
            rect.x.round() as i32,
            rect.y.round() as i32,
            rect.width.round() as i32,
            rect.height.round() as i32,
        ),
        None => "whole window".to_string(),
    };
    ui_set_text(
        &mut commands,
        readout,
        &format!("picture {picture}   interface {interface}"),
    );
}

pub fn drive_menu(
    mut resources: ResMut<TemplateResources>,
    mut dirty: ResMut<nightshade_ui::resources::RetainedUiDirty>,
    mut viewport: ResMut<Viewport>,
    frame: Res<RetainedUiFrame>,
    mut commands: Commands,
) {
    let handles = resources.hud;
    if handles.root == Entity::default() {
        return;
    }

    let mut menu_toggled = false;
    let mut extent_stepped = false;
    for entity in ui_button_clicks(&frame) {
        if entity == handles.menu_button {
            menu_toggled = true;
        } else if entity == handles.extent_button {
            extent_stepped = true;
        }
    }

    if menu_toggled {
        resources.menu_open = !resources.menu_open;
        ui_set_visible(
            &mut dirty,
            &mut commands,
            handles.menu_panel,
            resources.menu_open,
        );
    }

    if extent_stepped {
        resources.extent = match resources.extent {
            ViewportExtent::Unobstructed => ViewportExtent::WindowContent,
            ViewportExtent::WindowContent => ViewportExtent::FullWindow,
            ViewportExtent::FullWindow => ViewportExtent::Unobstructed,
        };
        ui_set_text(
            &mut commands,
            handles.extent_button,
            &extent_label(resources.extent),
        );
        viewport.extent = resources.extent;
    }
}

fn extent_label(extent: ViewportExtent) -> String {
    let name = match extent {
        ViewportExtent::FullWindow => "the whole window",
        ViewportExtent::Unobstructed => "clear of the camera",
        ViewportExtent::WindowContent => "clear of the bars",
    };
    format!("Viewport   {name}")
}

pub fn menu_style() -> nightshade_ui_menu::prelude::MenuStyle {
    nightshade_ui_menu::prelude::MenuStyle {
        background: PANEL_BACKGROUND,
        border: PANEL_BORDER,
        row_color: TEXT_DIM,
        padding: MARGIN,
        ..Default::default()
    }
}
