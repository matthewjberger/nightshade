pub const EVENT_CHANNEL_CAPACITY: usize = 262_144;

#[derive(Clone)]
pub struct EventChannel<T> {
    pub events: Vec<T>,
    pub base_sequence: u64,
    pub previous_update_sequence: u64,
}

impl<T> Default for EventChannel<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> EventChannel<T> {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            base_sequence: 0,
            previous_update_sequence: 0,
        }
    }

    #[inline]
    pub fn send(&mut self, event: T) {
        if self.events.len() >= EVENT_CHANNEL_CAPACITY {
            self.drop_oldest_half();
        }
        self.events.push(event);
    }

    #[cold]
    fn drop_oldest_half(&mut self) {
        let half = self.events.len() / 2;
        self.events.drain(..half);
        self.base_sequence += half as u64;
    }

    #[inline]
    pub fn sequence(&self) -> u64 {
        self.base_sequence + self.events.len() as u64
    }

    #[inline]
    pub fn events_since(&self, cursor: u64) -> &[T] {
        let start = cursor
            .saturating_sub(self.base_sequence)
            .min(self.events.len() as u64) as usize;
        &self.events[start..]
    }

    #[inline]
    pub fn consume(&self, cursor: &mut u64) -> &[T] {
        let events = self.events_since(*cursor);
        *cursor = self.sequence();
        events
    }

    #[inline]
    pub fn read_frame(&self) -> &[T] {
        let end = self
            .previous_update_sequence
            .saturating_sub(self.base_sequence)
            .min(self.events.len() as u64) as usize;
        &self.events[..end]
    }

    pub fn trim(&mut self, up_to_sequence: u64) {
        let drop_count = up_to_sequence
            .saturating_sub(self.base_sequence)
            .min(self.events.len() as u64) as usize;
        self.events.drain(..drop_count);
        self.base_sequence += drop_count as u64;
    }

    pub fn update(&mut self) {
        let expire = self.previous_update_sequence;
        self.trim(expire);
        self.previous_update_sequence = self.sequence();
    }

    pub fn clear(&mut self) {
        let sequence = self.sequence();
        self.trim(sequence);
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}
