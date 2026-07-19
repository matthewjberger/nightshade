//! System-parameter functions over [`DynWorld`]: functions whose
//! arguments are [`Res`], [`ResMut`], and [`Query`] resolve into runnable
//! systems the existing [`Schedule`] accepts, with no
//! `unsafe` and no new executor.
//!
//! A system is any function whose parameters are system parameters. Register
//! it on a [`Schedule`] over a [`DynWorld`] with
//! [`ScheduleExt::add_system`]:
//!
//! ```rust
//! use nightshade_ecs::Schedule;
//! use nightshade_ecs::dynamic::DynWorld;
//! use nightshade_ecs::system_param::{Query, Res, ResMut, ScheduleExt};
//!
//! #[derive(Default, Clone, Debug)]
//! struct Position { x: f32, y: f32 }
//! #[derive(Default, Clone, Debug)]
//! struct Velocity { x: f32, y: f32 }
//!
//! nightshade_ecs::impl_component!(Position, Velocity);
//!
//! struct DeltaTime(f32);
//! struct Score(u32);
//!
//! fn movement(dt: Res<DeltaTime>, mut score: ResMut<Score>, query: Query<(&mut Position, &Velocity)>) {
//!     query.for_each(|_entity, (position, velocity)| {
//!         position.x += velocity.x * dt.0;
//!         position.y += velocity.y * dt.0;
//!     });
//!     score.0 += 1;
//! }
//!
//! let mut world = DynWorld::new();
//! world.insert_resources((DeltaTime(0.5), Score(0)));
//! world.spawn((Position { x: 0.0, y: 0.0 }, Velocity { x: 2.0, y: 4.0 }));
//!
//! let mut schedule = Schedule::new();
//! schedule.add_system("movement", movement);
//! schedule.run(&mut world);
//!
//! assert_eq!(world.resource::<Score>().unwrap().0, 1);
//! ```
//!
//! Resource parameters ([`Res`], [`ResMut`]) resolve out of the world's
//! [`ResourceMap`](crate::dynamic::ResourceMap) through the same take/put
//! scope [`resources_scope`](crate::dynamic::ResourceHostExt::resources_scope)
//! uses, so they never alias a query's table borrow. Resource parameters
//! come first, query parameters after.
//!
//! A single query parameter borrows the world directly and pays nothing
//! beyond the query itself. Several query parameters in one system, such as
//! `fn(a: Query<&mut Position>, b: Query<&Velocity>)`, share the world
//! through a cell and each take it only for the span of one
//! [`for_each`](Query::for_each), so the cost is one borrow check per call
//! rather than anything per entity. Because `for_each` consumes the query,
//! two of them run in sequence, never nested. Type-level filters ([`With`],
//! [`Without`], [`Changed`], [`Added`], [`WithTag`], [`WithoutTag`], and
//! tuples of them) narrow a query as `Query<(&mut Position,), With<Player>>`.
//! [`ParamSet`] groups queries behind `p0()`/`p1()` accessors when you would
//! rather name a set than list the queries.
//!
//! Resource-only systems, and systems that end in a `&mut W` host argument,
//! run over any [`ResourceHost`], not just
//! [`DynWorld`]. So an engine wrapper that implements `ResourceHost` can
//! register `fn(Res<A>, ResMut<B>, &mut MyWorld)` on its own
//! `Schedule<MyWorld>`, pulling resources out of the host's map while the
//! `&mut MyWorld` stays free for the wrapper's own queries. [`Query`]
//! parameters resolve against [`DynWorld`], and an unfiltered `Query<Q>` also
//! resolves against a [`DynEcs`] group through
//! `query_join`. [`ParamSet`], multiple query parameters, and type-level
//! query filters are [`DynWorld`] only.
//!
//! ```rust
//! use nightshade_ecs::dynamic::DynWorld;
//! use nightshade_ecs::system_param::{Query, ScheduleExt};
//!
//! #[derive(Default, Clone, Debug)]
//! struct Position { x: f32, y: f32 }
//! #[derive(Default, Clone, Debug)]
//! struct Velocity { x: f32, y: f32 }
//!
//! nightshade_ecs::impl_component!(Position, Velocity);
//!
//! fn drift(positions: Query<&mut Position>, velocities: Query<&Velocity>) {
//!     let mut total = (0.0, 0.0);
//!     velocities.for_each(|_entity, velocity| {
//!         total.0 += velocity.x;
//!         total.1 += velocity.y;
//!     });
//!     positions.for_each(|_entity, position| {
//!         position.x += total.0;
//!         position.y += total.1;
//!     });
//! }
//!
//! let mut world = DynWorld::new();
//! world.spawn((Position { x: 0.0, y: 0.0 }, Velocity { x: 1.0, y: 2.0 }));
//!
//! let mut schedule = nightshade_ecs::Schedule::new();
//! schedule.add_system("drift", drift);
//! schedule.run(&mut world);
//!
//! assert_eq!(world.get::<Position>(world.query_ref::<&Position>().single().unwrap().0).unwrap().x, 1.0);
//! ```
//!
//! [`EventReader`] and [`EventWriter`] are extract parameters over the host's
//! event bus. A writer buffers its sends and flushes them after the system
//! returns; a reader keeps its own cursor in the runner, so it sees each event
//! once and coexists with a [`Query`] borrowing the world in the same system.
//!
//! ```rust
//! use nightshade_ecs::Schedule;
//! use nightshade_ecs::dynamic::DynWorld;
//! use nightshade_ecs::system_param::{EventReader, EventWriter, ResMut, ScheduleExt};
//!
//! #[derive(Clone)]
//! struct Damaged { amount: u32 }
//! struct Total(u32);
//!
//! fn emit(mut writer: EventWriter<Damaged>) {
//!     writer.send(Damaged { amount: 3 });
//!     writer.send(Damaged { amount: 4 });
//! }
//! fn accumulate(reader: EventReader<Damaged>, mut total: ResMut<Total>) {
//!     for event in &reader {
//!         total.0 += event.amount;
//!     }
//! }
//!
//! let mut world = DynWorld::new();
//! world.insert_resource(Total(0));
//!
//! let mut schedule = Schedule::new();
//! schedule.add_system("emit", emit);
//! schedule.add_system("accumulate", accumulate);
//! schedule.run(&mut world);
//!
//! assert_eq!(world.resource::<Total>().unwrap().0, 7);
//! ```

use crate::Entity;
use crate::Schedule;
use crate::dynamic::{DynEcs, DynJoin, DynQuery, DynWorld, EventBus, QueryTuple, ResourceHost};
use std::cell::RefCell;
use std::marker::PhantomData;

/// A shared reference to a resource of type `T`, resolved for a system
/// parameter. Dereferences to `T`.
pub struct Res<'world, T> {
    value: &'world T,
}

impl<T> std::ops::Deref for Res<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.value
    }
}

/// An exclusive reference to a resource of type `T`, resolved for a system
/// parameter. Dereferences to `T` and, mutably, writes through to it.
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

/// A host that owns an [`EventBus`], so [`EventReader`] and [`EventWriter`]
/// parameters resolve against it. [`DynWorld`] and [`DynEcs`] both embed one;
/// a bare [`ResourceHost`] does not, which is why event parameters are
/// offered over those two containers rather than any host.
pub trait EventHost {
    /// The host's event bus, for reading and writing events.
    fn event_bus_mut(&mut self) -> &mut EventBus;

    /// The same bus, shared, for reading events behind a `&self`, such as the
    /// frame-settled broadcast [`EventBus::read_frame`](crate::dynamic::EventBus::read_frame).
    /// Must return the same bus as [`event_bus_mut`](Self::event_bus_mut).
    fn event_bus(&self) -> &EventBus;
}

impl EventHost for DynWorld {
    fn event_bus_mut(&mut self) -> &mut EventBus {
        &mut self.events
    }

    fn event_bus(&self) -> &EventBus {
        &self.events
    }
}

impl EventHost for DynEcs {
    fn event_bus_mut(&mut self) -> &mut EventBus {
        &mut self.events
    }

