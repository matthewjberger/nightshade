use crate::Command;
use crate::Entity;
use crate::Schedule;
use crate::access::{Access, SystemAccess};
use crate::schedule::SystemRunner;
use crate::{
    Bundle, CloneBundle, EventBus, QueryBuilder, QueryTuple, ResourceHost, ResourceMap,
    ResourceSlot, StagedComponents, Storage, World,
};
use std::cell::RefCell;
use std::marker::PhantomData;

pub struct Res<'world, T> {
    value: &'world T,
}

impl<T> std::ops::Deref for Res<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.value
    }
}

pub struct ResMut<'world, T> {
    value: &'world mut T,
}

impl<T> std::ops::Deref for ResMut<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.value
    }
}

impl<T> std::ops::DerefMut for ResMut<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.value
    }
}

pub trait EventHost {
    fn event_bus_mut(&mut self) -> &mut EventBus;

    fn event_bus(&self) -> &EventBus;
}

impl EventHost for World {
    fn event_bus_mut(&mut self) -> &mut EventBus {
        &mut self.events
    }

    fn event_bus(&self) -> &EventBus {
        &self.events
    }
}

pub trait EcsHost {
    fn split_mut(&mut self) -> (&mut Storage, &mut ResourceMap);

    fn storage(&self) -> &Storage;
}

impl EcsHost for World {
    fn split_mut(&mut self) -> (&mut Storage, &mut ResourceMap) {
        (&mut self.storage, &mut self.resources)
    }

    fn storage(&self) -> &Storage {
        &self.storage
    }
}

pub trait ExtractParam<W> {
    type State: Send + 'static;
    type Owned;
    type Item<'item>;
    fn init() -> Self::State;
    fn extract(state: &mut Self::State, host: &W) -> Self::Owned;
    fn resources() -> Vec<ResourceNeed> {
        Vec::new()
    }
    fn build<'a, I: Iterator<Item = Option<ResourceSlot<'a>>>>(
        owned: &'a mut Self::Owned,
        slots: &mut I,
        resources: &'a ResourceMap,
    ) -> Self::Item<'a>;
    fn apply(state: &mut Self::State, owned: Self::Owned, host: &W);
    fn access() -> SystemAccess {
        SystemAccess::exclusive()
    }
    fn warm(_host: &mut W) {}
    fn flush(_host: &mut W) {}
}

macro_rules! impl_resource_param {
    ($wrapper:ident, $need:ident, $access:ident, $next:ident) => {
        impl<W: ResourceHost, T: Send + Sync + 'static> ExtractParam<W> for $wrapper<'_, T> {
            type State = ();
            type Owned = ();
            type Item<'item> = $wrapper<'item, T>;
            fn init() -> Self::State {}
            fn resources() -> Vec<ResourceNeed> {
                vec![ResourceNeed::$need::<T>()]
            }
            fn extract(_state: &mut (), _host: &W) {}
            fn build<'a, I: Iterator<Item = Option<ResourceSlot<'a>>>>(
                _owned: &'a mut (),
                slots: &mut I,
                _resources: &'a ResourceMap,
            ) -> $wrapper<'a, T> {
                $wrapper {
                    value: $next(slots).expect("the runner lends a slot for a required resource"),
                }
            }
            fn apply(_state: &mut (), _owned: (), _host: &W) {}
            fn access() -> SystemAccess {
                SystemAccess::empty().with_resource(std::any::TypeId::of::<T>(), Access::$access)
            }
        }

        impl<W: ResourceHost, T: Send + Sync + 'static> ExtractParam<W>
            for Option<$wrapper<'_, T>>
        {
            type State = ();
            type Owned = ();
            type Item<'item> = Option<$wrapper<'item, T>>;
            fn init() -> Self::State {}
            fn resources() -> Vec<ResourceNeed> {
                vec![ResourceNeed::optional(ResourceNeed::$need::<T>())]
            }
            fn extract(_state: &mut (), _host: &W) {}
            fn build<'a, I: Iterator<Item = Option<ResourceSlot<'a>>>>(
                _owned: &'a mut (),
                slots: &mut I,
                _resources: &'a ResourceMap,
            ) -> Option<$wrapper<'a, T>> {
                $next(slots).map(|value| $wrapper { value })
            }
            fn apply(_state: &mut (), _owned: (), _host: &W) {}
            fn access() -> SystemAccess {
                SystemAccess::empty().with_resource(std::any::TypeId::of::<T>(), Access::$access)
            }
        }
    };
}

impl_resource_param!(Res, read, Read, next_resource_ref);
impl_resource_param!(ResMut, write, Write, next_resource_mut);

pub struct EventReader<'a, T: Clone + Send + Sync + 'static> {
    events: &'a [T],
}

impl<T: Clone + Send + Sync + 'static> EventReader<'_, T> {
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.events.iter()
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

impl<'a, T: Clone + Send + Sync + 'static> IntoIterator for &'a EventReader<'_, T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.events.iter()
    }
}

impl<W: EventHost, T: Clone + Send + Sync + 'static> ExtractParam<W> for EventReader<'_, T> {
    type State = u64;
    type Owned = Vec<T>;
    type Item<'item> = EventReader<'item, T>;
    fn init() -> u64 {
        0
    }
    fn extract(state: &mut u64, host: &W) -> Vec<T> {
        host.event_bus().consume::<T>(state).to_vec()
    }
    fn build<'a, I: Iterator<Item = Option<ResourceSlot<'a>>>>(
        owned: &'a mut Vec<T>,
        _slots: &mut I,
        _resources: &'a ResourceMap,
    ) -> EventReader<'a, T> {
        EventReader { events: owned }
    }
    fn apply(_state: &mut u64, _owned: Vec<T>, _host: &W) {}
    fn access() -> SystemAccess {
        SystemAccess::empty().with_event(std::any::TypeId::of::<T>(), Access::Read)
    }
}

