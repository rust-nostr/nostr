// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr parser

use core::fmt;
use core::iter::Skip;
use core::str::{Chars, FromStr};

use bech32::Fe32;

use crate::nips::nip19::Nip19Prefix;
use crate::nips::nip21::{self, Nip21};
use crate::types::url::{ParseError, Url};

const BECH32_SEPARATOR: u8 = b'1';
const URL_SCHEME_SEPARATOR: &[u8] = b"://";
const HASHTAG_BYTE: u8 = b'#';
const LINE_BREAK_BYTE: u8 = b'\n';
const LINE_BREAK: &str = "\n";
const WHITESPACE: &str = " ";

/// Parser error
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// NIP21 error
    NIP21(nip21::Error),
    /// Url error
    Url(ParseError),
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NIP21(e) => write!(f, "{e}"),
            Self::Url(e) => write!(f, "{e}"),
        }
    }
}

impl From<nip21::Error> for Error {
    fn from(e: nip21::Error) -> Self {
        Self::NIP21(e)
    }
}

impl From<ParseError> for Error {
    fn from(e: ParseError) -> Self {
        Self::Url(e)
    }
}

/// Nostr parsed token
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Token<'a> {
    /// Nostr URI
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/21.md>
    Nostr(Nip21),
    /// Url
    Url(Url),
    /// Hashtag
    Hashtag(&'a str),
    /// Other text
    ///
    /// Spaces at the beginning or end of a text are parsed as [`Token::Whitespace`].
    Text(&'a str),
    /// Line break
    LineBreak,
    /// A whitespace
    Whitespace,
}

#[derive(Debug, Clone, Copy)]
struct Match {
    r#type: MatchType,
    start: usize,
    end: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum MatchType {
    NostrUri,
    Url,
    Hashtag,
    LineBreak,
}

/// Nostr parser
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct NostrParser {}

impl Default for NostrParser {
    fn default() -> Self {
        Self::new()
    }
}

impl NostrParser {
    /// Construct a new nostr parser
    ///
    /// # Patterns
    ///
    /// ## Urls
    ///
    /// Pattern details:
    /// - captures a scheme like `http`, `https`, `ftp`, etc.
    /// - matches the literal `://`
    /// - ensures the next part of the URL starts properly
    ///
    /// ## Nostr URIs
    ///
    /// Pattern details:
    /// - `nostr:`: the nostr URI prefix
    /// - `[a-z]`: the entity prefix (i.e., npub, naddr, ncryptsec)
    /// - `1`: the bech32 separator
    /// - `[qpzry9x8gf2tvdw0s3jn54khce6mua7l]`: the bech32 chars
    ///
    /// ## Hashtags
    ///
    /// Pattern details:
    /// - either the start of the string or a whitespace character;
    /// - `#`: the hashtag prefix;
    /// - `.,!?()[]{}\"'@#;:&*+=<>/\\|^~%$` and '`': chars that aren't allowed in the hashtag;
    ///
    /// ## Line breaks
    ///
    /// Pattern: `\n`
    #[inline]
    pub const fn new() -> Self {
        Self {}
    }

    /// Parse text
    #[inline]
    pub fn parse<'a>(&self, text: &'a str) -> NostrParserIter<'a> {
        NostrParserIter::new(text)
    }
}

/// Parsing options
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NostrParserOptions {
    /// Parse nostr URIs
    pub nostr_uris: bool,
    /// Parse URLs
    pub urls: bool,
    /// Parse hashtags
    pub hashtags: bool,
    /// Parse text, line breaks and whitespaces
    pub text: bool,
}

impl Default for NostrParserOptions {
    /// By default, parsing of all supported tokens is enabled.
    #[inline]
    fn default() -> Self {
        Self::enable_all()
    }
}

impl NostrParserOptions {
    const fn new(enabled: bool) -> Self {
        Self {
            nostr_uris: enabled,
            urls: enabled,
            hashtags: enabled,
            text: enabled,
        }
    }

    /// Enable parsing of all supported tokens
    #[inline]
    pub const fn enable_all() -> Self {
        Self::new(true)
    }

    /// Disable parsing of all supported tokens
    ///
    /// If you don't enable at least one pattern, nothing will be parsed!
    #[inline]
    pub const fn disable_all() -> Self {
        Self::new(false)
    }

    /// Enable parsing of nostr URIs ([`Token::Nostr`]).
    #[inline]
    pub const fn nostr_uris(mut self, enable: bool) -> Self {
        self.nostr_uris = enable;
        self
    }

    /// Enable parsing of URLs ([`Token::Url`]).
    #[inline]
    pub const fn urls(mut self, enable: bool) -> Self {
        self.urls = enable;
        self
    }

    /// Enable parsing of hashtags ([`Token::Hashtag`]).
    #[inline]
    pub const fn hashtags(mut self, enable: bool) -> Self {
        self.hashtags = enable;
        self
    }

    /// Include text and parse line breaks and whitespaces ([`Token::Text`], [`Token::LineBreak`] and [`Token::Whitespace`]).
    #[inline]
    pub const fn text(mut self, enable: bool) -> Self {
        self.text = enable;
        self
    }
}

struct FindMatches<'a> {
    // Text and bytes to parse
    text: &'a str,
    bytes: &'a [u8],
    // Current position
    pos: usize,
    // Options
    opts: NostrParserOptions,
}

impl<'a> FindMatches<'a> {
    #[inline]
    fn new(text: &'a str) -> Self {
        Self {
            text,
            bytes: text.as_bytes(),
            pos: 0,
            opts: NostrParserOptions::default(),
        }
    }

    fn try_parse_line_break(&self) -> Option<Match> {
        // Check if the first byte IS NOT '\n'
        if self.bytes[self.pos] != LINE_BREAK_BYTE {
            return None;
        }

        Some(Match {
            r#type: MatchType::LineBreak,
            start: self.pos,
            end: self.pos + 1,
        })
    }

    fn try_parse_hashtag(&self) -> Option<Match> {
        // Check if the first byte IS NOT '#'
        if self.bytes[self.pos] != HASHTAG_BYTE {
            return None;
        }

        // If the position isn't 0, meaning that is not the start of the string,
        // check if the previous character IS NOT a whitespace.
        if self.pos != 0 && !self.bytes[self.pos - 1].is_ascii_whitespace() {
            return None;
        }

        let start: usize = self.pos;
        let mut end: usize = self.pos + 1;

        // Get the char iterator
        // Skip the first character, because it's the '#' char.
        let chars: Skip<Chars> = self.text[start..].chars().skip(1);

        // Iterate over the characters, checking for forbidden characters.
        for ch in chars.into_iter() {
            // Check forbidden chars
            if is_forbidden_hashtag_char(ch) {
                break;
            }

            end += ch.len_utf8();
        }

        // Must have at least one character after #
        if end <= start + 1 {
            return None;
        }

        Some(Match {
            r#type: MatchType::Hashtag,
            start,
            end,
        })
    }

