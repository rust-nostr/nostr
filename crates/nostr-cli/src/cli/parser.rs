// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::sync::LazyLock;

use regex::Regex;

static MAIN_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?m:\s*(?:([^\s\\'"]+)|'([^']*)'|"((?:[^"\\]|\\.)*)"|(\\.?)|(\S))(\s|\z)?)"#)
        .unwrap()
});
static ESCAPE_PATTERN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\\(.)").unwrap());
static METACHAR_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"\\([$`"\\\n])"#).unwrap());

/// Splits a string into a vector of words in the same way the UNIX Bourne shell does.
///
/// This function does not behave like a full command line parser. Only single quotes, double
/// quotes, and backslashes are treated as metacharacters. Within double quoted strings,
/// backslashes are only treated as metacharacters when followed by one of the following
/// characters:
///
/// * $
/// * `
/// * "
/// * \
/// * newline
///
/// # Errors
///
/// If the input contains mismatched quotes (a quoted string missing a matching ending quote),
/// a `MismatchedQuotes` error is returned.
pub fn split(input: &str) -> Result<Vec<String>, MismatchedQuotes> {
    let mut words = Vec::new();
    let mut field = String::new();

    for capture in MAIN_PATTERN.captures_iter(input) {
        if let Some(word) = capture.get(1) {
            field.push_str(word.as_str());
        } else if let Some(single_quoted_word) = capture.get(2) {
            field.push_str(single_quoted_word.as_str());
        } else if let Some(double_quoted_word) = capture.get(3) {
            field.push_str(&METACHAR_PATTERN.replace_all(double_quoted_word.as_str(), "$1"));
        } else if let Some(escape) = capture.get(4) {
            field.push_str(&ESCAPE_PATTERN.replace_all(escape.as_str(), "$1"));
        } else if capture.get(5).is_some() {
            return Err(MismatchedQuotes);
        }

        if capture.get(6).is_some() {
            words.push(field);
            field = String::new();
        }
    }

    Ok(words)
}

/// An error when splitting a string with mismatched quotes.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct MismatchedQuotes;

impl std::fmt::Display for MismatchedQuotes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Mismatched quotes")
    }
}

impl std::error::Error for MismatchedQuotes {}

#[cfg(test)]
mod tests {
    use super::{MismatchedQuotes, split};

    #[test]
    fn nothing_special() {
        assert_eq!(split("a b c d").unwrap(), ["a", "b", "c", "d"]);
    }

    #[test]
    fn quoted_strings() {
        assert_eq!(split("a \"b b\" a").unwrap(), ["a", "b b", "a"]);
    }

    #[test]
    fn escaped_double_quotes() {
        assert_eq!(split("a \"\\\"b\\\" c\" d").unwrap(), ["a", "\"b\" c", "d"]);
    }

    #[test]
    fn escaped_single_quotes() {
        assert_eq!(split("a \"'b' c\" d").unwrap(), ["a", "'b' c", "d"]);
    }

    #[test]
    fn escaped_spaces() {
        assert_eq!(split("a b\\ c d").unwrap(), ["a", "b c", "d"]);
    }

    #[test]
    fn bad_double_quotes() {
        assert_eq!(split("a \"b c d e").unwrap_err(), MismatchedQuotes);
    }

    #[test]
    fn bad_single_quotes() {
        assert_eq!(split("a 'b c d e").unwrap_err(), MismatchedQuotes);
    }

    #[test]
    fn bad_quotes() {
        assert_eq!(split("one '\"\"\"").unwrap_err(), MismatchedQuotes);
    }

    #[test]
    fn trailing_whitespace() {
        assert_eq!(split("a b c d ").unwrap(), ["a", "b", "c", "d"]);
    }

    #[test]
    fn percent_signs() {
        assert_eq!(split("abc '%foo bar%'").unwrap(), ["abc", "%foo bar%"]);
    }
}
