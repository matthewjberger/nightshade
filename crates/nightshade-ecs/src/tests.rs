use crate::access::{Access, SystemAccess};
use crate::system_param::{
    Commands, EventReader, EventWriter, MainThread, Query, Res, ResMut, ScheduleExt, With, Without,
};
use crate::*;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Default, Clone, Debug, PartialEq)]
struct Position(f32);

#[derive(Default, Clone, Debug, PartialEq)]
struct Velocity(f32);

#[derive(Default, Clone, Debug, PartialEq)]
struct Health(i32);

#[derive(Default, Clone, Debug, PartialEq)]
struct Label(String);

#[derive(Default, Clone, Debug, PartialEq)]
struct Zero;

struct Player;
struct Hidden;

struct Counter(u32);
struct Total(i32);

#[derive(Clone, Debug, PartialEq)]
struct Ping(u32);

struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1);
        self.0 >> 33
    }

    fn below(&mut self, limit: usize) -> usize {
        (self.next() as usize) % limit
    }
}

fn world_with(rows: &[(f32, f32)]) -> (World, Vec<Entity>) {
    let mut world = World::tracking();
    let entities = rows
        .iter()
        .map(|&(position, velocity)| world.spawn((Position(position), Velocity(velocity))))
        .collect();
    (world, entities)
}

fn positions(world: &World) -> Vec<f32> {
    let mut found: Vec<f32> = world
        .query_ref::<&Position>()
        .iter()
        .map(|(_entity, position)| position.0)
        .collect();
    found.sort_by(f32::total_cmp);
    found
}

// --- storage -----------------------------------------------------------------

#[test]
fn a_spawned_entity_holds_the_bundle_it_was_given() {
    let (world, entities) = world_with(&[(1.0, 2.0), (3.0, 4.0)]);
    assert_eq!(world.entity_count(), 2);
    assert_eq!(world.get::<Position>(entities[0]), Some(&Position(1.0)));
    assert_eq!(world.get::<Velocity>(entities[1]), Some(&Velocity(4.0)));
    assert!(world.has::<Position>(entities[0]));
    assert!(!world.has::<Health>(entities[0]));
}

#[test]
fn a_typed_write_reaches_the_column_it_names() {
    let (mut world, entities) = world_with(&[(1.0, 0.0)]);
    world.set(entities[0], Health(7));
    world.get_mut::<Position>(entities[0]).unwrap().0 = 9.0;
    assert_eq!(world.get::<Health>(entities[0]), Some(&Health(7)));
    assert_eq!(world.get::<Position>(entities[0]), Some(&Position(9.0)));
}

#[test]
fn a_set_of_an_absent_component_migrates_the_row_and_keeps_the_rest() {
    let (mut world, entities) = world_with(&[(1.0, 2.0), (3.0, 4.0)]);
    world.set(entities[0], Label("first".into()));
    assert_eq!(world.get::<Label>(entities[0]).unwrap().0, "first");
    assert_eq!(world.get::<Position>(entities[0]), Some(&Position(1.0)));
    assert_eq!(world.get::<Velocity>(entities[0]), Some(&Velocity(2.0)));
    assert_eq!(world.get::<Position>(entities[1]), Some(&Position(3.0)));
    assert!(world.get::<Label>(entities[1]).is_none());
}

#[test]
fn adding_and_removing_a_bundle_moves_the_row_between_tables() {
    let (mut world, entities) = world_with(&[(1.0, 2.0)]);
    assert!(world.add_components::<(Health, Label)>(entities[0]));
    assert!(world.entity_has_components::<(Position, Health, Label)>(entities[0]));
    assert!(world.remove_components::<(Health, Label)>(entities[0]));
    assert!(!world.has::<Health>(entities[0]));
    assert_eq!(world.get::<Position>(entities[0]), Some(&Position(1.0)));
}

#[test]
fn a_removed_component_leaves_the_others_in_place() {
    let (mut world, entities) = world_with(&[(1.0, 2.0)]);
    assert!(world.remove::<Velocity>(entities[0]));
    assert!(!world.has::<Velocity>(entities[0]));
    assert_eq!(world.get::<Position>(entities[0]), Some(&Position(1.0)));
}

#[test]
fn a_despawn_frees_the_handle_and_refuses_a_stale_one() {
    let (mut world, entities) = world_with(&[(1.0, 0.0), (2.0, 0.0)]);
    let stale = entities[0];
    assert_eq!(world.despawn_entities(&[stale]), vec![stale]);
    assert!(world.despawn_entities(&[stale]).is_empty());
    assert!(!world.is_alive(stale));
    assert!(world.get::<Position>(stale).is_none());
    assert_eq!(positions(&world), vec![2.0]);

    let reused = world.spawn((Position(5.0),));
    assert_eq!(reused.id, stale.id);
    assert_ne!(reused.generation, stale.generation);
    assert!(world.get::<Position>(stale).is_none());
    assert_eq!(world.get::<Position>(reused), Some(&Position(5.0)));
}

#[test]
fn a_batch_spawn_fills_every_row_with_defaults() {
    let mut world = World::tracking();
    let entities = world.spawn_entities::<(Position, Velocity)>(300);
    assert_eq!(entities.len(), 300);
    assert_eq!(world.entity_count(), 300);
    assert!(
        entities
            .iter()
            .all(|&entity| world.get::<Position>(entity) == Some(&Position(0.0)))
    );
}

#[derive(Clone, PartialEq, Debug)]
struct Bulk([u64; 64]);

impl Default for Bulk {
    fn default() -> Self {
        Bulk([0; 64])
    }
}

#[test]
fn a_large_archetype_splits_into_shards_that_still_address_each_row() {
    let mut world = World::tracking();
    let entities = world.spawn_entities::<(Bulk,)>(5_000);
    for (index, &entity) in entities.iter().enumerate() {
        world.get_mut::<Bulk>(entity).unwrap().0[0] = index as u64;
    }
    assert!(
        world.tables.len() > 1,
        "a table past its row budget must spill into a shard"
    );
    for (index, &entity) in entities.iter().enumerate() {
        assert_eq!(world.get::<Bulk>(entity).unwrap().0[0], index as u64);
    }
    assert_eq!(world.entity_count(), entities.len());
}