    fn try_parse_nostr_uri(&self) -> Option<Match> {
        let uri_prefix: &[u8] = nip21::SCHEME_WITH_COLON.as_bytes();

        let start: usize = self.pos;
        let mut end: usize = start + uri_prefix.len(); // start + "nostr:".len()

        // Check for "nostr:" prefix
        if self.bytes.get(start..end) != Some(uri_prefix) {
            return None;
        }

        // Get data post URI prefix
        let data_post_uri_prefix: &str = self.text.get(end..)?;

        // Check NIP19 prefix
        let nip19_prefix: Nip19Prefix = Nip19Prefix::from_str(data_post_uri_prefix).ok()?;

        // Update end position
        // end += <nip19_prefix>
        end += nip19_prefix.len();

        // Look for bech32 separator '1'
        if self.bytes.get(end) != Some(&BECH32_SEPARATOR) {
            return None;
        }
        end += 1;

        // Get bech32 data
        let bech32_data: &str = self.text.get(end..)?;

        // This will be used to check if the post-iteration increased the end position,
        // meaning that there are some valid bech32 chars.
        let bech32_data_start: usize = end;

        // Find the end of the valid bech32 chars
        for c in bech32_data.chars() {
            // Check if it's a valid bech32 char
            match Fe32::from_char(c) {
                Ok(..) => {
                    // The UTF8 len for bech32 chars will always be `1`
                    end += 1;
                }
                Err(..) => break,
            }
        }

        // Same end position as before: no valid bech32 chars found
        if end == bech32_data_start {
            return None;
        }

        Some(Match {
            r#type: MatchType::NostrUri,
            start,
            end,
        })
    }

    /// Identifies and validates URL schemes (the part before `://`, like `http://`, `https://`, `ftp://`, etc.)
    /// and returns the position immediately after the scheme separator if a valid scheme is found.
    fn find_post_url_scheme_position(&self) -> Option<usize> {
        if !self.bytes[self.pos].is_ascii_alphabetic() {
            return None;
        }

        let mut end: usize = self.pos + 1;

        while end < self.bytes.len() {
            let byte: u8 = self.bytes[end];

            if byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'-' | b'.') {
                end += 1;
                continue;
            }

            if self.bytes.get(end..end + URL_SCHEME_SEPARATOR.len()) == Some(URL_SCHEME_SEPARATOR) {
                return Some(end + URL_SCHEME_SEPARATOR.len());
            }

            break;
        }

        None
    }

    fn try_parse_url(&self) -> Option<Match> {
        let start: usize = self.pos;

        // Look for the post-scheme position
        let after_scheme_pos: usize = self.find_post_url_scheme_position()?;

        // Find the end of valid URL characters
        let end: usize = self.find_url_end(after_scheme_pos)?;

        // Trim trailing punctuation and handle parentheses
        let actual_end: usize = self.trim_url_end(start, after_scheme_pos, end);

        // Ensure we found at least some content after the scheme
        if actual_end <= after_scheme_pos {
            return None;
        }

        Some(Match {
            r#type: MatchType::Url,
            start,
            end: actual_end,
        })
    }

    /// Find the end position of valid URL characters
    #[inline]
    fn find_url_end(&self, after_scheme_pos: usize) -> Option<usize> {
        // Set the end as the post-scheme position
        let mut end: usize = after_scheme_pos;

        // Check URL characters
        while end < self.bytes.len() {
            let byte: u8 = self.bytes[end];

            // Stop at whitespace or non-allowed URL byte
            if byte.is_ascii_whitespace() || !is_allowed_url_byte(byte) {
                break;
            }

            end += 1;
        }

        // Ensure we found at least some content after the scheme
        if end <= after_scheme_pos {
            None
        } else {
            Some(end)
        }
    }

    /// Trim trailing punctuation and handle parentheses matching
    #[inline]
    fn trim_url_end(&self, start: usize, after_scheme_pos: usize, end: usize) -> usize {
        let mut actual_end: usize = end;

        // Remove trailing punctuation
        while actual_end > after_scheme_pos {
            let byte: u8 = self.bytes[actual_end - 1];
            if is_url_trailing_punctuation(byte) {
                actual_end -= 1;
            } else {
                break;
            }
        }

        // Handle unmatched closing parentheses
        if actual_end > after_scheme_pos && self.bytes[actual_end - 1] == b')' {
            actual_end = self.handle_parentheses_matching(start, actual_end);
        }

        actual_end
    }

    /// Handle parentheses matching for URLs
    #[inline]
    fn handle_parentheses_matching(&self, start: usize, mut actual_end: usize) -> usize {
        let url_bytes: &[u8] = &self.bytes[start..actual_end];

        // Count parentheses
        let mut open_count: usize = 0;
        let mut close_count: usize = 0;

        for &byte in url_bytes.iter() {
            match byte {
                b'(' => open_count += 1,
                b')' => close_count += 1,
                _ => {}
            }
        }

        // Remove trailing ')' if unmatched
        if close_count > open_count {
            actual_end -= 1;
        }

        actual_end
    }
}

impl Iterator for FindMatches<'_> {
    type Item = Match;

    fn next(&mut self) -> Option<Self::Item> {
        // Loop through the texts till a match is found
        while self.pos < self.bytes.len() {
            // Check if text parsing is enabled
            if self.opts.text {
                // Check for line break
                if let Some(mat) = self.try_parse_line_break() {
                    self.pos = mat.end;
                    return Some(mat);
                }
            }

            // Check if hashtags parsing is enabled
            if self.opts.hashtags {
                // Check for hashtag
                if let Some(mat) = self.try_parse_hashtag() {
                    self.pos = mat.end;
                    return Some(mat);
                }
            }

            // Check if nostr URIs parsing is enabled
            if self.opts.nostr_uris {
                // Check for nostr URI
                if let Some(mat) = self.try_parse_nostr_uri() {
                    self.pos = mat.end;
                    return Some(mat);
                }
            }

            // Check if URLs parsing is enabled
            if self.opts.urls {
                // Check for URL
                if let Some(mat) = self.try_parse_url() {
                    self.pos = mat.end;
                    return Some(mat);
                }
            }

            // Move to the next character (handle UTF-8)
            self.pos += if self.bytes[self.pos].is_ascii() {
                1
            } else {
                // For non-ASCII, we need to skip the full UTF-8 sequence
                self.text[self.pos..]
                    .chars()
                    .next()
                    .map(|c| c.len_utf8())
                    .unwrap_or(1)
            };
        }

        // No match is found
        None
    }
}

