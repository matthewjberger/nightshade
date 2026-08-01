use super::*;

pub trait QueryElement: sealed::SealedElement {
    type Fetch<'table>: Send;
    type Item<'item>;
    const REQUIRED: bool;
    fn component_mask(world: &mut Storage) -> Mask;
    fn lookup_mask(world: &Storage) -> Option<Mask>;
    fn component_access(access: crate::access::SystemAccess) -> crate::access::SystemAccess;
    fn fetch<'table>(slot: Option<*mut ColumnSlot>, tick: u32) -> Self::Fetch<'table>;
    fn changed_newer(fetch: &Self::Fetch<'_>, index: usize, since_tick: u32) -> bool;
    fn item<'fetch>(fetch: &'fetch mut Self::Fetch<'_>, index: usize) -> Self::Item<'fetch>;
    fn mark_all(fetch: &mut Self::Fetch<'_>);
    unsafe fn item_unchecked<'fetch>(
        fetch: &'fetch mut Self::Fetch<'_>,
        index: usize,
    ) -> Self::Item<'fetch>;
}

#[inline]
fn newer_at(ticks: &[u32], index: usize, since_tick: u32) -> bool {
    ticks
        .get(index)
        .is_some_and(|&value| tick_is_newer(value, since_tick))
}

impl<T: Send + Sync + Default + 'static> sealed::SealedElement for &T {}

impl<T: Send + Sync + Default + 'static> QueryElement for &T {
    type Fetch<'table> = (&'table [T], &'table [u32]);
    type Item<'item> = &'item T;
    const REQUIRED: bool = true;

    fn component_mask(world: &mut Storage) -> Mask {
        world.component_key::<T>().mask
    }

    fn lookup_mask(world: &Storage) -> Option<Mask> {
        world.lookup_key::<T>().map(|key| key.mask)
    }

    fn component_access(access: crate::access::SystemAccess) -> crate::access::SystemAccess {
        access.with_component(TypeId::of::<T>(), crate::access::Access::Read)
    }

    fn fetch<'table>(slot: Option<*mut ColumnSlot>, _tick: u32) -> Self::Fetch<'table> {
        let slot = slot.expect("required query element column missing");
        let slot: &'table ColumnSlot = unsafe { &*slot };
        (column_vec::<T>(&slot.data), slot.changed.as_slice())
    }

    fn changed_newer(fetch: &Self::Fetch<'_>, index: usize, since_tick: u32) -> bool {
        newer_at(fetch.1, index, since_tick)
    }

    fn item<'fetch>(fetch: &'fetch mut Self::Fetch<'_>, index: usize) -> Self::Item<'fetch> {
        &fetch.0[index]
    }

    fn mark_all(_fetch: &mut Self::Fetch<'_>) {}

    unsafe fn item_unchecked<'fetch>(
        fetch: &'fetch mut Self::Fetch<'_>,
        index: usize,
    ) -> Self::Item<'fetch> {
        unsafe { fetch.0.get_unchecked(index) }
    }
}

impl<T: Send + Sync + Default + 'static> sealed::SealedElement for &mut T {}

impl<T: Send + Sync + Default + 'static> QueryElement for &mut T {
    type Fetch<'table> = (&'table mut [T], &'table mut [u32], u32);
    type Item<'item> = &'item mut T;
    const REQUIRED: bool = true;

    fn component_mask(world: &mut Storage) -> Mask {
        world.component_key::<T>().mask
    }

    fn lookup_mask(world: &Storage) -> Option<Mask> {
        world.lookup_key::<T>().map(|key| key.mask)
    }

    fn component_access(access: crate::access::SystemAccess) -> crate::access::SystemAccess {
        access.with_component(TypeId::of::<T>(), crate::access::Access::Write)
    }

    fn fetch<'table>(slot: Option<*mut ColumnSlot>, tick: u32) -> Self::Fetch<'table> {
        let slot = slot.expect("required query element column missing");
        let slot: &'table mut ColumnSlot = unsafe { &mut *slot };
        slot.peak_changed = tick;
        let ColumnSlot { data, changed, .. } = slot;
        (column_vec_mut::<T>(data), changed.as_mut_slice(), tick)
    }

    fn changed_newer(fetch: &Self::Fetch<'_>, index: usize, since_tick: u32) -> bool {
        newer_at(fetch.1, index, since_tick)
    }

    fn item<'fetch>(fetch: &'fetch mut Self::Fetch<'_>, index: usize) -> Self::Item<'fetch> {
        if let Some(cell) = fetch.1.get_mut(index) {
            *cell = fetch.2;
        }
        &mut fetch.0[index]
    }

    fn mark_all(fetch: &mut Self::Fetch<'_>) {
        fetch.1.fill(fetch.2);
    }

    unsafe fn item_unchecked<'fetch>(
        fetch: &'fetch mut Self::Fetch<'_>,
        index: usize,
    ) -> Self::Item<'fetch> {
        unsafe { fetch.0.get_unchecked_mut(index) }
    }
}

