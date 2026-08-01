use super::*;
use std::alloc::{self, Layout};
use std::cell::RefCell;
use std::ptr::{self, NonNull};

#[derive(Default)]
pub struct TypeIdHasher(u64);

impl std::hash::Hasher for TypeIdHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        let mut hash = self.0;
        for &byte in bytes {
            hash = (hash ^ byte as u64).wrapping_mul(0x0000_0100_0000_01b3);
        }
        self.0 = hash;
    }

    fn write_u64(&mut self, value: u64) {
        self.0 = (self.0 ^ value).wrapping_mul(0x9e37_79b9_7f4a_7c15);
    }

    fn write_u128(&mut self, value: u128) {
        self.write_u64(value as u64);
        self.write_u64((value >> 64) as u64);
    }
}

pub(crate) type TypeIdMap<V> = HashMap<TypeId, V, std::hash::BuildHasherDefault<TypeIdHasher>>;

pub(crate) type MaskMap<V> = HashMap<Mask, V, std::hash::BuildHasherDefault<TypeIdHasher>>;

pub(crate) type TablePairMap<V> = HashMap<u64, V, std::hash::BuildHasherDefault<TypeIdHasher>>;

const MAX_POOLED_PER_BUCKET: usize = 32;

struct ArenaPool {
    buckets: HashMap<(usize, usize), Vec<NonNull<u8>>>,
}

impl Drop for ArenaPool {
    fn drop(&mut self) {
        for ((size, align), bucket) in self.buckets.drain() {
            let layout =
                Layout::from_size_align(size, align).expect("pooled arena layout overflow");
            for pointer in bucket {
                unsafe { alloc::dealloc(pointer.as_ptr(), layout) };
            }
        }
    }
}

thread_local! {
    static ARENA_POOL: RefCell<ArenaPool> = RefCell::new(ArenaPool {
        buckets: HashMap::new(),
    });
}

fn pool_take(size: usize, align: usize) -> Option<NonNull<u8>> {
    ARENA_POOL.with(|pool| pool.borrow_mut().buckets.get_mut(&(size, align))?.pop())
}

fn pool_return(pointer: NonNull<u8>, size: usize, align: usize) {
    ARENA_POOL.with(|pool| {
        let mut pool = pool.borrow_mut();
        let bucket = pool.buckets.entry((size, align)).or_default();
        if bucket.len() < MAX_POOLED_PER_BUCKET {
            bucket.push(pointer);
            return;
        }
        let layout = Layout::from_size_align(size, align).expect("pooled arena layout overflow");
        unsafe { alloc::dealloc(pointer.as_ptr(), layout) };
    });
}

pub(crate) struct TableArena {
    pointer: NonNull<u8>,
    size: usize,
    align: usize,
}

unsafe impl Send for TableArena {}
unsafe impl Sync for TableArena {}

impl Default for TableArena {
    fn default() -> Self {
        TableArena {
            pointer: NonNull::dangling(),
            size: 0,
            align: 1,
        }
    }
}

impl TableArena {
    pub(crate) fn allocate(size: usize, align: usize) -> Self {
        if size == 0 {
            return TableArena {
                pointer: NonNull::new(align as *mut u8).expect("alignment is never zero"),
                size: 0,
                align,
            };
        }
        if let Some(pooled) = pool_take(size, align) {
            return TableArena {
                pointer: pooled,
                size,
                align,
            };
        }
        let layout = Layout::from_size_align(size, align).expect("table arena layout overflow");
        let block = unsafe { alloc::alloc(layout) };
        TableArena {
            pointer: NonNull::new(block).unwrap_or_else(|| alloc::handle_alloc_error(layout)),
            size,
            align,
        }
    }

    #[inline]
    pub(crate) fn base(&self) -> *mut u8 {
        self.pointer.as_ptr()
    }
}

impl Drop for TableArena {
    fn drop(&mut self) {
        if self.size != 0 {
            pool_return(self.pointer, self.size, self.align);
        }
    }
}

pub struct ErasedColumn {
    pointer: NonNull<u8>,
    len: usize,
    capacity: usize,
    item_size: usize,
    item_align: usize,
    drop_fn: Option<unsafe fn(*mut u8)>,
}

unsafe impl Send for ErasedColumn {}
unsafe impl Sync for ErasedColumn {}

unsafe fn drop_in_place_as<T>(pointer: *mut u8) {
    unsafe { ptr::drop_in_place(pointer.cast::<T>()) }
}

