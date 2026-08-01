pub fn stage_of(progress: f32) -> usize {
    if progress >= 1.0 {
        3
    } else {
        ((progress * 3.0) as usize).min(2)
    }
}