#[test]
fn a_zero_sized_component_still_joins_the_archetype() {
    let mut world = World::tracking();
    let entity = world.spawn((Zero, Position(1.0)));
    assert!(world.has::<Zero>(entity));
    assert_eq!(world.query_entities::<(Zero, Position)>().count(), 1);
    world.remove::<Zero>(entity);
    assert!(!world.has::<Zero>(entity));
}

static LIVE: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
struct Tracked(u32);

impl Default for Tracked {
    fn default() -> Self {
        LIVE.fetch_add(1, Ordering::SeqCst);
        Tracked(0)
    }
}

impl Clone for Tracked {
    fn clone(&self) -> Self {
        LIVE.fetch_add(1, Ordering::SeqCst);
        Tracked(self.0)
    }
}

impl Drop for Tracked {
    fn drop(&mut self) {
        LIVE.fetch_sub(1, Ordering::SeqCst);
    }
}

#[test]
fn every_column_value_is_dropped_exactly_once() {
    LIVE.store(0, Ordering::SeqCst);
    {
        let mut world = World::tracking();
        let entities = world.spawn_entities::<(Tracked, Position)>(500);
        assert_eq!(LIVE.load(Ordering::SeqCst), 500);
        for &entity in entities.iter().take(200) {
            world.set(entity, Label("moved".into()));
        }
        assert_eq!(LIVE.load(Ordering::SeqCst), 500);
        world.despawn_entities(&entities[..100]);
        assert_eq!(LIVE.load(Ordering::SeqCst), 400);
        for &entity in &entities[100..150] {
            world.remove::<Tracked>(entity);
        }
        assert_eq!(LIVE.load(Ordering::SeqCst), 350);
    }
    assert_eq!(LIVE.load(Ordering::SeqCst), 0);
}

#[test]
fn arena_growth_keeps_every_value_of_every_column() {
    let mut world = World::tracking();
    let mut entities = Vec::new();
    for index in 0..1000 {
        entities.push(world.spawn((Position(index as f32), Label(format!("{index}")))));
    }
    for (index, &entity) in entities.iter().enumerate() {
        assert_eq!(world.get::<Position>(entity), Some(&Position(index as f32)));
        assert_eq!(world.get::<Label>(entity).unwrap().0, format!("{index}"));
    }
}

// --- queries -----------------------------------------------------------------

#[test]
fn a_mutable_tuple_query_visits_every_matching_row() {
    let (mut world, _) = world_with(&[(1.0, 10.0), (2.0, 20.0), (3.0, 30.0)]);
    world
        .query::<(&mut Position, &Velocity)>()
        .for_each(|_entity, (position, velocity)| position.0 += velocity.0);
    assert_eq!(positions(&world), vec![11.0, 22.0, 33.0]);
}

#[test]
fn a_single_element_query_needs_no_tuple() {
    let (mut world, _) = world_with(&[(1.0, 0.0), (2.0, 0.0)]);
    world
        .query::<&mut Position>()
        .for_each(|_entity, position| position.0 *= 10.0);
    assert_eq!(positions(&world), vec![10.0, 20.0]);
    let read: f32 = world
        .query_ref::<&Position>()
        .iter()
        .map(|(_entity, position)| position.0)
        .sum();
    assert_eq!(read, 30.0);
}

