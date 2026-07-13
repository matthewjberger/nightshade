/// Flushes the pending WebGPU commands with an empty submit, then yields to
/// the JavaScript event loop so the browser processes them as a small batch.
/// Chrome executes each flushed batch as one uninterruptible task on the GPU
/// process main thread, where pipeline compilation runs inline and the
/// compositor also lives; without these breaks the renderer's startup
/// compiles arrive as multi-second batches that freeze the whole browser.
/// A no-op outside wasm.
#[cfg(all(target_arch = "wasm32", feature = "browser_yield"))]
pub async fn yield_to_event_loop(queue: &wgpu::Queue) {
    use wasm_bindgen::JsCast;

    queue.submit(std::iter::empty());
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        let global = js_sys::global();
        let scheduled =
            js_sys::Reflect::get(&global, &wasm_bindgen::JsValue::from_str("setTimeout"))
                .ok()
                .and_then(|set_timeout| set_timeout.dyn_into::<js_sys::Function>().ok())
                .and_then(|set_timeout| {
                    set_timeout
                        .call2(&global, &resolve, &wasm_bindgen::JsValue::from_f64(0.0))
                        .ok()
                })
                .is_some();
        if !scheduled {
            let _ = resolve.call0(&wasm_bindgen::JsValue::NULL);
        }
    });
    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
}

#[cfg(not(all(target_arch = "wasm32", feature = "browser_yield")))]
pub async fn yield_to_event_loop(_queue: &wgpu::Queue) {}

/// Waits until the GPU process has executed every command queued so far,
/// including the inline pipeline compiles, by chasing an empty submit with
/// `on_submitted_work_done`. Running the first frame before the creation
/// batches drain merges everything into one giant browser-freezing task; this
/// lets the renderer hand control back only once startup is digested.
/// A no-op outside wasm.
#[cfg(all(target_arch = "wasm32", feature = "browser_yield"))]
pub async fn drain_queued_work(queue: &wgpu::Queue) {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    queue.submit(std::iter::empty());
    let done = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&done);
    queue.on_submitted_work_done(move || {
        flag.store(true, Ordering::Release);
    });
    while !done.load(Ordering::Acquire) {
        yield_to_event_loop(queue).await;
    }
}

#[cfg(not(all(target_arch = "wasm32", feature = "browser_yield")))]
pub async fn drain_queued_work(_queue: &wgpu::Queue) {}
