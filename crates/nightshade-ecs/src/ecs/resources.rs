use super::*;

#[derive(Default)]
pub struct EventBus {
    slots: Vec<EventSlot>,
    by_type: HashMap<TypeId, usize>,
}

impl EventBus {
    fn slot_index<T: Send + Sync + 'static>(&mut self) -> usize {
        if let Some(&index) = self.by_type.get(&TypeId::of::<T>()) {
            return index;
        }
        let index = self.slots.len();
        self.slots.push(EventSlot::holding::<T>());
        self.by_type.insert(TypeId::of::<T>(), index);
        index
    }

    fn slot<T: Send + Sync + 'static>(&self) -> Option<&EventSlot> {
        let index = *self.by_type.get(&TypeId::of::<T>())?;
        Some(&self.slots[index])
    }

    fn channel<T: Send + Sync + 'static>(&self) -> Option<&EventChannel<T>> {
        self.slot::<T>().map(EventSlot::read::<T>)
    }

    pub fn send<T: Send + Sync + 'static>(&mut self, event: T) {
        let index = self.slot_index::<T>();
        unsafe { &mut *self.slots[index].lend::<T>() }.send(event);
    }

    pub fn send_shared<T: Send + Sync + 'static>(&self, event: T) {
        let slot = self
            .slot::<T>()
            .expect("a shared send needs the channel to exist, so warm the writer first");
        unsafe { &mut *slot.lend::<T>() }.send(event);
    }

    pub fn ensure_channel<T: Send + Sync + 'static>(&mut self) {
        self.slot_index::<T>();
    }

    pub fn read<T: Send + Sync + 'static>(&self) -> &[T] {
        self.channel::<T>()
            .map(|channel| channel.events.as_slice())
            .unwrap_or(&[])
    }

    pub fn read_frame<T: Send + Sync + 'static>(&self) -> &[T] {
        self.channel::<T>()
            .map(|channel| channel.read_frame())
            .unwrap_or(&[])
    }

    pub fn consume<T: Send + Sync + 'static>(&self, cursor: &mut u64) -> &[T] {
        match self.channel::<T>() {
            Some(channel) => channel.consume(cursor),
            None => &[],
        }
    }

    pub fn clear<T: Send + Sync + 'static>(&mut self) {
        let index = self.slot_index::<T>();
        unsafe { &mut *self.slots[index].lend::<T>() }.clear();
    }

    pub fn channel_count(&self) -> usize {
        self.slots.len()
    }

    pub fn update(&self) {
        for slot in &self.slots {
            slot.advance();
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct ResourceId(usize);

#[derive(Clone, Copy, Debug)]
pub struct ResourceBorrow {
    pub id: ResourceId,
    pub write: bool,
}

pub enum ResourceSlot<'map> {
    Shared(&'map BoxedAny),
    Exclusive(&'map mut BoxedAny),
}

impl<'map> ResourceSlot<'map> {
    pub fn into_ref<T: Send + Sync + 'static>(self) -> &'map T {
        let value: &'map BoxedAny = match self {
            ResourceSlot::Shared(value) => value,
            ResourceSlot::Exclusive(value) => value,
        };
        value
            .downcast_ref::<T>()
            .expect("a resource slot holds its own type")
    }

    pub fn into_mut<T: Send + Sync + 'static>(self) -> &'map mut T {
        match self {
            ResourceSlot::Exclusive(value) => value
                .downcast_mut::<T>()
                .expect("a resource slot holds its own type"),
            ResourceSlot::Shared(_) => panic!(
                "resource {} was borrowed for reading and is written",
                std::any::type_name::<T>()
            ),
        }
    }
}

#[derive(Default)]
pub struct ResourceCell(std::cell::UnsafeCell<Option<BoxedAny>>);

unsafe impl Sync for ResourceCell {}

impl ResourceCell {
    fn held(value: Option<BoxedAny>) -> Self {
        Self(std::cell::UnsafeCell::new(value))
    }

    #[inline]
    fn read(&self) -> Option<&BoxedAny> {
        unsafe { (*self.0.get()).as_ref() }
    }

    #[inline]
    fn lend(&self) -> *mut Option<BoxedAny> {
        self.0.get()
    }

    #[inline]
    fn owned(&mut self) -> &mut Option<BoxedAny> {
        self.0.get_mut()
    }
}

#[derive(Default)]
pub struct ResourceMap {
    slots: Vec<ResourceCell>,
    ids: HashMap<TypeId, ResourceId>,
}

impl ResourceMap {
    fn id_for<T: Send + Sync + 'static>(&mut self) -> ResourceId {
        self.id_for_type(TypeId::of::<T>())
    }

    pub fn id_for_type(&mut self, type_id: TypeId) -> ResourceId {
        if let Some(&id) = self.ids.get(&type_id) {
            return id;
        }
        let id = ResourceId(self.slots.len());
        self.slots.push(ResourceCell::held(None));
        self.ids.insert(type_id, id);
        id
    }

    pub fn id_of_type(&self, type_id: &TypeId) -> Option<ResourceId> {
        self.ids.get(type_id).copied()
    }

    pub fn holds(&self, type_id: TypeId) -> bool {
        self.id_of_type(&type_id)
            .is_some_and(|id| self.slots[id.0].read().is_some())
    }

    pub fn insert<T: Send + Sync + 'static>(&mut self, value: T) {
        let id = self.id_for::<T>();
        *self.slots[id.0].owned() = Some(Box::new(value));
    }

    pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.slots[self.id_of_type(&TypeId::of::<T>())?.0]
            .read()
            .and_then(|value| value.downcast_ref::<T>())
    }

    pub fn get_mut<T: Send + Sync + 'static>(&mut self) -> Option<&mut T> {
        let id = self.id_of_type(&TypeId::of::<T>())?;
        self.slots[id.0]
            .owned()
            .as_mut()
            .and_then(|value| value.downcast_mut::<T>())
    }

    pub fn res<T: Send + Sync + 'static>(&self) -> &T {
        self.get::<T>().unwrap_or_else(|| {
            panic!(
                "resource {} is missing; compose the plugin that owns it",
                std::any::type_name::<T>()
            )
        })
    }

    pub fn res_mut<T: Send + Sync + 'static>(&mut self) -> &mut T {
        self.get_mut::<T>().unwrap_or_else(|| {
            panic!(
                "resource {} is missing; compose the plugin that owns it",
                std::any::type_name::<T>()
            )
        })
    }

    pub fn res_disjoint_mut<const N: usize>(
        &mut self,
        type_ids: [TypeId; N],
    ) -> [&mut BoxedAny; N] {
        let ids = type_ids.map(|type_id| {
            self.id_of_type(&type_id)
                .expect("a resource named here is missing")
        });
        let mut borrowed = self.disjoint_mut(&ids).into_iter();
        std::array::from_fn(|_| borrowed.next().expect("one borrow per id"))
    }

    pub fn disjoint_slots(
        &self,
        borrows: &[Option<ResourceBorrow>],
    ) -> Vec<Option<ResourceSlot<'_>>> {
        let mut seen: Vec<ResourceId> = Vec::new();
        for borrow in borrows.iter().flatten() {
            assert!(
                !seen.contains(&borrow.id),
                "a resource was named twice in one borrow"
            );
            seen.push(borrow.id);
        }

        borrows
            .iter()
            .map(|borrow| {
                borrow.map(|borrow| {
                    let cell = self
                        .slots
                        .get(borrow.id.0)
                        .expect("resource id out of range");
                    match borrow.write {
                        true => ResourceSlot::Exclusive(
                            unsafe { (*cell.lend()).as_mut() }
                                .expect("a resource named here is missing or lent out"),
                        ),
                        false => ResourceSlot::Shared(
                            cell.read()
                                .expect("a resource named here is missing or lent out"),
                        ),
                    }
                })
            })
            .collect()
    }

    pub fn disjoint_mut(&mut self, ids: &[ResourceId]) -> Vec<&mut BoxedAny> {
        let borrows: Vec<Option<ResourceBorrow>> = ids
            .iter()
            .map(|&id| Some(ResourceBorrow { id, write: true }))
            .collect();
        self.disjoint_slots(&borrows)
            .into_iter()
            .map(|slot| match slot {
                Some(ResourceSlot::Exclusive(value)) => value,
                _ => panic!("a resource named here is missing or lent out"),
            })
            .collect()
    }

    pub fn len(&self) -> usize {
        self.slots
            .iter()
            .filter(|slot| slot.read().is_some())
            .count()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn contains<T: Send + Sync + 'static>(&self) -> bool {
        self.holds(TypeId::of::<T>())
    }

    pub fn remove<T: Send + Sync + 'static>(&mut self) -> Option<T> {
        let id = self.id_of_type(&TypeId::of::<T>())?;
        self.slots[id.0]
            .owned()
            .take()?
            .downcast::<T>()
            .ok()
            .map(|value| *value)
    }

    pub fn take_boxed(&mut self, type_id: &TypeId) -> Option<BoxedAny> {
        let id = self.id_of_type(type_id)?;
        self.slots[id.0].owned().take()
    }

    pub fn restore_boxed(&mut self, type_id: &TypeId, value: BoxedAny) {
        match self.id_of_type(type_id) {
            Some(id) => *self.slots[id.0].owned() = Some(value),
            None => panic!("a resource was restored to a map that never held it"),
        }
    }

    pub fn restore<T: Send + Sync + 'static>(&mut self, value: T) {
        match self.id_of_type(&TypeId::of::<T>()) {
            Some(id) => *self.slots[id.0].owned() = Some(Box::new(value)),
            None => panic!(
                "resource {} was restored to a map that never held it",
                std::any::type_name::<T>()
            ),
        }
    }
}