#[test]
fn a_query_of_eight_elements_reaches_every_column() {
    #[derive(Default, Clone)]
    struct Slot(u8);
    macro_rules! slots {
        ($($name:ident),+) => { $( #[derive(Default, Clone)] struct $name(u8); )+ };
    }
    slots!(S1, S2, S3, S4, S5, S6, S7);

    let mut world = World::tracking();
    let entity = world.spawn((Slot(0), S1(1), S2(2), S3(3), S4(4), S5(5), S6(6), S7(7)));
    let mut seen = Vec::new();
    world
        .query::<(&Slot, &S1, &S2, &S3, &S4, &S5, &S6, &mut S7)>()
        .for_each(|found, (a, b, c, d, e, f, g, h)| {
            assert_eq!(found, entity);
            seen = vec![a.0, b.0, c.0, d.0, e.0, f.0, g.0, h.0];
            h.0 = 70;
        });
    assert_eq!(seen, vec![0, 1, 2, 3, 4, 5, 6, 7]);
    assert_eq!(world.get::<S7>(entity).map(|slot| slot.0), Some(70));
}

#[test]
fn an_optional_element_is_none_where_the_column_is_absent() {
    let mut world = World::tracking();
    let with_health = world.spawn((Position(1.0), Health(5)));
    let without_health = world.spawn((Position(2.0),));

    let mut seen: BTreeMap<u32, Option<i32>> = BTreeMap::new();
    world
        .query::<(&Position, Option<&Health>)>()
        .for_each(|entity, (_position, health)| {
            seen.insert(entity.id, health.map(|health| health.0));
        });
    assert_eq!(seen.get(&with_health.id), Some(&Some(5)));
    assert_eq!(seen.get(&without_health.id), Some(&None));

    let read: Vec<Option<i32>> = world
        .query_ref::<(&Position, Option<&Health>)>()
        .iter()
        .map(|(_entity, (_position, health))| health.map(|health| health.0))
        .collect();
    assert_eq!(read.len(), 2);
    assert!(read.contains(&Some(5)) && read.contains(&None));
}

#[test]
fn an_optional_mutable_element_writes_only_where_the_column_is_there() {
    let mut world = World::tracking();
    let with_health = world.spawn((Position(1.0), Health(5)));
    let plain = world.spawn((Position(2.0),));
    world
        .query::<(&Position, Option<&mut Health>)>()
        .for_each(|_entity, (_position, health)| {
            if let Some(health) = health {
                health.0 += 1;
            }
        });
    assert_eq!(world.get::<Health>(with_health), Some(&Health(6)));
    assert!(world.get::<Health>(plain).is_none());
}

#[test]
fn with_and_without_narrow_the_match() {
    let mut world = World::tracking();
    let both = world.spawn((Position(1.0), Velocity(0.0), Health(1)));
    let plain = world.spawn((Position(2.0), Velocity(0.0)));

    let mut kept = Vec::new();
    world
        .query::<&Position>()
        .with::<Health>()
        .for_each(|entity, _position| kept.push(entity));
    assert_eq!(kept, vec![both]);

    let mut dropped = Vec::new();
    world
        .query::<&Position>()
        .without::<Health>()
        .for_each(|entity, _position| dropped.push(entity));
    assert_eq!(dropped, vec![plain]);

    let read: Vec<Entity> = world
        .query_ref::<&Position>()
        .without::<Health>()
        .iter()
        .map(|(entity, _position)| entity)
        .collect();
    assert_eq!(read, vec![plain]);
}

#[test]
fn a_changed_filter_visits_only_the_rows_a_writer_touched() {
    let (mut world, entities) = world_with(&[(1.0, 0.0), (2.0, 0.0), (3.0, 0.0)]);
    world.step();
    world.get_mut::<Position>(entities[1]).unwrap().0 = 20.0;

    let mut touched = Vec::new();
    world
        .query::<&Position>()
        .changed::<Position>()
        .for_each(|entity, _position| touched.push(entity));
    assert_eq!(touched, vec![entities[1]]);

    let read: Vec<Entity> = world
        .query_ref::<&Position>()
        .changed::<Position>()
        .iter()
        .map(|(entity, _position)| entity)
        .collect();
    assert_eq!(read, vec![entities[1]]);
}

#[test]
fn an_added_filter_matches_a_spawn_and_a_new_component_only() {
    let mut world = World::tracking();
    world.step();
    let first = world.spawn((Position(1.0),));
    assert_eq!(collect_added(&mut world), vec![first]);

    world.step();
    assert!(collect_added(&mut world).is_empty());

    let late = world.spawn((Position(9.0),));
    assert_eq!(collect_added(&mut world), vec![late]);
}

fn collect_added(world: &mut World) -> Vec<Entity> {
    let mut found = Vec::new();
    world
        .query::<&Position>()
        .added::<Position>()
        .for_each(|entity, _position| found.push(entity));
    found
}

#[test]
#[should_panic(expected = "must name components present in the query tuple")]
fn a_changed_filter_outside_the_tuple_is_refused() {
    let (mut world, _) = world_with(&[(1.0, 0.0)]);
    world
        .query::<&Position>()
        .changed::<Velocity>()
        .for_each(|_entity, _position| {});
}

#[test]
#[should_panic(expected = "must not repeat a component type")]
fn a_query_tuple_that_repeats_a_component_is_refused() {
    let (mut world, _) = world_with(&[(1.0, 0.0)]);
    world
        .query::<(&mut Position, &Position)>()
        .for_each(|_entity, _row| {});
}

#[test]
fn for_entity_visits_one_row_and_obeys_its_filters() {
    let (mut world, entities) = world_with(&[(1.0, 0.0), (2.0, 0.0)]);
    world
        .query::<&mut Position>()
        .for_entity(entities[1], |position| position.0 = 99.0);
    assert_eq!(positions(&world), vec![1.0, 99.0]);

    let mut visited = false;
    world
        .query::<&Position>()
        .with::<Health>()
        .for_entity(entities[0], |_position| visited = true);
    assert!(!visited, "a filter that fails must skip the row");
}

#[test]
fn single_matches_exactly_one_row() {
    let mut world = World::tracking();
    assert!(world.query_ref::<&Position>().single().is_none());
    let only = world.spawn((Position(1.0),));
    assert_eq!(world.query_ref::<&Position>().single().unwrap().0, only);
    world.spawn((Position(2.0),));
    assert!(world.query_ref::<&Position>().single().is_none());
}

#[test]
fn a_read_query_of_an_unregistered_type_matches_nothing_and_registers_nothing() {
    let world = World::tracking();
    assert_eq!(world.query_ref::<&Position>().iter().count(), 0);
    assert_eq!(world.stats().component_count, 0);
}

#[test]
fn the_fast_path_and_the_filtered_path_see_the_same_rows() {
    let mut world = World::tracking();
    for index in 0..200 {
        let entity = world.spawn((Position(index as f32), Velocity(1.0)));
        if index % 3 == 0 {
            world.set(entity, Health(index));
        }
    }
    let mut fast = Vec::new();
    world
        .query::<(&Position, &Velocity)>()
        .for_each(|entity, _row| fast.push(entity));
    let mut filtered = Vec::new();
    world
        .query::<(&Position, &Velocity)>()
        .without::<Zero>()
        .for_each(|entity, _row| filtered.push(entity));
    fast.sort_by_key(|entity| entity.id);
    filtered.sort_by_key(|entity| entity.id);
    assert_eq!(fast, filtered);
    assert_eq!(fast.len(), 200);
}

#[test]
fn a_raw_pass_reaches_the_columns_of_every_matching_table() {
    let (mut world, _) = world_with(&[(1.0, 0.0), (2.0, 0.0)]);
    let key = world.component_key::<Position>();
    world.for_each_mut::<(Position,), ()>(|_entity, table, index| {
        table.column_mut(key)[index].0 += 1.0;
    });
    assert_eq!(positions(&world), vec![2.0, 3.0]);

    let mut total = 0.0;
    world.for_each::<(Position,), ()>(|_entity, table, index| {
        total += table.column(key)[index].0;
    });
    assert_eq!(total, 5.0);
}

// --- tags --------------------------------------------------------------------

#[test]
fn a_tag_marks_an_entity_without_moving_its_row() {
    let (mut world, entities) = world_with(&[(1.0, 0.0), (2.0, 0.0)]);
    let table_before = world.tables.len();
    world.add_tag_type::<Player>(entities[0]);
    assert_eq!(world.tables.len(), table_before);
    assert!(world.has_tag_type::<Player>(entities[0]));
    assert!(!world.has_tag_type::<Player>(entities[1]));
    assert_eq!(
        world.query_tag_type::<Player>().collect::<Vec<_>>(),
        vec![entities[0]]
    );
    assert!(world.remove_tag_type::<Player>(entities[0]));
    assert!(!world.has_tag_type::<Player>(entities[0]));
}

#[test]
fn a_tag_filter_narrows_a_query_on_both_sides() {
    let (mut world, entities) = world_with(&[(1.0, 0.0), (2.0, 0.0), (3.0, 0.0)]);
    world.add_tag_type::<Player>(entities[0]);
    world.add_tag_type::<Hidden>(entities[1]);

    let mut kept = Vec::new();
    world
        .query::<&Position>()
        .with_tag_type::<Player>()
        .for_each(|entity, _position| kept.push(entity));
    assert_eq!(kept, vec![entities[0]]);

    let mut rest: Vec<Entity> = world
        .query_ref::<&Position>()
        .without_tag_type::<Hidden>()
        .iter()
        .map(|(entity, _position)| entity)
        .collect();
    rest.sort_by_key(|entity| entity.id);
    assert_eq!(rest, vec![entities[0], entities[2]]);
}

#[test]
fn a_tag_filter_on_an_unregistered_type_matches_nothing() {
    let (world, _) = world_with(&[(1.0, 0.0)]);
    assert_eq!(
        world
            .query_ref::<&Position>()
            .with_tag_type::<Player>()
            .iter()
            .count(),
        0
    );
    assert_eq!(world.query_tag_type::<Player>().count(), 0);
}

#[test]
fn a_despawn_drops_every_tag_the_entity_held() {
    let (mut world, entities) = world_with(&[(1.0, 0.0)]);
    world.add_tag_type::<Player>(entities[0]);
    world.despawn_entities(&entities);
    assert!(!world.has_tag_type::<Player>(entities[0]));
    assert_eq!(world.query_tag_type::<Player>().count(), 0);
}

// --- masks -------------------------------------------------------------------

#[test]
fn a_mask_answers_the_set_questions_it_is_asked() {
    let mut mask = Mask::EMPTY;
    assert!(mask.is_empty());
    for index in [0u32, 5, 64, 200] {
        mask.insert(index);
    }
    assert_eq!(mask.indices().collect::<Vec<_>>(), vec![0, 5, 64, 200]);
    assert_eq!(mask.count(), 4);
    assert_eq!(mask.count_below(64), 2);
    assert!(mask.contains_bit(200) && !mask.contains_bit(201));
    assert!(mask.contains(Mask::bit(5)));
    assert!(mask.intersects(Mask::bit(64)));
    assert_eq!(mask.column_of(Mask::bit(64)), Some(2));
    assert_eq!(mask.column_of(Mask::bit(9)), None);
    assert_eq!(Mask::bit(7).single_slot(), Some(7));
    assert_eq!(mask.single_slot(), None);
    mask.remove(200);
    assert_eq!(mask.count(), 3);
    assert_eq!(Mask::lowest(3).indices().collect::<Vec<_>>(), vec![0, 1, 2]);
    assert_eq!(
        Mask::highest(2).indices().collect::<Vec<_>>(),
        vec![MASK_BITS - 2, MASK_BITS - 1]
    );
    assert_eq!((Mask::bit(1) | Mask::bit(2)) & Mask::bit(2), Mask::bit(2));
    assert!(!(!Mask::EMPTY).is_empty());
}

// --- change detection and the structural log ---------------------------------

#[test]
fn a_world_without_tracking_keeps_no_tick_columns_and_still_migrates() {
    let mut world = World::new();
    assert!(!world.change_detection());
    let entity = world.spawn((Position(1.0), Velocity(2.0)));
    world.set(entity, Health(3));
    assert_eq!(world.get::<Position>(entity), Some(&Position(1.0)));
    assert_eq!(world.get::<Health>(entity), Some(&Health(3)));
    assert!(world.structural_changes_since(0).is_empty());

    world.set_change_detection(true);
    world.step();
    world.get_mut::<Position>(entity).unwrap().0 = 5.0;
    let mut touched = Vec::new();
    world
        .query::<&Position>()
        .changed::<Position>()
        .for_each(|entity, _position| touched.push(entity));
    assert_eq!(touched, vec![entity]);
}

#[test]
fn the_structural_log_records_what_changed_the_shape_of_the_world() {
    let mut world = World::tracking();
    let entity = world.spawn((Position(1.0),));
    world.set(entity, Health(1));
    world.remove::<Health>(entity);
    world.add_tag_type::<Player>(entity);
    world.remove_tag_type::<Player>(entity);
    world.despawn_entities(&[entity]);

    let kinds: Vec<StructuralChangeKind> = world
        .structural_changes_since(0)
        .iter()
        .map(|change| change.kind)
        .collect();
    assert_eq!(
        kinds,
        vec![
            StructuralChangeKind::Spawned,
            StructuralChangeKind::ComponentsAdded,
            StructuralChangeKind::ComponentsRemoved,
            StructuralChangeKind::TagsAdded,
            StructuralChangeKind::TagsRemoved,
            StructuralChangeKind::Despawned,
        ]
    );
    let cursor = world.structural_sequence();
    assert!(world.structural_changes_since(cursor).is_empty());
    world.trim_structural_log(cursor);
    assert_eq!(world.stats().structural_log_entries, 0);
}

#[test]
fn a_change_mask_tells_which_bundle_a_log_entry_touches() {
    let mut world = World::tracking();
    let entity = world.spawn((Position(1.0),));
    world.set(entity, Health(1));
    let change = world.structural_changes_since(1)[0];
    assert!(world.change_touches::<(Health,)>(&change));
    assert!(!world.change_touches::<(Velocity,)>(&change));
}

#[test]
fn a_tick_comparison_survives_the_wrap_of_the_counter() {
    assert!(tick_is_newer(2, 1));
    assert!(!tick_is_newer(1, 2));
    assert!(tick_is_newer(0, u32::MAX));
}

// --- events ------------------------------------------------------------------

#[test]
fn an_event_channel_delivers_each_event_once_to_each_cursor() {
    let mut channel = EventChannel::<u32>::new();
    let (mut first, mut second) = (0, 0);
    channel.send(1);
    channel.send(2);
    assert_eq!(channel.consume(&mut first), &[1, 2]);
    assert!(channel.consume(&mut first).is_empty());
    assert_eq!(channel.consume(&mut second), &[1, 2]);
    channel.send(3);
    assert_eq!(channel.consume(&mut first), &[3]);
    assert_eq!(channel.len(), 3);
}

#[test]
fn an_event_lives_for_two_frames_and_then_expires() {
    let mut channel = EventChannel::<u32>::new();
    channel.send(1);
    channel.update();
    assert_eq!(channel.read_frame(), &[1]);
    channel.send(2);
    channel.update();
    assert_eq!(channel.read_frame(), &[2]);
    channel.update();
    assert!(channel.read_frame().is_empty());
}

#[test]
fn an_event_channel_drops_its_oldest_half_at_the_capacity_backstop() {
    let mut channel = EventChannel::<u32>::new();
    for index in 0..EVENT_CHANNEL_CAPACITY as u32 + 1 {
        channel.send(index);
    }
    assert!(channel.len() <= EVENT_CHANNEL_CAPACITY);
    let mut cursor = 0;
    assert!(!channel.consume(&mut cursor).is_empty());
    assert_eq!(cursor, channel.sequence());
}

#[test]
fn the_bus_keeps_one_channel_for_each_type() {
    let mut world = World::tracking();
    world.events.send(Ping(1));
    world.events.send(7u8);
    assert_eq!(world.events.read::<Ping>(), &[Ping(1)]);
    assert_eq!(world.events.read::<u8>(), &[7]);
    assert_eq!(world.events.channel_count(), 2);
    world.events.clear::<Ping>();
    assert!(world.events.read::<Ping>().is_empty());
}

// --- resources ---------------------------------------------------------------

#[test]
fn a_resource_is_a_singleton_the_world_lends_by_type() {
    let mut world = World::tracking();
    world.insert_resources((Counter(1), Total(2)));
    assert_eq!(world.res::<Counter>().0, 1);
    world.res_mut::<Total>().0 = 5;
    assert_eq!(world.resources.get::<Total>().map(|total| total.0), Some(5));
    assert_eq!(
        world.remove_resource::<Total>().map(|total| total.0),
        Some(5)
    );
    assert!(world.resources.get::<Total>().is_none());
    assert_eq!(world.resource_or_insert_default::<Zero>(), &mut Zero);
}

#[test]
#[should_panic(expected = "is missing")]
fn a_missing_resource_fails_loudly() {
    let world = World::tracking();
    world.res::<Counter>();
}

#[test]
fn a_resource_scope_lends_the_value_and_puts_it_back_even_after_a_panic() {
    let mut world = World::tracking();
    world.insert_resource(Counter(1));
    world.resource_scope(|world: &mut World, counter: &mut Counter| {
        counter.0 += 1;
        world.spawn((Position(1.0),));
    });
    assert_eq!(world.res::<Counter>().0, 2);

    let escaped = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        world.resource_scope(|_world: &mut World, counter: &mut Counter| {
            counter.0 = 9;
            panic!("the system failed");
        })
    }));
    assert!(escaped.is_err());
    assert_eq!(world.res::<Counter>().0, 9);
}