pub struct EventWriter<'a, T: Send + Sync + 'static> {
    pending: &'a mut Vec<T>,
}

impl<T: Send + Sync + 'static> EventWriter<'_, T> {
    pub fn send(&mut self, event: T) {
        self.pending.push(event);
    }

    pub fn send_batch(&mut self, events: impl IntoIterator<Item = T>) {
        self.pending.extend(events);
    }
}

impl<W: EventHost, T: Send + Sync + 'static> ExtractParam<W> for EventWriter<'_, T> {
    type State = ();
    type Owned = Vec<T>;
    type Item<'item> = EventWriter<'item, T>;
    fn init() -> Self::State {}
    fn extract(_state: &mut (), _host: &W) -> Vec<T> {
        Vec::new()
    }
    fn build<'a, I: Iterator<Item = Option<ResourceSlot<'a>>>>(
        owned: &'a mut Vec<T>,
        _slots: &mut I,
        _resources: &'a ResourceMap,
    ) -> EventWriter<'a, T> {
        EventWriter { pending: owned }
    }
    fn apply(_state: &mut (), owned: Vec<T>, host: &W) {
        for event in owned {
            host.event_bus().send_shared::<T>(event);
        }
    }
    fn access() -> SystemAccess {
        SystemAccess::empty().with_event(std::any::TypeId::of::<T>(), Access::Write)
    }
    fn warm(host: &mut W) {
        host.event_bus_mut().ensure_channel::<T>();
    }
}

pub struct MainThread;

impl<W> ExtractParam<W> for MainThread {
    type State = ();
    type Owned = ();
    type Item<'item> = MainThread;

    fn init() -> Self::State {}

    fn extract(_state: &mut Self::State, _host: &W) -> Self::Owned {}

    fn build<'a, I: Iterator<Item = Option<ResourceSlot<'a>>>>(
        _owned: &'a mut Self::Owned,
        _slots: &mut I,
        _resources: &'a ResourceMap,
    ) -> Self::Item<'a> {
        MainThread
    }

    fn apply(_state: &mut Self::State, _owned: Self::Owned, _host: &W) {}

    fn access() -> SystemAccess {
        SystemAccess::empty().main_thread()
    }
}

#[derive(Default)]
pub struct CommandBuffer {
    commands: Vec<Command>,
    reserved: Vec<Entity>,
    ids: Option<crate::IdCursor>,
    staged: StagedComponents,
}

pub struct Commands<'a> {
    pending: &'a mut CommandBuffer,
}

impl CommandBuffer {
    pub fn drawing_from(cursor: crate::IdCursor) -> Self {
        Self {
            ids: Some(cursor),
            ..Self::default()
        }
    }

    pub fn take_reserved(&mut self) -> Vec<Entity> {
        std::mem::take(&mut self.reserved)
    }

    pub(crate) fn take_commands(&mut self) -> Vec<Command> {
        std::mem::take(&mut self.commands)
    }
}

impl<'a> Commands<'a> {
    pub fn over(pending: &'a mut CommandBuffer) -> Self {
        Self { pending }
    }
}

impl Commands<'_> {
    pub fn queue(
        &mut self,
        command: impl FnOnce(&mut Storage, &mut ResourceMap) + Send + Sync + 'static,
    ) {
        self.pending
            .commands
            .push(Command::Closure(Box::new(command)));
    }

    pub fn reserve_entity(&mut self) -> Entity {
        let id = match &self.pending.ids {
            Some(cursor) => cursor.next(),
            None => panic!("a command buffer reserves ids from the world's cursor"),
        };
        let entity = Entity { id, generation: 0 };
        self.pending.reserved.push(entity);
        entity
    }

    pub fn spawn<B: CloneBundle + Send + Sync + 'static>(&mut self, bundle: B) -> Entity {
        let entity = self.reserve_entity();
        self.spawn_reserved(entity, bundle);
        entity
    }

    pub fn spawn_reserved<B: CloneBundle + Send + Sync + 'static>(
        &mut self,
        entity: Entity,
        bundle: B,
    ) {
        bundle.stage(entity, &mut self.pending.staged);
        self.queue(move |storage, _| {
            let mask = B::component_mask(storage);
            if !storage.contains_entity(entity) {
                storage.insert_row(entity, mask);
            }
            bundle.write(storage, entity);
        });
    }

    pub fn despawn(&mut self, entity: Entity) {
        self.pending.commands.push(Command::DespawnEntity(entity));
    }

    pub fn despawn_batch(&mut self, entities: Vec<Entity>) {
        self.pending
            .commands
            .push(Command::DespawnEntities(entities));
    }

    pub fn despawn_recursive(&mut self, entity: Entity) {
        self.queue(move |storage, resources| {
            let mut targets = resources.res::<crate::HierarchyIndex>().descendants(entity);
            targets.insert(0, entity);
            storage.despawn_entities(&targets);
        });
    }

    pub fn set<T: Clone + Send + Sync + Default + 'static>(&mut self, entity: Entity, value: T) {
        self.pending.staged.insert(
            (std::any::TypeId::of::<T>(), entity),
            Box::new(value.clone()),
        );
        self.queue(move |storage, _| storage.set(entity, value));
    }

    pub fn staged<T: Send + Sync + 'static>(&self, entity: Entity) -> Option<&T> {
        self.pending
            .staged
            .get(&(std::any::TypeId::of::<T>(), entity))
            .and_then(|value| value.downcast_ref::<T>())
    }

    pub fn staged_mut<T: Send + Sync + 'static>(&mut self, entity: Entity) -> Option<&mut T> {
        self.pending
            .staged
            .get_mut(&(std::any::TypeId::of::<T>(), entity))
            .and_then(|value| value.downcast_mut::<T>())
    }

    pub fn restage<T: Clone + Send + Sync + Default + 'static>(&mut self, entity: Entity) {
        let Some(value) = self.staged::<T>(entity).cloned() else {
            return;
        };
        self.queue(move |storage, _| storage.set(entity, value));
    }

    pub fn insert_bundle<B: Bundle + Send + Sync + 'static>(&mut self, entity: Entity, bundle: B) {
        self.queue(move |storage, _| storage.insert_bundle(entity, bundle));
    }

    pub fn remove<B: Bundle + Send + Sync + 'static>(&mut self, entity: Entity) {
        self.queue(move |storage, _| {
            storage.remove_components::<B>(entity);
        });
    }

    pub fn add_tag_type<T: 'static>(&mut self, entity: Entity) {
        self.queue(move |storage, _| storage.add_tag_type::<T>(entity));
    }

    pub fn remove_tag_type<T: 'static>(&mut self, entity: Entity) {
        self.queue(move |storage, _| {
            storage.remove_tag_type::<T>(entity);
        });
    }
}

