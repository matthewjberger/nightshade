use crate::components::CharStyle;

use super::text_cursor::{
    char_position_from_line_col, line_col_from_char_position, line_count, line_start_char_index,
    line_text, next_word_boundary, prev_word_boundary,
};

pub(super) struct EditBuffer {
    pub text: String,
    pub styles: Vec<CharStyle>,
    pub styled: bool,
    pub cursor: usize,
    pub selection: Option<usize>,
    pub multiline: bool,
}

impl EditBuffer {
    pub fn new(text: String, cursor: usize, selection: Option<usize>, multiline: bool) -> Self {
        let length = text.chars().count();
        Self {
            text,
            styles: Vec::new(),
            styled: false,
            cursor: cursor.min(length),
            selection: selection.map(|anchor| anchor.min(length)),
            multiline,
        }
    }

    pub fn styled(
        text: String,
        styles: Vec<CharStyle>,
        cursor: usize,
        selection: Option<usize>,
        multiline: bool,
    ) -> Self {
        let length = text.chars().count();
        Self {
            text,
            styles,
            styled: true,
            cursor: cursor.min(length),
            selection: selection.map(|anchor| anchor.min(length)),
            multiline,
        }
    }

    pub fn length(&self) -> usize {
        self.text.chars().count()
    }

    pub fn selection_range(&self) -> Option<(usize, usize)> {
        self.selection
            .map(|anchor| (anchor.min(self.cursor), anchor.max(self.cursor)))
    }

    pub fn selected_text(&self) -> Option<String> {
        self.selection_range().map(|(min, max)| {
            self.text
                .chars()
                .skip(min)
                .take(max.saturating_sub(min))
                .collect()
        })
    }

    fn remove_range(&mut self, min: usize, max: usize) {
        let chars: Vec<char> = self.text.chars().collect();
        let mut next: String = chars[..min].iter().collect();
        next.extend(chars[max..].iter());
        self.text = next;
        if self.styled && max <= self.styles.len() {
            self.styles.drain(min..max);
        }
    }

    pub fn delete_selection(&mut self) -> bool {
        if let Some(anchor) = self.selection {
            let min = anchor.min(self.cursor);
            let max = anchor.max(self.cursor);
            self.remove_range(min, max);
            self.cursor = min;
            self.selection = None;
            true
        } else {
            false
        }
    }

    pub fn insert_char(&mut self, character: char, style: CharStyle) {
        if let Some(anchor) = self.selection {
            let min = anchor.min(self.cursor);
            let max = anchor.max(self.cursor);
            self.remove_range(min, max);
            self.cursor = min;
            self.selection = None;
        }
        let chars: Vec<char> = self.text.chars().collect();
        let mut next: String = chars[..self.cursor].iter().collect();
        next.push(character);
        next.extend(chars[self.cursor..].iter());
        self.text = next;
        if self.styled {
            self.styles.insert(self.cursor, style);
        }
        self.cursor += 1;
    }

    pub fn insert_str(&mut self, value: &str, style: CharStyle) {
        if let Some(anchor) = self.selection {
            let min = anchor.min(self.cursor);
            let max = anchor.max(self.cursor);
            self.remove_range(min, max);
            self.cursor = min;
            self.selection = None;
        }
        let inserted = value.chars().count();
        let chars: Vec<char> = self.text.chars().collect();
        let mut next: String = chars[..self.cursor].iter().collect();
        next.push_str(value);
        next.extend(chars[self.cursor..].iter());
        self.text = next;
        if self.styled {
            for offset in 0..inserted {
                self.styles.insert(self.cursor + offset, style.clone());
            }
        }
        self.cursor += inserted;
    }

    pub fn backspace(&mut self, ctrl: bool) -> bool {
        if self.delete_selection() {
            return true;
        }
        if self.cursor == 0 {
            return false;
        }
        let new_cursor = if ctrl {
            prev_word_boundary(&self.text, self.cursor)
        } else {
            self.cursor - 1
        };
        self.remove_range(new_cursor, self.cursor);
        self.cursor = new_cursor;
        true
    }

    pub fn delete_forward(&mut self, ctrl: bool) -> bool {
        if self.delete_selection() {
            return true;
        }
        let length = self.length();
        if self.cursor >= length {
            return false;
        }
        let end = if ctrl {
            next_word_boundary(&self.text, self.cursor)
        } else {
            self.cursor + 1
        };
        self.remove_range(self.cursor, end);
        true
    }

    pub fn move_left(&mut self, ctrl: bool, shift: bool) {
        if shift {
            if self.selection.is_none() {
                self.selection = Some(self.cursor);
            }
        } else if self.selection.is_some() {
            self.cursor = self.selection.unwrap().min(self.cursor);
            self.selection = None;
            return;
        }
        if ctrl {
            self.cursor = prev_word_boundary(&self.text, self.cursor);
        } else {
            self.cursor = self.cursor.saturating_sub(1);
        }
        if !shift {
            self.selection = None;
        }
    }

    pub fn move_right(&mut self, ctrl: bool, shift: bool) {
        if shift {
            if self.selection.is_none() {
                self.selection = Some(self.cursor);
            }
        } else if self.selection.is_some() {
            self.cursor = self.selection.unwrap().max(self.cursor);
            self.selection = None;
            return;
        }
        if ctrl {
            self.cursor = next_word_boundary(&self.text, self.cursor);
        } else if self.cursor < self.length() {
            self.cursor += 1;
        }
        if !shift {
            self.selection = None;
        }
    }