macro_rules! impl_optional_element {
    ($($element:ty),+) => {
        $(
            impl<'element, T: Send + Sync + Default + 'static> sealed::SealedElement
                for Option<$element> {}

            impl<'element, T: Send + Sync + Default + 'static> QueryElement for Option<$element> {
                type Fetch<'table> = Option<<$element as QueryElement>::Fetch<'table>>;
                type Item<'item> = Option<<$element as QueryElement>::Item<'item>>;
                const REQUIRED: bool = false;

                fn component_mask(world: &mut Storage) -> Mask {
                    <$element as QueryElement>::component_mask(world)
                }

                fn lookup_mask(world: &Storage) -> Option<Mask> {
                    <$element as QueryElement>::lookup_mask(world)
                }

                fn component_access(
                    access: crate::access::SystemAccess,
                ) -> crate::access::SystemAccess {
                    <$element as QueryElement>::component_access(access)
                }

                fn fetch<'table>(
                    slot: Option<*mut ColumnSlot>,
                    tick: u32,
                ) -> Self::Fetch<'table> {
                    slot.map(|slot| <$element as QueryElement>::fetch(Some(slot), tick))
                }

                fn changed_newer(
                    fetch: &Self::Fetch<'_>,
                    index: usize,
                    since_tick: u32,
                ) -> bool {
                    fetch.as_ref().is_some_and(|inner| {
                        <$element as QueryElement>::changed_newer(inner, index, since_tick)
                    })
                }

                fn item<'fetch>(
                    fetch: &'fetch mut Self::Fetch<'_>,
                    index: usize,
                ) -> Self::Item<'fetch> {
                    fetch
                        .as_mut()
                        .map(|inner| <$element as QueryElement>::item(inner, index))
                }

                fn mark_all(fetch: &mut Self::Fetch<'_>) {
                    if let Some(inner) = fetch.as_mut() {
                        <$element as QueryElement>::mark_all(inner);
                    }
                }

                unsafe fn item_unchecked<'fetch>(
                    fetch: &'fetch mut Self::Fetch<'_>,
                    index: usize,
                ) -> Self::Item<'fetch> {
                    fetch
                        .as_mut()
                        .map(|inner| unsafe {
                            <$element as QueryElement>::item_unchecked(inner, index)
                        })
                }
            }
        )+
    };
}

impl_optional_element!(&'element T, &'element mut T);

pub trait ReadQueryElement: QueryElement {
    type ReadFetch<'table>: Copy;
    fn lookup_mask(world: &Storage) -> Option<Mask>;
    fn read_fetch<'table>(slot: Option<&'table ColumnSlot>) -> Self::ReadFetch<'table>;
    fn read_newer(fetch: Self::ReadFetch<'_>, index: usize, since: u32, added: bool) -> bool;
    fn read_item<'table>(fetch: Self::ReadFetch<'table>, index: usize) -> Self::Item<'table>;
}

impl<T: Send + Sync + Default + 'static> ReadQueryElement for &T {
    type ReadFetch<'table> = Option<(&'table [T], &'table [u32], &'table [u32])>;

    fn lookup_mask(world: &Storage) -> Option<Mask> {
        world.lookup_key::<T>().map(|key| key.mask)
    }

    fn read_fetch<'table>(slot: Option<&'table ColumnSlot>) -> Self::ReadFetch<'table> {
        slot.map(|slot| {
            (
                column_vec::<T>(&slot.data),
                slot.changed.as_slice(),
                slot.added.as_slice(),
            )
        })
    }

    fn read_newer(fetch: Self::ReadFetch<'_>, index: usize, since: u32, added: bool) -> bool {
        fetch.is_some_and(|fetch| newer_at(if added { fetch.2 } else { fetch.1 }, index, since))
    }

    fn read_item<'table>(fetch: Self::ReadFetch<'table>, index: usize) -> Self::Item<'table> {
        &fetch.expect("required query element column missing").0[index]
    }
}

impl<T: Send + Sync + Default + 'static> ReadQueryElement for Option<&T> {
    type ReadFetch<'table> = <&'table T as ReadQueryElement>::ReadFetch<'table>;

    fn lookup_mask(world: &Storage) -> Option<Mask> {
        world.lookup_key::<T>().map(|key| key.mask)
    }

    fn read_fetch<'table>(slot: Option<&'table ColumnSlot>) -> Self::ReadFetch<'table> {
        <&T as ReadQueryElement>::read_fetch(slot)
    }

    fn read_newer(fetch: Self::ReadFetch<'_>, index: usize, since: u32, added: bool) -> bool {
        <&T as ReadQueryElement>::read_newer(fetch, index, since, added)
    }

    fn read_item<'table>(fetch: Self::ReadFetch<'table>, index: usize) -> Self::Item<'table> {
        fetch.map(|fetch| &fetch.0[index])
    }
}