pub trait ResourceHost {
    fn resource_map_mut(&mut self) -> &mut ResourceMap;

    fn resource_map(&self) -> &ResourceMap;
}

pub trait ResourceHostExt: ResourceHost + Sized {
    fn resource_scope<R: Send + Sync + 'static, T>(
        &mut self,
        f: impl FnOnce(&mut Self, &mut R) -> T,
    ) -> T {
        let mut resource = self.resource_map_mut().remove::<R>().unwrap_or_else(|| {
            panic!(
                "resource_scope requires {} to be present",
                std::any::type_name::<R>()
            )
        });
        let result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(self, &mut resource)));
        self.resource_map_mut().insert(resource);
        debug_assert!(
            self.resource_map_mut().contains::<R>(),
            "ResourceHost returned a different map across calls, so {} was reinserted into a map the host no longer exposes",
            std::any::type_name::<R>()
        );
        match result {
            Ok(value) => value,
            Err(panic) => std::panic::resume_unwind(panic),
        }
    }

    fn resources_scope<B: ResourceBundle, T>(
        &mut self,
        f: impl FnOnce(&mut Self, &mut B) -> T,
    ) -> T {
        let mut bundle = B::take(self.resource_map_mut());
        let result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(self, &mut bundle)));
        bundle.put(self.resource_map_mut());
        debug_assert!(
            B::contains_all(self.resource_map_mut()),
            "ResourceHost returned a different map across calls, so the bundle was reinserted into a map the host no longer exposes"
        );
        match result {
            Ok(value) => value,
            Err(panic) => std::panic::resume_unwind(panic),
        }
    }
}