#[test]
fn a_resource_tuple_scope_takes_and_restores_every_member() {
    let mut world = World::tracking();
    world.insert_resources((Counter(1), Total(2)));
    world.resources_scope(
        |_world: &mut World, (counter, total): &mut (Counter, Total)| {
            counter.0 = 10;
            total.0 = 20;
        },
    );
    assert_eq!(world.res::<Counter>().0, 10);
    assert_eq!(world.res::<Total>().0, 20);
}

#[test]
fn disjoint_borrows_lend_several_resources_at_once() {
    let mut world = World::tracking();
    world.insert_resources((Counter(1), Total(2)));
    let [counter, total] = world.resources.res_disjoint_mut([
        std::any::TypeId::of::<Counter>(),
        std::any::TypeId::of::<Total>(),
    ]);
    counter.downcast_mut::<Counter>().unwrap().0 = 4;
    total.downcast_mut::<Total>().unwrap().0 = 8;
    assert_eq!(world.res::<Counter>().0, 4);
    assert_eq!(world.res::<Total>().0, 8);
}

// --- commands ----------------------------------------------------------------

#[test]
fn a_command_buffer_spawns_despawns_and_writes_when_it_is_applied() {
    let mut world = World::tracking();
    let doomed = world.spawn((Position(1.0),));
    let kept = world.spawn((Position(2.0),));

    let spawned = world.with_commands(|_world, commands| {
        commands.despawn(doomed);
        commands.set(kept, Position(20.0));
        commands.spawn((Position(3.0), Velocity(0.5)))
    });

    assert!(!world.is_alive(doomed));
    assert_eq!(world.get::<Position>(kept), Some(&Position(20.0)));
    assert!(world.is_alive(spawned));
    assert_eq!(world.get::<Position>(spawned), Some(&Position(3.0)));
    assert_eq!(world.get::<Velocity>(spawned), Some(&Velocity(0.5)));
}

