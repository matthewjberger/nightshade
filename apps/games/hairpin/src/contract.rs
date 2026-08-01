#[derive(Clone, Debug, PartialEq)]
pub enum HairpinEvent {
    LapFinished { lap: u32, seconds: f32, best: bool },
}