enum HandleMatch<'a> {
    /// Found a valid token
    Token(Token<'a>),
    /// No valid token found, perform recursion.
    Recursion,
}

/// Nostr parser iterator
pub struct NostrParserIter<'a> {
    /// The original text
    text: &'a str,
    /// Matches found
    matches: FindMatches<'a>,
    /// A pending match
    pending_match: Option<Match>,
    /// A pending match string
    pending_str_match: Option<&'a str>,
    /// Pending token
    pending_token: Option<Token<'a>>,
    /// Last match end index
    last_match_end: usize,
}

impl<'a> NostrParserIter<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            text,
            matches: FindMatches::new(text),
            pending_match: None,
            pending_str_match: None,
            pending_token: None,
            last_match_end: 0,
        }
    }

    /// Update parsing options
    #[inline]
    pub fn opts(mut self, opts: NostrParserOptions) -> Self {
        self.matches.opts = opts;
        self
    }

    fn handle_match(&mut self, mat: Match) -> HandleMatch<'a> {
        // Update last match end
        self.last_match_end = mat.end;

        // Extract the matched string
        let data: &str = &self.text[mat.start..mat.end];

        // Handle match type
        match mat.r#type {
            MatchType::Url => match Url::parse(data) {
                Ok(url) => HandleMatch::Token(Token::Url(url)),
                // If the URL parsing is invalid, the fallback is to treat it as text.
                // But may happen that the FindMatches failed to identify an invalid URL,
                // so this additional check prevents pushing a `Token::Text` if the `text` options isn't enabled.
                Err(_) => {
                    if self.matches.opts.text {
                        HandleMatch::Token(self.handle_str_as_text(data))
                    } else {
                        HandleMatch::Recursion
                    }
                }
            },
            MatchType::NostrUri => match Nip21::parse(data) {
                Ok(uri) => HandleMatch::Token(Token::Nostr(uri)),
                // If the nostr URI parsing is invalid, the fallback is to treat it as text.
                // But may happen that the FindMatches failed to identify an invalid nostr URI,
                // so this additional check prevents pushing a `Token::Text` if the `text` options isn't enabled.
                Err(_) => {
                    if self.matches.opts.text {
                        HandleMatch::Token(self.handle_str_as_text(data))
                    } else {
                        HandleMatch::Recursion
                    }
                }
            },
            MatchType::Hashtag => {
                if data.len() > 1 {
                    HandleMatch::Token(Token::Hashtag(&data[1..]))
                } else if self.matches.opts.text {
                    HandleMatch::Token(self.handle_str_as_text(data))
                } else {
                    HandleMatch::Recursion
                }
            }
            MatchType::LineBreak => HandleMatch::Token(Token::LineBreak),
        }
    }

    fn handle_str_as_text(&mut self, text_str: &'a str) -> Token<'a> {
        match text_str {
            // Line break
            LINE_BREAK => Token::LineBreak,
            // Line break with other stuff
            m if m.starts_with(LINE_BREAK) && m.len() > 1 => {
                self.pending_str_match = Some(&m[1..]);
                Token::LineBreak
            }
            // Whitespace
            WHITESPACE => Token::Whitespace,
            // Whitespace with other stuff
            m if m.starts_with(WHITESPACE) && m.len() > 1 => {
                self.pending_str_match = Some(&m[1..]);
                Token::Whitespace
            }
            // Stuff that terminate with a whitespace
            m if m.ends_with(WHITESPACE) && m.len() > 1 => {
                self.pending_token = Some(Token::Whitespace);

                // Handle it as text
                Token::Text(&m[..m.len() - 1])
            }
            // Fallback: treat it as plain text
            m => Token::Text(m),
        }
    }
}

impl<'a> Iterator for NostrParserIter<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // Return a pending token
        if let Some(pending_token) = self.pending_token.take() {
            #[cfg(all(feature = "std", debug_assertions, test))]
            dbg!(&pending_token);

            return Some(pending_token);
        }

        // Return a pending match string
        if let Some(pending_str_match) = self.pending_str_match.take() {
            #[cfg(all(feature = "std", debug_assertions, test))]
            dbg!(&pending_str_match);

            return Some(self.handle_str_as_text(pending_str_match));
        }

        // Handle a pending match
        if let Some(pending_match) = self.pending_match.take() {
            #[cfg(all(feature = "std", debug_assertions, test))]
            dbg!(&pending_match, self.last_match_end);

            return match self.handle_match(pending_match) {
                HandleMatch::Token(token) => Some(token),
                HandleMatch::Recursion => self.next(),
            };
        }

        match self.matches.next() {
            Some(mat) => {
                #[cfg(all(feature = "std", debug_assertions, test))]
                dbg!(&mat, self.last_match_end);

                // Capture text that appears before this match (if any)
                if self.matches.opts.text && mat.start > self.last_match_end {
                    // Update pending match
                    // This will be handled at next iteration, in `handle_match` method.
                    self.pending_match = Some(mat);

                    // Capture text that appears between last match and this match start
                    let data: &str = &self.text[self.last_match_end..mat.start];

                    // Return the token
                    return Some(self.handle_str_as_text(data));
                }

                // Handle match
                match self.handle_match(mat) {
                    HandleMatch::Token(token) => Some(token),
                    HandleMatch::Recursion => self.next(),
                }
            }
            None => {
                // Text disabled
                if !self.matches.opts.text {
                    return None;
                }

                // No text left
                if self.last_match_end >= self.text.len() {
                    return None;
                }

                // Handle missing text
                let data: &str = &self.text[self.last_match_end..];

                // Update last match end
                self.last_match_end = self.text.len();

                // Return the token
                Some(self.handle_str_as_text(data))
            }
        }
    }
}

#[inline]
fn is_forbidden_hashtag_char(ch: char) -> bool {
    // Whitespace and control characters
    if ch.is_whitespace() || ch.is_control() {
        return true;
    }

    matches!(
        ch,
        '.' | ','
            | '!'
            | '?'
            | '('
            | ')'
            | '['
            | ']'
            | '{'
            | '}'
            | '"'
            | '\''
            | '@'
            | '#'
            | ';'
            | ':'
            | '&'
            | '*'
            | '+'
            | '='
            | '<'
            | '>'
            | '/'
            | '\\'
            | '|'
            | '^'
            | '~'
            | '%'
            | '$'
            | '`'
    )
}

/// Allow URL characters: alphanumeric, and common URL symbols
#[inline]
const fn is_allowed_url_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric()
        || matches!(
            byte,
            b'-' | b'.'
                | b'_'
                | b'~'
                | b':'
                | b'/'
                | b'?'
                | b'#'
                | b'['
                | b']'
                | b'@'
                | b'!'
                | b'$'
                | b'&'
                | b'\''
                | b'('
                | b')'
                | b'*'
                | b'+'
                | b','
                | b';'
                | b'='
                | b'%'
        )
}

