use nightshade_render::data::config::SkinnedChannelData;
use std::collections::HashMap;

#[derive(Default)]
pub struct ChannelCache(pub HashMap<u64, SkinnedChannelData>);

#[derive(Default)]
pub struct FrozenJoints(pub Vec<nightshade::ecs::Entity>);
