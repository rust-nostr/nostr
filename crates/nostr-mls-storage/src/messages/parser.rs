//! This module contains the logic for parsing Nostr events into tokens.

use nostr::parser::{NostrParser, Token};
use serde::{Deserialize, Serialize};

/// Serializable Token
/// This is a parallel of the `Token` enum from the `nostr` crate, modified so that we can serialize it for DB storage
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(into = "TokenObject", from = "TokenObject")]
pub enum SerializableToken {
    /// Nostr URI converted to a string
    Nostr(String),
    /// Url converted to a string
    Url(String),
    /// Hashtag
    Hashtag(String),
    /// Other text
    ///
    /// Spaces at the beginning or end of a text are parsed as [`Token::Whitespace`].
    Text(String),
    /// Line break
    LineBreak,
    /// A whitespace
    Whitespace,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum TokenObject {
    Text {
        #[serde(rename = "Text")]
        text: String,
    },
    Url {
        #[serde(rename = "Url")]
        url: String,
    },
    Hashtag {
        #[serde(rename = "Hashtag")]
        hashtag: String,
    },
    Nostr {
        #[serde(rename = "Nostr")]
        nostr: String,
    },
    LineBreak {
        #[serde(rename = "LineBreak")]
        line_break: Option<()>,
    },
    Whitespace {
        #[serde(rename = "Whitespace")]
        whitespace: Option<()>,
    },
}

impl From<SerializableToken> for TokenObject {
    fn from(token: SerializableToken) -> Self {
        match token {
            SerializableToken::Text(s) => TokenObject::Text { text: s },
            SerializableToken::Url(s) => TokenObject::Url { url: s },
            SerializableToken::Hashtag(s) => TokenObject::Hashtag { hashtag: s },
            SerializableToken::Nostr(s) => TokenObject::Nostr { nostr: s },
            SerializableToken::LineBreak => TokenObject::LineBreak { line_break: None },
            SerializableToken::Whitespace => TokenObject::Whitespace { whitespace: None },
        }
    }
}

impl From<TokenObject> for SerializableToken {
    fn from(obj: TokenObject) -> Self {
        match obj {
            TokenObject::Text { text } => SerializableToken::Text(text),
            TokenObject::Url { url } => SerializableToken::Url(url),
            TokenObject::Hashtag { hashtag } => SerializableToken::Hashtag(hashtag),
            TokenObject::Nostr { nostr } => SerializableToken::Nostr(nostr),
            TokenObject::LineBreak { .. } => SerializableToken::LineBreak,
            TokenObject::Whitespace { .. } => SerializableToken::Whitespace,
        }
    }
}

// We use From instead of TryFrom because we want to show an error if the underlying token enum changes.
impl<'a> From<Token<'a>> for SerializableToken {
    fn from(value: Token<'a>) -> Self {
        match value {
            Token::Nostr(n) => SerializableToken::Nostr(match n.to_nostr_uri() {
                Ok(uri) => uri,
                Err(e) => {
                    // handle or return a default/fallback
                    format!("invalid_nostr_uri:{}", e)
                }
            }),
            Token::Url(u) => SerializableToken::Url(u.to_string()),
            Token::Hashtag(h) => SerializableToken::Hashtag(h.to_string()),
            Token::Text(t) => SerializableToken::Text(t.to_string()),
            Token::LineBreak => SerializableToken::LineBreak,
            Token::Whitespace => SerializableToken::Whitespace,
        }
    }
}

/// Parses a string into a vector of serializable tokens.
///
/// This function takes a string content and returns a vector of `SerializableToken`s,
/// which can be used for database storage or frontend communication.
///
/// # Arguments
/// * `content` - The string content to parse
///
/// # Returns
/// A vector of `SerializableToken`s representing the parsed content
pub fn parse(content: &str) -> Vec<SerializableToken> {
    let parser = NostrParser::new();
    parser.parse(content).map(SerializableToken::from).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use nostr::serde_json::json;

    #[test]
    fn test_parse_basic_text() {
        let content = "Hello, world!";
        let tokens = parse(content);
        assert_eq!(
            tokens,
            vec![SerializableToken::Text("Hello, world!".to_string())]
        );
    }

    #[test]
    fn test_parse_with_whitespace() {
        let content = "  Hello  world  ";
        let tokens = parse(content);
        assert_eq!(
            tokens,
            vec![
                SerializableToken::Whitespace,
                SerializableToken::Whitespace,
                SerializableToken::Text("Hello  world ".to_string()),
                SerializableToken::Whitespace,
            ]
        );
    }

    #[test]
    fn test_parse_with_line_breaks() {
        let content = "Hello\nworld";
        let tokens = parse(content);
        assert_eq!(
            tokens,
            vec![
                SerializableToken::Text("Hello".to_string()),
                SerializableToken::LineBreak,
                SerializableToken::Text("world".to_string()),
            ]
        );
    }

    #[test]
    fn test_parse_with_hashtags() {
        let content = "Hello #nostr world";
        let tokens = parse(content);
        assert_eq!(
            tokens,
            vec![
                SerializableToken::Text("Hello".to_string()),
                SerializableToken::Whitespace,
                SerializableToken::Hashtag("nostr".to_string()),
                SerializableToken::Whitespace,
                SerializableToken::Text("world".to_string()),
            ]
        );
    }

    #[test]
    fn test_parse_with_nostr_uri() {
        let content =
            "Check out nostr:npub1l2vyh47mk2p0qlsku7hg0vn29faehy9hy34ygaclpn66ukqp3afqutajft";
        let tokens = parse(content);
        assert_eq!(
            tokens,
            vec![
                SerializableToken::Text("Check out".to_string()),
                SerializableToken::Whitespace,
                SerializableToken::Nostr(
                    "nostr:npub1l2vyh47mk2p0qlsku7hg0vn29faehy9hy34ygaclpn66ukqp3afqutajft"
                        .to_string()
                ),
            ]
        );
    }

    #[test]
    fn test_parse_with_url() {
        let content = "Visit https://example.com";
        let tokens = parse(content);
        assert_eq!(
            tokens,
            vec![
                SerializableToken::Text("Visit".to_string()),
                SerializableToken::Whitespace,
                SerializableToken::Url("https://example.com/".to_string()),
            ]
        );
    }

    #[test]
    fn test_parse_empty_string() {
        let content = "";
        let tokens = parse(content);
        assert_eq!(tokens, vec![]);
    }

    #[test]
    fn test_parse_complex_content() {
        let content = "Hello #nostr! Check out https://example.com and nostr:npub1l2vyh47mk2p0qlsku7hg0vn29faehy9hy34ygaclpn66ukqp3afqutajft\n\nBye!";
        let tokens = parse(content);
        assert_eq!(
            tokens,
            vec![
                SerializableToken::Text("Hello".to_string()),
                SerializableToken::Whitespace,
                SerializableToken::Hashtag("nostr".to_string()),
                SerializableToken::Text("! Check out".to_string()),
                SerializableToken::Whitespace,
                SerializableToken::Url("https://example.com/".to_string()),
                SerializableToken::Whitespace,
                SerializableToken::Text("and".to_string()),
                SerializableToken::Whitespace,
                SerializableToken::Nostr(
                    "nostr:npub1l2vyh47mk2p0qlsku7hg0vn29faehy9hy34ygaclpn66ukqp3afqutajft"
                        .to_string()
                ),
                SerializableToken::LineBreak,
                SerializableToken::LineBreak,
                SerializableToken::Text("Bye!".to_string()),
            ]
        );
    }

    #[test]
    fn test_token_to_token_object_conversion() {
        let tokens = vec![
            SerializableToken::Text("Hello".to_string()),
            SerializableToken::Whitespace,
            SerializableToken::Hashtag("nostr".to_string()),
            SerializableToken::LineBreak,
            SerializableToken::Url("https://example.com".to_string()),
            SerializableToken::Nostr("nostr:npub1...".to_string()),
        ];

        for token in tokens {
            let token_object: TokenObject = token.clone().into();
            let back_to_token: SerializableToken = token_object.into();
            assert_eq!(token, back_to_token);
        }
    }

    #[test]
    fn test_token_object_serialization() {
        let token = SerializableToken::Text("Hello".to_string());
        let token_object: TokenObject = token.into();
        let json = nostr::serde_json::to_value(&token_object).unwrap();

        assert_eq!(
            json,
            json!({
                "type": "Text",
                "Text": "Hello"
            })
        );

        let token = SerializableToken::Hashtag("nostr".to_string());
        let token_object: TokenObject = token.into();
        let json = nostr::serde_json::to_value(&token_object).unwrap();

        assert_eq!(
            json,
            json!({
                "type": "Hashtag",
                "Hashtag": "nostr"
            })
        );

        let token = SerializableToken::LineBreak;
        let token_object: TokenObject = token.into();
        let json = nostr::serde_json::to_value(&token_object).unwrap();

        assert_eq!(
            json,
            json!({
                "type": "LineBreak",
                "LineBreak": null
            })
        );

        let token = SerializableToken::Whitespace;
        let token_object: TokenObject = token.into();
        let json = nostr::serde_json::to_value(&token_object).unwrap();

        assert_eq!(
            json,
            json!({
                "type": "Whitespace",
                "Whitespace": null
            })
        );
    }

    #[test]
    fn test_token_object_deserialization() {
        let json = json!({
            "type": "Text",
            "Text": "Hello"
        });
        let token_object: TokenObject = nostr::serde_json::from_value(json).unwrap();
        let token: SerializableToken = token_object.into();
        assert_eq!(token, SerializableToken::Text("Hello".to_string()));

        let json = json!({
            "type": "Hashtag",
            "Hashtag": "nostr"
        });
        let token_object: TokenObject = nostr::serde_json::from_value(json).unwrap();
        let token: SerializableToken = token_object.into();
        assert_eq!(token, SerializableToken::Hashtag("nostr".to_string()));

        let json = json!({
            "type": "LineBreak",
            "LineBreak": null
        });
        let token_object: TokenObject = nostr::serde_json::from_value(json).unwrap();
        let token: SerializableToken = token_object.into();
        assert_eq!(token, SerializableToken::LineBreak);

        let json = json!({
            "type": "Whitespace",
            "Whitespace": null
        });
        let token_object: TokenObject = nostr::serde_json::from_value(json).unwrap();
        let token: SerializableToken = token_object.into();
        assert_eq!(token, SerializableToken::Whitespace);
    }

    #[test]
    fn test_roundtrip_serialization() {
        let tokens = vec![
            SerializableToken::Text("Hello".to_string()),
            SerializableToken::Whitespace,
            SerializableToken::Hashtag("nostr".to_string()),
            SerializableToken::LineBreak,
            SerializableToken::Url("https://example.com".to_string()),
            SerializableToken::Nostr("nostr:npub1...".to_string()),
        ];

        for token in tokens {
            let token_object: TokenObject = token.clone().into();
            let json = nostr::serde_json::to_value(&token_object).unwrap();
            let deserialized_object: TokenObject = nostr::serde_json::from_value(json).unwrap();
            let back_to_token: SerializableToken = deserialized_object.into();
            assert_eq!(token, back_to_token, "Failed for token: {:?}", token);
        }
    }

    #[test]
    fn test_url_edge_cases() {
        let test_cases = vec![
            (
                "https://example.com?param=value",
                vec![SerializableToken::Url(
                    "https://example.com/?param=value".to_string(),
                )],
            ),
            (
                "https://example.com#fragment",
                vec![SerializableToken::Url(
                    "https://example.com/#fragment".to_string(),
                )],
            ),
            (
                "https://example.com/path/to/resource",
                vec![SerializableToken::Url(
                    "https://example.com/path/to/resource".to_string(),
                )],
            ),
            (
                "not a valid url",
                vec![SerializableToken::Text("not a valid url".to_string())],
            ),
            (
                "https://example.com with text",
                vec![
                    SerializableToken::Url("https://example.com/".to_string()),
                    SerializableToken::Whitespace,
                    SerializableToken::Text("with text".to_string()),
                ],
            ),
        ];

        for (input, expected) in test_cases {
            let tokens = parse(input);
            assert_eq!(tokens, expected, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_whitespace_edge_cases() {
        let test_cases = vec![
            (
                "\t\t",
                vec![
                    SerializableToken::Text("\t\t".to_string()), // TODO: This should be updated upstream to handle tabs as whitespace
                ],
            ),
            (
                "  \t  ",
                vec![
                    SerializableToken::Whitespace,
                    SerializableToken::Whitespace,
                    SerializableToken::Text("\t ".to_string()), // TODO: This should be updated upstream to handle tabs as whitespace
                    SerializableToken::Whitespace,
                ],
            ),
            (
                "\n\t",
                vec![
                    SerializableToken::LineBreak,
                    SerializableToken::Text("\t".to_string()), // TODO: This should be updated upstream to handle tabs as whitespace
                ],
            ),
            (
                "text\ttext",
                vec![SerializableToken::Text("text\ttext".to_string())], // TODO: This should be updated upstream to handle tabs as whitespace
            ),
        ];

        for (input, expected) in test_cases {
            let tokens = parse(input);
            assert_eq!(tokens, expected, "Failed for input: {:?}", input);
        }
    }

    #[test]
    fn test_text_edge_cases() {
        let test_cases = vec![
            (
                "Hello, 世界!",
                vec![SerializableToken::Text("Hello, 世界!".to_string())],
            ),
            (
                "Text with emoji 😊",
                vec![SerializableToken::Text("Text with emoji 😊".to_string())],
            ),
            (
                "Text with \"quotes\"",
                vec![SerializableToken::Text("Text with \"quotes\"".to_string())],
            ),
            (
                "Text with \\escaped\\ chars",
                vec![SerializableToken::Text(
                    "Text with \\escaped\\ chars".to_string(),
                )],
            ),
        ];

        for (input, expected) in test_cases {
            let tokens = parse(input);
            assert_eq!(tokens, expected, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_error_cases() {
        // Test with a very long string
        let long_string = "a".repeat(10000);
        let tokens = parse(&long_string);
        assert!(!tokens.is_empty(), "Should handle long strings");

        // Test with a string containing null bytes
        let null_string = "text\0text";
        let tokens = parse(null_string);
        assert_eq!(
            tokens,
            vec![SerializableToken::Text("text\0text".to_string())],
            "Should handle null bytes"
        );

        // Test with invalid UTF-8 (this will panic if not handled properly)
        let invalid_utf8 = unsafe { String::from_utf8_unchecked(vec![0xFF, 0xFF]) };
        let tokens = parse(&invalid_utf8);
        assert!(!tokens.is_empty(), "Should handle invalid UTF-8");
    }
}
