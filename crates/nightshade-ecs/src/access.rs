use std::any::TypeId;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Access {
    Read,
    Write,
}

impl Access {
    fn conflicts(self, other: Access) -> bool {
        self == Access::Write || other == Access::Write
    }
}

#[derive(Clone, Debug)]
pub struct SystemAccess {
    components: Vec<(TypeId, Access)>,
    resources: Vec<(TypeId, Access)>,
    events: Vec<(TypeId, Access)>,
    exclusive: bool,
    components_exclusive: bool,
    events_exclusive: bool,
    main_thread: bool,
    entity_scope: Option<EntityScope>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct EntityScope {
    requires: Vec<TypeId>,
    excludes: Vec<TypeId>,
}

impl Default for SystemAccess {
    fn default() -> Self {
        Self {
            components: Vec::new(),
            resources: Vec::new(),
            events: Vec::new(),
            exclusive: false,
            components_exclusive: false,
            events_exclusive: false,
            main_thread: false,
            entity_scope: Some(EntityScope::default()),
        }
    }
}

impl SystemAccess {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn exclusive() -> Self {
        Self {
            exclusive: true,
            ..Self::default()
        }
    }

    pub fn component_exclusive() -> Self {
        Self {
            components_exclusive: true,
            entity_scope: None,
            ..Self::default()
        }
    }

    pub fn is_exclusive(&self) -> bool {
        self.exclusive
    }

    pub fn main_thread(mut self) -> Self {
        self.main_thread = true;
        self
    }

    pub fn is_main_thread(&self) -> bool {
        self.main_thread
    }

    pub fn is_component_exclusive(&self) -> bool {
        self.components_exclusive
    }

    pub fn event_exclusive() -> Self {
        Self {
            events_exclusive: true,
            ..Self::default()
        }
    }

    pub fn is_event_exclusive(&self) -> bool {
        self.events_exclusive
    }

    pub fn components(&self) -> &[(TypeId, Access)] {
        &self.components
    }

    pub fn resources(&self) -> &[(TypeId, Access)] {
        &self.resources
    }

    pub fn events(&self) -> &[(TypeId, Access)] {
        &self.events
    }

    pub fn with_component(mut self, type_id: TypeId, access: Access) -> Self {
        insert(&mut self.components, type_id, access);
        self
    }

    pub fn with_resource(mut self, type_id: TypeId, access: Access) -> Self {
        insert(&mut self.resources, type_id, access);
        self
    }

    pub fn with_event(mut self, type_id: TypeId, access: Access) -> Self {
        insert(&mut self.events, type_id, access);
        self
    }

    pub fn requiring(mut self, type_id: TypeId) -> Self {
        if let Some(scope) = self.entity_scope.as_mut()
            && let Err(index) = scope.requires.binary_search(&type_id)
        {
            scope.requires.insert(index, type_id);
        }
        self
    }

    pub fn excluding(mut self, type_id: TypeId) -> Self {
        if let Some(scope) = self.entity_scope.as_mut()
            && let Err(index) = scope.excludes.binary_search(&type_id)
        {
            scope.excludes.insert(index, type_id);
        }
        self
    }

    pub fn merge(mut self, other: Self) -> Self {
        self.exclusive |= other.exclusive;
        self.components_exclusive |= other.components_exclusive;
        self.events_exclusive |= other.events_exclusive;
        self.main_thread |= other.main_thread;
        self.entity_scope = if self.components_exclusive {
            None
        } else if self.components.is_empty() {
            other.entity_scope.clone()
        } else if other.components.is_empty() || self.entity_scope == other.entity_scope {
            self.entity_scope.take()
        } else {
            None
        };
        for (type_id, access) in other.components {
            insert(&mut self.components, type_id, access);
        }
        for (type_id, access) in other.resources {
            insert(&mut self.resources, type_id, access);
        }
        for (type_id, access) in other.events {
            insert(&mut self.events, type_id, access);
        }
        self
    }

    pub fn conflicts_with(&self, other: &Self) -> bool {
        if self.exclusive || other.exclusive {
            return true;
        }
        if overlaps(&self.events, &other.events) || overlaps(&self.resources, &other.resources) {
            return true;
        }
        if (self.events_exclusive && other.touches_a_channel())
            || (other.events_exclusive && self.touches_a_channel())
        {
            return true;
        }
        if self.components_exclusive || other.components_exclusive {
            return self.touches_a_component() && other.touches_a_component();
        }
        if self.visits_no_entity_in_common(other) {
            return false;
        }
        overlaps(&self.components, &other.components)
    }

    fn touches_a_component(&self) -> bool {
        self.components_exclusive || !self.components.is_empty()
    }

    fn touches_a_channel(&self) -> bool {
        self.events_exclusive || !self.events.is_empty()
    }

    fn visits_no_entity_in_common(&self, other: &Self) -> bool {
        let (Some(mine), Some(theirs)) = (&self.entity_scope, &other.entity_scope) else {
            return false;
        };
        mine.requires
            .iter()
            .any(|type_id| theirs.excludes.binary_search(type_id).is_ok())
            || theirs
                .requires
                .iter()
                .any(|type_id| mine.excludes.binary_search(type_id).is_ok())
    }
}

fn insert(list: &mut Vec<(TypeId, Access)>, type_id: TypeId, access: Access) {
    match list.binary_search_by_key(&type_id, |(existing, _)| *existing) {
        Ok(index) => {
            if access == Access::Write {
                list[index].1 = Access::Write;
            }
        }
        Err(index) => list.insert(index, (type_id, access)),
    }
}

fn overlaps(left: &[(TypeId, Access)], right: &[(TypeId, Access)]) -> bool {
    let (mut left_index, mut right_index) = (0, 0);
    while left_index < left.len() && right_index < right.len() {
        let (left_type, left_access) = left[left_index];
        let (right_type, right_access) = right[right_index];
        match left_type.cmp(&right_type) {
            std::cmp::Ordering::Less => left_index += 1,
            std::cmp::Ordering::Greater => right_index += 1,
            std::cmp::Ordering::Equal => {
                if left_access.conflicts(right_access) {
                    return true;
                }
                left_index += 1;
                right_index += 1;
            }
        }
    }
    false
}
