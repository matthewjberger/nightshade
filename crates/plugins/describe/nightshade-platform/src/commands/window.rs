use crate::contract::window::Window;

pub fn set_cursor_locked(window: &mut Window, locked: bool) {
    window.cursor_locked = locked;
}

#[cfg(target_arch = "wasm32")]
pub fn sync_cursor_lock_state(window: &mut Window) {
    window.cursor_locked = web_sys::window()
        .and_then(|browser_window| browser_window.document())
        .and_then(|document| document.pointer_lock_element())
        .is_some();
}

pub fn set_cursor_visible(window: &mut Window, visible: bool) {
    window.show_cursor = visible;
}

pub fn set_pointer_grab(window: &mut Window, grabbed: bool) {
    set_cursor_locked(window, grabbed);
    set_cursor_visible(window, !grabbed);
}