impl<W: EcsHost> ExtractParam<W> for Commands<'_> {
    type State = ();
    type Owned = CommandBuffer;
    type Item<'item> = Commands<'item>;

    fn init() -> Self::State {}

    fn extract(_state: &mut (), host: &W) -> CommandBuffer {
        CommandBuffer::drawing_from(host.storage().allocator.next_id.clone())
    }

    fn build<'a, I: Iterator<Item = Option<ResourceSlot<'a>>>>(
        owned: &'a mut CommandBuffer,
        _slots: &mut I,
        _resources: &'a ResourceMap,
    ) -> Commands<'a> {
        Commands { pending: owned }
    }

    fn apply(_state: &mut (), owned: CommandBuffer, host: &W) {
        host.storage()
            .defer_commands(owned.commands, owned.reserved);
    }

    fn flush(host: &mut W) {
        apply_deferred_commands(host);
    }

    fn access() -> SystemAccess {
        SystemAccess::empty()
    }
}

pub fn apply_deferred_commands<W: EcsHost>(host: &mut W) {
    let (storage, resources) = host.split_mut();
    for parked in storage.take_deferred_commands() {
        for entity in parked.reserved {
            storage.allocator.revive(entity);
        }
        storage.extend_commands(parked.commands);
        let commands = storage.take_commands();
        crate::apply_commands_to(commands, storage, resources);
    }
}

pub trait QueryFilter {
    fn apply<Q: QueryTuple>(query: QueryBuilder<'_, Q>) -> QueryBuilder<'_, Q>;

    fn register(_world: &mut Storage) {}

    fn filter_access(access: SystemAccess) -> SystemAccess {
        access
    }
}

impl QueryFilter for () {
    fn apply<Q: QueryTuple>(query: QueryBuilder<'_, Q>) -> QueryBuilder<'_, Q> {
        query
    }
}

macro_rules! impl_query_filter {
    ($filter:ident, $bound:path, $method:ident, $register:ident $(, $access:expr)?) => {
        pub struct $filter<T>(PhantomData<fn() -> T>);

        impl<T: $bound> QueryFilter for $filter<T> {
            fn apply<Q: QueryTuple>(query: QueryBuilder<'_, Q>) -> QueryBuilder<'_, Q> {
                query.$method::<T>()
            }

            fn register(world: &mut Storage) {
                world.$register::<T>();
            }

            $(
                fn filter_access(access: SystemAccess) -> SystemAccess {
                    let build: fn(SystemAccess, std::any::TypeId) -> SystemAccess = $access;
                    build(access, std::any::TypeId::of::<T>())
                }
            )?
        }
    };
}

impl_query_filter!(
    With,
    ComponentType,
    with,
    component_key,
    SystemAccess::requiring
);
impl_query_filter!(
    Without,
    ComponentType,
    without,
    component_key,
    SystemAccess::excluding
);
impl_query_filter!(
    Changed,
    ComponentType,
    changed,
    component_key,
    |access, type_id| access.with_component(type_id, Access::Read)
);
impl_query_filter!(
    Added,
    ComponentType,
    added,
    component_key,
    |access, type_id| access.with_component(type_id, Access::Read)
);
impl_query_filter!(WithTag, TagType, with_tag_type, tag_key);
impl_query_filter!(WithoutTag, TagType, without_tag_type, tag_key);

pub trait ComponentType: Send + Sync + Default + 'static {}

impl<T: Send + Sync + Default + 'static> ComponentType for T {}

pub trait TagType: 'static {}

impl<T: 'static> TagType for T {}

macro_rules! impl_query_filter_tuple {
    ($($filter:ident),+) => {
        impl<$($filter: QueryFilter),+> QueryFilter for ($($filter,)+) {
            fn apply<Q: QueryTuple>(query: QueryBuilder<'_, Q>) -> QueryBuilder<'_, Q> {
                $(let query = $filter::apply(query);)+
                query
            }

            fn filter_access(access: SystemAccess) -> SystemAccess {
                $(let access = $filter::filter_access(access);)+
                access
            }
        }
    };
}

impl_query_filter_tuple!(A);
impl_query_filter_tuple!(A, B);
impl_query_filter_tuple!(A, B, C);
impl_query_filter_tuple!(A, B, C, D);

enum QueryState<'world, Q: QueryTuple> {
    Eager(Box<QueryBuilder<'world, Q>>),
    Lazy(&'world RefCell<&'world Storage>),
}

pub struct Query<'world, Q: QueryTuple, F: QueryFilter = ()> {
    state: QueryState<'world, Q>,
    filter: PhantomData<fn() -> F>,
}

impl<'world, Q: QueryTuple, F: QueryFilter> Query<'world, Q, F> {
    pub fn for_each(self, f: impl for<'item> FnMut(Entity, Q::Item<'item>)) {
        match self.state {
            QueryState::Eager(query) => F::apply(*query).for_each(f),
            QueryState::Lazy(cell) => {
                let guard = cell.borrow_mut();
                F::apply(guard.query_shared::<Q>()).for_each(f);
            }
        }
    }

    pub fn for_each_mut(&mut self, f: impl for<'item> FnMut(Entity, Q::Item<'item>)) {
        match &mut self.state {
            QueryState::Eager(query) => F::apply(query.reborrow()).for_each(f),
            QueryState::Lazy(cell) => {
                let guard = cell.borrow_mut();
                F::apply(guard.query_shared::<Q>()).for_each(f);
            }
        }
    }