pub trait QueryTuple: sealed::SealedQueryTuple {
    type Fetch<'table>: Send;
    type Item<'item>;
    const ARITY: usize;
    const ALL_REQUIRED: bool;
    fn component_mask(world: &mut Storage) -> Mask;
    fn component_access(access: crate::access::SystemAccess) -> crate::access::SystemAccess;
    fn element_masks(world: &mut Storage) -> [Mask; 8];
    fn lookup_element_masks(world: &Storage) -> Option<([Mask; 8], Mask)>;
    fn required_mask_from(element_masks: &[Mask; 8]) -> Mask;
    fn fetch<'table>(
        positions: &[Option<usize>; 8],
        columns: &'table [ColumnCell],
        tick: u32,
    ) -> Self::Fetch<'table>;
    fn changed_newer(
        fetch: &Self::Fetch<'_>,
        index: usize,
        element_masks: &[Mask; 8],
        changed_mask: Mask,
        since_tick: u32,
    ) -> bool;
    fn item<'fetch>(fetch: &'fetch mut Self::Fetch<'_>, index: usize) -> Self::Item<'fetch>;
    fn mark_all(fetch: &mut Self::Fetch<'_>);
    unsafe fn item_unchecked<'fetch>(
        fetch: &'fetch mut Self::Fetch<'_>,
        index: usize,
    ) -> Self::Item<'fetch>;

    fn fast_for_each<F>(mut fetch: Self::Fetch<'_>, entities: &[Entity], f: &mut F)
    where
        F: for<'item> FnMut(Entity, Self::Item<'item>),
    {
        Self::mark_all(&mut fetch);
        for (index, &entity) in entities.iter().enumerate() {
            f(entity, unsafe { Self::item_unchecked(&mut fetch, index) });
        }
    }
}

pub trait ReadQueryTuple: QueryTuple {
    type ReadFetch<'table>: Copy;
    fn lookup_masks(world: &Storage) -> Option<([Mask; 8], Mask)>;
    fn read_fetch<'table>(
        table_mask: Mask,
        columns: &'table [ColumnCell],
        element_masks: &[Mask; 8],
    ) -> Self::ReadFetch<'table>;
    fn read_newer(
        fetch: Self::ReadFetch<'_>,
        index: usize,
        element_masks: &[Mask; 8],
        wanted: Mask,
        since_tick: u32,
        added: bool,
    ) -> bool;
    fn read_item<'table>(fetch: Self::ReadFetch<'table>, index: usize) -> Self::Item<'table>;
}

#[inline]
fn slot_positions(
    column_index: &[u16],
    element_masks: &[Mask; 8],
    arity: usize,
) -> [Option<usize>; 8] {
    let mut positions = [None; 8];
    for (position, mask) in positions.iter_mut().zip(element_masks.iter()).take(arity) {
        *position = mask
            .first()
            .and_then(|slot| match column_index.get(slot as usize) {
                Some(&found) if found != COLUMN_ABSENT => Some(found as usize),
                _ => None,
            });
    }
    positions
}

#[inline]
fn planned_positions(planned: &[u16; 8], ranks: &[usize; 8], arity: usize) -> [Option<usize>; 8] {
    let mut positions = [None; 8];
    for (index, position) in positions.iter_mut().enumerate().take(arity) {
        *position = Some(planned[ranks[index]] as usize);
    }
    positions
}

fn required_mask(elements: &[(Mask, bool)]) -> Mask {
    let mut seen = Mask::EMPTY;
    let mut required = Mask::EMPTY;
    for &(mask, is_required) in elements {
        assert!(
            !seen.intersects(mask),
            "query tuples must not repeat a component type"
        );
        seen |= mask;
        if is_required {
            required |= mask;
        }
    }
    required
}

fn lookup_masks_from(elements: &[(Option<Mask>, bool)]) -> Option<([Mask; 8], Mask)> {
    let mut masks = [Mask::EMPTY; 8];
    let mut seen = Mask::EMPTY;
    let mut required = Mask::EMPTY;
    for (position, &(mask, is_required)) in elements.iter().enumerate() {
        match mask {
            Some(mask) => {
                assert!(
                    !seen.intersects(mask),
                    "query tuples must not repeat a component type"
                );
                seen |= mask;
                if is_required {
                    required |= mask;
                }
                masks[position] = mask;
            }
            None if is_required => return None,
            None => {}
        }
    }
    Some((masks, required))
}

#[inline]
fn distribute_slots(
    columns: &[ColumnCell],
    positions: &[Option<usize>; 8],
) -> [Option<*mut ColumnSlot>; 8] {
    std::array::from_fn(|index| {
        positions[index].map(|position| {
            assert!(
                position < columns.len(),
                "query column position out of bounds"
            );
            columns[position].write()
        })
    })
}