    fn event_bus(&self) -> &EventBus {
        &self.events
    }
}

/// A system parameter resolved by taking data out of the host before the
/// system runs and writing data back after. [`Res`], [`ResMut`],
/// [`EventReader`], and [`EventWriter`] are the extract parameters. Each
/// carries a [`State`](Self::State) kept in the runner between runs (an event
/// reader's cursor, say), produces an [`Owned`](Self::Owned) value the
/// parameter borrows for the call, and flushes through [`apply`](Self::apply)
/// afterward. Extract parameters lead a system's argument list, ahead of a
/// [`Query`], [`ParamSet`], or trailing `&mut W` host argument. Because the
/// owned value is lifted out of the host before the run, an extract parameter
/// never aliases the world a query borrows.
pub trait ExtractParam<W> {
    /// Per-system state kept in the runner across runs.
    type State: Send + 'static;
    /// The owned value produced for one run; the parameter borrows it.
    type Owned;
    /// The parameter value handed to the system for a given borrow.
    type Item<'item>;
    /// The initial state for a freshly registered system.
    fn init() -> Self::State;
    /// Lifts the owned value out of the host before the run.
    fn extract(state: &mut Self::State, host: &mut W) -> Self::Owned;
    /// Wraps the owned value as the parameter value.
    fn build(owned: &mut Self::Owned) -> Self::Item<'_>;
    /// Writes back after the run: a resource returns to the map, buffered
    /// events flush to the bus, a reader does nothing.
    fn apply(state: &mut Self::State, owned: Self::Owned, host: &mut W);
}

impl<W: ResourceHost, T: Send + Sync + 'static> ExtractParam<W> for Res<'_, T> {
    type State = ();
    type Owned = T;
    type Item<'item> = Res<'item, T>;
    fn init() -> Self::State {}
    fn extract(_state: &mut (), host: &mut W) -> T {
        host.resource_map_mut().remove::<T>().unwrap_or_else(|| {
            panic!(
                "system requires resource {} to be present",
                std::any::type_name::<T>()
            )
        })
    }
    fn build(owned: &mut T) -> Res<'_, T> {
        Res { value: owned }
    }
    fn apply(_state: &mut (), owned: T, host: &mut W) {
        host.resource_map_mut().insert(owned);
    }
}

impl<W: ResourceHost, T: Send + Sync + 'static> ExtractParam<W> for ResMut<'_, T> {
    type State = ();
    type Owned = T;
    type Item<'item> = ResMut<'item, T>;
    fn init() -> Self::State {}
    fn extract(_state: &mut (), host: &mut W) -> T {
        host.resource_map_mut().remove::<T>().unwrap_or_else(|| {
            panic!(
                "system requires resource {} to be present",
                std::any::type_name::<T>()
            )
        })
    }
    fn build(owned: &mut T) -> ResMut<'_, T> {
        ResMut { value: owned }
    }
    fn apply(_state: &mut (), owned: T, host: &mut W) {
        host.resource_map_mut().insert(owned);
    }
}

/// A system parameter that reads events of type `T` from the host's event
/// bus, delivering each event to this system exactly once across frames. The
/// reader keeps its own cursor in the runner, so two systems reading the same
/// event type advance independently and neither disturbs the bus's two-frame
/// buffer. Events are copied out for the run, so `T` is [`Clone`], and a
/// [`Query`] in the same system borrows the world freely alongside it.
pub struct EventReader<'a, T: Clone + Send + Sync + 'static> {
    events: &'a [T],
}

impl<T: Clone + Send + Sync + 'static> EventReader<'_, T> {
    /// Visits the events delivered to this reader this run, oldest first.
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.events.iter()
    }

    /// The number of events delivered to this reader this run.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Whether no events were delivered to this reader this run.
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
    fn extract(state: &mut u64, host: &mut W) -> Vec<T> {
        host.event_bus_mut().consume::<T>(state).to_vec()
    }
    fn build(owned: &mut Vec<T>) -> EventReader<'_, T> {
        EventReader { events: owned }
    }
    fn apply(_state: &mut u64, _owned: Vec<T>, _host: &mut W) {}
}

/// A system parameter that writes events of type `T` to the host's event bus.
/// Sends are buffered during the run and flushed to the bus after the system
/// returns, so a writer and a [`Query`] coexist without contending for the
/// world. Readers pick the events up through the bus's two-frame buffer.
pub struct EventWriter<'a, T: Send + Sync + 'static> {
    pending: &'a mut Vec<T>,
}

impl<T: Send + Sync + 'static> EventWriter<'_, T> {
    /// Buffers one event to flush to the bus after the system runs.
    pub fn send(&mut self, event: T) {
        self.pending.push(event);
    }

    /// Buffers a batch of events to flush after the system runs.
    pub fn send_batch(&mut self, events: impl IntoIterator<Item = T>) {
        self.pending.extend(events);
    }
}

impl<W: EventHost, T: Send + Sync + 'static> ExtractParam<W> for EventWriter<'_, T> {
    type State = ();
    type Owned = Vec<T>;
    type Item<'item> = EventWriter<'item, T>;
    fn init() -> Self::State {}
    fn extract(_state: &mut (), _host: &mut W) -> Vec<T> {
        Vec::new()
    }
    fn build(owned: &mut Vec<T>) -> EventWriter<'_, T> {
        EventWriter { pending: owned }
    }
    fn apply(_state: &mut (), owned: Vec<T>, host: &mut W) {
        for event in owned {
            host.event_bus_mut().send::<T>(event);
        }
    }
}