    pub fn for_entity(&mut self, entity: Entity, f: impl for<'item> FnOnce(Q::Item<'item>)) {
        match &mut self.state {
            QueryState::Eager(query) => F::apply(query.reborrow()).for_entity(entity, f),
            QueryState::Lazy(cell) => {
                let guard = cell.borrow_mut();
                F::apply(guard.query_shared::<Q>()).for_entity(entity, f);
            }
        }
    }
}

pub fn warm_query<Q: QueryTuple, F: QueryFilter>(world: &mut Storage) {
    F::register(world);
    let element_masks = Q::element_masks(world);
    let required = Q::required_mask_from(&element_masks);
    let (include, exclude) = {
        let builder = F::apply(QueryBuilder::<Q>::over(&*world, element_masks, required));
        (builder.include, builder.exclude)
    };
    world.warm_masks(include, exclude);
}

pub trait WorldParam {
    type Item<'world>;
    fn build(world: &Storage) -> Self::Item<'_>;
    fn access() -> SystemAccess;
    fn warm(world: &mut Storage);
}

impl<'a, Q: QueryTuple, F: QueryFilter> WorldParam for Query<'a, Q, F> {
    type Item<'world> = Query<'world, Q, F>;
    fn build(world: &Storage) -> Query<'_, Q, F> {
        Query {
            state: QueryState::Eager(Box::new(world.query_shared::<Q>())),
            filter: PhantomData,
        }
    }
    fn access() -> SystemAccess {
        F::filter_access(Q::component_access(SystemAccess::empty()))
    }
    fn warm(world: &mut Storage) {
        warm_query::<Q, F>(world);
    }
}

pub trait StorageParam {
    type Item<'world>;
    fn build(world: &mut Storage) -> Self::Item<'_>;
    fn access() -> SystemAccess;
}

impl StorageParam for &mut Storage {
    type Item<'world> = &'world mut Storage;
    fn build(world: &mut Storage) -> &mut Storage {
        world
    }
    fn access() -> SystemAccess {
        SystemAccess::exclusive()
    }
}

pub trait MultiQueryParam {
    type Item<'world>;
    fn build_lazy<'world>(cell: &'world RefCell<&'world Storage>) -> Self::Item<'world>;
    fn access() -> SystemAccess;
    fn warm(world: &mut Storage);
}

impl<'a, Q: QueryTuple, F: QueryFilter> MultiQueryParam for Query<'a, Q, F> {
    type Item<'world> = Query<'world, Q, F>;
    fn build_lazy<'world>(cell: &'world RefCell<&'world Storage>) -> Query<'world, Q, F> {
        Query {
            state: QueryState::Lazy(cell),
            filter: PhantomData,
        }
    }
    fn access() -> SystemAccess {
        F::filter_access(Q::component_access(SystemAccess::empty()))
    }
    fn warm(world: &mut Storage) {
        warm_query::<Q, F>(world);
    }
}

pub trait QueryBundle: MultiQueryParam {}

#[derive(Clone, Copy, Debug)]
pub struct ResourceNeed {
    pub type_id: std::any::TypeId,
    pub name: &'static str,
    pub optional: bool,
    pub write: bool,
}

impl ResourceNeed {
    pub fn read<T: 'static>() -> Self {
        Self::of::<T>(false, false)
    }

    pub fn write<T: 'static>() -> Self {
        Self::of::<T>(false, true)
    }

    pub fn optional_read<T: 'static>() -> Self {
        Self::of::<T>(true, false)
    }

    pub fn optional_write<T: 'static>() -> Self {
        Self::of::<T>(true, true)
    }

    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }

    fn of<T: 'static>(optional: bool, write: bool) -> Self {
        Self {
            type_id: std::any::TypeId::of::<T>(),
            name: std::any::type_name::<T>(),
            optional,
            write,
        }
    }
}

pub fn declared_resources(per_param: &[Vec<ResourceNeed>]) -> Vec<ResourceNeed> {
    per_param.iter().flatten().copied().collect()
}

pub fn assert_resources_named_once<Func>(declared: &[ResourceNeed]) {
    for (index, need) in declared.iter().enumerate() {
        if declared[..index]
            .iter()
            .any(|seen| seen.type_id == need.type_id)
        {
            panic!(
                "system {} names resource {} through two parameters",
                std::any::type_name::<Func>(),
                need.name
            );
        }
    }
}

pub fn resource_borrows(
    resources: &ResourceMap,
    declared: &[ResourceNeed],
) -> Vec<Option<crate::ResourceBorrow>> {
    declared
        .iter()
        .map(|need| match resources.id_of_type(&need.type_id) {
            Some(id) if resources.holds(need.type_id) => Some(crate::ResourceBorrow {
                id,
                write: need.write,
            }),
            _ if need.optional => None,
            _ => panic!("system requires resource {} to be present", need.name),
        })
        .collect()
}

fn next_resource_ref<'a, T: Send + Sync + 'static, I: Iterator<Item = Option<ResourceSlot<'a>>>>(
    slots: &mut I,
) -> Option<&'a T> {
    slots
        .next()
        .expect("the runner lends one slot per resource a parameter names")
        .map(|slot| slot.into_ref::<T>())
}

fn next_resource_mut<'a, T: Send + Sync + 'static, I: Iterator<Item = Option<ResourceSlot<'a>>>>(
    slots: &mut I,
) -> Option<&'a mut T> {
    slots
        .next()
        .expect("the runner lends one slot per resource a parameter names")
        .map(|slot| slot.into_mut::<T>())
}

