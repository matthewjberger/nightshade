use crate::data::systems::SystemFn;

#[derive(Default)]
pub struct RetainedUiSchedule {
    pub systems: std::sync::Mutex<Vec<SystemFn>>,
}