#[inline]
const fn is_url_trailing_punctuation(byte: u8) -> bool {
    matches!(
        byte,
        b'.' | b',' | b';' | b':' | b'!' | b'?' | b')' | b']' | b'}'
    )
}

#[cfg(test)]
mod tests {
    use alloc::vec;
    use alloc::vec::Vec;

    use super::*;
    use crate::nips::nip19::{FromBech32, Nip19Event, Nip19Profile};
    use crate::PublicKey;

    const PARSER: NostrParser = NostrParser::new();

    #[test]
    fn test_is_forbidden_hashtag_char() {
        assert!(is_forbidden_hashtag_char('!'));
        assert!(is_forbidden_hashtag_char('@'));
        assert!(is_forbidden_hashtag_char('#'));
        assert!(is_forbidden_hashtag_char('$'));
        assert!(is_forbidden_hashtag_char('%'));
        assert!(is_forbidden_hashtag_char('^'));
        assert!(is_forbidden_hashtag_char('&'));
        assert!(is_forbidden_hashtag_char('*'));
        assert!(is_forbidden_hashtag_char('('));
    }

    #[test]
    fn test_is_allowed_url_byte() {
        assert!(is_allowed_url_byte(b'a'));
        assert!(is_allowed_url_byte(b'A'));
        assert!(is_allowed_url_byte(b'0'));
        assert!(is_allowed_url_byte(b'9'));
        assert!(is_allowed_url_byte(b'.'));
        assert!(is_allowed_url_byte(b'_'));
        assert!(is_allowed_url_byte(b'~'));
        assert!(is_allowed_url_byte(b':'));
        assert!(is_allowed_url_byte(b'/'));
    }

    #[test]
    fn test_parse_links() {
        let test_cases = vec![
            "https://example.com",
            "http://test.org",
            "ftp://files.example.com",
            "https://example.com/path",
            "https://example.com/path/to/resource",
            "https://example.com/path/to/resource.html",
            "https://example.com/api/v1/users",
            "https://example.com/path_with_underscores",
            "https://example.com/path-with-hyphens",
            "https://example.com?param=value",
            "https://example.com?param1=value1&param2=value2",
            "https://example.com/path?query=test&sort=asc",
            "https://example.com?empty=&filled=value",
            "https://example.com?special=%20%21%40%23",
            "https://search.com?q=rust+programming",
            "https://example.com#section",
            "https://example.com/page#top",
            "https://example.com/docs#installation",
            "https://example.com?param=value#fragment",
            "http://localhost:8080",
            "https://example.com:443",
            "http://192.168.1.1:3000",
            "https://api.example.com:8443/v1",
            "https://user:pass@example.com",
            "ftp://username@files.example.com",
            "https://api_key@api.example.com/data",
        ];

        for input in test_cases {
            let tokens: Vec<Token> = PARSER.parse(input).collect();
            assert_eq!(tokens, vec![Token::Url(Url::parse(input).unwrap())]);
        }
    }

    #[test]
    fn test_parse_urls_with_trailing_punctuation() {
        let test_vector = vec![
            (
                "Check this out: https://example.com.",
                vec![
                    Token::Text("Check this out:"),
                    Token::Whitespace,
                    Token::Url(Url::parse("https://example.com").unwrap()),
                    Token::Text("."),
                ],
            ),
            (
                "Visit https://example.com!",
                vec![
                    Token::Text("Visit"),
                    Token::Whitespace,
                    Token::Url(Url::parse("https://example.com").unwrap()),
                    Token::Text("!"),
                ],
            ),
            (
                "See https://example.com?",
                vec![
                    Token::Text("See"),
                    Token::Whitespace,
                    Token::Url(Url::parse("https://example.com").unwrap()),
                    Token::Text("?"),
                ],
            ),
            (
                "Go to https://example.com;",
                vec![
                    Token::Text("Go to"),
                    Token::Whitespace,
                    Token::Url(Url::parse("https://example.com").unwrap()),
                    Token::Text(";"),
                ],
            ),
            (
                "Link: https://example.com,",
                vec![
                    Token::Text("Link:"),
                    Token::Whitespace,
                    Token::Url(Url::parse("https://example.com").unwrap()),
                    Token::Text(","),
                ],
            ),
            (
                "URL https://example.com:",
                vec![
                    Token::Text("URL"),
                    Token::Whitespace,
                    Token::Url(Url::parse("https://example.com").unwrap()),
                    Token::Text(":"),
                ],
            ),
        ];

        for (text, expected) in test_vector.into_iter() {
            let tokens = PARSER.parse(text).collect::<Vec<_>>();
            assert_eq!(tokens, expected);
        }
    }

    #[test]
    fn test_urls_with_parentheses() {
        let test_cases = vec![
            // Balanced parentheses should be included
            (
                "Check (https://example.com/path)",
                vec![
                    Token::Text("Check ("),
                    Token::Url(Url::parse("https://example.com/path").unwrap()),
                    Token::Text(")"),
                ],
            ),
            // Unbalanced closing parentheses should be excluded
            (
                "See https://example.com)",
                vec![
                    Token::Text("See"),
                    Token::Whitespace,
                    Token::Url(Url::parse("https://example.com").unwrap()),
                    Token::Text(")"),
                ],
            ),
            (
                "Visit https://example.com/page)",
                vec![
                    Token::Text("Visit"),
                    Token::Whitespace,
                    Token::Url(Url::parse("https://example.com/page").unwrap()),
                    Token::Text(")"),
                ],
            ),
            // Multiple unbalanced closing parentheses
            (
                "URL https://example.com))",
                vec![
                    Token::Text("URL"),
                    Token::Whitespace,
                    Token::Url(Url::parse("https://example.com").unwrap()),
                    Token::Text("))"),
                ],
            ),
        ];

        for (input, expected) in test_cases {
            let tokens: Vec<Token> = PARSER.parse(input).collect();
            assert_eq!(tokens, expected);
        }
    }

    #[test]
    fn test_urls_with_brackets() {
        let test_cases = vec![
            (
                "Check [https://example.com]",
                vec![
                    Token::Text("Check ["),
                    Token::Url(Url::parse("https://example.com").unwrap()),
                    Token::Text("]"),
                ],
            ),
            (
                "Link [https://example.com/page]",
                vec![
                    Token::Text("Link ["),
                    Token::Url(Url::parse("https://example.com/page").unwrap()),
                    Token::Text("]"),
                ],
            ),
            (
                "See https://example.com]",
                vec![
                    Token::Text("See"),
                    Token::Whitespace,
                    Token::Url(Url::parse("https://example.com").unwrap()),
                    Token::Text("]"),
                ],
            ),
        ];

        for (input, expected) in test_cases {
            let tokens: Vec<Token> = PARSER.parse(input).collect();
            assert_eq!(tokens, expected);
        }
    }