pub trait IntoSystem<W, Marker>: Sized {
    fn into_runner(self) -> SystemRunner<W>;

    fn access() -> SystemAccess {
        SystemAccess::exclusive()
    }

    fn warmer() -> Option<fn(&mut W)> {
        None
    }

    fn flusher() -> Option<fn(&mut W)> {
        None
    }

    fn declared_resources() -> Vec<ResourceNeed> {
        Vec::new()
    }
}

pub trait ScheduleExt<W> {
    fn add_system<Marker, S>(&mut self, name: &'static str, system: S) -> &mut Self
    where
        S: IntoSystem<W, Marker>;
}

impl<W> ScheduleExt<W> for Schedule<W> {
    fn add_system<Marker, S>(&mut self, name: &'static str, system: S) -> &mut Self
    where
        S: IntoSystem<W, Marker>,
    {
        self.push_with_access(
            name,
            S::access(),
            S::warmer(),
            S::flusher(),
            system.into_runner(),
        )
    }
}

pub fn into_standalone_runner<W, Marker, S>(system: S) -> impl FnMut(&mut W) + Send + 'static
where
    S: IntoSystem<W, Marker>,
    W: 'static,
{
    let warmer = S::warmer();
    let flusher = S::flusher();
    let mut runner = system.into_runner();
    move |world: &mut W| {
        if let Some(warm) = warmer {
            warm(world);
        }
        runner.run(world);
        if let Some(flush) = flusher {
            flush(world);
        }
    }
}

pub fn run_system<H, Marker>(host: &mut H, system: impl IntoSystem<H, Marker>) {
    system.into_runner().run(host);
}

pub struct ExtractSystemMarker<E>(PhantomData<fn() -> E>);

pub struct QuerySystemMarker<E, Q>(PhantomData<fn() -> (E, Q)>);

pub struct MultiQuerySystemMarker<R, Q>(PhantomData<fn() -> (R, Q)>);

pub struct BundleSystemMarker<E, B>(PhantomData<fn() -> (E, B)>);

pub struct StorageSystemMarker<E, S>(PhantomData<fn() -> (E, S)>);

impl<H, Func, S> IntoSystem<H, StorageSystemMarker<(), S>> for Func
where
    H: EcsHost + 'static,
    S: StorageParam,
    Func: FnMut(S) + for<'world> FnMut(S::Item<'world>) + Send + 'static,
{
    fn into_runner(mut self) -> SystemRunner<H> {
        SystemRunner::Exclusive(Box::new(move |host: &mut H| {
            self(S::build(host.split_mut().0));
        }))
    }

    fn access() -> SystemAccess {
        S::access()
    }
}

macro_rules! impl_storage_system {
    ($($param:ident $state:ident $owned:ident),+) => {
        impl<H, Func, $($param,)+ S> IntoSystem<H, StorageSystemMarker<($($param,)+), S>> for Func
        where
            H: EcsHost + ResourceHost + 'static,
            $($param: ExtractParam<H>,)+
            S: StorageParam,
            Func: FnMut($($param,)+ S)
                + for<'item> FnMut($($param::Item<'item>,)+ S::Item<'item>)
                + Send
                + 'static,
        {
            fn into_runner(mut self) -> SystemRunner<H> {
                let declared = extract_preamble!(Func, H, $($param),+);
                $(let mut $state = <$param as ExtractParam<H>>::init();)+
                SystemRunner::Exclusive(Box::new(move |host: &mut H| {
                    $(let mut $owned =
                        <$param as ExtractParam<H>>::extract(&mut $state, &*host);)+
                    let result = {
                        let (storage, resources) = host.split_mut();
                        let resources = &*resources;
                        let borrows = resource_borrows(resources, &declared);
                        let mut slots = resources.disjoint_slots(&borrows).into_iter();
                        $(let $owned = <$param as ExtractParam<H>>::build(
                            &mut $owned,
                            &mut slots,
                            resources,
                        );)+
                        let rows = S::build(storage);
                        let this = &mut self;
                        std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
                            this($($owned,)+ rows);
                        }))
                    };
                    $(<$param as ExtractParam<H>>::apply(&mut $state, $owned, &*host);)+
                    if let Err(panic) = result {
                        std::panic::resume_unwind(panic);
                    }
                }))
            }

            fn access() -> SystemAccess {
                SystemAccess::empty()
                    $(.merge(<$param as ExtractParam<H>>::access()))+
                    .merge(S::access())
            }

            fn declared_resources() -> Vec<ResourceNeed> {
                declared_resources(&[$(<$param as ExtractParam<H>>::resources(),)+])
            }

            fn warmer() -> Option<fn(&mut H)> {
                Some(|host: &mut H| {
                    $(<$param as ExtractParam<H>>::warm(host);)+
                })
            }

            fn flusher() -> Option<fn(&mut H)> {
                Some(|host: &mut H| {
                    $(<$param as ExtractParam<H>>::flush(host);)+
                })
            }
        }
    };
}

impl<H, Func, Q> IntoSystem<H, QuerySystemMarker<(), Q>> for Func
where
    H: EcsHost + 'static,
    Q: WorldParam,
    Func: FnMut(Q) + for<'world> FnMut(Q::Item<'world>) + Send + 'static,
{
    fn into_runner(mut self) -> SystemRunner<H> {
        SystemRunner::Shared(Box::new(move |host: &H| {
            self(Q::build(host.storage()));
        }))
    }

    fn access() -> SystemAccess {
        Q::access()
    }

    fn warmer() -> Option<fn(&mut H)> {
        Some(|host: &mut H| Q::warm(host.split_mut().0))
    }
}

