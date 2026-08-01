use nightshade_platform::contract::window::Window;
#[cfg(target_os = "android")]
use nightshade_platform::contract::window::WindowRect;

#[cfg(target_os = "android")]
fn rect_from_edges(
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
    width: i32,
    height: i32,
) -> WindowRect {
    let left = left.clamp(0, width);
    let top = top.clamp(0, height);
    let right = right.clamp(left, width);
    let bottom = bottom.clamp(top, height);
    WindowRect {
        x: left as f32,
        y: top as f32,
        width: (right - left) as f32,
        height: (bottom - top) as f32,
    }
}

pub fn refresh_content_rect(window: &mut Window) {
    #[cfg(target_os = "android")]
    {
        let Some(handle) = window.handle.as_ref() else {
            return;
        };

        let size = handle.inner_size();
        if size.width == 0 || size.height == 0 {
            return;
        }
        let width = size.width as i32;
        let height = size.height as i32;

        let insets = super::display_area::display_insets();
        let [hole_left, hole_top, hole_right, hole_bottom] = insets.cutout;
        let [bar_left, bar_top, bar_right, bar_bottom] = insets.bars;

        window.unobstructed_rect = Some(rect_from_edges(
            hole_left,
            hole_top,
            width - hole_right,
            height - hole_bottom,
            width,
            height,
        ));
        window.content_rect = Some(rect_from_edges(
            hole_left.max(bar_left),
            hole_top.max(bar_top),
            width - hole_right.max(bar_right),
            height - hole_bottom.max(bar_bottom),
            width,
            height,
        ));
    }

    #[cfg(not(target_os = "android"))]
    {
        window.content_rect = None;
        window.unobstructed_rect = None;
    }
}