#[test]
fn a_staged_component_is_readable_before_the_command_is_applied() {
    let mut world = World::tracking();
    world.with_commands(|_world, commands| {
        let entity = commands.spawn((Position(1.0),));
        assert_eq!(commands.staged::<Position>(entity), Some(&Position(1.0)));
        commands.staged_mut::<Position>(entity).unwrap().0 = 4.0;
        commands.restage::<Position>(entity);
    });
    assert_eq!(positions(&world), vec![4.0]);
}

#[test]
fn a_reserved_handle_is_live_before_its_row_exists() {
    let mut world = World::tracking();
    let entity = world.with_commands(|_world, commands| {
        let entity = commands.reserve_entity();
        commands.spawn_reserved(entity, (Position(7.0),));
        entity
    });
    assert!(world.is_alive(entity));
    assert_eq!(world.get::<Position>(entity), Some(&Position(7.0)));
}

#[test]
fn a_queued_closure_runs_against_the_storage_and_the_resources() {
    let mut world = World::tracking();
    world.insert_resource(Counter(0));
    let entity = world.spawn((Position(1.0),));
    world.queue_set(entity, Health(3));
    world.queue(|_storage, resources| resources.res_mut::<Counter>().0 = 5);
    assert_eq!(world.command_count(), 2);
    world.apply_commands();
    assert_eq!(world.get::<Health>(entity), Some(&Health(3)));
    assert_eq!(world.res::<Counter>().0, 5);
}

#[test]
fn commands_add_and_remove_what_they_are_told_to() {
    let mut world = World::tracking();
    let entity = world.spawn((Position(1.0), Velocity(1.0)));
    world.with_commands(|_world, commands| {
        commands.add_tag_type::<Player>(entity);
        commands.remove::<(Velocity,)>(entity);
        commands.insert_bundle(entity, (Health(4),));
    });
    assert!(world.has_tag_type::<Player>(entity));
    assert!(!world.has::<Velocity>(entity));
    assert_eq!(world.get::<Health>(entity), Some(&Health(4)));

    world.with_commands(|_world, commands| commands.remove_tag_type::<Player>(entity));
    assert!(!world.has_tag_type::<Player>(entity));
}

