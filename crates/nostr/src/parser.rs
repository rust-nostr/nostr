// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr parser

use core::fmt;

use regex::{Match, Matches, Regex};

use crate::nips::nip21::{self, Nip21};
use crate::types::url::{ParseError, Url};

const URL_SCHEME_SEPARATOR: &str = "://";
const HASHTAG: char = '#';
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

/// Nostr parser
#[derive(Debug, Clone)]
pub struct NostrParser {
    re: Regex,
}

impl Default for NostrParser {
    fn default() -> Self {
        Self::new()
    }
}

impl NostrParser {
    const RE: &'static str = r##"(?i)\b((?:[a-z][a-z0-9+\-.]*)://[a-z0-9._~:/?#\[\]@!$&'()*+;=%-]+)|(nostr:[a-z]+1[qpzry9x8gf2tvdw0s3jn54khce6mua7l]+)|((^|\s)#[^\s!@#$%^&*(),.?":{}|<>]+)|(\n)"##;

    /// Construct new nostr parser
    ///
    /// It's suggested to construct this once and reuse it, to avoid regex re-compilation.
    ///
    /// # Patterns
    ///
    /// ## Urls
    ///
    /// Regex: `(?i)\b((?:[a-z][a-z0-9+\-.]*)://[a-z0-9._~:/?#\[\]@!$&'()*+;=%-]+)`
    ///
    /// Pattern details:
    /// - `(?:[a-z][a-z0-9+\-.]*)`: captures a scheme like `http`, `https`, `ftp`, etc.
    /// - `://`: matches the literal `://`
    /// - `[a-z0-9._~:/?#\[\]@!$&'()*+;=%-]+`: ensures the next part of the URL starts properly
    ///
    /// ## Nostr URIs
    ///
    /// Regex: `nostr:[a-z]+1[qpzry9x8gf2tvdw0s3jn54khce6mua7l]+`
    ///
    /// Pattern details:
    /// - `nostr:`: the nostr URI prefix
    /// - `[a-z]`: the entity prefix (i.e., npub, naddr, ncryptsec)
    /// - `1`: the bech32 separator
    /// - `[qpzry9x8gf2tvdw0s3jn54khce6mua7l]`: the bech32 chars
    ///
    /// ## Hashtags
    ///
    /// Regex: `(^|\s)#[^\s!@#$%^&*(),.?":{}|<>]+`.
    ///
    /// Pattern details:
    /// - `(^|\s)`: either the start of the string (`^`) or a whitespace character (`\s`);
    /// - `#`: the hashtag prefix;
    /// - `[^\s!@#$%^&*(),.?":{}|<>]`: chars that aren't allowed in the hashtag;
    ///
    /// ## Line breaks
    ///
    /// Regex: `\n`
    pub fn new() -> Self {
        Self {
            re: Regex::new(Self::RE).expect("Failed to compile regex"),
        }
    }

    /// Parse text
    pub fn parse<'a, 'p>(&'p self, text: &'a str) -> NostrParserIter<'a, 'p> {
        NostrParserIter {
            text,
            matches: self.re.find_iter(text),
            pending_match: None,
            pending_str_match: None,
            pending_token: None,
            last_match_end: 0,
        }
    }
}

/// Nostr parser iterator
pub struct NostrParserIter<'a, 'p> {
    /// The original text
    text: &'a str,
    /// Regex matches
    matches: Matches<'p, 'a>,
    /// A pending match
    pending_match: Option<Match<'a>>,
    /// A pending match string
    pending_str_match: Option<&'a str>,
    /// Pending token
    pending_token: Option<Token<'a>>,
    /// Last regex match end index
    last_match_end: usize,
}

