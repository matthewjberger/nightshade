#[derive(Default)]
pub struct TextState {
    pub cache: TextCache,
    pub font_engine: crate::contract::data::font_engine::FontEngine,
}

#[derive(Default)]
pub struct TextCache {
    pub text_strings: Vec<Option<String>>,
    pub next_free_slot: usize,
    pub slot_generations: Vec<u64>,
    pub generation: u64,
}

impl TextCache {
    pub fn new() -> Self {
        Self {
            text_strings: Vec::new(),
            next_free_slot: 0,
            slot_generations: Vec::new(),
            generation: 0,
        }
    }

    pub fn add_text(&mut self, content: impl Into<String>) -> usize {
        let content = content.into();
        self.generation += 1;

        if self.next_free_slot < self.text_strings.len() {
            let slot = self.next_free_slot;
            self.text_strings[slot] = Some(content);
            self.slot_generations[slot] += 1;
            self.next_free_slot = self.find_next_free_slot();
            slot
        } else {
            let slot = self.text_strings.len();
            self.text_strings.push(Some(content));
            self.slot_generations.push(1);
            self.next_free_slot = slot + 1;
            slot
        }
    }

    pub fn get_text(&self, index: usize) -> Option<&str> {
        self.text_strings.get(index).and_then(|opt| opt.as_deref())
    }

    pub fn set_text(&mut self, index: usize, content: impl Into<String>) -> bool {
        let Some(text_opt) = self.text_strings.get_mut(index) else {
            return false;
        };
        let content = content.into();
        if text_opt.as_deref() == Some(content.as_str()) {
            return true;
        }
        *text_opt = Some(content);
        self.slot_generations[index] += 1;
        self.generation += 1;
        true
    }

    pub fn remove_text(&mut self, index: usize) -> bool {
        if let Some(text_opt) = self.text_strings.get_mut(index) {
            if text_opt.is_some() {
                *text_opt = None;
                self.slot_generations[index] += 1;
                if index < self.next_free_slot {
                    self.next_free_slot = index;
                }
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn is_slot_valid(&self, index: usize) -> bool {
        self.text_strings
            .get(index)
            .is_some_and(|opt| opt.is_some())
    }

    pub fn get_all_text_with_indices(&self) -> Vec<(usize, &str)> {
        self.text_strings
            .iter()
            .enumerate()
            .filter_map(|(i, opt)| opt.as_deref().map(|s| (i, s)))
            .collect()
    }

    pub fn get_valid_slots(&self) -> Vec<usize> {
        self.text_strings
            .iter()
            .enumerate()
            .filter_map(|(i, opt)| if opt.is_some() { Some(i) } else { None })
            .collect()
    }

    pub fn compact(&mut self) -> Vec<(usize, usize)> {
        let mut new_strings = Vec::new();
        let mut new_generations = Vec::new();
        let mut index_mapping = Vec::new();

        for (old_index, opt) in self.text_strings.iter().enumerate() {
            if let Some(text) = opt {
                index_mapping.push((old_index, new_strings.len()));
                new_strings.push(Some(text.clone()));
                new_generations.push(self.slot_generations.get(old_index).copied().unwrap_or(1));
            }
        }

        self.text_strings = new_strings;
        self.slot_generations = new_generations;
        self.next_free_slot = self.text_strings.len();

        index_mapping
    }

    fn find_next_free_slot(&self) -> usize {
        for (i, opt) in self.text_strings.iter().enumerate() {
            if opt.is_none() {
                return i;
            }
        }
        self.text_strings.len()
    }
}