    pub fn move_up(&mut self, shift: bool) {
        if shift && self.selection.is_none() {
            self.selection = Some(self.cursor);
        }
        let (line, column) = line_col_from_char_position(&self.text, self.cursor);
        if line > 0 {
            self.cursor = char_position_from_line_col(&self.text, line - 1, column);
        } else {
            self.cursor = 0;
        }
        if !shift {
            self.selection = None;
        }
    }

    pub fn move_down(&mut self, shift: bool) {
        if shift && self.selection.is_none() {
            self.selection = Some(self.cursor);
        }
        let (line, column) = line_col_from_char_position(&self.text, self.cursor);
        if line + 1 < line_count(&self.text) {
            self.cursor = char_position_from_line_col(&self.text, line + 1, column);
        } else {
            self.cursor = self.length();
        }
        if !shift {
            self.selection = None;
        }
    }

    pub fn move_home(&mut self, shift: bool) {
        if shift && self.selection.is_none() {
            self.selection = Some(self.cursor);
        }
        self.cursor = if self.multiline {
            let (line, _) = line_col_from_char_position(&self.text, self.cursor);
            line_start_char_index(&self.text, line)
        } else {
            0
        };
        if !shift {
            self.selection = None;
        }
    }

    pub fn move_end(&mut self, shift: bool) {
        if shift && self.selection.is_none() {
            self.selection = Some(self.cursor);
        }
        self.cursor = if self.multiline {
            let (line, _) = line_col_from_char_position(&self.text, self.cursor);
            line_start_char_index(&self.text, line) + line_text(&self.text, line).chars().count()
        } else {
            self.length()
        };
        if !shift {
            self.selection = None;
        }
    }

    pub fn insert_newline(&mut self, style: CharStyle) {
        self.insert_char('\n', style);
    }

    pub fn select_all(&mut self) {
        self.selection = Some(0);
        self.cursor = self.length();
    }

    pub fn select_word_at_cursor(&mut self) {
        let length = self.length();
        let position = self.cursor.min(length);
        let word_start = prev_word_boundary(&self.text, position);
        let word_end = next_word_boundary(&self.text, position).min(length);
        self.selection = Some(word_start);
        self.cursor = word_end;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn buffer(text: &str, cursor: usize) -> EditBuffer {
        EditBuffer::new(text.to_string(), cursor, None, false)
    }

    #[test]
    fn insert_char_advances_cursor() {
        let mut edit = buffer("ac", 1);
        edit.insert_char('b', CharStyle::default());
        assert_eq!(edit.text, "abc");
        assert_eq!(edit.cursor, 2);
    }

    #[test]
    fn insert_str_inserts_at_cursor() {
        let mut edit = buffer("ad", 1);
        edit.insert_str("bc", CharStyle::default());
        assert_eq!(edit.text, "abcd");
        assert_eq!(edit.cursor, 3);
    }

    #[test]
    fn backspace_removes_previous_char() {
        let mut edit = buffer("abc", 2);
        assert!(edit.backspace(false));
        assert_eq!(edit.text, "ac");
        assert_eq!(edit.cursor, 1);
    }

    #[test]
    fn backspace_at_start_is_noop() {
        let mut edit = buffer("abc", 0);
        assert!(!edit.backspace(false));
        assert_eq!(edit.text, "abc");
    }

    #[test]
    fn ctrl_backspace_removes_word() {
        let mut edit = buffer("hello world", 11);
        assert!(edit.backspace(true));
        assert_eq!(edit.text, "hello ");
        assert_eq!(edit.cursor, 6);
    }

    #[test]
    fn delete_forward_removes_next_char() {
        let mut edit = buffer("abc", 1);
        assert!(edit.delete_forward(false));
        assert_eq!(edit.text, "ac");
        assert_eq!(edit.cursor, 1);
    }

    #[test]
    fn insert_over_selection_replaces() {
        let mut edit = EditBuffer::new("abcd".to_string(), 3, Some(1), false);
        edit.insert_char('X', CharStyle::default());
        assert_eq!(edit.text, "aXd");
        assert_eq!(edit.cursor, 2);
        assert_eq!(edit.selection, None);
    }

    #[test]
    fn select_all_then_backspace_clears() {
        let mut edit = buffer("hello", 0);
        edit.select_all();
        assert_eq!(edit.selection_range(), Some((0, 5)));
        assert!(edit.backspace(false));
        assert_eq!(edit.text, "");
        assert_eq!(edit.cursor, 0);
    }

    #[test]
    fn shift_move_left_extends_selection() {
        let mut edit = buffer("abc", 3);
        edit.move_left(false, true);
        edit.move_left(false, true);
        assert_eq!(edit.cursor, 1);
        assert_eq!(edit.selection_range(), Some((1, 3)));
        assert_eq!(edit.selected_text(), Some("bc".to_string()));
    }

    #[test]
    fn move_right_without_shift_collapses_selection() {
        let mut edit = EditBuffer::new("abc".to_string(), 0, Some(2), false);
        edit.move_right(false, false);
        assert_eq!(edit.cursor, 2);
        assert_eq!(edit.selection, None);
    }

    #[test]
    fn select_word_at_cursor_selects_word() {
        let mut edit = buffer("hello world", 8);
        edit.select_word_at_cursor();
        assert_eq!(edit.selected_text(), Some("world".to_string()));
    }
}
