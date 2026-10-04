//! A backreference that folds case by ASCII alone: `(?i-u:\1)`.
//!
//! Python's `re.ASCII | re.IGNORECASE` compares a reference so: an ASCII letter
//! matches either case of itself, and every other character only itself. Unicode
//! folding is wider -- `é` against `É`, `k` against KELVIN SIGN -- so a pattern
//! translated from Python needs this per reference, without turning Unicode off
//! anywhere else.

use fancy_regex::{Error, ParseError, Regex};

fn found(pattern: &str, haystack: &str) -> Option<String> {
    Regex::new(pattern)
        .unwrap()
        .find(haystack)
        .unwrap()
        .map(|m| m.as_str().to_string())
}

#[test]
fn ascii_letters_fold() {
    assert_eq!(found(r"(s)(?i-u:\1)", "sS").as_deref(), Some("sS"));
    assert_eq!(found(r"(\w+) (?i-u:\1)", "ab AB").as_deref(), Some("ab AB"));
    assert_eq!(found(r"(?i)(s)(?-u:\1)", "sS").as_deref(), Some("sS"));
}

#[test]
fn every_other_character_matches_only_itself() {
    assert_eq!(found(r"(é)(?i-u:\1)", "éÉ"), None);
    assert_eq!(found(r"(é)(?i-u:\1)", "éé").as_deref(), Some("éé"));
    assert_eq!(found(r"(s)(?i-u:\1)", "sſ"), None);
    assert_eq!(found(r"(k)(?i-u:\1)", "k\u{212A}"), None);
    // Unicode folding, for contrast, takes the wide pair.
    assert_eq!(found(r"(é)(?i:\1)", "éÉ").as_deref(), Some("éÉ"));
}

#[test]
fn mixed_text_folds_its_ascii_and_keeps_the_rest() {
    assert_eq!(found(r"(.+) (?i-u:\1)", "éS és").as_deref(), Some("éS és"));
    assert_eq!(found(r"(.+) (?i-u:\1)", "éS És"), None);
    assert_eq!(found(r"(.)(?i-u:\1)", "sS").as_deref(), Some("sS"));
}

#[test]
fn named_references_fold_too() {
    assert_eq!(found(r"(?P<n>k)(?i-u:(?P=n))", "kK").as_deref(), Some("kK"));
    assert_eq!(found(r"(?<n>é)(?i-u:\k<n>)", "éÉ"), None);
    assert_eq!(found(r"(a)(b)(?i-u:\1\2)", "abAB").as_deref(), Some("abAB"));
}

#[test]
fn without_i_it_is_an_exact_comparison() {
    assert_eq!(found(r"(s)(?-u:\1)", "sS"), None);
    assert_eq!(found(r"(s)(?-u:\1)", "ss").as_deref(), Some("ss"));
}

/// Anything but backreferences in the group keeps today's refusal.
#[test]
fn any_other_unicode_change_is_still_refused() {
    for pattern in [
        r"(?-u)a",
        r"(?-u:a)",
        r"(a)(?-u:\1a)",
        r"(a)(?-u:\1|\1)",
        r"(?-u:)",
        r"(?i-u)",
    ] {
        assert!(
            matches!(
                Regex::new(pattern),
                Err(Error::ParseError(
                    _,
                    ParseError::ChangingUnicodeModeUnsupported
                ))
            ),
            "{pattern}: {:?}",
            Regex::new(pattern).err()
        );
    }
}