#[inline]
fn read_column(
    table_mask: Mask,
    columns: &[ColumnCell],
    element_mask: Mask,
) -> Option<&ColumnSlot> {
    table_mask
        .column_of(element_mask)
        .map(|position| &*columns[position as usize])
}

macro_rules! impl_query_tuple {
    ($(($element:ident, $position:tt)),+) => {
        impl<$($element: QueryElement),+> sealed::SealedQueryTuple for ($($element,)+) {}

        impl<$($element: QueryElement),+> QueryTuple for ($($element,)+) {
            type Fetch<'table> = ($($element::Fetch<'table>,)+);
            type Item<'item> = ($($element::Item<'item>,)+);
            const ARITY: usize = 0 $(+ { let _ = stringify!($element); 1 })+;
            const ALL_REQUIRED: bool = true $(&& $element::REQUIRED)+;

            fn component_mask(world: &mut Storage) -> Mask {
                required_mask(&[$(($element::component_mask(world), $element::REQUIRED),)+])
            }

            fn component_access(
                access: crate::access::SystemAccess,
            ) -> crate::access::SystemAccess {
                $(let access = $element::component_access(access);)+
                access
            }

            fn element_masks(world: &mut Storage) -> [Mask; 8] {
                let mut masks = [Mask::EMPTY; 8];
                $(masks[$position] = $element::component_mask(world);)+
                masks
            }

            fn lookup_element_masks(world: &Storage) -> Option<([Mask; 8], Mask)> {
                lookup_masks_from(&[$(($element::lookup_mask(world), $element::REQUIRED),)+])
            }

            fn required_mask_from(element_masks: &[Mask; 8]) -> Mask {
                required_mask(&[$((element_masks[$position], $element::REQUIRED),)+])
            }

            fn fetch<'table>(
                positions: &[Option<usize>; 8],
                columns: &'table [ColumnCell],
                tick: u32,
            ) -> Self::Fetch<'table> {
                let mut slots = distribute_slots(columns, positions);
                ($($element::fetch(slots[$position].take(), tick),)+)
            }

            fn changed_newer(
                fetch: &Self::Fetch<'_>,
                index: usize,
                element_masks: &[Mask; 8],
                changed_mask: Mask,
                since_tick: u32,
            ) -> bool {
                false $(|| (changed_mask.intersects(element_masks[$position])
                    && $element::changed_newer(&fetch.$position, index, since_tick)))+
            }

            fn item<'fetch>(
                fetch: &'fetch mut Self::Fetch<'_>,
                index: usize,
            ) -> Self::Item<'fetch> {
                ($($element::item(&mut fetch.$position, index),)+)
            }

            fn mark_all(fetch: &mut Self::Fetch<'_>) {
                $($element::mark_all(&mut fetch.$position);)+
            }

            unsafe fn item_unchecked<'fetch>(
                fetch: &'fetch mut Self::Fetch<'_>,
                index: usize,
            ) -> Self::Item<'fetch> {
                ($(unsafe { $element::item_unchecked(&mut fetch.$position, index) },)+)
            }
        }
    };
}

impl_query_tuple!((A, 0));
impl_query_tuple!((A, 0), (B, 1));
impl_query_tuple!((A, 0), (B, 1), (C, 2));
impl_query_tuple!((A, 0), (B, 1), (C, 2), (D, 3));
impl_query_tuple!((A, 0), (B, 1), (C, 2), (D, 3), (E, 4));
impl_query_tuple!((A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5));
impl_query_tuple!((A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5), (G, 6));
impl_query_tuple!(
    (A, 0),
    (B, 1),
    (C, 2),
    (D, 3),
    (E, 4),
    (F, 5),
    (G, 6),
    (H, 7)
);