impl<H, Func, B> IntoSystem<H, BundleSystemMarker<(), B>> for Func
where
    H: EcsHost + 'static,
    B: QueryBundle,
    Func: FnMut(B) + for<'world> FnMut(B::Item<'world>) + Send + 'static,
{
    fn into_runner(mut self) -> SystemRunner<H> {
        SystemRunner::Shared(Box::new(move |host: &H| {
            let cell = RefCell::new(host.storage());
            self(B::build_lazy(&cell));
        }))
    }

    fn access() -> SystemAccess {
        <B as MultiQueryParam>::access()
    }

    fn warmer() -> Option<fn(&mut H)> {
        Some(|host: &mut H| <B as MultiQueryParam>::warm(host.split_mut().0))
    }
}

macro_rules! impl_multi_query_bare {
    ($($query:ident $qbind:ident),+) => {
        impl<H, Func, $($query,)+>
            IntoSystem<H, MultiQuerySystemMarker<(), ($($query,)+)>> for Func
        where
            H: EcsHost + 'static,
            $($query: MultiQueryParam,)+
            Func: FnMut($($query,)+)
                + for<'world> FnMut($($query::Item<'world>,)+)
                + Send
                + 'static,
        {
            fn into_runner(mut self) -> SystemRunner<H> {
                SystemRunner::Shared(Box::new(move |host: &H| {
                    let cell = RefCell::new(host.storage());
                    $(let $qbind = $query::build_lazy(&cell);)+
                    self($($qbind,)+);
                }))
            }

            fn access() -> SystemAccess {
                SystemAccess::empty()
                    $(.merge(<$query as MultiQueryParam>::access()))+
            }

            fn warmer() -> Option<fn(&mut H)> {
                Some(|host: &mut H| {
                    $(<$query as MultiQueryParam>::warm(host.split_mut().0);)+
                })
            }
        }
    };
}

impl_multi_query_bare!(Q0 q0, Q1 q1);
impl_multi_query_bare!(Q0 q0, Q1 q1, Q2 q2);
impl_multi_query_bare!(Q0 q0, Q1 q1, Q2 q2, Q3 q3);
impl_multi_query_bare!(Q0 q0, Q1 q1, Q2 q2, Q3 q3, Q4 q4);
impl_multi_query_bare!(Q0 q0, Q1 q1, Q2 q2, Q3 q3, Q4 q4, Q5 q5);
impl_multi_query_bare!(Q0 q0, Q1 q1, Q2 q2, Q3 q3, Q4 q4, Q5 q5, Q6 q6);
impl_multi_query_bare!(Q0 q0, Q1 q1, Q2 q2, Q3 q3, Q4 q4, Q5 q5, Q6 q6, Q7 q7);

macro_rules! extract_preamble {
    ($func:ty, $world:ty, $($param:ident),+) => {{
        let declared = declared_resources(&[$(<$param as ExtractParam<$world>>::resources(),)+]);
        assert_resources_named_once::<$func>(&declared);
        declared
    }};
}

macro_rules! impl_extract_system {
    ($($param:ident $state:ident $owned:ident),+) => {
        impl<W, Func, $($param,)+> IntoSystem<W, ExtractSystemMarker<($($param,)+)>> for Func
        where
            W: ResourceHost + 'static,
            $($param: ExtractParam<W>,)+
            Func: FnMut($($param,)+)
                + for<'item> FnMut($($param::Item<'item>,)+)
                + Send
                + 'static,
        {
            fn into_runner(mut self) -> SystemRunner<W> {
                let declared = extract_preamble!(Func, W, $($param),+);
                $(let mut $state = <$param as ExtractParam<W>>::init();)+
                SystemRunner::Shared(Box::new(move |host: &W| {
                    $(let mut $owned = <$param as ExtractParam<W>>::extract(&mut $state, host);)+
                    let result = {
                        let resources = host.resource_map();
                        let borrows = resource_borrows(resources, &declared);
                        let mut slots = resources.disjoint_slots(&borrows).into_iter();
                        $(let $owned = <$param as ExtractParam<W>>::build(
                            &mut $owned,
                            &mut slots,
                            resources,
                        );)+
                        let this = &mut self;
                        std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
                            this($($owned,)+);
                        }))
                    };
                    $(<$param as ExtractParam<W>>::apply(&mut $state, $owned, host);)+
                    if let Err(panic) = result {
                        std::panic::resume_unwind(panic);
                    }
                }))
            }

            fn access() -> SystemAccess {
                SystemAccess::empty() $(.merge(<$param as ExtractParam<W>>::access()))+
            }

            fn declared_resources() -> Vec<ResourceNeed> {
                declared_resources(&[$(<$param as ExtractParam<W>>::resources(),)+])
            }

            fn warmer() -> Option<fn(&mut W)> {
                Some(|host: &mut W| {
                    $(<$param as ExtractParam<W>>::warm(host);)+
                })
            }

            fn flusher() -> Option<fn(&mut W)> {
                Some(|host: &mut W| {
                    $(<$param as ExtractParam<W>>::flush(host);)+
                })
            }
        }
    };
}

macro_rules! impl_extract_query_system {
    ($($param:ident $state:ident $owned:ident),+) => {
        impl<H, Func, $($param,)+ Q> IntoSystem<H, QuerySystemMarker<($($param,)+), Q>> for Func
        where
            H: EcsHost + ResourceHost + 'static,
            $($param: ExtractParam<H>,)+
            Q: WorldParam,
            Func: FnMut($($param,)+ Q)
                + for<'item> FnMut($($param::Item<'item>,)+ Q::Item<'item>)
                + Send
                + 'static,
        {
            fn into_runner(mut self) -> SystemRunner<H> {
                let declared = extract_preamble!(Func, H, $($param),+);
                $(let mut $state = <$param as ExtractParam<H>>::init();)+
                SystemRunner::Shared(Box::new(move |host: &H| {
                    $(let mut $owned = <$param as ExtractParam<H>>::extract(&mut $state, host);)+
                    let result = {
                        let resources = host.resource_map();
                        let borrows = resource_borrows(resources, &declared);
                        let mut slots = resources.disjoint_slots(&borrows).into_iter();
                        $(let $owned = <$param as ExtractParam<H>>::build(
                            &mut $owned,
                            &mut slots,
                            resources,
                        );)+
                        let query = Q::build(host.storage());
                        let this = &mut self;
                        std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
                            this($($owned,)+ query);
                        }))
                    };
                    $(<$param as ExtractParam<H>>::apply(&mut $state, $owned, host);)+
                    if let Err(panic) = result {
                        std::panic::resume_unwind(panic);
                    }
                }))
            }

            fn access() -> SystemAccess {
                SystemAccess::empty()
                    $(.merge(<$param as ExtractParam<H>>::access()))+
                    .merge(Q::access())
            }

            fn declared_resources() -> Vec<ResourceNeed> {
                declared_resources(&[$(<$param as ExtractParam<H>>::resources(),)+])
            }

            fn warmer() -> Option<fn(&mut H)> {
                Some(|host: &mut H| {
                    $(<$param as ExtractParam<H>>::warm(host);)+
                    Q::warm(host.split_mut().0);
                })
            }

            fn flusher() -> Option<fn(&mut H)> {
                Some(|host: &mut H| {
                    $(<$param as ExtractParam<H>>::flush(host);)+
                })
            }
        }
    };
}

