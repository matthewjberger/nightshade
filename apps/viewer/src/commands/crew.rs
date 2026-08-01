use crate::resources::Crew;

#[cfg(not(target_arch = "wasm32"))]
pub fn hire<Work>(crew: &mut Crew, work: Work)
where
    Work: FnOnce() + Send + 'static,
{
    if let Ok(mut hands) = crew.hands.lock() {
        hands.push(std::thread::spawn(work));
    }
}

#[cfg(target_arch = "wasm32")]
pub fn hire<Work>(crew: &mut Crew, work: Work)
where
    Work: FnOnce() + Send + 'static,
{
    if let Ok(mut hands) = crew.hands.lock() {
        hands.clear();
    }
    work();
}

#[cfg(not(target_arch = "wasm32"))]
pub fn hold_the_agent(crew: &mut Crew, server: std::sync::Arc<tiny_http::Server>) {
    if let Ok(mut slot) = crew.agent.lock() {
        *slot = Some(server);
    }
}

pub fn wait_for_the_crew(crew: &mut Crew) {
    #[cfg(not(target_arch = "wasm32"))]
    if let Ok(mut slot) = crew.agent.lock()
        && let Some(server) = slot.take()
    {
        server.unblock();
    }
    let waiting = crew
        .hands
        .lock()
        .map(|mut hands| std::mem::take(&mut *hands))
        .unwrap_or_default();
    for hand in waiting {
        hand.join().ok();
    }
}