impl<H: ResourceHost> ResourceHostExt for H {}

impl ResourceHost for ResourceMap {
    fn resource_map_mut(&mut self) -> &mut ResourceMap {
        self
    }
    fn resource_map(&self) -> &ResourceMap {
        self
    }
}

impl ResourceHost for World {
    fn resource_map_mut(&mut self) -> &mut ResourceMap {
        &mut self.resources
    }
    fn resource_map(&self) -> &ResourceMap {
        &self.resources
    }
}

#[derive(Default, Clone)]
pub(crate) struct MigrationPlan {
    pub(crate) gained: Vec<(u32, u32)>,
    pub(crate) shared: Vec<(u32, u32)>,
    pub(crate) removed: Vec<u32>,
}

pub trait ResourceBundle: sealed::SealedResourceBundle + Sized {
    fn take(resources: &mut ResourceMap) -> Self;
    fn put(self, resources: &mut ResourceMap);
    fn contains_all(resources: &ResourceMap) -> bool;
}

macro_rules! impl_resource_bundle {
    ($($element:ident $value:ident),+) => {
        impl<$($element: Send + Sync + 'static),+> sealed::SealedResourceBundle for ($($element,)+) {}

        impl<$($element: Send + Sync + 'static),+> ResourceBundle for ($($element,)+) {
            fn contains_all(resources: &ResourceMap) -> bool {
                true $(&& resources.contains::<$element>())+
            }

            fn take(resources: &mut ResourceMap) -> Self {
                let elements = [$((TypeId::of::<$element>(), std::any::type_name::<$element>()),)+];
                for (index, (type_id, name)) in elements.iter().enumerate() {
                    assert!(
                        !elements[..index].iter().any(|(seen, _)| seen == type_id),
                        "resource scopes must not repeat a resource type: {name}"
                    );
                    assert!(
                        resources.holds(*type_id),
                        "resources_scope requires {name} to be present"
                    );
                }
                ($(
                    resources
                        .remove::<$element>()
                        .expect("presence was checked before any removal"),
                )+)
            }

            fn put(self, resources: &mut ResourceMap) {
                let ($($value,)+) = self;
                $(resources.insert($value);)+
            }
        }
    };
}

impl_resource_bundle!(A a);
impl_resource_bundle!(A a, B b);
impl_resource_bundle!(A a, B b, C c);
impl_resource_bundle!(A a, B b, C c, D d);
impl_resource_bundle!(A a, B b, C c, D d, E e);
impl_resource_bundle!(A a, B b, C c, D d, E e, F f);
impl_resource_bundle!(A a, B b, C c, D d, E e, F f, G g);
impl_resource_bundle!(A a, B b, C c, D d, E e, F f, G g, H h);
