use nightshade::ecs::Entity;
use std::marker::PhantomData;

pub struct ConfigPanel<S> {
    pub rows: Vec<Entity>,
    pub reset: Option<Entity>,
    schema: PhantomData<fn() -> S>,
}

impl<S> Default for ConfigPanel<S> {
    fn default() -> Self {
        Self {
            rows: Vec::new(),
            reset: None,
            schema: PhantomData,
        }
    }
}