impl<'a> NostrParserIter<'a, '_> {
    fn handle_match(&mut self, mat: Match<'a>) -> Token<'a> {
        // Update last match end
        self.last_match_end = mat.end();

        // Match str
        self.handle_str(mat.as_str())
    }

    fn handle_str(&mut self, match_str: &'a str) -> Token<'a> {
        match match_str {
            // URL
            m if m.contains(URL_SCHEME_SEPARATOR) => match Url::parse(m) {
                Ok(url) => Token::Url(url),
                // Fallback to text
                Err(_) => self.handle_str_as_text(m),
            },
            // Nostr URI
            m if m.starts_with(nip21::SCHEME_WITH_COLON) => match Nip21::parse(m) {
                Ok(uri) => Token::Nostr(uri),
                // Fallback to text
                Err(_) => self.handle_str_as_text(m),
            },
            // Hashtag
            m if m.starts_with(HASHTAG) && m.len() > 1 => Token::Hashtag(&m[1..]),
            // Fallback: treat it as text
            m => self.handle_str_as_text(m),
        }
    }

    fn handle_str_as_text(&mut self, match_str: &'a str) -> Token<'a> {
        match match_str {
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

impl<'a> Iterator for NostrParserIter<'a, '_> {
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

            return Some(self.handle_str(pending_str_match));
        }

        // Handle a pending match
        if let Some(pending_match) = self.pending_match.take() {
            #[cfg(all(feature = "std", debug_assertions, test))]
            dbg!(&pending_match, self.last_match_end);

            return Some(self.handle_match(pending_match));
        }

        match self.matches.next() {
            Some(mat) => {
                #[cfg(all(feature = "std", debug_assertions, test))]
                dbg!(&mat, self.last_match_end);

                // Capture text that appears before this match (if any)
                if mat.start() > self.last_match_end {
                    // Update pending match
                    // This will be handled at next iteration, in `handle_match` method.
                    self.pending_match = Some(mat);

                    // Capture text that appears between last match and this match start
                    let data: &str = &self.text[self.last_match_end..mat.start()];

                    // Return the token
                    return Some(self.handle_str_as_text(data));
                }

                // Handle match
                Some(self.handle_match(mat))
            }
            None => {
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

#[cfg(test)]
mod tests {
    use alloc::vec;
    use alloc::vec::Vec;

    use super::*;
    use crate::nips::nip19::{FromBech32, Nip19Profile};
    use crate::PublicKey;

    #[test]
    fn test_parser() {
        let test_vectors: Vec<(&str, Vec<Token>)> = vec![
            ("Hello nostr:npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet, take a look at https://example.com/foo/bar.html. Thanks!", vec![
                Token::Text("Hello"),
                Token::Whitespace,
                Token::Nostr(Nip21::Pubkey(PublicKey::parse("npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet").unwrap())),
                Token::Text(", take a look at"),
                Token::Whitespace,
                Token::Url(Url::parse("https://example.com/foo/bar.html.").unwrap()),
                Token::Whitespace,
                Token::Text("Thanks!"),
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
            ("https://github.com/rust-lang/rustlings/", vec![Token::Url(Url::parse("https://github.com/rust-lang/rustlings/").unwrap())]),
            ("nostr:npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet", vec![
                Token::Nostr(Nip21::Pubkey(PublicKey::parse("nostr:npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet").unwrap()))
            ]),
            ("test", vec![Token::Text("test")]),
            ("#rust-nostr", vec![Token::Hashtag("rust-nostr")]),
            ("#rustnostr #rust-nostr #rust #kotlin", vec![Token::Hashtag("rustnostr"), Token::Whitespace, Token::Hashtag("rust-nostr"), Token::Whitespace, Token::Hashtag("rust"), Token::Whitespace, Token::Hashtag("kotlin")]),
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
            ("https://example.com#title-1", vec![
                Token::Url(Url::parse("https://example.com#title-1").unwrap()),
            ]),
            ("bob#alice", vec![
                Token::Text("bob#alice"),
            ]),
            ("#alice #bob\n#carol", vec![
                Token::Hashtag("alice"),
                Token::Whitespace,
                Token::Hashtag("bob"),
                Token::LineBreak,
                Token::Hashtag("carol"),
            ]),
            ("#hashtag ", vec![
                Token::Hashtag("hashtag"),
                Token::Whitespace,
            ]),
            ("#", vec![
                Token::Text("#"),
            ]),
            ("# ", vec![
                Token::Text("#"),
                Token::Whitespace,
            ]),
            ("#\n", vec![
                Token::Text("#"),
                Token::LineBreak,
            ]),
            ("#$", vec![
                Token::Text("#$"),
            ]),
        ];

        let parser: NostrParser = NostrParser::new();

        for (text, expected) in test_vectors.into_iter() {
            let tokens = parser.parse(text).collect::<Vec<_>>();
            assert_eq!(tokens, expected);
        }
    }
}

#[cfg(bench)]
mod benches {
    use test::{black_box, Bencher};

    use super::*;

    #[bench]
    pub fn bench_parse_text_with_nostr_uris(bh: &mut Bencher) {
        let text: &str = "I have never been very active in discussions but working on rust-nostr (at the time called nostr-rs-sdk) since September 2022 🦀 \n\nIf I remember correctly there were also nostr:nprofile1qqsqfyvdlsmvj0nakmxq6c8n0c2j9uwrddjd8a95ynzn9479jhlth3gpvemhxue69uhkv6tvw3jhytnwdaehgu3wwa5kuef0dec82c33w94xwcmdd3cxketedsux6ertwecrgues0pk8xdrew33h27pkd4unvvpkw3nkv7pe0p68gat58ycrw6ps0fenwdnvva48w0mzwfhkzerrv9ehg0t5wf6k2qgnwaehxw309ac82unsd3jhqct89ejhxtcpz4mhxue69uhhyetvv9ujuerpd46hxtnfduhsh8njvk and nostr:nprofile1qqswuyd9ml6qcxd92h6pleptfrcqucvvjy39vg4wx7mv9wm8kakyujgpypmhxue69uhkx6r0wf6hxtndd94k2erfd3nk2u3wvdhk6w35xs6z7qgwwaehxw309ahx7uewd3hkctcpypmhxue69uhkummnw3ezuetfde6kuer6wasku7nfvuh8xurpvdjj7a0nq40";

        let parser = NostrParser::new();

        bh.iter(|| {
            black_box(parser.parse(text).collect::<Vec<_>>());
        });
    }

    #[bench]
    pub fn bench_parse_text_with_urls(bh: &mut Bencher) {
        let text: &str = "I've uses both the book and rustlings: https://github.com/rust-lang/rustlings/\n\nThere is also the \"Rust by example\" book: https://doc.rust-lang.org/rust-by-example/\n\nWhile you read the book, try to make projects from scratch (not just simple ones). At the end, writing code is the best way to learn it.";

        let parser = NostrParser::new();

        bh.iter(|| {
            black_box(parser.parse(text).collect::<Vec<_>>());
        });
    }

    #[bench]
    pub fn bench_parse_text_with_hashtags(bh: &mut Bencher) {
        let text: &str = "Hey #rust-nostr fans, can you enlighten me please:\nWhen I am calculating my Web of Trust I do the following:\n0. Create client with outbox model enabled\n1. Get my follows, mutes, reports in one fetch call\n2. Get follows, mutes, reports of my follows in another fetch call, using an authors filter that has all follows in it\n3. Calculate scores with my weights locally\n\nQuestion:\nWhy did step 2. take hours to complete?\n\nIt seems like it's trying to connect to loads of relays.\nMy guess is either I am doing sth horribly wrong or there is no smart relay set calculation for filters in the pool.\n\nIn ndk this calculation takes under 10 seconds to complete, even without any caching. It will first look at the filters and calculate a relay set that has all authors in it then does the fetching.\n\n#asknostr #rust";

        let parser = NostrParser::new();

        bh.iter(|| {
            black_box(parser.parse(text).collect::<Vec<_>>());
        });
    }
}