impl ErasedColumn {
    pub(crate) fn new<T: Send + Sync + 'static>() -> Self {
        let layout = Layout::new::<T>();
        ErasedColumn {
            pointer: NonNull::new(layout.align() as *mut u8).expect("alignment is never zero"),
            len: 0,
            capacity: 0,
            item_size: layout.size(),
            item_align: layout.align(),
            drop_fn: std::mem::needs_drop::<T>()
                .then_some(drop_in_place_as::<T> as unsafe fn(*mut u8)),
        }
    }

    #[inline]
    fn element_pointer(&self, index: usize) -> *mut u8 {
        unsafe { self.pointer.as_ptr().add(index * self.item_size) }
    }

    #[inline]
    pub(crate) fn slice<T: 'static>(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.pointer.as_ptr().cast::<T>(), self.len) }
    }

    #[inline]
    pub(crate) fn slice_mut<T: 'static>(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.pointer.as_ptr().cast::<T>(), self.len) }
    }

    #[inline]
    pub(crate) fn item_layout(&self) -> (usize, usize) {
        (self.item_size, self.item_align)
    }

    #[inline]
    pub(crate) fn live_bytes(&self) -> usize {
        self.len * self.item_size
    }

    #[inline]
    pub(crate) fn base_pointer(&self) -> *mut u8 {
        self.pointer.as_ptr()
    }

    #[inline]
    pub(crate) fn rebind(&mut self, base: *mut u8, capacity: usize) {
        if self.item_size != 0 {
            self.pointer = NonNull::new(base).expect("table arena base is never null");
        }
        self.capacity = capacity;
    }

    pub(crate) fn push<T: 'static>(&mut self, value: T) {
        if self.item_size == 0 {
            std::mem::forget(value);
            self.len += 1;
            return;
        }
        assert!(
            self.len < self.capacity,
            "column pushed past the reserved table capacity"
        );
        unsafe { ptr::write(self.element_pointer(self.len).cast::<T>(), value) }
        self.len += 1;
    }

    fn extend_with<T: 'static>(&mut self, count: usize, mut make: impl FnMut() -> T) {
        if count == 0 {
            return;
        }
        if self.item_size == 0 {
            for _ in 0..count {
                std::mem::forget(make());
            }
            self.len += count;
            return;
        }
        assert!(
            self.len + count <= self.capacity,
            "column extended past the reserved table capacity"
        );
        let base = self.element_pointer(self.len).cast::<T>();
        for index in 0..count {
            unsafe { ptr::write(base.add(index), make()) };
        }
        self.len += count;
    }

    pub(crate) fn swap_remove(&mut self, index: usize) {
        assert!(index < self.len, "column swap_remove index out of bounds");
        if let Some(drop_fn) = self.drop_fn {
            unsafe { drop_fn(self.element_pointer(index)) }
        }
        self.len -= 1;
        if index != self.len && self.item_size != 0 {
            unsafe {
                ptr::copy_nonoverlapping(
                    self.element_pointer(self.len),
                    self.element_pointer(index),
                    self.item_size,
                )
            }
        }
    }

    pub(crate) fn swap_remove_into(&mut self, index: usize, destination: &mut ErasedColumn) {
        assert!(
            index < self.len,
            "column swap_remove_into index out of bounds"
        );
        assert!(
            destination.item_size == 0 || destination.len < destination.capacity,
            "migration destination was not reserved"
        );
        if self.item_size != 0 {
            unsafe {
                ptr::copy_nonoverlapping(
                    self.element_pointer(index),
                    destination.element_pointer(destination.len),
                    self.item_size,
                )
            }
        }
        destination.len += 1;
        self.len -= 1;
        if index != self.len && self.item_size != 0 {
            unsafe {
                ptr::copy_nonoverlapping(
                    self.element_pointer(self.len),
                    self.element_pointer(index),
                    self.item_size,
                )
            }
        }
    }
}

impl Drop for ErasedColumn {
    fn drop(&mut self) {
        if let Some(drop_fn) = self.drop_fn {
            for index in 0..self.len {
                unsafe { drop_fn(self.element_pointer(index)) }
            }
        }
    }
}

pub(crate) fn column_new<T: Send + Sync + Default + 'static>() -> ErasedColumn {
    ErasedColumn::new::<T>()
}

pub(crate) fn column_vec<T: 'static>(column: &ErasedColumn) -> &[T] {
    column.slice::<T>()
}

pub(crate) fn column_vec_mut<T: 'static>(column: &mut ErasedColumn) -> &mut [T] {
    column.slice_mut::<T>()
}

pub(crate) fn column_push_default<T: Send + Sync + Default + 'static>(
    column: &mut ErasedColumn,
    count: usize,
) {
    column.extend_with(count, T::default);
}
