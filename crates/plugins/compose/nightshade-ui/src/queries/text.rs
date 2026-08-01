pub fn thousands(value: u64) -> String {
    let digits = value.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            out.push(',');
        }
        out.push(digit);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_short_count_is_left_alone() {
        assert_eq!(thousands(0), "0");
        assert_eq!(thousands(999), "999");
    }

    #[test]
    fn a_long_count_is_grouped_in_threes() {
        assert_eq!(thousands(1_000), "1,000");
        assert_eq!(thousands(100_000), "100,000");
        assert_eq!(thousands(1_000_000), "1,000,000");
        assert_eq!(thousands(4_250_000), "4,250,000");
    }
}
