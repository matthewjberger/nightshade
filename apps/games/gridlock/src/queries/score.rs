use crate::data::look::{
    FLASH_COLORS, FLASH_GROWTH, FLASH_POP, FLASH_POP_MAX, FLASH_POP_STEP, FLASH_WORDS,
};
use nightshade::prelude::Vec4;

pub fn flash_value(base: u64, streak: u32) -> u64 {
    (0..streak.saturating_sub(1)).fold(base, |value, _| value.saturating_mul(FLASH_GROWTH))
}

pub fn flash_text(value: u64, streak: u32) -> String {
    let digits = value.to_string();
    let mut grouped = String::new();
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    let word = FLASH_WORDS[(streak as usize).min(FLASH_WORDS.len() - 1)];
    let bangs = "!".repeat((streak as usize / 3).min(5));
    if word.is_empty() {
        format!("+{grouped}{bangs}")
    } else {
        format!("+{grouped} {word}{bangs}")
    }
}

pub fn flash_color(streak: u32) -> Vec4 {
    FLASH_COLORS[streak as usize % FLASH_COLORS.len()]
}

pub fn flash_pop(streak: u32) -> f32 {
    (FLASH_POP + streak as f32 * FLASH_POP_STEP).min(FLASH_POP_MAX)
}