    #[test]
    fn test_urls_with_angle_brackets() {
        let test_cases = vec![
            (
                "Check <https://example.com>",
                vec![
                    Token::Text("Check <"),
                    Token::Url(Url::parse("https://example.com").unwrap()),
                    Token::Text(">"),
                ],
            ),
            (
                "Link <https://example.com/page>",
                vec![
                    Token::Text("Link <"),
                    Token::Url(Url::parse("https://example.com/page").unwrap()),
                    Token::Text(">"),
                ],
            ),
            (
                "See https://example.com>",
                vec![
                    Token::Text("See"),
                    Token::Whitespace,
                    Token::Url(Url::parse("https://example.com").unwrap()),
                    Token::Text(">"),
                ],
            ),
        ];

        for (input, expected) in test_cases {
            let tokens: Vec<Token> = PARSER.parse(input).collect();
            assert_eq!(tokens, expected);
        }
    }

    #[test]
    fn test_parse_link_with_only_scheme() {
        let text: &str = "https://";
        let tokens = PARSER.parse(text).collect::<Vec<_>>();
        assert_eq!(tokens, vec![Token::Text("https://")]);
    }

    #[test]
    fn test_parse_list_of_links() {
        let text: &str =
            "https://example.com#title-1, https://rust-nostr.org/donate, https://duckduckgo.com.";
        let tokens = PARSER.parse(text).collect::<Vec<_>>();
        assert_eq!(
            tokens,
            vec![
                Token::Url(Url::parse("https://example.com#title-1").unwrap()),
                Token::Text(","),
                Token::Whitespace,
                Token::Url(Url::parse("https://rust-nostr.org/donate").unwrap()),
                Token::Text(","),
                Token::Whitespace,
                Token::Url(Url::parse("https://duckduckgo.com").unwrap()),
                Token::Text("."),
            ]
        );
    }

    #[test]
    fn test_urls_in_text() {
        let input =
            "Visit https://example.com for more info about https://rust-lang.org programming.";
        let tokens: Vec<Token> = PARSER.parse(input).collect();
        assert_eq!(
            tokens,
            vec![
                Token::Text("Visit"),
                Token::Whitespace,
                Token::Url(Url::parse("https://example.com").unwrap()),
                Token::Whitespace,
                Token::Text("for more info about"),
                Token::Whitespace,
                Token::Url(Url::parse("https://rust-lang.org").unwrap()),
                Token::Whitespace,
                Token::Text("programming."),
            ]
        );
    }

    #[test]
    fn test_parse_ascii_hashtags() {
        // Valid hashtag
        let text: &str = "#bitcoin";
        let tokens = PARSER.parse(text).collect::<Vec<_>>();
        assert_eq!(tokens, vec![Token::Hashtag("bitcoin"),]);

        // List of hashtags
        let text: &str = "#bitcoin #lightning";
        let tokens = PARSER.parse(text).collect::<Vec<_>>();
        assert_eq!(
            tokens,
            vec![
                Token::Hashtag("bitcoin"),
                Token::Whitespace,
                Token::Hashtag("lightning"),
            ]
        );
    }

    #[test]
    fn test_parse_unicode_hashtags() {
        let test_vector: Vec<(&str, Vec<Token>)> = vec![
            ("#🚀", vec![Token::Hashtag("🚀")]),           // Emoji
            ("#₿", vec![Token::Hashtag("₿")]),             // Bitcoin symbol
            ("#東京", vec![Token::Hashtag("東京")]),       // Japanese
            ("#中文", vec![Token::Hashtag("中文")]),       // Chinese
            ("#москва", vec![Token::Hashtag("москва")]),   // Cyrillic
            ("#العربية", vec![Token::Hashtag("العربية")]), // Arabic
        ];

        for (text, expected) in test_vector.into_iter() {
            let tokens = PARSER.parse(text).collect::<Vec<_>>();
            assert_eq!(tokens, expected);
        }
    }

    #[test]
    fn test_parse_invalid_hashtags() {
        let test_vector: Vec<(&str, Vec<Token>)> = vec![
            ("bob#alice", vec![Token::Text("bob#alice")]),
            ("#", vec![Token::Text("#")]),
            ("# ", vec![Token::Text("#"), Token::Whitespace]),
            ("#\n", vec![Token::Text("#"), Token::LineBreak]),
            ("#$", vec![Token::Text("#$")]),
            ("#$", vec![Token::Text("#$")]),
        ];

        for (text, expected) in test_vector.into_iter() {
            let tokens = PARSER.parse(text).collect::<Vec<_>>();
            assert_eq!(tokens, expected);
        }
    }

    #[test]
    fn test_nostr_uri_with_brackets() {
        let test_cases = vec![
            (
                "Check [nostr:npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet]",
                vec![
                    Token::Text("Check ["),
                    Token::Nostr(
                        Nip21::parse(
                            "nostr:npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet",
                        )
                        .unwrap(),
                    ),
                    Token::Text("]"),
                ],
            ),
            (
                "See nostr:npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet]",
                vec![
                    Token::Text("See"),
                    Token::Whitespace,
                    Token::Nostr(
                        Nip21::parse(
                            "nostr:npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet",
                        )
                        .unwrap(),
                    ),
                    Token::Text("]"),
                ],
            ),
        ];

        for (input, expected) in test_cases {
            let tokens: Vec<Token> = PARSER.parse(input).collect();
            assert_eq!(tokens, expected);
        }
    }

    #[test]
    fn test_parse_text() {
        let text: &str = "Simple text.";
        let tokens = PARSER.parse(text).collect::<Vec<_>>();
        assert_eq!(tokens, vec![Token::Text("Simple text.")]);
    }

    #[test]
    fn test_parse_empty_text() {
        let text: &str = "";
        let tokens = PARSER.parse(text).collect::<Vec<_>>();
        assert_eq!(tokens, vec![]);
    }