// --- hierarchy ---------------------------------------------------------------

#[test]
fn the_hierarchy_index_follows_links_relinks_and_unlinks() {
    let mut world = World::tracking();
    let mut index = HierarchyIndex::new();
    let root = world.spawn((Position(0.0),));
    let first = world.spawn((Position(1.0), ChildOf(root)));
    let second = world.spawn((Position(2.0), ChildOf(root)));
    let grandchild = world.spawn((Position(3.0), ChildOf(first)));
    index.sync(&mut world);
    assert_eq!(index.children(root), &[first, second]);
    assert_eq!(index.children(first), &[grandchild]);
    let mut descendants = index.descendants(root);
    descendants.sort_by_key(|entity| entity.id);
    assert_eq!(descendants, vec![first, second, grandchild]);

    world.set(second, ChildOf(first));
    index.sync(&mut world);
    assert_eq!(index.children(root), &[first]);
    assert_eq!(index.children(first), &[second, grandchild]);

    world.remove::<ChildOf>(second);
    index.sync(&mut world);
    assert_eq!(index.children(first), &[grandchild]);
}

#[test]
fn the_incremental_index_agrees_with_a_full_scan() {
    let mut world = World::tracking();
    let mut index = HierarchyIndex::new();
    let mut random = Lcg(7);
    let mut entities = vec![world.spawn((Position(0.0),))];

    for step in 0..200 {
        match step % 4 {
            0 => {
                let parent = entities[random.below(entities.len())];
                entities.push(world.spawn((Position(step as f32), ChildOf(parent))));
            }
            1 if entities.len() > 1 => {
                let child = entities[1 + random.below(entities.len() - 1)];
                let parent = entities[random.below(entities.len())];
                if child != parent {
                    world.set(child, ChildOf(parent));
                }
            }
            2 if entities.len() > 2 => {
                let victim = entities.remove(1 + random.below(entities.len() - 1));
                index.despawn_recursive(&mut world, victim);
                entities.retain(|entity| world.is_alive(*entity));
            }
            _ => {}
        }
        index.sync(&mut world);

        let mut oracle: std::collections::HashMap<Entity, Vec<Entity>> =
            std::collections::HashMap::new();
        for (entity, child_of) in world.query_ref::<&ChildOf>().iter() {
            oracle.entry(child_of.0).or_default().push(entity);
        }
        for (parent, mut children) in oracle {
            children.sort_by_key(|entity| entity.id);
            let mut held: Vec<Entity> = index
                .children(parent)
                .iter()
                .copied()
                .filter(|entity| world.is_alive(*entity))
                .collect();
            held.sort_by_key(|entity| entity.id);
            assert_eq!(
                held, children,
                "the index and a scan disagree at step {step}"
            );
        }
    }
}

#[test]
fn a_recursive_despawn_takes_the_whole_subtree() {
    let mut world = World::tracking();
    let root = world.spawn((Position(0.0),));
    let child = world.spawn((Position(1.0), ChildOf(root)));
    let grandchild = world.spawn((Position(2.0), ChildOf(child)));
    let bystander = world.spawn((Position(3.0),));

    let removed = world.despawn_recursive(root);
    assert_eq!(removed.len(), 3);
    for entity in [root, child, grandchild] {
        assert!(!world.is_alive(entity));
    }
    assert!(world.is_alive(bystander));
}

// --- schedules ---------------------------------------------------------------

fn add_one(mut counter: ResMut<Counter>) {
    counter.0 += 1;
}

fn read_counter(counter: Res<Counter>, mut total: ResMut<Total>) {
    total.0 = counter.0 as i32;
}

fn advance(query: Query<(&mut Position, &Velocity)>) {
    query.for_each(|_entity, (position, velocity)| position.0 += velocity.0);
}

#[test]
fn a_schedule_runs_its_systems_in_registration_order() {
    let mut world = World::tracking();
    world.insert_resources((Counter(0), Total(0)));
    let mut schedule = Schedule::new();
    schedule
        .add_system("add one", add_one)
        .add_system("read counter", read_counter);
    schedule.run(&mut world);
    assert_eq!(world.res::<Total>().0, 1);
    assert_eq!(
        schedule.names().collect::<Vec<_>>(),
        ["add one", "read counter"]
    );
    assert_eq!(schedule.len(), 2);
}

#[test]
fn an_order_moves_a_later_system_ahead_of_an_earlier_one() {
    let mut world = World::tracking();
    world.insert_resources((Counter(0), Total(0)));
    let mut schedule = Schedule::new();
    schedule
        .add_system("add one", add_one)
        .add_system("read counter", read_counter);
    schedule.order("read counter", "add one");
    schedule.run(&mut world);
    assert_eq!(world.res::<Total>().0, 0, "the reader must run first now");
}

#[test]
fn an_order_naming_a_system_that_is_not_there_is_inert() {
    let mut schedule: Schedule<World> = Schedule::new();
    schedule.push("only", |_world: &mut World| {});
    schedule.order("absent", "only");
    schedule.order("only", "only");
    schedule.apply_orderings();
    assert_eq!(schedule.names().collect::<Vec<_>>(), ["only"]);
}

#[test]
#[should_panic(expected = "order each other in a cycle")]
fn an_order_cycle_names_the_systems_that_make_it() {
    let mut schedule: Schedule<World> = Schedule::new();
    schedule.push("first", |_world: &mut World| {});
    schedule.push("second", |_world: &mut World| {});
    schedule.order("first", "second");
    schedule.order("second", "first");
    schedule.apply_orderings();
}

#[test]
fn systems_that_touch_no_common_data_share_a_batch() {
    fn write_counter(mut counter: ResMut<Counter>) {
        counter.0 += 1;
    }
    fn write_total(mut total: ResMut<Total>) {
        total.0 += 1;
    }
    let mut schedule: Schedule<World> = Schedule::new();
    schedule
        .add_system("counter", write_counter)
        .add_system("total", write_total);
    assert_eq!(schedule.batches().len(), 1);
}

#[test]
fn two_writers_of_one_resource_never_share_a_batch() {
    fn first(mut counter: ResMut<Counter>) {
        counter.0 += 1;
    }
    fn second(mut counter: ResMut<Counter>) {
        counter.0 += 1;
    }
    let mut schedule: Schedule<World> = Schedule::new();
    schedule
        .add_system("first", first)
        .add_system("second", second);
    assert_eq!(schedule.batches().len(), 2);
}