/// A type-level query filter: [`With`], [`Without`], [`Changed`], [`Added`],
/// the unit type for no filter, or a tuple of filters applied in order.
pub trait QueryFilter {
    /// Applies this filter to a query builder.
    fn apply<Q: QueryTuple>(query: DynQuery<'_, Q>) -> DynQuery<'_, Q>;
}

impl QueryFilter for () {
    fn apply<Q: QueryTuple>(query: DynQuery<'_, Q>) -> DynQuery<'_, Q> {
        query
    }
}

/// Restricts a query to entities that also carry component `T`, without
/// fetching it.
pub struct With<T>(PhantomData<fn() -> T>);

impl<T: Send + Sync + Default + 'static> QueryFilter for With<T> {
    fn apply<Q: QueryTuple>(query: DynQuery<'_, Q>) -> DynQuery<'_, Q> {
        query.with::<T>()
    }
}

/// Restricts a query to entities that do not carry component `T`.
pub struct Without<T>(PhantomData<fn() -> T>);

impl<T: Send + Sync + Default + 'static> QueryFilter for Without<T> {
    fn apply<Q: QueryTuple>(query: DynQuery<'_, Q>) -> DynQuery<'_, Q> {
        query.without::<T>()
    }
}

/// Restricts a query to entities whose component `T` changed since the last
/// step. `T` must appear in the query tuple.
pub struct Changed<T>(PhantomData<fn() -> T>);

impl<T: Send + Sync + Default + 'static> QueryFilter for Changed<T> {
    fn apply<Q: QueryTuple>(query: DynQuery<'_, Q>) -> DynQuery<'_, Q> {
        query.changed::<T>()
    }
}

/// Restricts a query to entities that gained component `T` since the last
/// step. `T` must appear in the query tuple.
pub struct Added<T>(PhantomData<fn() -> T>);

impl<T: Send + Sync + Default + 'static> QueryFilter for Added<T> {
    fn apply<Q: QueryTuple>(query: DynQuery<'_, Q>) -> DynQuery<'_, Q> {
        query.added::<T>()
    }
}

/// Restricts a query to entities carrying the marker tag type `T`.
pub struct WithTag<T>(PhantomData<fn() -> T>);

impl<T: 'static> QueryFilter for WithTag<T> {
    fn apply<Q: QueryTuple>(query: DynQuery<'_, Q>) -> DynQuery<'_, Q> {
        query.with_tag_type::<T>()
    }
}

/// Restricts a query to entities not carrying the marker tag type `T`.
pub struct WithoutTag<T>(PhantomData<fn() -> T>);

impl<T: 'static> QueryFilter for WithoutTag<T> {
    fn apply<Q: QueryTuple>(query: DynQuery<'_, Q>) -> DynQuery<'_, Q> {
        query.without_tag_type::<T>()
    }
}

macro_rules! impl_query_filter_tuple {
    ($($filter:ident),+) => {
        impl<$($filter: QueryFilter),+> QueryFilter for ($($filter,)+) {
            fn apply<Q: QueryTuple>(query: DynQuery<'_, Q>) -> DynQuery<'_, Q> {
                $(let query = $filter::apply(query);)+
                query
            }
        }
    };
}

impl_query_filter_tuple!(A);
impl_query_filter_tuple!(A, B);
impl_query_filter_tuple!(A, B, C);
impl_query_filter_tuple!(A, B, C, D);

enum QueryState<'world, Q: QueryTuple> {
    Eager(DynQuery<'world, Q>),
    Lazy(&'world RefCell<&'world mut DynWorld>),
    Join(DynJoin<'world, Q>),
}

/// A query system parameter over tuple `Q` with type-level filter `F`. The
/// filter defaults to none. A single query parameter borrows the world
/// directly; several query parameters in one system share the world through
/// a cell and take it one [`for_each`](Self::for_each) at a time.
pub struct Query<'world, Q: QueryTuple, F: QueryFilter = ()> {
    state: QueryState<'world, Q>,
    filter: PhantomData<fn() -> F>,
}

impl<'world, Q: QueryTuple, F: QueryFilter> Query<'world, Q, F> {
    /// Visits every matching entity with its fetched components.
    pub fn for_each(self, f: impl for<'item> FnMut(Entity, Q::Item<'item>)) {
        match self.state {
            QueryState::Eager(query) => F::apply(query).for_each(f),
            QueryState::Lazy(cell) => {
                let mut guard = cell.borrow_mut();
                let world: &mut DynWorld = &mut guard;
                F::apply(world.query::<Q>()).for_each(f);
            }
            QueryState::Join(join) => join.for_each(f),
        }
    }

    /// The parallel form of [`for_each`](Self::for_each), table-granular.
    #[cfg(not(target_family = "wasm"))]
    pub fn par_for_each<Fun>(self, f: Fun)
    where
        Fun: for<'item> Fn(Entity, Q::Item<'item>) + Send + Sync,
    {
        match self.state {
            QueryState::Eager(query) => F::apply(query).par_for_each(f),
            QueryState::Lazy(cell) => {
                let mut guard = cell.borrow_mut();
                let world: &mut DynWorld = &mut guard;
                F::apply(world.query::<Q>()).par_for_each(f);
            }
            QueryState::Join(join) => join.par_for_each(f),
        }
    }
}

/// A world-borrowing system parameter in the single-query slot, resolved
/// after the resource parameters are lifted out. Implemented for [`Query`]
/// and [`ParamSet`].
pub trait WorldParam {
    /// The parameter value handed to the system for a given borrow.
    type Item<'world>;
    /// Resolves the parameter against the world.
    fn build(world: &mut DynWorld) -> Self::Item<'_>;
}

impl<'a, Q: QueryTuple, F: QueryFilter> WorldParam for Query<'a, Q, F> {
    type Item<'world> = Query<'world, Q, F>;
    fn build(world: &mut DynWorld) -> Query<'_, Q, F> {
        Query {
            state: QueryState::Eager(world.query::<Q>()),
            filter: PhantomData,
        }
    }
}

/// A query parameter resolved against a [`DynEcs`] group through
/// [`query_join`](crate::dynamic::DynEcs::query_join), so a system over a
/// `Schedule<DynEcs>` can take a `Query`. Group queries join across member
/// worlds under the driver rule, and carry no type-level filter, so only the
/// unfiltered [`Query<Q>`](Query) is an `EcsParam`. Filter a group query by
/// taking the group as a `&mut DynEcs` host argument and calling
/// `query_join` with its builder methods.
pub trait EcsParam {
    /// The parameter value handed to the system for a given borrow.
    type Item<'ecs>;
    /// Resolves the parameter against the group.
    fn build_ecs(ecs: &mut DynEcs) -> Self::Item<'_>;
}

impl<'a, Q: QueryTuple> EcsParam for Query<'a, Q, ()> {
    type Item<'ecs> = Query<'ecs, Q, ()>;
    fn build_ecs(ecs: &mut DynEcs) -> Query<'_, Q, ()> {
        Query {
            state: QueryState::Join(ecs.query_join::<Q>()),
            filter: PhantomData,
        }
    }
}

/// A query parameter that shares the world with sibling query parameters
/// through a cell, so a system can take several [`Query`] arguments. Each
/// query borrows the world only for the span of one
/// [`for_each`](Query::for_each), so two queries never iterate at once.
pub trait MultiQueryParam {
    /// The parameter value handed to the system for a given borrow.
    type Item<'world>;
    /// Resolves the parameter against the shared world cell.
    fn build_lazy<'world>(cell: &'world RefCell<&'world mut DynWorld>) -> Self::Item<'world>;
}

impl<'a, Q: QueryTuple, F: QueryFilter> MultiQueryParam for Query<'a, Q, F> {
    type Item<'world> = Query<'world, Q, F>;
    fn build_lazy<'world>(cell: &'world RefCell<&'world mut DynWorld>) -> Query<'world, Q, F> {
        Query {
            state: QueryState::Lazy(cell),
            filter: PhantomData,
        }
    }
}

/// A set of conflicting query parameters accessed one at a time. Holds the
/// world and lends each member query through [`p0`](Self::p0),
/// [`p1`](Self::p1), and so on, so two mutable queries never live at once.
pub struct ParamSet<'world, T> {
    world: &'world mut DynWorld,
    marker: PhantomData<fn() -> T>,
}

impl<'a, T: 'static> WorldParam for ParamSet<'a, T>
where
    ParamSet<'a, T>: ParamSetAccess,
{
    type Item<'world> = ParamSet<'world, T>;
    fn build(world: &mut DynWorld) -> ParamSet<'_, T> {
        ParamSet {
            world,
            marker: PhantomData,
        }
    }
}

/// Marks the [`ParamSet`] tuples that expose member accessors.
pub trait ParamSetAccess {}

impl<P0: WorldParam, P1: WorldParam> ParamSetAccess for ParamSet<'_, (P0, P1)> {}

impl<'world, P0: WorldParam, P1: WorldParam> ParamSet<'world, (P0, P1)> {
    /// Borrows the world for the first member query.
    pub fn p0(&mut self) -> P0::Item<'_> {
        P0::build(self.world)
    }

    /// Borrows the world for the second member query.
    pub fn p1(&mut self) -> P1::Item<'_> {
        P1::build(self.world)
    }
}

impl<P0: WorldParam, P1: WorldParam, P2: WorldParam> ParamSetAccess for ParamSet<'_, (P0, P1, P2)> {}

impl<'world, P0: WorldParam, P1: WorldParam, P2: WorldParam> ParamSet<'world, (P0, P1, P2)> {
    /// Borrows the world for the first member query.
    pub fn p0(&mut self) -> P0::Item<'_> {
        P0::build(self.world)
    }

    /// Borrows the world for the second member query.
    pub fn p1(&mut self) -> P1::Item<'_> {
        P1::build(self.world)
    }

    /// Borrows the world for the third member query.
    pub fn p2(&mut self) -> P2::Item<'_> {
        P2::build(self.world)
    }
}

/// Converts a system-parameter function into a runner the
/// [`Schedule`] accepts, over a world type `W`. `Marker` is inferred from
/// the function's parameter types. Resource-only systems (`fn(Res<A>,
/// ResMut<B>)`) and systems that take the host as a final `&mut W` argument
/// work over any [`ResourceHost`]; systems with [`Query`]/[`ParamSet`]
/// parameters resolve against [`DynWorld`] specifically. Register through
/// [`ScheduleExt::add_system`].
pub trait IntoSystem<W, Marker>: Sized {
    /// Wraps the function into a closure that resolves its parameters from
    /// the world on each call.
    fn into_runner(self) -> impl FnMut(&mut W) + Send + 'static;
}