macro_rules! impl_bare_element_query {
    ($($element:ty),+) => {
        $(
            impl<'element, T: Send + Sync + Default + 'static> sealed::SealedQueryTuple
                for $element {}

            impl<'element, T: Send + Sync + Default + 'static> QueryTuple for $element {
                type Fetch<'table> = <$element as QueryElement>::Fetch<'table>;
                type Item<'item> = <$element as QueryElement>::Item<'item>;
                const ARITY: usize = 1;
                const ALL_REQUIRED: bool = <$element as QueryElement>::REQUIRED;

                fn component_mask(world: &mut Storage) -> Mask {
                    required_mask(&[(
                        <$element as QueryElement>::component_mask(world),
                        <$element as QueryElement>::REQUIRED,
                    )])
                }

                fn component_access(
                    access: crate::access::SystemAccess,
                ) -> crate::access::SystemAccess {
                    <$element as QueryElement>::component_access(access)
                }

                fn element_masks(world: &mut Storage) -> [Mask; 8] {
                    let mut masks = [Mask::EMPTY; 8];
                    masks[0] = <$element as QueryElement>::component_mask(world);
                    masks
                }

                fn lookup_element_masks(world: &Storage) -> Option<([Mask; 8], Mask)> {
                    lookup_masks_from(&[(
                        <$element as QueryElement>::lookup_mask(world),
                        <$element as QueryElement>::REQUIRED,
                    )])
                }

                fn required_mask_from(element_masks: &[Mask; 8]) -> Mask {
                    if <$element as QueryElement>::REQUIRED {
                        element_masks[0]
                    } else {
                        Mask::EMPTY
                    }
                }

                fn fetch<'table>(
                    positions: &[Option<usize>; 8],
                    columns: &'table [ColumnCell],
                    tick: u32,
                ) -> Self::Fetch<'table> {
                    let mut slots = distribute_slots(columns, positions);
                    <$element as QueryElement>::fetch(slots[0].take(), tick)
                }

                fn changed_newer(
                    fetch: &Self::Fetch<'_>,
                    index: usize,
                    element_masks: &[Mask; 8],
                    changed_mask: Mask,
                    since_tick: u32,
                ) -> bool {
                    changed_mask.intersects(element_masks[0])
                        && <$element as QueryElement>::changed_newer(fetch, index, since_tick)
                }

                fn item<'fetch>(
                    fetch: &'fetch mut Self::Fetch<'_>,
                    index: usize,
                ) -> Self::Item<'fetch> {
                    <$element as QueryElement>::item(fetch, index)
                }

                fn mark_all(fetch: &mut Self::Fetch<'_>) {
                    <$element as QueryElement>::mark_all(fetch);
                }

                unsafe fn item_unchecked<'fetch>(
                    fetch: &'fetch mut Self::Fetch<'_>,
                    index: usize,
                ) -> Self::Item<'fetch> {
                    unsafe { <$element as QueryElement>::item_unchecked(fetch, index) }
                }
            }
        )+
    };
}

impl_bare_element_query!(
    &'element T,
    &'element mut T,
    Option<&'element T>,
    Option<&'element mut T>
);

macro_rules! impl_read_query_tuple {
    ($(($element:ident, $position:tt)),+) => {
        impl<$($element: ReadQueryElement),+> ReadQueryTuple for ($($element,)+) {
            type ReadFetch<'table> = ($($element::ReadFetch<'table>,)+);

            fn lookup_masks(world: &Storage) -> Option<([Mask; 8], Mask)> {
                lookup_masks_from(&[$((
                    <$element as ReadQueryElement>::lookup_mask(world),
                    $element::REQUIRED,
                ),)+])
            }

            fn read_fetch<'table>(
                table_mask: Mask,
                columns: &'table [ColumnCell],
                element_masks: &[Mask; 8],
            ) -> Self::ReadFetch<'table> {
                ($(
                    $element::read_fetch(
                        read_column(table_mask, columns, element_masks[$position]),
                    ),
                )+)
            }

            fn read_newer(
                fetch: Self::ReadFetch<'_>,
                index: usize,
                element_masks: &[Mask; 8],
                wanted: Mask,
                since_tick: u32,
                added: bool,
            ) -> bool {
                false $(|| (wanted.intersects(element_masks[$position])
                    && $element::read_newer(fetch.$position, index, since_tick, added)))+
            }

            fn read_item<'table>(
                fetch: Self::ReadFetch<'table>,
                index: usize,
            ) -> Self::Item<'table> {
                ($($element::read_item(fetch.$position, index),)+)
            }
        }
    };
}

impl_read_query_tuple!((A, 0));
impl_read_query_tuple!((A, 0), (B, 1));
impl_read_query_tuple!((A, 0), (B, 1), (C, 2));
impl_read_query_tuple!((A, 0), (B, 1), (C, 2), (D, 3));
impl_read_query_tuple!((A, 0), (B, 1), (C, 2), (D, 3), (E, 4));
impl_read_query_tuple!((A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5));
impl_read_query_tuple!((A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5), (G, 6));
impl_read_query_tuple!(
    (A, 0),
    (B, 1),
    (C, 2),
    (D, 3),
    (E, 4),
    (F, 5),
    (G, 6),
    (H, 7)
);

