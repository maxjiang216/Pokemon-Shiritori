//! Map English Pokémon names to first/last letters (`a`–`z` indices).
//!
//! Non-ASCII letters are dropped; ASCII letters are lowercased. Spaces and
//! punctuation (including apostrophes and gender symbols) are ignored.

/// Returns zero-based indices `0..=25` for `a`–`z`, or `None` if there are no letters.
pub fn first_last_letters(name: &str) -> Option<(u8, u8)> {
    let bytes: Vec<u8> = name
        .chars()
        .flat_map(|c| c.to_lowercase())
        .filter(|c| c.is_ascii_alphabetic())
        .map(|c| (c as u8).saturating_sub(b'a'))
        .collect();
    if bytes.is_empty() {
        return None;
    }
    Some((*bytes.first().unwrap(), *bytes.last().unwrap()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mr_mime() {
        assert_eq!(first_last_letters("Mr. Mime"), Some((b'm' - b'a', b'e' - b'a')));
    }

    #[test]
    fn farfetchd_curly_apostrophe() {
        assert_eq!(
            first_last_letters("Farfetch’d"),
            Some((b'f' - b'a', b'd' - b'a'))
        );
    }

    #[test]
    fn farfetchd_ascii_apostrophe() {
        assert_eq!(
            first_last_letters("Farfetch'd"),
            Some((b'f' - b'a', b'd' - b'a'))
        );
    }

    #[test]
    fn nidoran_forms_share_letters_only() {
        let a = first_last_letters("Nidoran♀").unwrap();
        let b = first_last_letters("Nidoran♂").unwrap();
        assert_eq!(a, b);
        assert_eq!(a, (b'n' - b'a', b'n' - b'a'));
    }

    #[test]
    fn pikachu() {
        assert_eq!(first_last_letters("Pikachu"), Some((b'p' - b'a', b'u' - b'a')));
    }
}