/// Registers system-parameter functions on a [`Schedule<W>`](crate::Schedule),
/// resolving [`Res`]/[`ResMut`]/[`Query`] parameters per run.
pub trait ScheduleExt<W> {
    /// Adds a system-parameter function under `name`, like
    /// [`Schedule::push`](crate::Schedule::push) for plain systems. Returns
    /// `&mut Self`, so calls chain in builder style.
    fn add_system<Marker>(
        &mut self,
        name: &'static str,
        system: impl IntoSystem<W, Marker>,
    ) -> &mut Self;

    /// Adds a system-parameter function that runs only on the passes where
    /// `condition` holds, the param-system form of
    /// [`Schedule::push_if`](crate::Schedule::push_if). The condition reads
    /// the world each pass; a false skips the system for that pass without
    /// removing it. This is how a run condition composes with resource,
    /// query, and event parameters, so gating a system on a state the host
    /// carries stays declarative: pass `|world| in_state(world, ...)` as the
    /// condition. The state machinery itself belongs to the host, not here.
    fn add_system_if<Marker>(
        &mut self,
        name: &'static str,
        condition: impl Fn(&W) -> bool + Send + 'static,
        system: impl IntoSystem<W, Marker>,
    ) -> &mut Self;

    /// Adds a tuple of system-parameter functions in one call, each named
    /// after its function type, so `schedule.add_systems((movement, score))`
    /// stands in for one [`add_system`](Self::add_system) per system. Reach
    /// for `add_system` when a system needs an explicit name for later
    /// [`replace`](crate::Schedule::replace) or removal.
    fn add_systems<Marker>(&mut self, systems: impl IntoSystems<W, Marker>) -> &mut Self;
}

impl<W> ScheduleExt<W> for Schedule<W> {
    fn add_system<Marker>(
        &mut self,
        name: &'static str,
        system: impl IntoSystem<W, Marker>,
    ) -> &mut Self {
        self.push(name, system.into_runner())
    }

    fn add_system_if<Marker>(
        &mut self,
        name: &'static str,
        condition: impl Fn(&W) -> bool + Send + 'static,
        system: impl IntoSystem<W, Marker>,
    ) -> &mut Self {
        self.push_if(name, condition, system.into_runner())
    }

    fn add_systems<Marker>(&mut self, systems: impl IntoSystems<W, Marker>) -> &mut Self {
        systems.register(self);
        self
    }
}

fn add_named_system<W, Marker>(schedule: &mut Schedule<W>, system: impl IntoSystem<W, Marker>) {
    let base = std::any::type_name_of_val(&system);
    let name = if schedule.contains(base) {
        let mut suffix = 2;
        loop {
            let candidate = format!("{base}#{suffix}");
            if !schedule.contains(&candidate) {
                break Box::leak(candidate.into_boxed_str()) as &'static str;
            }
            suffix += 1;
        }
    } else {
        base
    };
    schedule.push(name, system.into_runner());
}

/// A tuple of system-parameter functions registered together by
/// [`ScheduleExt::add_systems`]. Implemented for tuples of up to eight
/// systems, each of which is an [`IntoSystem`].
pub trait IntoSystems<W, Marker> {
    /// Registers each system in the tuple onto `schedule`.
    fn register(self, schedule: &mut Schedule<W>);
}

macro_rules! impl_into_systems {
    ($(($system:ident, $marker:ident)),+) => {
        impl<W, $($system, $marker,)+> IntoSystems<W, ($($marker,)+)> for ($($system,)+)
        where
            $($system: IntoSystem<W, $marker>,)+
        {
            #[allow(non_snake_case)]
            fn register(self, schedule: &mut Schedule<W>) {
                let ($($system,)+) = self;
                $(add_named_system(schedule, $system);)+
            }
        }
    };
}

impl_into_systems!((S0, M0));
impl_into_systems!((S0, M0), (S1, M1));
impl_into_systems!((S0, M0), (S1, M1), (S2, M2));
impl_into_systems!((S0, M0), (S1, M1), (S2, M2), (S3, M3));
impl_into_systems!((S0, M0), (S1, M1), (S2, M2), (S3, M3), (S4, M4));
impl_into_systems!((S0, M0), (S1, M1), (S2, M2), (S3, M3), (S4, M4), (S5, M5));
impl_into_systems!(
    (S0, M0),
    (S1, M1),
    (S2, M2),
    (S3, M3),
    (S4, M4),
    (S5, M5),
    (S6, M6)
);
impl_into_systems!(
    (S0, M0),
    (S1, M1),
    (S2, M2),
    (S3, M3),
    (S4, M4),
    (S5, M5),
    (S6, M6),
    (S7, M7)
);

/// The marker for a system whose parameters are all extract parameters.
pub struct ExtractSystemMarker<E>(PhantomData<fn() -> E>);

/// The marker for a system with extract parameters and one trailing
/// world-borrowing parameter.
pub struct QuerySystemMarker<E, Q>(PhantomData<fn() -> (E, Q)>);

impl<Func, Q> IntoSystem<DynWorld, QuerySystemMarker<(), Q>> for Func
where
    Q: WorldParam,
    Func: FnMut(Q) + for<'world> FnMut(Q::Item<'world>) + Send + 'static,
{
    fn into_runner(mut self) -> impl FnMut(&mut DynWorld) + Send + 'static {
        move |world: &mut DynWorld| {
            let query = Q::build(world);
            self(query);
        }
    }
}

/// The marker for a system whose parameters are all extract parameters
/// followed by a final `&mut W` host argument.
pub struct HostSystemMarker<E>(PhantomData<fn() -> E>);

macro_rules! impl_extract_system {
    ($($param:ident $state:ident $owned:ident),+) => {
        impl<W, Func, $($param,)+> IntoSystem<W, ExtractSystemMarker<($($param,)+)>>
            for Func
        where
            W: 'static,
            $($param: ExtractParam<W>,)+
            Func: FnMut($($param,)+)
                + for<'item> FnMut($($param::Item<'item>,)+)
                + Send
                + 'static,
        {
            fn into_runner(mut self) -> impl FnMut(&mut W) + Send + 'static {
                $(let mut $state = <$param as ExtractParam<W>>::init();)+
                move |host: &mut W| {
                    $(let mut $owned = <$param as ExtractParam<W>>::extract(&mut $state, host);)+
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        self($(<$param as ExtractParam<W>>::build(&mut $owned),)+);
                    }));
                    $(<$param as ExtractParam<W>>::apply(&mut $state, $owned, host);)+
                    if let Err(panic) = result {
                        std::panic::resume_unwind(panic);
                    }
                }
            }
        }

        impl<W, Func, $($param,)+> IntoSystem<W, HostSystemMarker<($($param,)+)>>
            for Func
        where
            W: 'static,
            $($param: ExtractParam<W>,)+
            Func: FnMut($($param,)+ &mut W)
                + for<'item> FnMut($($param::Item<'item>,)+ &'item mut W)
                + Send
                + 'static,
        {
            fn into_runner(mut self) -> impl FnMut(&mut W) + Send + 'static {
                $(let mut $state = <$param as ExtractParam<W>>::init();)+
                move |host: &mut W| {
                    $(let mut $owned = <$param as ExtractParam<W>>::extract(&mut $state, host);)+
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        self($(<$param as ExtractParam<W>>::build(&mut $owned),)+ host);
                    }));
                    $(<$param as ExtractParam<W>>::apply(&mut $state, $owned, host);)+
                    if let Err(panic) = result {
                        std::panic::resume_unwind(panic);
                    }
                }
            }
        }
    };
}