macro_rules! impl_bare_element_read_query {
    ($($element:ty),+) => {
        $(
            impl<'element, T: Send + Sync + Default + 'static> ReadQueryTuple for $element {
                type ReadFetch<'table> = <$element as ReadQueryElement>::ReadFetch<'table>;

                fn lookup_masks(world: &Storage) -> Option<([Mask; 8], Mask)> {
                    lookup_masks_from(&[(
                        <$element as ReadQueryElement>::lookup_mask(world),
                        <$element as QueryElement>::REQUIRED,
                    )])
                }

                fn read_fetch<'table>(
                    table_mask: Mask,
                    columns: &'table [ColumnCell],
                    element_masks: &[Mask; 8],
                ) -> Self::ReadFetch<'table> {
                    <$element as ReadQueryElement>::read_fetch(
                        read_column(table_mask, columns, element_masks[0]),
                    )
                }

                fn read_newer(
                    fetch: Self::ReadFetch<'_>,
                    index: usize,
                    element_masks: &[Mask; 8],
                    wanted: Mask,
                    since_tick: u32,
                    added: bool,
                ) -> bool {
                    wanted.intersects(element_masks[0])
                        && <$element as ReadQueryElement>::read_newer(
                            fetch, index, since_tick, added,
                        )
                }

                fn read_item<'table>(
                    fetch: Self::ReadFetch<'table>,
                    index: usize,
                ) -> Self::Item<'table> {
                    <$element as ReadQueryElement>::read_item(fetch, index)
                }
            }
        )+
    };
}

impl_bare_element_read_query!(&'element T, Option<&'element T>);

pub(crate) fn tags_match(
    tags: &[SparseTagSet],
    entity: Entity,
    tag_include: Mask,
    tag_exclude: Mask,
) -> bool {
    if tag_include.is_empty() && tag_exclude.is_empty() {
        return true;
    }
    for (tag_index, tag_set) in tags.iter().enumerate() {
        let tag_mask = Mask::bit(MASK_BITS - 1 - tag_index as u32);
        if tag_include.intersects(tag_mask) && !tag_set.contains(entity) {
            return false;
        }
        if tag_exclude.intersects(tag_mask) && tag_set.contains(entity) {
            return false;
        }
    }
    true
}

pub struct QueryBuilder<'world, Q: QueryTuple> {
    pub world: &'world Storage,
    pub include: Mask,
    pub exclude: Mask,
    pub changed_mask: Mask,
    pub added_mask: Mask,
    pub element_masks: [Mask; 8],
    pub dead: bool,
    pub marker: PhantomData<Q>,
}

impl<'world, Q: QueryTuple> QueryBuilder<'world, Q> {
    pub(crate) fn over(world: &'world Storage, element_masks: [Mask; 8], include: Mask) -> Self {
        QueryBuilder {
            world,
            include,
            exclude: Mask::EMPTY,
            changed_mask: Mask::EMPTY,
            added_mask: Mask::EMPTY,
            element_masks,
            dead: false,
            marker: PhantomData,
        }
    }