    #[test]
    fn test_parse_complex() {
        let test_vectors: Vec<(&str, Vec<Token>)> = vec![
            ("Hello nostr:npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet, take a look at https://example.com/foo/bar.html. Thanks!", vec![
                Token::Text("Hello"),
                Token::Whitespace,
                Token::Nostr(Nip21::Pubkey(PublicKey::parse("npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet").unwrap())),
                Token::Text(", take a look at"),
                Token::Whitespace,
                Token::Url(Url::parse("https://example.com/foo/bar.html").unwrap()),
                Token::Text(". Thanks!"),
            ]),
            ("I have never been very active in discussions but working on rust-nostr (at the time called nostr-rs-sdk) since September 2022 🦀 \n\nIf I remember correctly there were also nostr:nprofile1qqsqfyvdlsmvj0nakmxq6c8n0c2j9uwrddjd8a95ynzn9479jhlth3gpvemhxue69uhkv6tvw3jhytnwdaehgu3wwa5kuef0dec82c33w94xwcmdd3cxketedsux6ertwecrgues0pk8xdrew33h27pkd4unvvpkw3nkv7pe0p68gat58ycrw6ps0fenwdnvva48w0mzwfhkzerrv9ehg0t5wf6k2qgnwaehxw309ac82unsd3jhqct89ejhxtcpz4mhxue69uhhyetvv9ujuerpd46hxtnfduhsh8njvk and nostr:nprofile1qqswuyd9ml6qcxd92h6pleptfrcqucvvjy39vg4wx7mv9wm8kakyujgpypmhxue69uhkx6r0wf6hxtndd94k2erfd3nk2u3wvdhk6w35xs6z7qgwwaehxw309ahx7uewd3hkctcpypmhxue69uhkummnw3ezuetfde6kuer6wasku7nfvuh8xurpvdjj7a0nq40", vec![
                Token::Text("I have never been very active in discussions but working on rust-nostr (at the time called nostr-rs-sdk) since September 2022 🦀"),
                Token::Whitespace,
                Token::LineBreak,
                Token::LineBreak,
                Token::Text("If I remember correctly there were also"),
                Token::Whitespace,
                Token::Nostr(Nip21::Profile(Nip19Profile::from_bech32("nprofile1qqsqfyvdlsmvj0nakmxq6c8n0c2j9uwrddjd8a95ynzn9479jhlth3gpvemhxue69uhkv6tvw3jhytnwdaehgu3wwa5kuef0dec82c33w94xwcmdd3cxketedsux6ertwecrgues0pk8xdrew33h27pkd4unvvpkw3nkv7pe0p68gat58ycrw6ps0fenwdnvva48w0mzwfhkzerrv9ehg0t5wf6k2qgnwaehxw309ac82unsd3jhqct89ejhxtcpz4mhxue69uhhyetvv9ujuerpd46hxtnfduhsh8njvk").unwrap())),
                Token::Whitespace,
                Token::Text("and"),
                Token::Whitespace,
                Token::Nostr(Nip21::Profile(Nip19Profile::from_bech32("nprofile1qqswuyd9ml6qcxd92h6pleptfrcqucvvjy39vg4wx7mv9wm8kakyujgpypmhxue69uhkx6r0wf6hxtndd94k2erfd3nk2u3wvdhk6w35xs6z7qgwwaehxw309ahx7uewd3hkctcpypmhxue69uhkummnw3ezuetfde6kuer6wasku7nfvuh8xurpvdjj7a0nq40").unwrap())),
            ]),
            ("nostr:nprofile1qqs8a474cw4lqmapcq8hr7res4nknar2ey34fsffk0k42cjsdyn7yqqpz9mhxue69uhkummnw3ezuamfdejj7qgwwaehxw309ahx7uewd3hkctcpzemhxue69uhk2er9dchxummnw3ezumrpdejz73pa0sl or anyone else who knows rust for that matter do you know if any good rust resources other than the rust book?", vec![
                Token::Nostr(Nip21::Profile(Nip19Profile::from_bech32("nprofile1qqs8a474cw4lqmapcq8hr7res4nknar2ey34fsffk0k42cjsdyn7yqqpz9mhxue69uhkummnw3ezuamfdejj7qgwwaehxw309ahx7uewd3hkctcpzemhxue69uhk2er9dchxummnw3ezumrpdejz73pa0sl").unwrap())),
                Token::Whitespace,
                Token::Text("or anyone else who knows rust for that matter do you know if any good rust resources other than the rust book?"),
            ]),
            ("I've uses both the book and rustlings: https://github.com/rust-lang/rustlings/\n\nThere is also the \"Rust by example\" book: https://doc.rust-lang.org/rust-by-example/\n\nWhile you read the book, try to make projects from scratch (not just simple ones). At the end, writing code is the best way to learn it.", vec![
                Token::Text("I've uses both the book and rustlings:"),
                Token::Whitespace,
                Token::Url(Url::parse("https://github.com/rust-lang/rustlings/").unwrap()),
                Token::LineBreak,
                Token::LineBreak,
                Token::Text("There is also the \"Rust by example\" book:"),
                Token::Whitespace,
                Token::Url(Url::parse("https://doc.rust-lang.org/rust-by-example/").unwrap()),
                Token::LineBreak,
                Token::LineBreak,
                Token::Text("While you read the book, try to make projects from scratch (not just simple ones). At the end, writing code is the best way to learn it."),
            ]),
            ("nostr:npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet", vec![
                Token::Nostr(Nip21::Pubkey(PublicKey::parse("nostr:npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet").unwrap()))
            ]),
            ("Hey nostr:nprofile1qqsx3kq3vkgczq9hmfplc28h687py42yvms3zkyxh8nmkvn0vhkyyuspz4mhxue69uhkummnw3ezummcw3ezuer9wchsz9thwden5te0wfjkccte9ejxzmt4wvhxjme0qy88wumn8ghj7mn0wvhxcmmv9u0uehfp  and #rust-nostr fans, can you enlighten me please:\nWhen I am calculating my Web of Trust I do the following:\n0. Create client with outbox model enabled\n1. Get my follows, mutes, reports in one fetch call\n2. Get follows, mutes, reports of my follows in another fetch call, using an authors filter that has all follows in it\n3. Calculate scores with my weights locally\n\nQuestion:\nWhy did step 2. take hours to complete?\n\nIt seems like it's trying to connect to loads of relays.\nMy guess is either I am doing sth horribly wrong or there is no smart relay set calculation for filters in the pool.\n\nIn ndk this calculation takes under 10 seconds to complete, even without any caching. It will first look at the filters and calculate a relay set that has all authors in it then does the fetching.\n\n#asknostr #rust", vec![
                Token::Text("Hey"),
                Token::Whitespace,
                Token::Nostr(Nip21::Profile(Nip19Profile::from_bech32("nprofile1qqsx3kq3vkgczq9hmfplc28h687py42yvms3zkyxh8nmkvn0vhkyyuspz4mhxue69uhkummnw3ezummcw3ezuer9wchsz9thwden5te0wfjkccte9ejxzmt4wvhxjme0qy88wumn8ghj7mn0wvhxcmmv9u0uehfp").unwrap())),
                Token::Whitespace,
                Token::Whitespace,
                Token::Text("and"),
                Token::Whitespace,
                Token::Hashtag("rust-nostr"),
                Token::Whitespace,
                Token::Text("fans, can you enlighten me please:"),
                Token::LineBreak,
                Token::Text("When I am calculating my Web of Trust I do the following:"),
                Token::LineBreak,
                Token::Text("0. Create client with outbox model enabled"),
                Token::LineBreak,
                Token::Text("1. Get my follows, mutes, reports in one fetch call"),
                Token::LineBreak,
                Token::Text("2. Get follows, mutes, reports of my follows in another fetch call, using an authors filter that has all follows in it"),
                Token::LineBreak,
                Token::Text("3. Calculate scores with my weights locally"),
                Token::LineBreak,
                Token::LineBreak,
                Token::Text("Question:"),
                Token::LineBreak,
                Token::Text("Why did step 2. take hours to complete?"),
                Token::LineBreak,
                Token::LineBreak,
                Token::Text("It seems like it's trying to connect to loads of relays."),
                Token::LineBreak,
                Token::Text("My guess is either I am doing sth horribly wrong or there is no smart relay set calculation for filters in the pool."),
                Token::LineBreak,
                Token::LineBreak,
                Token::Text("In ndk this calculation takes under 10 seconds to complete, even without any caching. It will first look at the filters and calculate a relay set that has all authors in it then does the fetching."),
                Token::LineBreak,
                Token::LineBreak,
                Token::Hashtag("asknostr"),
                Token::Whitespace,
                Token::Hashtag("rust"),
            ]),
            ("Try this FTP server: ftp://example.com", vec![
                Token::Text("Try this FTP server:"),
                Token::Whitespace,
                Token::Url(Url::parse("ftp://example.com").unwrap())
            ]),
            ("My relays are: wss://relay.damus.io, wss://nos.lol and wss://example.com", vec![
                Token::Text("My relays are:"),
                Token::Whitespace,
                Token::Url(Url::parse("wss://relay.damus.io").unwrap()),
                Token::Text(","),
                Token::Whitespace,
                Token::Url(Url::parse("wss://nos.lol").unwrap()),
                Token::Whitespace,
                Token::Text("and"),
                Token::Whitespace,
                Token::Url(Url::parse("wss://example.com").unwrap())
            ]),
            (
                "#alice #bob\n#carol",
                vec![
                    Token::Hashtag("alice"),
                    Token::Whitespace,
                    Token::Hashtag("bob"),
                    Token::LineBreak,
                    Token::Hashtag("carol"),
                ],
            ),
            (
                "#hashtag ",
                vec![Token::Hashtag("hashtag"), Token::Whitespace],
            ),
        ];

        for (text, expected) in test_vectors.into_iter() {
            let tokens = PARSER.parse(text).collect::<Vec<_>>();
            assert_eq!(tokens, expected);
        }
    }

