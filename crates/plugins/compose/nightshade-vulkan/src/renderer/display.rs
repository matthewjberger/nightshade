#[derive(Clone, Copy, Debug)]
pub struct DisplayLight {
    pub peak_nits: f32,
    pub floor_nits: f32,
    pub frame_nits: f32,
}

#[cfg(windows)]
pub fn display_light(window: &impl raw_window_handle::HasWindowHandle) -> Option<DisplayLight> {
    use raw_window_handle::RawWindowHandle;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory1, IDXGIFactory1, IDXGIOutput6};
    use windows::Win32::Graphics::Gdi::{MONITOR_DEFAULTTONEAREST, MonitorFromWindow};
    use windows::core::Interface;

    let RawWindowHandle::Win32(handle) = window.window_handle().ok()?.as_raw() else {
        return None;
    };
    let hwnd = HWND(handle.hwnd.get() as *mut std::ffi::c_void);
    let monitor = unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) };
    let factory: IDXGIFactory1 = unsafe { CreateDXGIFactory1() }.ok()?;
    let mut adapter_index = 0;
    while let Ok(adapter) = unsafe { factory.EnumAdapters1(adapter_index) } {
        adapter_index += 1;
        let mut output_index = 0;
        while let Ok(output) = unsafe { adapter.EnumOutputs(output_index) } {
            output_index += 1;
            let Ok(output) = output.cast::<IDXGIOutput6>() else {
                continue;
            };
            let Ok(description) = (unsafe { output.GetDesc1() }) else {
                continue;
            };
            if description.Monitor != monitor {
                continue;
            }
            return Some(DisplayLight {
                peak_nits: description.MaxLuminance,
                floor_nits: description.MinLuminance,
                frame_nits: description.MaxFullFrameLuminance,
            });
        }
    }
    None
}

#[cfg(not(windows))]
pub fn display_light(window: &impl raw_window_handle::HasWindowHandle) -> Option<DisplayLight> {
    window.window_handle().ok().and(None)
}
