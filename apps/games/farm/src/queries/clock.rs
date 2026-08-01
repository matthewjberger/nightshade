use crate::theme;

pub fn is_waking_hour(hour: f32) -> bool {
    (theme::WAKE_HOUR..theme::BED_HOUR).contains(&hour)
}

pub fn is_bed_hour(hour: f32) -> bool {
    hour >= theme::SLEEP_FROM || hour < theme::SLEEP_UNTIL
}

pub fn clock_text(hour: f32) -> String {
    let whole = hour.floor();
    let minutes = ((hour - whole) * 60.0).floor() as u32;
    format!("{:02}:{:02}", whole as u32 % 24, minutes)
}