    pub fn reborrow(&self) -> QueryBuilder<'world, Q> {
        QueryBuilder {
            world: self.world,
            include: self.include,
            exclude: self.exclude,
            changed_mask: self.changed_mask,
            added_mask: self.added_mask,
            element_masks: self.element_masks,
            dead: self.dead,
            marker: PhantomData,
        }
    }

    pub fn with<T: Send + Sync + Default + 'static>(mut self) -> Self {
        match self.world.lookup_key::<T>() {
            Some(key) => self.include |= key.mask,
            None => self.dead = true,
        }
        self
    }

    pub fn without<T: Send + Sync + Default + 'static>(mut self) -> Self {
        if let Some(key) = self.world.lookup_key::<T>() {
            self.exclude |= key.mask;
        }
        self
    }

    pub fn with_tag_type<T: 'static>(mut self) -> Self {
        match self.world.lookup_tag_key::<T>() {
            Some(key) => self.include |= key.mask,
            None => self.dead = true,
        }
        self
    }

    pub fn without_tag_type<T: 'static>(mut self) -> Self {
        if let Some(key) = self.world.lookup_tag_key::<T>() {
            self.exclude |= key.mask;
        }
        self
    }

    pub fn changed<T: Send + Sync + Default + 'static>(mut self) -> Self {
        match self.world.lookup_key::<T>() {
            Some(key) => self.changed_mask |= key.mask,
            None => self.dead = true,
        }
        self
    }

    pub fn added<T: Send + Sync + Default + 'static>(mut self) -> Self {
        match self.world.lookup_key::<T>() {
            Some(key) => self.added_mask |= key.mask,
            None => self.dead = true,
        }
        self
    }

    fn tuple_mask(&self) -> Mask {
        self.element_masks
            .iter()
            .take(Q::ARITY)
            .fold(Mask::EMPTY, |mask, element| mask | *element)
    }

    pub fn for_entity(self, entity: Entity, f: impl for<'item> FnOnce(Q::Item<'item>)) {
        if self.dead {
            return;
        }
        let Some((table_index, array_index)) =
            crate::ecs::world::get_location(&self.world.entity_locations, entity)
        else {
            return;
        };
        let Some((component_include, component_exclude, tag_include, tag_exclude)) =
            self.world.split_masks(self.include, self.exclude)
        else {
            return;
        };
        if !tags_match(&self.world.tags, entity, tag_include, tag_exclude) {
            return;
        }

        let since_tick = self.world.last_tick;
        let current_tick = self.world.current_tick;
        let table = &self.world.tables[table_index];
        if !table.mask.contains(component_include) || table.mask.intersects(component_exclude) {
            return;
        }
        if !row_was_added(table, array_index, self.added_mask, since_tick) {
            return;
        }

        let positions = slot_positions(&table.column_index, &self.element_masks, Q::ARITY);
        let mut fetch = Q::fetch(&positions, &table.columns, current_tick);
        if self.changed_mask != Mask::EMPTY
            && !Q::changed_newer(
                &fetch,
                array_index,
                &self.element_masks,
                self.changed_mask,
                since_tick,
            )
        {
            return;
        }
        f(Q::item(&mut fetch, array_index));
    }

    pub fn for_each(self, mut f: impl for<'item> FnMut(Entity, Q::Item<'item>)) {
        assert_eq!(
            (self.changed_mask | self.added_mask) & !self.tuple_mask(),
            Mask::EMPTY,
            "changed and added filters must name components present in the query tuple"
        );
        if self.dead {
            return;
        }

        let Some((component_include, component_exclude, tag_include, tag_exclude)) =
            self.world.split_masks(self.include, self.exclude)
        else {
            return;
        };

        let world = self.world;
        let since_tick = world.last_tick;
        let current_tick = world.current_tick;
        let changed_mask = self.changed_mask;
        let added_mask = self.added_mask;
        let element_masks = self.element_masks;
        let filtered = tag_include != Mask::EMPTY
            || tag_exclude != Mask::EMPTY
            || changed_mask != Mask::EMPTY
            || added_mask != Mask::EMPTY;

        if !filtered && Q::ALL_REQUIRED && component_exclude == Mask::EMPTY {
            let mut ranks = [0usize; 8];
            for (rank, mask) in ranks.iter_mut().zip(element_masks.iter()).take(Q::ARITY) {
                if let Some(slot) = mask.first() {
                    *rank = component_include.count_below(slot) as usize;
                }
            }
            let planned =
                archetype_read_planned_tables(&world.query_plans, &world.tables, component_include);
            for entry in planned.iter() {
                let table = &world.tables[entry.table as usize];
                if table.entity_indices.is_empty() {
                    continue;
                }
                let positions = planned_positions(&entry.positions, &ranks, Q::ARITY);
                let fetch = Q::fetch(&positions, &table.columns, current_tick);
                Q::fast_for_each(fetch, table.entity_indices.as_slice(), &mut f);
            }
            return;
        }

        let table_indices = archetype_read_cached_tables(
            &world.query_cache,
            world.tables.iter().map(|table| table.mask),
            component_include,
        );

        for &table_index in table_indices.iter() {
            let table = &world.tables[table_index];
            if table.mask.intersects(component_exclude) || table.entity_indices.is_empty() {
                continue;
            }

            let positions = slot_positions(&table.column_index, &element_masks, Q::ARITY);
            let added_rows = added_rows_of(table, added_mask, since_tick);
            let mut fetch = Q::fetch(&positions, &table.columns, current_tick);
            for (index, &entity) in table.entity_indices.iter().enumerate() {
                if !tags_match(&world.tags, entity, tag_include, tag_exclude) {
                    continue;
                }
                if changed_mask != Mask::EMPTY
                    && !Q::changed_newer(&fetch, index, &element_masks, changed_mask, since_tick)
                {
                    continue;
                }
                if !added_rows.is_empty() && !added_rows[index] {
                    continue;
                }
                f(entity, Q::item(&mut fetch, index));
            }
        }
    }
}

#[inline]
fn row_was_added(table: &ComponentArrays, index: usize, added_mask: Mask, since_tick: u32) -> bool {
    if added_mask == Mask::EMPTY {
        return true;
    }
    table.columns.iter().any(|column| {
        added_mask.contains_bit(column.component_index)
            && newer_at(&column.added, index, since_tick)
    })
}

fn added_rows_of(table: &ComponentArrays, added_mask: Mask, since_tick: u32) -> Vec<bool> {
    if added_mask == Mask::EMPTY {
        return Vec::new();
    }
    (0..table.entity_indices.len())
        .map(|index| row_was_added(table, index, added_mask, since_tick))
        .collect()
}

pub struct QueryRef<'world, Q: ReadQueryTuple> {
    pub world: &'world Storage,
    pub include: Mask,
    pub exclude: Mask,
    pub changed_mask: Mask,
    pub added_mask: Mask,
    pub dead: bool,
    pub marker: PhantomData<Q>,
}

