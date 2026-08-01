use nightshade::prelude::App;
use nightshade_platform::prelude::Time;
use std::time::{Duration, Instant};

pub fn keep_serving(app: App) -> Result<(), Box<dyn std::error::Error>> {
    let (mut world, mut startup, mut stages) = app.into_schedules();
    startup.run(&mut world);
    let step = Duration::from_secs_f32(crate::theme::TICK_SECONDS);
    let mut last = Instant::now();
    loop {
        let now = Instant::now();
        let delta = now.duration_since(last).as_secs_f32();
        last = now;
        let time = world.res_mut::<Time>();
        time.raw_delta_time = delta;
        time.delta_time = delta;
        time.uptime_seconds += delta as f64;
        time.uptime_milliseconds = (time.uptime_seconds * 1000.0) as u64;
        stages.run(&mut world);
        let spent = Instant::now().duration_since(now);
        if let Some(rest) = step.checked_sub(spent) {
            std::thread::sleep(rest);
        }
    }
}