macro_rules! impl_lazy_system {
    (
        $marker:ident,
        ($($param:ident $state:ident $owned:ident),+),
        ($($query:ident $qbind:ident),+),
        $build:ident
    ) => {
        impl<H, Func, $($param,)+ $($query,)+>
            IntoSystem<H, $marker<($($param,)+), ($($query,)+)>> for Func
        where
            H: EcsHost + ResourceHost + 'static,
            $($param: ExtractParam<H>,)+
            $($query: $build,)+
            Func: FnMut($($param,)+ $($query,)+)
                + for<'item> FnMut($($param::Item<'item>,)+ $($query::Item<'item>,)+)
                + Send
                + 'static,
        {
            fn into_runner(mut self) -> SystemRunner<H> {
                let declared = extract_preamble!(Func, H, $($param),+);
                $(let mut $state = <$param as ExtractParam<H>>::init();)+
                SystemRunner::Shared(Box::new(move |host: &H| {
                    $(let mut $owned = <$param as ExtractParam<H>>::extract(&mut $state, host);)+
                    let result = {
                        let resources = host.resource_map();
                        let borrows = resource_borrows(resources, &declared);
                        let mut slots = resources.disjoint_slots(&borrows).into_iter();
                        $(let $owned = <$param as ExtractParam<H>>::build(
                            &mut $owned,
                            &mut slots,
                            resources,
                        );)+
                        let cell = RefCell::new(host.storage());
                        $(let $qbind = $query::build_lazy(&cell);)+
                        let this = &mut self;
                        std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
                            this($($owned,)+ $($qbind,)+);
                        }))
                    };
                    $(<$param as ExtractParam<H>>::apply(&mut $state, $owned, host);)+
                    if let Err(panic) = result {
                        std::panic::resume_unwind(panic);
                    }
                }))
            }

            fn access() -> SystemAccess {
                SystemAccess::empty()
                    $(.merge(<$param as ExtractParam<H>>::access()))+
                    $(.merge(<$query as MultiQueryParam>::access()))+
            }

            fn declared_resources() -> Vec<ResourceNeed> {
                declared_resources(&[$(<$param as ExtractParam<H>>::resources(),)+])
            }

            fn warmer() -> Option<fn(&mut H)> {
                Some(|host: &mut H| {
                    $(<$param as ExtractParam<H>>::warm(host);)+
                    $(<$query as MultiQueryParam>::warm(host.split_mut().0);)+
                })
            }

            fn flusher() -> Option<fn(&mut H)> {
                Some(|host: &mut H| {
                    $(<$param as ExtractParam<H>>::flush(host);)+
                })
            }
        }
    };
}

macro_rules! impl_param_systems {
    ($($param:ident $state:ident $owned:ident),+) => {
        impl_extract_system!($($param $state $owned),+);
        impl_extract_query_system!($($param $state $owned),+);
        impl_storage_system!($($param $state $owned),+);
        impl_lazy_system!(
            BundleSystemMarker, ($($param $state $owned),+), (B0 b0), QueryBundle
        );
        impl_lazy_system!(
            MultiQuerySystemMarker, ($($param $state $owned),+),
            (Q0 q0, Q1 q1), MultiQueryParam
        );
        impl_lazy_system!(
            MultiQuerySystemMarker, ($($param $state $owned),+),
            (Q0 q0, Q1 q1, Q2 q2), MultiQueryParam
        );
        impl_lazy_system!(
            MultiQuerySystemMarker, ($($param $state $owned),+),
            (Q0 q0, Q1 q1, Q2 q2, Q3 q3), MultiQueryParam
        );
        impl_lazy_system!(
            MultiQuerySystemMarker, ($($param $state $owned),+),
            (Q0 q0, Q1 q1, Q2 q2, Q3 q3, Q4 q4), MultiQueryParam
        );
        impl_lazy_system!(
            MultiQuerySystemMarker, ($($param $state $owned),+),
            (Q0 q0, Q1 q1, Q2 q2, Q3 q3, Q4 q4, Q5 q5), MultiQueryParam
        );
        impl_lazy_system!(
            MultiQuerySystemMarker, ($($param $state $owned),+),
            (Q0 q0, Q1 q1, Q2 q2, Q3 q3, Q4 q4, Q5 q5, Q6 q6), MultiQueryParam
        );
        impl_lazy_system!(
            MultiQuerySystemMarker, ($($param $state $owned),+),
            (Q0 q0, Q1 q1, Q2 q2, Q3 q3, Q4 q4, Q5 q5, Q6 q6, Q7 q7), MultiQueryParam
        );
    };
}

