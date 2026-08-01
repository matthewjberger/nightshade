use serde_json::json;

use crate::commands::agent::{answer_call, take_pending};
use crate::resources::AgentChannel;

pub fn serve(
    channel: nightshade::prelude::Res<AgentChannel>,
    mut stage: crate::resources::Stage,
    storage: &mut nightshade::ecs::Storage,
) {
    let channel = channel.clone();
    for pending in take_pending(&channel) {
        let answer = answer_call(&mut stage, storage, &pending.name, pending.arguments);
        let value = match answer {
            Ok(value) => value,
            Err(message) => json!({ "error": message }),
        };
        if pending.reply.send(value).is_err() {
            continue;
        }
    }
}