    #[test]
    fn test_parse_only_nostr_uris() {
        let pubkey = Nip21::Pubkey(
            PublicKey::from_bech32(
                "npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet",
            )
            .unwrap(),
        );
        let pubkey2 = Nip21::Pubkey(
            PublicKey::from_bech32(
                "npub1acg6thl5psv62405rljzkj8spesceyfz2c32udakc2ak0dmvfeyse9p35c",
            )
            .unwrap(),
        );
        let event = Nip21::Event(Nip19Event::from_bech32("nevent1qqsz8xjlh82ykfr3swjk5fw0l3v33pcsaq4z6f7q0zy2dxrfm7x2yeqpz4mhxue69uhkummnw3ezummcw3ezuer9wchsygrgmqgktyvpqzma5slu9rmarlqj24zxdcg3tzrtneamxfhktmzzwgpsgqqqqqqsmxphku").unwrap());
        let profile1 = Nip21::Profile(Nip19Profile::from_bech32("nprofile1qqsqfyvdlsmvj0nakmxq6c8n0c2j9uwrddjd8a95ynzn9479jhlth3gpvemhxue69uhkv6tvw3jhytnwdaehgu3wwa5kuef0dec82c33w94xwcmdd3cxketedsux6ertwecrgues0pk8xdrew33h27pkd4unvvpkw3nkv7pe0p68gat58ycrw6ps0fenwdnvva48w0mzwfhkzerrv9ehg0t5wf6k2qgnwaehxw309ac82unsd3jhqct89ejhxtcpz4mhxue69uhhyetvv9ujuerpd46hxtnfduhsh8njvk").unwrap());
        let profile2 = Nip21::Profile(Nip19Profile::from_bech32("nprofile1qqswuyd9ml6qcxd92h6pleptfrcqucvvjy39vg4wx7mv9wm8kakyujgpypmhxue69uhkx6r0wf6hxtndd94k2erfd3nk2u3wvdhk6w35xs6z7qgwwaehxw309ahx7uewd3hkctcpypmhxue69uhkummnw3ezuetfde6kuer6wasku7nfvuh8xurpvdjj7a0nq40").unwrap());

        let vector = vec![
            ("#rustnostr #nostr #kotlin #jvm\n\nnostr:nevent1qqsz8xjlh82ykfr3swjk5fw0l3v33pcsaq4z6f7q0zy2dxrfm7x2yeqpz4mhxue69uhkummnw3ezummcw3ezuer9wchsygrgmqgktyvpqzma5slu9rmarlqj24zxdcg3tzrtneamxfhktmzzwgpsgqqqqqqsmxphku", vec![Token::Nostr(event)]),
            ("I have never been very active in discussions but working on rust-nostr (at the time called nostr-rs-sdk) since September 2022 🦀 \n\nIf I remember correctly there were also nostr:nprofile1qqsqfyvdlsmvj0nakmxq6c8n0c2j9uwrddjd8a95ynzn9479jhlth3gpvemhxue69uhkv6tvw3jhytnwdaehgu3wwa5kuef0dec82c33w94xwcmdd3cxketedsux6ertwecrgues0pk8xdrew33h27pkd4unvvpkw3nkv7pe0p68gat58ycrw6ps0fenwdnvva48w0mzwfhkzerrv9ehg0t5wf6k2qgnwaehxw309ac82unsd3jhqct89ejhxtcpz4mhxue69uhhyetvv9ujuerpd46hxtnfduhsh8njvk and nostr:nprofile1qqswuyd9ml6qcxd92h6pleptfrcqucvvjy39vg4wx7mv9wm8kakyujgpypmhxue69uhkx6r0wf6hxtndd94k2erfd3nk2u3wvdhk6w35xs6z7qgwwaehxw309ahx7uewd3hkctcpypmhxue69uhkummnw3ezuetfde6kuer6wasku7nfvuh8xurpvdjj7a0nq40", vec![Token::Nostr(profile1), Token::Nostr(profile2.clone())]),
            ("Test ending with full stop: nostr:npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet.", vec![Token::Nostr(pubkey.clone())]),
            ("nostr:npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet", vec![Token::Nostr(pubkey.clone())]),
            ("Public key without prefix npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet", vec![]),
            ("Public key `nostr:npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet+.", vec![Token::Nostr(pubkey.clone())]),
            ("Duplicated npub: nostr:npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet, nostr:npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet", vec![Token::Nostr(pubkey.clone()), Token::Nostr(pubkey)]),
            ("Uppercase nostr:npub1DRVpZev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eSEET", vec![]),
            ("Npub and nprofile that point to the same public key: nostr:npub1acg6thl5psv62405rljzkj8spesceyfz2c32udakc2ak0dmvfeyse9p35c and nostr:nprofile1qqswuyd9ml6qcxd92h6pleptfrcqucvvjy39vg4wx7mv9wm8kakyujgpypmhxue69uhkx6r0wf6hxtndd94k2erfd3nk2u3wvdhk6w35xs6z7qgwwaehxw309ahx7uewd3hkctcpypmhxue69uhkummnw3ezuetfde6kuer6wasku7nfvuh8xurpvdjj7a0nq40", vec![Token::Nostr(pubkey2), Token::Nostr(profile2)]),
            ("content without nostr URIs", vec![]),
        ];

        let opts = NostrParserOptions::disable_all().nostr_uris(true);

        for (content, expected) in vector {
            let objs = PARSER.parse(content).opts(opts).collect::<Vec<_>>();
            assert_eq!(objs, expected);
        }
    }

