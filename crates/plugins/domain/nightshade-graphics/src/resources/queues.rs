use crate::data::queues::{EcsCommand, RenderCommand};

#[derive(Default)]
pub struct CommandQueues {
    pub ecs: Vec<EcsCommand>,
    pub render: Vec<RenderCommand>,
}
