use nightshade::ecs::Entity;

#[derive(Default, Clone, Debug)]
pub struct MenuHandles {
    pub panel: Entity,
    pub heading: Entity,
    pub rows: Vec<Entity>,
    pub buttons: Vec<Entity>,
}

pub struct Menus<S> {
    pub panels: Vec<(S, Entity)>,
    pub shown: Option<S>,
}

impl<S> Default for Menus<S> {
    fn default() -> Self {
        Self {
            panels: Vec::new(),
            shown: None,
        }
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct SlotHandles {
    pub frame: Entity,
    pub icon: Entity,
    pub count: Entity,
}