#[test]
fn a_plain_system_is_a_barrier_and_a_main_thread_system_runs_alone() {
    fn on_the_main_thread(_thread: MainThread, mut counter: ResMut<Counter>) {
        counter.0 += 1;
    }
    fn reader(_counter: Res<Counter>) {}
    let mut schedule: Schedule<World> = Schedule::new();
    schedule.push("barrier", |_world: &mut World| {});
    schedule
        .add_system("main thread", on_the_main_thread)
        .add_system("reader", reader);
    assert_eq!(schedule.batches().len(), 3);

    let mut world = World::tracking();
    world.insert_resource(Counter(0));
    schedule.run(&mut world);
    assert_eq!(world.res::<Counter>().0, 1);
    assert!(schedule.batch_layout().to_string().contains("barrier"));
}

#[test]
fn a_batch_applies_every_write_that_it_holds() {
    fn bump_counter(mut counter: ResMut<Counter>) {
        counter.0 += 3;
    }
    fn bump_total(mut total: ResMut<Total>) {
        total.0 += 4;
    }
    let mut world = World::tracking();
    world.insert_resources((Counter(0), Total(0)));
    let mut schedule = Schedule::new();
    schedule
        .add_system("counter", bump_counter)
        .add_system("total", bump_total);
    schedule.run(&mut world);
    assert_eq!((world.res::<Counter>().0, world.res::<Total>().0), (3, 4));
}

#[test]
fn stages_run_in_the_order_they_were_declared() {
    let mut stages: Stages<World> = Stages::new();
    stages.add_stage("first");
    stages.add_stage("second");
    stages
        .stage_mut("first")
        .push("one", |world: &mut World| world.res_mut::<Total>().0 = 1);
    stages
        .stage_mut("second")
        .push("two", |world: &mut World| world.res_mut::<Total>().0 *= 10);

    let mut world = World::tracking();
    world.insert_resource(Total(0));
    stages.run(&mut world);
    assert_eq!(world.res::<Total>().0, 10);
}

#[test]
#[should_panic(expected = "is already declared")]
fn a_repeated_stage_name_is_refused() {
    let mut stages: Stages<World> = Stages::new();
    stages.add_stage("only");
    stages.add_stage("only");
}

#[test]
#[should_panic(expected = "no stage named")]
fn an_unknown_stage_names_the_ones_that_were_declared() {
    let mut stages: Stages<World> = Stages::new();
    stages.add_stage("only");
    stages.stage_mut("missing");
}

#[test]
fn a_gated_system_runs_only_while_its_condition_holds() {
    let mut world = World::tracking();
    world.insert_resources((Counter(0), Total(0)));
    let mut schedule = Schedule::new();
    schedule.push(
        "gated",
        gate::run_if(|world: &World| world.res::<Total>().0 == 0, add_one),
    );
    schedule.run(&mut world);
    assert_eq!(world.res::<Counter>().0, 1);
    world.res_mut::<Total>().0 = 1;
    schedule.run(&mut world);
    assert_eq!(world.res::<Counter>().0, 1);
}

// --- system parameters -------------------------------------------------------

#[test]
fn a_query_parameter_runs_over_the_world_it_is_given() {
    let (mut world, _) = world_with(&[(1.0, 1.0), (2.0, 2.0)]);
    world.insert_resource(Counter(0));
    let mut schedule = Schedule::new();
    schedule.add_system("advance", advance);
    schedule.run(&mut world);
    assert_eq!(positions(&world), vec![2.0, 4.0]);
}

#[test]
fn a_filtered_query_parameter_narrows_its_match() {
    fn only_tagged(query: Query<&mut Position, With<Health>>) {
        query.for_each(|_entity, position| position.0 = 100.0);
    }
    fn only_untagged(query: Query<&mut Velocity, Without<Health>>) {
        query.for_each(|_entity, velocity| velocity.0 = 50.0);
    }
    let mut world = World::tracking();
    let tagged = world.spawn((Position(1.0), Velocity(1.0), Health(1)));
    let plain = world.spawn((Position(2.0), Velocity(2.0)));

    crate::system_param::run_system(&mut world, only_tagged);
    crate::system_param::run_system(&mut world, only_untagged);
    assert_eq!(world.get::<Position>(tagged), Some(&Position(100.0)));
    assert_eq!(world.get::<Position>(plain), Some(&Position(2.0)));
    assert_eq!(world.get::<Velocity>(plain), Some(&Velocity(50.0)));
    assert_eq!(world.get::<Velocity>(tagged), Some(&Velocity(1.0)));
}

#[test]
fn two_queries_in_one_system_take_turns_on_the_storage() {
    fn pair(first: Query<&mut Position>, second: Query<&Velocity>) {
        let mut total = 0.0;
        second.for_each(|_entity, velocity| total += velocity.0);
        first.for_each(|_entity, position| position.0 = total);
    }
    let (mut world, _) = world_with(&[(0.0, 1.0), (0.0, 2.0)]);
    crate::system_param::run_system(&mut world, pair);
    assert_eq!(positions(&world), vec![3.0, 3.0]);
}

#[test]
fn a_system_takes_resources_a_query_and_a_command_buffer_together() {
    fn mixed(counter: Res<Counter>, mut commands: Commands, query: Query<&Position>) {
        query.for_each(|_entity, position| {
            commands.spawn((Position(position.0 + counter.0 as f32),));
        });
    }
    let (mut world, _) = world_with(&[(1.0, 0.0)]);
    world.insert_resource(Counter(10));
    let mut schedule = Schedule::new();
    schedule.add_system("mixed", mixed);
    schedule.run(&mut world);
    assert_eq!(positions(&world), vec![1.0, 11.0]);
}