macro_rules! impl_extract_query_system {
    ($($param:ident $state:ident $owned:ident),+) => {
        impl<Func, $($param,)+ Q>
            IntoSystem<DynWorld, QuerySystemMarker<($($param,)+), Q>> for Func
        where
            $($param: ExtractParam<DynWorld>,)+
            Q: WorldParam,
            Func: FnMut($($param,)+ Q)
                + for<'item> FnMut($($param::Item<'item>,)+ Q::Item<'item>)
                + Send
                + 'static,
        {
            fn into_runner(mut self) -> impl FnMut(&mut DynWorld) + Send + 'static {
                $(let mut $state = <$param as ExtractParam<DynWorld>>::init();)+
                move |world: &mut DynWorld| {
                    $(let mut $owned =
                        <$param as ExtractParam<DynWorld>>::extract(&mut $state, world);)+
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        let query = Q::build(world);
                        self($(<$param as ExtractParam<DynWorld>>::build(&mut $owned),)+ query);
                    }));
                    $(<$param as ExtractParam<DynWorld>>::apply(&mut $state, $owned, world);)+
                    if let Err(panic) = result {
                        std::panic::resume_unwind(panic);
                    }
                }
            }
        }
    };
}

impl_extract_system!(P0 state0 owned0);
impl_extract_system!(P0 state0 owned0, P1 state1 owned1);
impl_extract_system!(P0 state0 owned0, P1 state1 owned1, P2 state2 owned2);
impl_extract_system!(P0 state0 owned0, P1 state1 owned1, P2 state2 owned2, P3 state3 owned3);
impl_extract_system!(
    P0 state0 owned0, P1 state1 owned1, P2 state2 owned2, P3 state3 owned3, P4 state4 owned4
);
impl_extract_system!(
    P0 state0 owned0, P1 state1 owned1, P2 state2 owned2, P3 state3 owned3, P4 state4 owned4,
    P5 state5 owned5
);
impl_extract_system!(
    P0 state0 owned0, P1 state1 owned1, P2 state2 owned2, P3 state3 owned3, P4 state4 owned4,
    P5 state5 owned5, P6 state6 owned6
);
impl_extract_system!(
    P0 state0 owned0, P1 state1 owned1, P2 state2 owned2, P3 state3 owned3, P4 state4 owned4,
    P5 state5 owned5, P6 state6 owned6, P7 state7 owned7
);

impl_extract_query_system!(P0 state0 owned0);
impl_extract_query_system!(P0 state0 owned0, P1 state1 owned1);
impl_extract_query_system!(P0 state0 owned0, P1 state1 owned1, P2 state2 owned2);
impl_extract_query_system!(P0 state0 owned0, P1 state1 owned1, P2 state2 owned2, P3 state3 owned3);
impl_extract_query_system!(
    P0 state0 owned0, P1 state1 owned1, P2 state2 owned2, P3 state3 owned3, P4 state4 owned4
);
impl_extract_query_system!(
    P0 state0 owned0, P1 state1 owned1, P2 state2 owned2, P3 state3 owned3, P4 state4 owned4,
    P5 state5 owned5
);
impl_extract_query_system!(
    P0 state0 owned0, P1 state1 owned1, P2 state2 owned2, P3 state3 owned3, P4 state4 owned4,
    P5 state5 owned5, P6 state6 owned6
);

impl<Func, Q> IntoSystem<DynEcs, QuerySystemMarker<(), Q>> for Func
where
    Q: EcsParam,
    Func: FnMut(Q) + for<'ecs> FnMut(Q::Item<'ecs>) + Send + 'static,
{
    fn into_runner(mut self) -> impl FnMut(&mut DynEcs) + Send + 'static {
        move |ecs: &mut DynEcs| {
            let query = Q::build_ecs(ecs);
            self(query);
        }
    }
}

macro_rules! impl_extract_ecs_query_system {
    ($($param:ident $state:ident $owned:ident),+) => {
        impl<Func, $($param,)+ Q>
            IntoSystem<DynEcs, QuerySystemMarker<($($param,)+), Q>> for Func
        where
            $($param: ExtractParam<DynEcs>,)+
            Q: EcsParam,
            Func: FnMut($($param,)+ Q)
                + for<'item> FnMut($($param::Item<'item>,)+ Q::Item<'item>)
                + Send
                + 'static,
        {
            fn into_runner(mut self) -> impl FnMut(&mut DynEcs) + Send + 'static {
                $(let mut $state = <$param as ExtractParam<DynEcs>>::init();)+
                move |ecs: &mut DynEcs| {
                    $(let mut $owned = <$param as ExtractParam<DynEcs>>::extract(&mut $state, ecs);)+
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        let query = Q::build_ecs(ecs);
                        self($(<$param as ExtractParam<DynEcs>>::build(&mut $owned),)+ query);
                    }));
                    $(<$param as ExtractParam<DynEcs>>::apply(&mut $state, $owned, ecs);)+
                    if let Err(panic) = result {
                        std::panic::resume_unwind(panic);
                    }
                }
            }
        }
    };
}

impl_extract_ecs_query_system!(P0 state0 owned0);
impl_extract_ecs_query_system!(P0 state0 owned0, P1 state1 owned1);
impl_extract_ecs_query_system!(P0 state0 owned0, P1 state1 owned1, P2 state2 owned2);
impl_extract_ecs_query_system!(
    P0 state0 owned0, P1 state1 owned1, P2 state2 owned2, P3 state3 owned3
);
impl_extract_ecs_query_system!(
    P0 state0 owned0, P1 state1 owned1, P2 state2 owned2, P3 state3 owned3, P4 state4 owned4
);
impl_extract_ecs_query_system!(
    P0 state0 owned0, P1 state1 owned1, P2 state2 owned2, P3 state3 owned3, P4 state4 owned4,
    P5 state5 owned5
);
impl_extract_ecs_query_system!(
    P0 state0 owned0, P1 state1 owned1, P2 state2 owned2, P3 state3 owned3, P4 state4 owned4,
    P5 state5 owned5, P6 state6 owned6
);

/// The marker for a system with resource parameters and two or more
/// world-borrowing query parameters resolved through a shared world cell.
pub struct MultiQuerySystemMarker<R, Q>(PhantomData<fn() -> (R, Q)>);

macro_rules! impl_multi_query_bare {
    ($($query:ident $qbind:ident),+) => {
        impl<Func, $($query,)+>
            IntoSystem<DynWorld, MultiQuerySystemMarker<(), ($($query,)+)>> for Func
        where
            $($query: MultiQueryParam,)+
            Func: FnMut($($query,)+)
                + for<'world> FnMut($($query::Item<'world>,)+)
                + Send
                + 'static,
        {
            #[allow(non_snake_case)]
            fn into_runner(mut self) -> impl FnMut(&mut DynWorld) + Send + 'static {
                move |world: &mut DynWorld| {
                    let cell = RefCell::new(world);
                    $(let $qbind = $query::build_lazy(&cell);)+
                    self($($qbind,)+);
                }
            }
        }
    };
}

macro_rules! impl_extract_multi_query {
    (($($param:ident $state:ident $owned:ident),+), ($($query:ident $qbind:ident),+)) => {
        impl<Func, $($param,)+ $($query,)+>
            IntoSystem<DynWorld, MultiQuerySystemMarker<($($param,)+), ($($query,)+)>> for Func
        where
            $($param: ExtractParam<DynWorld>,)+
            $($query: MultiQueryParam,)+
            Func: FnMut($($param,)+ $($query,)+)
                + for<'item> FnMut($($param::Item<'item>,)+ $($query::Item<'item>,)+)
                + Send
                + 'static,
        {
            fn into_runner(mut self) -> impl FnMut(&mut DynWorld) + Send + 'static {
                $(let mut $state = <$param as ExtractParam<DynWorld>>::init();)+
                move |world: &mut DynWorld| {
                    $(let mut $owned =
                        <$param as ExtractParam<DynWorld>>::extract(&mut $state, world);)+
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        let cell = RefCell::new(&mut *world);
                        $(let $qbind = $query::build_lazy(&cell);)+
                        self(
                            $(<$param as ExtractParam<DynWorld>>::build(&mut $owned),)+
                            $($qbind,)+
                        );
                    }));
                    $(<$param as ExtractParam<DynWorld>>::apply(&mut $state, $owned, world);)+
                    if let Err(panic) = result {
                        std::panic::resume_unwind(panic);
                    }
                }
            }
        }
    };
}

