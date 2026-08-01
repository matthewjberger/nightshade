pub fn config_label(key: &str) -> String {
    let mut label = String::with_capacity(key.len() + 4);
    for (position, letter) in key.char_indices() {
        if position > 0 && letter.is_ascii_uppercase() {
            label.push(' ');
        }
        label.push(letter);
    }
    label
}