    #[test]
    fn test_parse_only_nostr_uris_empty_result() {
        // Text without nostr URIs
        let text = "I follow #bitcoin hashtag and #lightning.\nWhat do you follow?";

        // Enable only nostr URIs
        let opts = NostrParserOptions::disable_all().nostr_uris(true);

        let tokens = PARSER.parse(text).opts(opts).collect::<Vec<_>>();
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_parse_only_urls() {
        let text = "My relays are: wss://relay.damus.io, wss://nos.lol and wss://example.com";

        let opts = NostrParserOptions::disable_all().urls(true);

        let tokens = PARSER.parse(text).opts(opts).collect::<Vec<_>>();
        assert_eq!(
            tokens,
            vec![
                Token::Url(Url::parse("wss://relay.damus.io").unwrap()),
                Token::Url(Url::parse("wss://nos.lol").unwrap()),
                Token::Url(Url::parse("wss://example.com").unwrap())
            ]
        );
    }

    #[test]
    fn test_parse_only_hashtags() {
        let text = "I follow #bitcoin hashtag and #lightning.";

        let opts = NostrParserOptions::disable_all().hashtags(true);

        let tokens = PARSER.parse(text).opts(opts).collect::<Vec<_>>();
        assert_eq!(
            tokens,
            vec![Token::Hashtag("bitcoin"), Token::Hashtag("lightning"),]
        );
    }

    #[test]
    fn test_parse_only_text() {
        let text = "I follow #bitcoin hashtag and #lightning.\nWhat do you follow?";

        let opts = NostrParserOptions::disable_all().text(true);

        let tokens = PARSER.parse(text).opts(opts).collect::<Vec<_>>();
        assert_eq!(
            tokens,
            vec![
                Token::Text("I follow #bitcoin hashtag and #lightning."),
                Token::LineBreak,
                Token::Text("What do you follow?"),
            ]
        );
    }
}

#[cfg(bench)]
mod benches {
    use test::{black_box, Bencher};

    use super::*;

    const PARSER: NostrParser = NostrParser::new();

    #[bench]
    pub fn bench_parse_text_with_nostr_uris(bh: &mut Bencher) {
        let text: &str = "I have never been very active in discussions but working on rust-nostr (at the time called nostr-rs-sdk) since September 2022 🦀 \n\nIf I remember correctly there were also nostr:nprofile1qqsqfyvdlsmvj0nakmxq6c8n0c2j9uwrddjd8a95ynzn9479jhlth3gpvemhxue69uhkv6tvw3jhytnwdaehgu3wwa5kuef0dec82c33w94xwcmdd3cxketedsux6ertwecrgues0pk8xdrew33h27pkd4unvvpkw3nkv7pe0p68gat58ycrw6ps0fenwdnvva48w0mzwfhkzerrv9ehg0t5wf6k2qgnwaehxw309ac82unsd3jhqct89ejhxtcpz4mhxue69uhhyetvv9ujuerpd46hxtnfduhsh8njvk and nostr:nprofile1qqswuyd9ml6qcxd92h6pleptfrcqucvvjy39vg4wx7mv9wm8kakyujgpypmhxue69uhkx6r0wf6hxtndd94k2erfd3nk2u3wvdhk6w35xs6z7qgwwaehxw309ahx7uewd3hkctcpypmhxue69uhkummnw3ezuetfde6kuer6wasku7nfvuh8xurpvdjj7a0nq40";

        bh.iter(|| {
            black_box(PARSER.parse(text).collect::<Vec<_>>());
        });
    }

    #[bench]
    pub fn bench_parse_text_with_urls(bh: &mut Bencher) {
        let text: &str = "I've uses both the book and rustlings: https://github.com/rust-lang/rustlings/\n\nThere is also the \"Rust by example\" book: https://doc.rust-lang.org/rust-by-example/\n\nWhile you read the book, try to make projects from scratch (not just simple ones). At the end, writing code is the best way to learn it.";

        bh.iter(|| {
            black_box(PARSER.parse(text).collect::<Vec<_>>());
        });
    }

    #[bench]
    pub fn bench_parse_text_with_hashtags(bh: &mut Bencher) {
        let text: &str = "Hey #rust-nostr fans, can you enlighten me please:\nWhen I am calculating my Web of Trust I do the following:\n0. Create client with outbox model enabled\n1. Get my follows, mutes, reports in one fetch call\n2. Get follows, mutes, reports of my follows in another fetch call, using an authors filter that has all follows in it\n3. Calculate scores with my weights locally\n\nQuestion:\nWhy did step 2. take hours to complete?\n\nIt seems like it's trying to connect to loads of relays.\nMy guess is either I am doing sth horribly wrong or there is no smart relay set calculation for filters in the pool.\n\nIn ndk this calculation takes under 10 seconds to complete, even without any caching. It will first look at the filters and calculate a relay set that has all authors in it then does the fetching.\n\n#asknostr #rust";

        bh.iter(|| {
            black_box(PARSER.parse(text).collect::<Vec<_>>());
        });
    }
}