impl_multi_query_bare!(Q0 q0, Q1 q1);
impl_multi_query_bare!(Q0 q0, Q1 q1, Q2 q2);
impl_multi_query_bare!(Q0 q0, Q1 q1, Q2 q2, Q3 q3);

impl_extract_multi_query!((P0 state0 owned0), (Q0 q0, Q1 q1));
impl_extract_multi_query!((P0 state0 owned0), (Q0 q0, Q1 q1, Q2 q2));
impl_extract_multi_query!((P0 state0 owned0, P1 state1 owned1), (Q0 q0, Q1 q1));
impl_extract_multi_query!((P0 state0 owned0, P1 state1 owned1), (Q0 q0, Q1 q1, Q2 q2));
impl_extract_multi_query!((P0 state0 owned0, P1 state1 owned1, P2 state2 owned2), (Q0 q0, Q1 q1));
impl_extract_multi_query!(
    (P0 state0 owned0, P1 state1 owned1, P2 state2 owned2),
    (Q0 q0, Q1 q1, Q2 q2)
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Schedule;
    use crate::dynamic::{ComponentRegistry, DynEcs, DynWorld, ResourceMap};

    struct Engine {
        resources: ResourceMap,
        frames: u32,
    }

    impl ResourceHost for Engine {
        fn resource_map_mut(&mut self) -> &mut ResourceMap {
            &mut self.resources
        }
        fn resource_map(&self) -> &ResourceMap {
            &self.resources
        }
    }

    #[derive(Default, Clone, Debug, PartialEq)]
    struct Position {
        x: f32,
        y: f32,
    }

    #[derive(Default, Clone, Debug, PartialEq)]
    struct Velocity {
        x: f32,
        y: f32,
    }

    #[derive(Default, Clone, Debug, PartialEq)]
    struct Health {
        value: f32,
    }

    crate::impl_component!(Position, Velocity, Health);

    struct DeltaTime(f32);
    struct Score(u32);
    struct Tally(u32);
    struct Seen(Vec<u32>);
    struct Enabled(bool);
    struct Frozen;

    #[derive(Clone, Debug, PartialEq)]
    struct Collision {
        entity: u32,
    }

    fn run<Marker>(world: &mut DynWorld, system: impl IntoSystem<DynWorld, Marker>) {
        let mut runner = system.into_runner();
        runner(world);
    }

    #[test]
    fn single_resource_param() {
        let mut world = DynWorld::new();
        world.insert_resource(Score(0));
        run(&mut world, |mut score: ResMut<Score>| score.0 += 5);
        assert_eq!(world.resource::<Score>().unwrap().0, 5);
    }

    #[test]
    fn multiple_resource_params() {
        let mut world = DynWorld::new();
        world.insert_resource(DeltaTime(2.0));
        world.insert_resource(Score(1));
        run(
            &mut world,
            |delta: Res<DeltaTime>, mut score: ResMut<Score>| {
                score.0 += delta.0 as u32;
            },
        );
        assert_eq!(world.resource::<Score>().unwrap().0, 3);
    }

    #[test]
    fn query_only_param() {
        let mut world = DynWorld::new();
        world.spawn((Position { x: 1.0, y: 2.0 },));
        run(&mut world, |query: Query<&mut Position>| {
            query.for_each(|_entity, position| position.x += 10.0);
        });
        let position = world.query_ref::<&Position>().single().unwrap().1.clone();
        assert_eq!(position, Position { x: 11.0, y: 2.0 });
    }

    #[test]
    fn resources_and_query_together() {
        let mut world = DynWorld::new();
        world.insert_resource(DeltaTime(0.5));
        world.insert_resource(Score(0));
        world.spawn((
            Position { x: 0.0, y: 0.0 },
            Velocity { x: 2.0, y: 4.0 },
            Health { value: 100.0 },
        ));
        world.spawn((Position { x: 5.0, y: 5.0 }, Velocity { x: 1.0, y: 1.0 }));

        run(
            &mut world,
            |delta: Res<DeltaTime>,
             mut score: ResMut<Score>,
             query: Query<(&mut Position, &Velocity, Option<&mut Health>)>| {
                query.for_each(|_entity, (position, velocity, health)| {
                    position.x += velocity.x * delta.0;
                    position.y += velocity.y * delta.0;
                    if let Some(health) = health {
                        health.value *= 0.9;
                    }
                });
                score.0 += 1;
            },
        );

        assert_eq!(world.resource::<Score>().unwrap().0, 1);
        let mut positions: Vec<(f32, f32)> = world
            .query_ref::<&Position>()
            .iter()
            .map(|(_entity, position)| (position.x, position.y))
            .collect();
        positions.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        assert_eq!(positions, vec![(1.0, 2.0), (5.5, 5.5)]);
    }

    #[test]
    fn with_filter_narrows_match() {
        let mut world = DynWorld::new();
        let frozen = world.spawn((Position { x: 0.0, y: 0.0 },));
        world.add_tag_type::<Frozen>(frozen);
        world.spawn((Position { x: 0.0, y: 0.0 },));

        run(&mut world, |query: Query<&mut Position, With<Health>>| {
            query.for_each(|_entity, position| position.x = 1.0);
        });
        assert!(
            world
                .query_ref::<&Position>()
                .iter()
                .all(|(_entity, position)| position.x == 0.0)
        );

        run(
            &mut world,
            |query: Query<&mut Position, WithoutTag<Frozen>>| {
                query.for_each(|_entity, position| position.x = 2.0);
            },
        );
        let moved = world
            .query_ref::<&Position>()
            .iter()
            .filter(|(_entity, position)| position.x == 2.0)
            .count();
        assert_eq!(moved, 1);
    }

    #[test]
    fn changed_filter_visits_only_mutated() {
        let mut world = DynWorld::new();
        world.set_change_detection(true);
        let entity = world.spawn((Position { x: 0.0, y: 0.0 },));
        world.spawn((Position { x: 0.0, y: 0.0 },));
        world.step();
        world.get_mut::<Position>(entity).unwrap().x = 1.0;

        let mut visited = 0;
        run(&mut world, |query: Query<&Position, Changed<Position>>| {
            query.for_each(|_entity, _position| {});
        });
        world
            .query_ref::<&Position>()
            .changed::<Position>()
            .iter()
            .for_each(|_| visited += 1);
        assert_eq!(visited, 1);
    }

    #[test]
    fn param_set_lends_conflicting_queries() {
        let mut world = DynWorld::new();
        world.spawn((Position { x: 1.0, y: 1.0 }, Velocity { x: 3.0, y: 3.0 }));

        run(
            &mut world,
            |mut set: ParamSet<(Query<&mut Position>, Query<&Velocity>)>| {
                let mut velocity = (0.0, 0.0);
                set.p1()
                    .for_each(|_entity, value| velocity = (value.x, value.y));
                set.p0().for_each(|_entity, position| {
                    position.x += velocity.0;
                    position.y += velocity.1;
                });
            },
        );

        let position = world.query_ref::<&Position>().single().unwrap().1.clone();
        assert_eq!(position, Position { x: 4.0, y: 4.0 });
    }

    #[test]
    fn two_direct_queries_disjoint_components() {
        let mut world = DynWorld::new();
        world.spawn((Position { x: 1.0, y: 1.0 }, Velocity { x: 3.0, y: 4.0 }));

        run(
            &mut world,
            |positions: Query<&mut Position>, velocities: Query<&Velocity>| {
                let mut sample = (0.0, 0.0);
                velocities.for_each(|_entity, velocity| sample = (velocity.x, velocity.y));
                positions.for_each(|_entity, position| {
                    position.x += sample.0;
                    position.y += sample.1;
                });
            },
        );

        let position = world.query_ref::<&Position>().single().unwrap().1.clone();
        assert_eq!(position, Position { x: 4.0, y: 5.0 });
    }

    #[test]
    fn two_direct_queries_same_component_run_sequentially() {
        let mut world = DynWorld::new();
        world.spawn((Position { x: 0.0, y: 0.0 },));

        run(
            &mut world,
            |first: Query<&mut Position>, second: Query<&mut Position>| {
                first.for_each(|_entity, position| position.x += 1.0);
                second.for_each(|_entity, position| position.x += 10.0);
            },
        );

        assert_eq!(world.query_ref::<&Position>().single().unwrap().1.x, 11.0);
    }

    #[test]
    fn resource_with_two_direct_queries() {
        let mut world = DynWorld::new();
        world.insert_resource(Score(0));
        world.spawn((Position { x: 0.0, y: 0.0 }, Velocity { x: 2.0, y: 0.0 }));

        run(
            &mut world,
            |mut score: ResMut<Score>,
             positions: Query<&mut Position>,
             velocities: Query<&Velocity>| {
                let mut velocity_x = 0.0;
                velocities.for_each(|_entity, velocity| velocity_x = velocity.x);
                positions.for_each(|_entity, position| position.x += velocity_x);
                score.0 += 1;
            },
        );

        assert_eq!(world.resource::<Score>().unwrap().0, 1);
        assert_eq!(world.query_ref::<&Position>().single().unwrap().1.x, 2.0);
    }

    #[test]
    fn schedule_add_system_runs_param_systems() {
        let mut world = DynWorld::new();
        world.insert_resource(Score(0));
        world.spawn((Position { x: 0.0, y: 0.0 }, Velocity { x: 1.0, y: 0.0 }));

        fn movement(query: Query<(&mut Position, &Velocity)>) {
            query.for_each(|_entity, (position, velocity)| position.x += velocity.x);
        }
        fn scoring(mut score: ResMut<Score>) {
            score.0 += 1;
        }

        let mut schedule = Schedule::new();
        schedule.add_system("movement", movement);
        schedule.add_system("scoring", scoring);
        schedule.run(&mut world);
        schedule.run(&mut world);

        assert_eq!(world.resource::<Score>().unwrap().0, 2);
        let position = world.query_ref::<&Position>().single().unwrap().1.clone();
        assert_eq!(position, Position { x: 2.0, y: 0.0 });
    }

    #[test]
    fn add_systems_registers_a_tuple() {
        let mut world = DynWorld::new();
        world.insert_resource(Score(0));
        world.spawn((Position { x: 0.0, y: 0.0 }, Velocity { x: 1.0, y: 0.0 }));

        fn movement(query: Query<(&mut Position, &Velocity)>) {
            query.for_each(|_entity, (position, velocity)| position.x += velocity.x);
        }
        fn scoring(mut score: ResMut<Score>) {
            score.0 += 1;
        }

        let mut schedule = Schedule::new();
        schedule.add_systems((movement, scoring));
        assert_eq!(schedule.len(), 2);
        schedule.run(&mut world);

        assert_eq!(world.resource::<Score>().unwrap().0, 1);
        assert_eq!(world.query_ref::<&Position>().single().unwrap().1.x, 1.0);
    }

    #[test]
    fn add_systems_dedups_repeated_registration() {
        fn noop(_query: Query<&Position>) {}

        let mut schedule = Schedule::<DynWorld>::new();
        schedule.add_systems((noop,));
        schedule.add_systems((noop,));
        assert_eq!(schedule.len(), 2);
    }

    #[test]
    fn insert_resources_batches_a_tuple() {
        let mut world = DynWorld::new();
        world.insert_resources((DeltaTime(0.5), Score(7)));
        assert_eq!(world.resource::<DeltaTime>().unwrap().0, 0.5);
        assert_eq!(world.resource::<Score>().unwrap().0, 7);
    }

    #[test]
    fn resource_system_runs_on_custom_host() {
        let mut engine = Engine {
            resources: ResourceMap::default(),
            frames: 0,
        };
        engine.resources.insert(Score(0));

        let mut schedule = Schedule::<Engine>::new();
        schedule.add_system("bump", |mut score: ResMut<Score>| score.0 += 3);
        schedule.run(&mut engine);

        assert_eq!(engine.resources.get::<Score>().unwrap().0, 3);
    }

    #[test]
    fn host_system_gets_resources_and_the_host() {
        fn tick(mut score: ResMut<Score>, engine: &mut Engine) {
            score.0 += 1;
            engine.frames += 1;
        }

        let mut engine = Engine {
            resources: ResourceMap::default(),
            frames: 0,
        };
        engine.resources.insert(Score(10));

        let mut schedule = Schedule::<Engine>::new();
        schedule.add_system("tick", tick);
        schedule.run(&mut engine);
        schedule.run(&mut engine);

        assert_eq!(engine.resources.get::<Score>().unwrap().0, 12);
        assert_eq!(engine.frames, 2);
    }

    #[test]
    fn host_system_on_dynworld_can_query() {
        fn count_positions(mut score: ResMut<Score>, world: &mut DynWorld) {
            score.0 += world.query_ref::<&Position>().iter().count() as u32;
        }

        let mut world = DynWorld::new();
        world.insert_resource(Score(0));
        world.spawn((Position { x: 0.0, y: 0.0 },));
        world.spawn((Position { x: 1.0, y: 1.0 },));

        run(&mut world, count_positions);

        assert_eq!(world.resource::<Score>().unwrap().0, 2);
    }

    #[test]
    fn query_param_runs_over_a_group() {
        fn movement(query: Query<(&mut Position, &Velocity)>) {
            query.for_each(|_entity, (position, velocity)| position.x += velocity.x);
        }

        let mut registry = ComponentRegistry::new();
        registry.register::<Position>();
        registry.register::<Velocity>();
        let mut ecs = DynEcs::new();
        ecs.add_world(registry);
        let entity = ecs.spawn_with((Position { x: 1.0, y: 0.0 }, Velocity { x: 3.0, y: 0.0 }));

        let mut schedule = Schedule::<DynEcs>::new();
        schedule.add_system("movement", movement);
        schedule.run(&mut ecs);

        assert_eq!(ecs.get::<Position>(entity).unwrap().x, 4.0);
    }

    #[test]
    fn resource_and_query_over_a_group() {
        fn tick(mut score: ResMut<Score>, query: Query<(&mut Position, &Velocity)>) {
            query.for_each(|_entity, (position, velocity)| position.x += velocity.x);
            score.0 += 1;
        }

        let mut registry = ComponentRegistry::new();
        registry.register::<Position>();
        registry.register::<Velocity>();
        let mut ecs = DynEcs::new();
        ecs.add_world(registry);
        ecs.insert_resource(Score(0));
        let entity = ecs.spawn_with((Position { x: 0.0, y: 0.0 }, Velocity { x: 5.0, y: 0.0 }));

        let mut schedule = Schedule::<DynEcs>::new();
        schedule.add_system("tick", tick);
        schedule.run(&mut ecs);

        assert_eq!(ecs.resource::<Score>().unwrap().0, 1);
        assert_eq!(ecs.get::<Position>(entity).unwrap().x, 5.0);
    }

    #[test]
    fn added_filter_visits_only_new() {
        let mut world = DynWorld::new();
        world.set_change_detection(true);
        world.spawn((Position { x: 0.0, y: 0.0 },));
        world.step();
        world.spawn((Position { x: 9.0, y: 9.0 },));

        run(
            &mut world,
            |query: Query<&mut Position, Added<Position>>| {
                query.for_each(|_entity, position| position.y = 100.0);
            },
        );
        let marked = world
            .query_ref::<&Position>()
            .iter()
            .filter(|(_entity, position)| position.y == 100.0)
            .map(|(_entity, position)| position.x)
            .collect::<Vec<_>>();
        assert_eq!(marked, vec![9.0]);
    }

    #[test]
    fn tuple_filter_combines_constraints() {
        let mut world = DynWorld::new();
        world.spawn((Position { x: 0.0, y: 0.0 }, Health { value: 1.0 }));
        world.spawn((
            Position { x: 0.0, y: 0.0 },
            Health { value: 1.0 },
            Velocity { x: 0.0, y: 0.0 },
        ));

        run(
            &mut world,
            |query: Query<&mut Position, (With<Health>, Without<Velocity>)>| {
                query.for_each(|_entity, position| position.x = 1.0);
            },
        );
        let moved = world
            .query_ref::<&Position>()
            .iter()
            .filter(|(_entity, position)| position.x == 1.0)
            .count();
        assert_eq!(moved, 1);
    }

    #[test]
    fn par_for_each_runs_over_query() {
        let mut world = DynWorld::new();
        for index in 0..64 {
            world.spawn((
                Position {
                    x: index as f32,
                    y: 0.0,
                },
                Velocity { x: 1.0, y: 0.0 },
            ));
        }
        run(&mut world, |query: Query<(&mut Position, &Velocity)>| {
            query.par_for_each(|_entity, (position, velocity)| position.x += velocity.x);
        });
        let total: f32 = world
            .query_ref::<&Position>()
            .iter()
            .map(|(_entity, position)| position.x)
            .sum();
        assert_eq!(total, (0..64).map(|index| index as f32 + 1.0).sum());
    }

    #[test]
    #[should_panic(expected = "DeltaTime")]
    fn missing_resource_panics() {
        let mut world = DynWorld::new();
        run(&mut world, |_delta: Res<DeltaTime>| {});
    }

    fn emit_pair(mut writer: EventWriter<Collision>) {
        writer.send(Collision { entity: 1 });
        writer.send(Collision { entity: 2 });
    }

    #[test]
    fn writer_flushes_after_run_and_reader_consumes_once() {
        let mut world = DynWorld::new();
        world.insert_resource(Score(0));

        fn tally(reader: EventReader<Collision>, mut score: ResMut<Score>) {
            score.0 += reader.len() as u32;
        }

        let mut schedule = Schedule::new();
        schedule.add_system("emit", emit_pair);
        schedule.add_system("tally", tally);

        schedule.run(&mut world);
        assert_eq!(world.resource::<Score>().unwrap().0, 2);

        world.step();
        schedule.run(&mut world);
        assert_eq!(
            world.resource::<Score>().unwrap().0,
            4,
            "the reader consumes only the events sent this frame"
        );
    }

    #[test]
    fn reader_coexists_with_a_query() {
        let mut world = DynWorld::new();
        world.events.send::<Collision>(Collision { entity: 0 });
        world.spawn((Position { x: 0.0, y: 0.0 },));

        fn shift(reader: EventReader<Collision>, query: Query<&mut Position>) {
            let delta = reader.len() as f32;
            query.for_each(|_entity, position| position.x += delta);
        }

        let mut schedule = Schedule::new();
        schedule.add_system("shift", shift);
        schedule.run(&mut world);

        assert_eq!(world.query_ref::<&Position>().single().unwrap().1.x, 1.0);
    }

    #[test]
    fn resource_reader_and_query_in_one_system() {
        let mut world = DynWorld::new();
        world.insert_resource(DeltaTime(2.0));
        world.events.send::<Collision>(Collision { entity: 7 });
        world.spawn((Position { x: 0.0, y: 0.0 }, Velocity { x: 1.0, y: 0.0 }));

        fn integrate(
            delta: Res<DeltaTime>,
            reader: EventReader<Collision>,
            query: Query<(&mut Position, &Velocity)>,
        ) {
            let bump = reader.len() as f32;
            query.for_each(|_entity, (position, velocity)| {
                position.x += velocity.x * delta.0 + bump;
            });
        }

        let mut schedule = Schedule::new();
        schedule.add_system("integrate", integrate);
        schedule.run(&mut world);

        assert_eq!(world.query_ref::<&Position>().single().unwrap().1.x, 3.0);
    }

    #[test]
    fn writer_coexists_with_a_query() {
        let mut world = DynWorld::new();
        world.spawn((Position { x: 3.0, y: 0.0 },));
        world.spawn((Position { x: 5.0, y: 0.0 },));

        fn report(mut writer: EventWriter<Collision>, query: Query<&Position>) {
            query.for_each(|_entity, position| {
                writer.send(Collision {
                    entity: position.x as u32,
                });
            });
        }

        let mut schedule = Schedule::new();
        schedule.add_system("report", report);
        schedule.run(&mut world);

        let mut ids: Vec<u32> = world
            .events
            .read::<Collision>()
            .iter()
            .map(|collision| collision.entity)
            .collect();
        ids.sort();
        assert_eq!(ids, vec![3, 5]);
    }

    #[test]
    fn two_readers_keep_independent_cursors() {
        let mut world = DynWorld::new();
        world.insert_resource(Score(0));
        world.insert_resource(Tally(0));
        world.events.send::<Collision>(Collision { entity: 1 });
        world.events.send::<Collision>(Collision { entity: 2 });
        world.events.send::<Collision>(Collision { entity: 3 });

        fn reader_a(reader: EventReader<Collision>, mut score: ResMut<Score>) {
            score.0 += reader.len() as u32;
        }
        fn reader_b(reader: EventReader<Collision>, mut tally: ResMut<Tally>) {
            tally.0 += reader.len() as u32;
        }

        let mut schedule = Schedule::new();
        schedule.add_system("reader_a", reader_a);
        schedule.add_system("reader_b", reader_b);
        schedule.run(&mut world);

        assert_eq!(world.resource::<Score>().unwrap().0, 3);
        assert_eq!(world.resource::<Tally>().unwrap().0, 3);
    }

    #[test]
    fn reader_iterates_each_event_once() {
        let mut world = DynWorld::new();
        world.insert_resource(Seen(Vec::new()));
        world.events.send::<Collision>(Collision { entity: 10 });
        world.events.send::<Collision>(Collision { entity: 20 });

        fn collect(reader: EventReader<Collision>, mut seen: ResMut<Seen>) {
            for collision in &reader {
                seen.0.push(collision.entity);
            }
        }

        let mut schedule = Schedule::new();
        schedule.add_system("collect", collect);
        schedule.run(&mut world);

        assert_eq!(world.resource::<Seen>().unwrap().0, vec![10, 20]);
    }

    #[test]
    fn reader_is_empty_before_any_send() {
        let mut world = DynWorld::new();
        world.insert_resource(Score(0));

        fn tally(reader: EventReader<Collision>, mut score: ResMut<Score>) {
            assert!(reader.is_empty());
            score.0 += reader.len() as u32;
        }

        let mut schedule = Schedule::new();
        schedule.add_system("tally", tally);
        schedule.run(&mut world);

        assert_eq!(world.resource::<Score>().unwrap().0, 0);
    }

    #[test]
    fn events_flow_over_a_group() {
        let mut ecs = DynEcs::new();
        ecs.insert_resource(Score(0));

        fn tally(reader: EventReader<Collision>, mut score: ResMut<Score>) {
            score.0 += reader.len() as u32;
        }

        let mut schedule = Schedule::<DynEcs>::new();
        schedule.add_system("emit", emit_pair);
        schedule.add_system("tally", tally);
        schedule.run(&mut ecs);

        assert_eq!(ecs.resource::<Score>().unwrap().0, 2);
    }

    #[test]
    fn add_system_if_gates_a_param_system() {
        let mut world = DynWorld::new();
        world.insert_resource(Score(0));
        world.insert_resource(Enabled(false));

        fn bump(mut score: ResMut<Score>) {
            score.0 += 1;
        }

        let mut schedule = Schedule::new();
        schedule.add_system_if(
            "bump",
            |world: &DynWorld| world.resource::<Enabled>().unwrap().0,
            bump,
        );

        schedule.run(&mut world);
        assert_eq!(
            world.resource::<Score>().unwrap().0,
            0,
            "the system is skipped while the condition is false"
        );

        world.resource_mut::<Enabled>().unwrap().0 = true;
        schedule.run(&mut world);
        assert_eq!(
            world.resource::<Score>().unwrap().0,
            1,
            "the system runs once the condition holds"
        );
    }
}
