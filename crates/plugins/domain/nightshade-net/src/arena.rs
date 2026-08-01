#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PeerId {
    index: u32,
    generation: u32,
}

struct Slot<Value> {
    generation: u32,
    value: Option<Value>,
}

pub(crate) struct Arena<Value> {
    slots: Vec<Slot<Value>>,
    free: Vec<u32>,
}

impl<Value> Default for Arena<Value> {
    fn default() -> Self {
        Self {
            slots: Vec::new(),
            free: Vec::new(),
        }
    }
}

impl<Value> Arena<Value> {
    pub(crate) fn insert(&mut self, value: Value) -> PeerId {
        if let Some(index) = self.free.pop() {
            let slot = &mut self.slots[index as usize];
            slot.value = Some(value);
            return PeerId {
                index,
                generation: slot.generation,
            };
        }
        let index = self.slots.len() as u32;
        self.slots.push(Slot {
            generation: 0,
            value: Some(value),
        });
        PeerId {
            index,
            generation: 0,
        }
    }

    pub(crate) fn get(&self, id: PeerId) -> Option<&Value> {
        let slot = self.slots.get(id.index as usize)?;
        if slot.generation != id.generation {
            return None;
        }
        slot.value.as_ref()
    }

    pub(crate) fn remove(&mut self, id: PeerId) -> Option<Value> {
        let slot = self.slots.get_mut(id.index as usize)?;
        if slot.generation != id.generation || slot.value.is_none() {
            return None;
        }
        slot.generation = slot.generation.wrapping_add(1);
        self.free.push(id.index);
        slot.value.take()
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (PeerId, &Value)> {
        self.slots.iter().enumerate().filter_map(|(index, slot)| {
            slot.value.as_ref().map(|value| {
                (
                    PeerId {
                        index: index as u32,
                        generation: slot.generation,
                    },
                    value,
                )
            })
        })
    }
}