impl_param_systems!(P0 s0 o0);
impl_param_systems!(P0 s0 o0, P1 s1 o1);
impl_param_systems!(P0 s0 o0, P1 s1 o1, P2 s2 o2);
impl_param_systems!(P0 s0 o0, P1 s1 o1, P2 s2 o2, P3 s3 o3);
impl_param_systems!(P0 s0 o0, P1 s1 o1, P2 s2 o2, P3 s3 o3, P4 s4 o4);
impl_param_systems!(P0 s0 o0, P1 s1 o1, P2 s2 o2, P3 s3 o3, P4 s4 o4, P5 s5 o5);
impl_param_systems!(P0 s0 o0, P1 s1 o1, P2 s2 o2, P3 s3 o3, P4 s4 o4, P5 s5 o5, P6 s6 o6);
impl_param_systems!(
    P0 s0 o0, P1 s1 o1, P2 s2 o2, P3 s3 o3, P4 s4 o4, P5 s5 o5, P6 s6 o6, P7 s7 o7
);
impl_param_systems!(
    P0 s0 o0, P1 s1 o1, P2 s2 o2, P3 s3 o3, P4 s4 o4, P5 s5 o5, P6 s6 o6, P7 s7 o7, P8 s8 o8
);
impl_param_systems!(
    P0 s0 o0, P1 s1 o1, P2 s2 o2, P3 s3 o3, P4 s4 o4, P5 s5 o5, P6 s6 o6, P7 s7 o7, P8 s8 o8,
    P9 s9 o9
);
impl_param_systems!(
    P0 s0 o0, P1 s1 o1, P2 s2 o2, P3 s3 o3, P4 s4 o4, P5 s5 o5, P6 s6 o6, P7 s7 o7, P8 s8 o8,
    P9 s9 o9, P10 s10 o10
);
impl_param_systems!(
    P0 s0 o0, P1 s1 o1, P2 s2 o2, P3 s3 o3, P4 s4 o4, P5 s5 o5, P6 s6 o6, P7 s7 o7, P8 s8 o8,
    P9 s9 o9, P10 s10 o10, P11 s11 o11
);

#[macro_export]
macro_rules! query_bundle {
    (
        $(#[$bundle_meta:meta])*
        $bundle_visibility:vis struct $name:ident<$world:lifetime, $item:lifetime> {
            $(
                $(#[$field_meta:meta])*
                $field_visibility:vis $field:ident: $query:ty,
            )+
        }
    ) => {
        $(#[$bundle_meta])*
        $bundle_visibility struct $name<$world, $item> {
            $(
                $(#[$field_meta])*
                $field_visibility $field: $query,
            )+
        }

        impl<$world, $item> $crate::system_param::MultiQueryParam for $name<$world, $item> {
            type Item<'bundle> = $name<'bundle, $item>;

            fn build_lazy<'bundle>(
                cell: &'bundle ::std::cell::RefCell<&'bundle $crate::Storage>,
            ) -> $name<'bundle, $item> {
                $name {
                    $($field: <$query as $crate::system_param::MultiQueryParam>::build_lazy(cell),)+
                }
            }

            fn access() -> $crate::SystemAccess {
                $crate::SystemAccess::empty()
                    $(.merge(<$query as $crate::system_param::MultiQueryParam>::access()))+
            }

            fn warm(storage: &mut $crate::Storage) {
                $(<$query as $crate::system_param::MultiQueryParam>::warm(storage);)+
            }
        }

        impl<$world, $item> $crate::system_param::QueryBundle for $name<$world, $item> {}
    };
}

#[macro_export]
macro_rules! resource_bundle {
    (
        $(#[$bundle_meta:meta])*
        $bundle_visibility:vis struct $name:ident<$map:lifetime> {
            write {
                $($write_field:ident: $write_type:ty,)+
            }
            read {
                $($read_field:ident: $read_type:ty,)*
            }
        }
    ) => {
        $(#[$bundle_meta])*
        $bundle_visibility struct $name<$map> {
            $(pub $write_field: &$map mut $write_type,)+
            $(pub $read_field: &$map $read_type,)*
        }

        impl $name<'_> {
            pub fn reborrow(&mut self) -> $name<'_> {
                $name {
                    $($write_field: self.$write_field,)+
                    $($read_field: self.$read_field,)*
                }
            }
        }

        impl<W: $crate::ResourceHost> $crate::system_param::ExtractParam<W> for $name<'_> {
            type State = ();
            type Owned = ();
            type Item<'item> = $name<'item>;

            fn init() -> Self::State {}

            fn resources() -> ::std::vec::Vec<$crate::system_param::ResourceNeed> {
                ::std::vec![
                    $($crate::system_param::ResourceNeed::write::<$write_type>(),)+
                    $($crate::system_param::ResourceNeed::read::<$read_type>(),)*
                ]
            }

            fn extract(_state: &mut (), _host: &W) {}

            fn build<'slots, I: ::std::iter::Iterator<Item = ::std::option::Option<$crate::ResourceSlot<'slots>>>>(
                _owned: &'slots mut (),
                slots: &mut I,
                _resources: &'slots $crate::ResourceMap,
            ) -> $name<'slots> {
                $name {
                    $($write_field: $crate::system_param::bundled_slot(slots).into_mut::<$write_type>(),)+
                    $($read_field: $crate::system_param::bundled_slot(slots).into_ref::<$read_type>(),)*
                }
            }

            fn apply(_state: &mut (), _owned: (), _host: &W) {}

            fn access() -> $crate::SystemAccess {
                use ::std::any::TypeId;
                $crate::SystemAccess::empty()
                    $(.with_resource(TypeId::of::<$write_type>(), $crate::Access::Write))+
                    $(.with_resource(TypeId::of::<$read_type>(), $crate::Access::Read))*
            }
        }
    };
}

pub fn bundled_slot<'map>(
    slots: &mut impl Iterator<Item = Option<ResourceSlot<'map>>>,
) -> ResourceSlot<'map> {
    slots
        .next()
        .expect("the runner borrows one slot for each resource a bundle names")
        .expect("a resource bundle names no resource it can do without")
}