#[test]
fn an_event_writer_and_a_reader_cross_a_frame_intact() {
    fn writer(mut events: EventWriter<Ping>) {
        events.send(Ping(1));
        events.send_batch([Ping(2)]);
    }
    fn reader(mut events: EventReader<Ping>, mut total: ResMut<Total>) {
        total.0 += events.iter().map(|ping| ping.0 as i32).sum::<i32>();
        assert_eq!(events.len(), events.iter().count());
        let _ = &mut events;
    }
    let mut world = World::tracking();
    world.insert_resource(Total(0));
    let mut schedule = Schedule::new();
    schedule
        .add_system("writer", writer)
        .add_system("reader", reader);

    schedule.run(&mut world);
    assert_eq!(world.res::<Total>().0, 3);
    schedule.run(&mut world);
    assert_eq!(
        world.res::<Total>().0,
        6,
        "a reader consumes each event once"
    );
}

#[test]
fn an_optional_resource_parameter_tolerates_an_absent_resource() {
    fn maybe(counter: Option<Res<Counter>>, mut total: ResMut<Total>) {
        total.0 = counter.map(|counter| counter.0 as i32).unwrap_or(-1);
    }
    let mut world = World::tracking();
    world.insert_resource(Total(0));
    crate::system_param::run_system(&mut world, maybe);
    assert_eq!(world.res::<Total>().0, -1);
    world.insert_resource(Counter(4));
    crate::system_param::run_system(&mut world, maybe);
    assert_eq!(world.res::<Total>().0, 4);
}

#[test]
#[should_panic(expected = "names resource")]
fn a_system_that_names_one_resource_twice_is_refused() {
    fn twice(_first: Res<Counter>, _second: Res<Counter>) {}
    let mut schedule: Schedule<World> = Schedule::new();
    schedule.add_system("twice", twice);
}

#[test]
fn a_system_reports_the_access_its_signature_gives() {
    fn reported<Marker, S: crate::system_param::IntoSystem<World, Marker>>(
        _system: &S,
    ) -> SystemAccess {
        S::access()
    }
    fn writer(_counter: ResMut<Counter>, _query: Query<&mut Position>) {}
    let reported = reported(&writer);
    assert!(!reported.is_exclusive());
    assert_eq!(
        reported.resources(),
        &[(std::any::TypeId::of::<Counter>(), Access::Write)]
    );
}

#[test]
fn a_query_bundle_reports_the_union_of_the_queries_it_holds() {
    crate::query_bundle! {
        struct Pair<'world, 'item> {
            positions: Query<'world, &'item mut Position>,
            velocities: Query<'world, &'item Velocity>,
        }
    }

    fn run(mut bundle: Pair) {
        let mut total = 0.0;
        bundle
            .velocities
            .for_each_mut(|_entity, velocity| total += velocity.0);
        bundle
            .positions
            .for_each_mut(|_entity, position| position.0 = total);
    }

    let (mut world, _) = world_with(&[(0.0, 2.0), (0.0, 3.0)]);
    crate::system_param::run_system(&mut world, run);
    assert_eq!(positions(&world), vec![5.0, 5.0]);
}

// --- the world as a whole ----------------------------------------------------

#[test]
fn the_stats_describe_what_the_world_holds() {
    let mut world = World::tracking();
    world.insert_resource(Counter(0));
    let entity = world.spawn((Position(1.0), Velocity(2.0)));
    world.add_tag_type::<Player>(entity);
    world.events.send(Ping(1));
    world.queue(|_storage, _resources| {});

    let stats = world.stats();
    assert_eq!(stats.entity_count, 1);
    assert_eq!(stats.component_count, 2);
    assert_eq!(stats.tag_count, 1);
    assert_eq!(stats.resource_count, 1);
    assert_eq!(stats.event_channels, 1);
    assert_eq!(stats.pending_commands, 1);
    assert_eq!(stats.remaining_mask_bits, MASK_BITS - 3);
    assert!(stats.table_count >= 1);
}

#[test]
fn a_bundle_struct_spawns_the_same_row_as_a_tuple() {
    crate::bundle! {
        struct Body {
            position: Position,
            velocity: Velocity,
        }
    }

    let mut world = World::tracking();
    let built = world.spawn(
        Body::builder()
            .position(Position(1.0))
            .velocity(Velocity(2.0)),
    );
    let tupled = world.spawn((Position(1.0), Velocity(2.0)));
    assert_eq!(
        world.component_mask(built).unwrap(),
        world.component_mask(tupled).unwrap()
    );
    assert_eq!(world.get::<Position>(built), Some(&Position(1.0)));
    assert_eq!(world.get::<Velocity>(built), Some(&Velocity(2.0)));
}

#[test]
#[should_panic(expected = "must not repeat a component type")]
fn a_bundle_that_repeats_a_component_is_refused() {
    let mut world = World::tracking();
    world.spawn((Position(1.0), Position(2.0)));
}

#[test]
fn the_world_survives_a_long_run_of_random_changes() {
    let mut world = World::tracking();
    let mut oracle: BTreeMap<u32, (Option<f32>, Option<i32>)> = BTreeMap::new();
    let mut live: Vec<Entity> = Vec::new();
    let mut random = Lcg(42);

    for step in 0..2000 {
        match random.below(5) {
            0 => {
                let position = step as f32;
                let entity = world.spawn((Position(position),));
                oracle.insert(entity.id, (Some(position), None));
                live.push(entity);
            }
            1 if !live.is_empty() => {
                let entity = live[random.below(live.len())];
                let health = step;
                world.set(entity, Health(health));
                oracle.get_mut(&entity.id).unwrap().1 = Some(health);
            }
            2 if !live.is_empty() => {
                let entity = live[random.below(live.len())];
                world.remove::<Health>(entity);
                oracle.get_mut(&entity.id).unwrap().1 = None;
            }
            3 if !live.is_empty() => {
                let index = random.below(live.len());
                let entity = live.swap_remove(index);
                world.despawn_entities(&[entity]);
                oracle.remove(&entity.id);
            }
            _ if !live.is_empty() => {
                let entity = live[random.below(live.len())];
                let position = (step * 2) as f32;
                world.get_mut::<Position>(entity).unwrap().0 = position;
                oracle.get_mut(&entity.id).unwrap().0 = Some(position);
            }
            _ => {}
        }
    }

    assert_eq!(world.entity_count(), oracle.len());
    for &entity in &live {
        let (position, health) = oracle[&entity.id];
        assert_eq!(world.get::<Position>(entity).map(|held| held.0), position);
        assert_eq!(world.get::<Health>(entity).map(|held| held.0), health);
    }
    let counted = world.query_ref::<&Position>().iter().count();
    assert_eq!(counted, oracle.len());
}