impl<'world, Q: ReadQueryTuple> QueryRef<'world, Q> {
    pub fn with<T: Send + Sync + Default + 'static>(mut self) -> Self {
        match self.world.lookup_key::<T>() {
            Some(key) => self.include |= key.mask,
            None => self.dead = true,
        }
        self
    }

    pub fn without<T: Send + Sync + Default + 'static>(mut self) -> Self {
        if let Some(key) = self.world.lookup_key::<T>() {
            self.exclude |= key.mask;
        }
        self
    }

    pub fn with_tag_type<T: 'static>(mut self) -> Self {
        match self.world.lookup_tag_key::<T>() {
            Some(key) => self.include |= key.mask,
            None => self.dead = true,
        }
        self
    }

    pub fn without_tag_type<T: 'static>(mut self) -> Self {
        if let Some(key) = self.world.lookup_tag_key::<T>() {
            self.exclude |= key.mask;
        }
        self
    }

    pub fn changed<T: Send + Sync + Default + 'static>(mut self) -> Self {
        match self.world.lookup_key::<T>() {
            Some(key) => self.changed_mask |= key.mask,
            None => self.dead = true,
        }
        self
    }

    pub fn added<T: Send + Sync + Default + 'static>(mut self) -> Self {
        match self.world.lookup_key::<T>() {
            Some(key) => self.added_mask |= key.mask,
            None => self.dead = true,
        }
        self
    }

    pub fn iter(self) -> QueryRefIter<'world, Q> {
        let mut done = self.dead;
        let mut element_masks = [Mask::EMPTY; 8];
        let mut include = self.include;
        match Q::lookup_masks(self.world) {
            Some((masks, required)) => {
                element_masks = masks;
                include |= required;
            }
            None => done = true,
        }

        if !done {
            let tuple_mask = element_masks
                .iter()
                .take(Q::ARITY)
                .fold(Mask::EMPTY, |mask, element| mask | *element);
            assert_eq!(
                (self.changed_mask | self.added_mask) & !tuple_mask,
                Mask::EMPTY,
                "changed and added filters must name components present in the query tuple"
            );
        }

        let mut split = (Mask::EMPTY, Mask::EMPTY, Mask::EMPTY, Mask::EMPTY);
        match self.world.split_masks(include, self.exclude) {
            Some(masks) => split = masks,
            None => done = true,
        }

        QueryRefIter {
            world: self.world,
            element_masks,
            include: split.0,
            exclude: split.1,
            tag_include: split.2,
            tag_exclude: split.3,
            changed_mask: self.changed_mask,
            added_mask: self.added_mask,
            since_tick: self.world.last_tick,
            cached_tables: self.world.query_cache.get(&split.0).map(Vec::as_slice),
            table_index: 0,
            row_index: 0,
            current: None,
            done,
        }
    }

    pub fn single(self) -> Option<(Entity, Q::Item<'world>)> {
        let mut matches = self.iter();
        let first = matches.next()?;
        matches.next().is_none().then_some(first)
    }
}

pub struct QueryRefIter<'world, Q: ReadQueryTuple> {
    pub world: &'world Storage,
    pub element_masks: [Mask; 8],
    pub include: Mask,
    pub exclude: Mask,
    pub tag_include: Mask,
    pub tag_exclude: Mask,
    pub changed_mask: Mask,
    pub added_mask: Mask,
    pub since_tick: u32,
    pub cached_tables: Option<&'world [usize]>,
    pub table_index: usize,
    pub row_index: usize,
    pub current: Option<(&'world [Entity], Q::ReadFetch<'world>)>,
    pub done: bool,
}

impl<'world, Q: ReadQueryTuple> Iterator for QueryRefIter<'world, Q> {
    type Item = (Entity, Q::Item<'world>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        loop {
            if let Some((entities, fetch)) = self.current {
                while self.row_index < entities.len() {
                    let index = self.row_index;
                    self.row_index += 1;
                    let entity = entities[index];
                    if !tags_match(&self.world.tags, entity, self.tag_include, self.tag_exclude) {
                        continue;
                    }
                    let passes = |wanted: Mask, added: bool| {
                        wanted == Mask::EMPTY
                            || Q::read_newer(
                                fetch,
                                index,
                                &self.element_masks,
                                wanted,
                                self.since_tick,
                                added,
                            )
                    };
                    if !passes(self.changed_mask, false) || !passes(self.added_mask, true) {
                        continue;
                    }
                    return Some((entity, Q::read_item(fetch, index)));
                }
                self.current = None;
            }

            loop {
                let table = match self.cached_tables {
                    Some(indices) => {
                        let &cached = indices.get(self.table_index)?;
                        self.table_index += 1;
                        &self.world.tables[cached]
                    }
                    None => {
                        let table = self.world.tables.get(self.table_index)?;
                        self.table_index += 1;
                        if !table.mask.contains(self.include) {
                            continue;
                        }
                        table
                    }
                };
                if !table.mask.intersects(self.exclude) && !table.entity_indices.is_empty() {
                    self.row_index = 0;
                    self.current = Some((
                        table.entity_indices.as_slice(),
                        Q::read_fetch(table.mask, &table.columns, &self.element_masks),
                    ));
                    break;
                }
            }
        }
    }
}
