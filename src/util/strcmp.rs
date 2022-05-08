/// Compares to string references `s0` and `s1` with case conversion.
/// 
/// Returns `true` if they are deemed equivalent without regarding case sensivity.
pub fn cmp_ignore_case(s0: &str, s1: &str) -> bool {
    if s0.len() != s1.len() { return false }
    let mut s0 = s0.chars();
    let mut s1 = s1.chars();
    while let Some(c) = s0.next() {
        if c.to_lowercase().cmp(s1.next().unwrap().to_lowercase()) != std::cmp::Ordering::Equal {
            return false
        }
    }
    true
}

/// Compares to string references `s0` and `s1` with only ascii case conversion.
/// 
/// Returns `true` if they are deemed equivalent without regarding ascii case sensivity.
pub fn cmp_ascii_ignore_case(s0: &str, s1: &str) -> bool {
    if s0.len() != s1.len() { return false }
    let mut s0 = s0.chars();
    let mut s1 = s1.chars();
    while let Some(c) = s0.next() {
        if c.to_ascii_lowercase() != s1.next().unwrap().to_ascii_lowercase() {
            return false
        }
    }
    true
}

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn it_works() {
        let s0 = "ABC";
        let s1 = "abc";
        assert_eq!(cmp_ignore_case(s0, s1), true);
        assert_eq!(cmp_ascii_ignore_case(s0, s1), true);

        // negative case: different lengths
        let s0 = "ABCD";
        let s1 = "abc";
        assert_eq!(cmp_ignore_case(s0, s1), false);
        assert_eq!(cmp_ascii_ignore_case(s0, s1), false);

        // negative case: different letter order
        let s0 = "cba";
        let s1 = "abc";
        assert_eq!(cmp_ignore_case(s0, s1), false);
        assert_eq!(cmp_ascii_ignore_case(s0, s1), false);

        // VHDL-2008 LRM p226
        let s0 = "ABCDEFGHIJKLMNOPQRSTUVWXYZÀÁÂÃÄÅÆÇÈÉÊËÌÍÎÏÐÑÒÓÔÕÖØÙÚÛÜÝÞ";
        let s1 = "abcdefghijklmnopqrstuvwxyzàáâãäåæçèéêëìíîïðñòóôõöøùúûüýþ";
        assert_eq!(cmp_ignore_case(s0, s1), true);
        assert_eq!(cmp_ascii_ignore_case(s0, s1), false);

        // these 2 letters do not have upper-case equivalents
        let s0 = "ß";
        let s1 = "ÿ";
        assert_eq!(cmp_ignore_case(s0, s1), false);
        assert_eq!(cmp_ascii_ignore_case(s0, s1), false);
    }
}