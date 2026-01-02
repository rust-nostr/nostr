// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2026 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::event::borrow::EventBorrow;
use tantivy_query_grammar::{parse_query, Occur, UserInputAst, UserInputLeaf};


/// Match a query against an event
#[inline]
pub(super) fn match_query(query: &str, event: &EventBorrow) -> bool {
    // Early exit if query is empty
    if query.is_empty() {
        return false;
    }

    // TODO: expose query parser error?
    if let Ok(ast) = parse_query(query) {
        return eval_ast(&ast, event);
    }

    false
}

/// Evaluate a UserInputAst against an event
fn eval_ast(ast: &UserInputAst, event: &EventBorrow) -> bool {
    match ast {
        UserInputAst::Clause(items) => eval_clause(items, event),
        UserInputAst::Boost(inner, _) => eval_ast(inner, event),
        UserInputAst::Leaf(leaf) => eval_leaf(leaf, event),
    }
}

/// Evaluate a clause with Occur semantics
///
/// Terms without explicit operators (None) are treated as AND (all must match).
/// Use `+term` for explicit Must, `-term` for MustNot.
fn eval_clause(items: &[(Option<Occur>, UserInputAst)], event: &EventBorrow) -> bool {
    if items.is_empty() {
        return true;
    }

    let mut has_should = false;
    let mut should_matched = false;

    for (occur, ast) in items {
        let matches = eval_ast(ast, event);

        match occur {
            // None (no explicit operator) and Must (+) both require the term to match
            Some(Occur::Must) | None => {
                if !matches {
                    return false;
                }
            }
            Some(Occur::MustNot) => {
                if matches {
                    return false;
                }
            }
            Some(Occur::Should) => {
                has_should = true;
                if matches {
                    should_matched = true;
                }
            }
        }
    }

    // If we only have Should items, at least one must match
    if has_should {
        should_matched
    } else {
        true
    }
}

/// Evaluate a leaf node against an event
fn eval_leaf(leaf: &UserInputLeaf, event: &EventBorrow) -> bool {
    match leaf {
        UserInputLeaf::Literal(lit) => {
            let phrase_lower = lit.phrase.to_ascii_lowercase();

            match &lit.field_name {
                // Field specified: match against specific tag
                Some(field) => {
                    let field_lower = field.to_ascii_lowercase();
                    event.tags.iter().any(|tag| {
                        if let Some(tag_content) = tag.content() {
                            tag.kind().eq_ignore_ascii_case(&field_lower)
                                && contains_word(&tag_content.to_ascii_lowercase(), &phrase_lower)
                        } else {
                            false
                        }
                    })
                }
                // No field: search content and multi-char tags
                None => {
                    // Check content first
                    if contains_word(&event.content.to_ascii_lowercase(), &phrase_lower) {
                        return true;
                    }

                    // Check tags with keys > 1 char (skip single-letter tags like "e", "p", "t")
                    event.tags.iter().any(|tag| {
                        if let Some(tag_content) = tag.content() {
                            tag.kind().len() > 1
                                && contains_word(&tag_content.to_ascii_lowercase(), &phrase_lower)
                        } else {
                            false
                        }
                    })
                }
            }
        }
        UserInputLeaf::All => true,
        // Range, Set, and Exists are field-specific operations not applicable here
        UserInputLeaf::Range { .. } | UserInputLeaf::Set { .. } | UserInputLeaf::Exists { .. } => {
            false
        }
    }
}

/// Check if text contains the word/phrase, matching at word boundaries.
/// For single words, matches whole words only.
/// For phrases (containing spaces), matches the exact phrase.
#[inline]
fn contains_word(text: &str, word: &str) -> bool {
    if word.contains(' ') {
        // Phrase: exact substring match
        text.contains(word)
    } else {
        // Single word: match at word boundaries
        text.split(|c: char| !c.is_alphanumeric())
            .any(|w| w == word)
    }
}

#[cfg(test)]
mod tests {
    use nostr::event::borrow::EventBorrow;
    use nostr::{Event, EventBuilder, Keys, Tag};

    use super::*;

    fn create_test_event(content: &str) -> Event {
        let keys = Keys::generate();
        EventBuilder::text_note(content)
            .sign_with_keys(&keys)
            .unwrap()
    }

    #[test]
    fn test_match_in_content() {
        let event = create_test_event("Hello World");
        let event: EventBorrow = (&event).into();

        // Case insensitive match
        assert!(match_query("hello", &event));
        assert!(match_query("world", &event));

        // No match
        assert!(!match_query("rust", &event));
    }

    #[test]
    fn test_match_in_tags() {
        let keys = Keys::generate();
        let event = EventBuilder::text_note("content")
            .tag(Tag::parse(["title", "Search userfacing tags"]).unwrap())
            .sign_with_keys(&keys)
            .unwrap();
        let event: EventBorrow = (&event).into();

        assert!(match_query("userfacing", &event));
        assert!(!match_query("bitcoin", &event));
    }

    #[test]
    fn test_empty_query() {
        let event = create_test_event("test");
        let event: EventBorrow = (&event).into();

        assert!(!match_query("", &event));
    }

    #[test]
    fn test_word_match() {
        let event = create_test_event("nostr protocol");
        let event: EventBorrow = (&event).into();

        // Whole word matches
        assert!(match_query("protocol", &event));
        assert!(match_query("nostr", &event));

        // Partial word does not match (word boundary matching)
        assert!(!match_query("proto", &event));
    }

    #[test]
    fn test_and_query() {
        let event = create_test_event("bitcoin and nostr are great");
        let event: EventBorrow = (&event).into();

        // Both terms present - should match
        assert!(match_query("+bitcoin +nostr", &event));

        // One term missing - should not match
        assert!(!match_query("+bitcoin +ethereum", &event));
    }

    #[test]
    fn test_multiple_terms_and() {
        let event = create_test_event("bitcoin maximalist");
        let event: EventBorrow = (&event).into();

        // Both terms present - should match (AND semantics)
        assert!(match_query("bitcoin maximalist", &event));

        // One term missing - should not match
        assert!(!match_query("bitcoin ethereum", &event));

        // Neither term present - should not match
        assert!(!match_query("solana ethereum", &event));
    }

    #[test]
    fn test_not_query() {
        let event = create_test_event("bitcoin is decentralized");
        let event: EventBorrow = (&event).into();

        // Must have bitcoin, must not have ethereum
        assert!(match_query("+bitcoin -ethereum", &event));

        // Must not have bitcoin - should not match
        assert!(!match_query("-bitcoin", &event));

        // Must have bitcoin, must not have decentralized - should not match
        assert!(!match_query("+bitcoin -decentralized", &event));
    }

    #[test]
    fn test_phrase_query() {
        let event = create_test_event("the quick brown fox jumps");
        let event: EventBorrow = (&event).into();

        // Exact phrase - should match
        assert!(match_query("\"quick brown\"", &event));

        // Phrase not in order - should not match
        assert!(!match_query("\"brown quick\"", &event));
    }

    #[test]
    fn test_complex_query() {
        let event = create_test_event("nostr protocol enables censorship resistant communication");
        let event: EventBorrow = (&event).into();

        // Complex query: must have nostr, should have protocol or communication, must not have bitcoin
        assert!(match_query("+nostr protocol communication -bitcoin", &event));

        // Same but content has bitcoin - should not match
        let event2 = create_test_event("nostr and bitcoin are related");
        let event2: EventBorrow = (&event2).into();
        assert!(!match_query("+nostr protocol communication -bitcoin", &event2));
    }

    #[test]
    fn test_field_tag_match() {
        let keys = Keys::generate();
        let event = EventBuilder::text_note("some content")
            .tag(Tag::parse(["title", "Introduction to Nostr"]).unwrap())
            .tag(Tag::parse(["t", "bitcoin"]).unwrap())
            .tag(Tag::parse(["summary", "A guide about the protocol"]).unwrap())
            .sign_with_keys(&keys)
            .unwrap();
        let event: EventBorrow = (&event).into();

        // Match specific tag by field name
        assert!(match_query("title:introduction", &event));

        // Match 't' tag
        assert!(match_query("t:bitcoin", &event));

        // Field name is case-insensitive
        assert!(match_query("TITLE:nostr", &event));

        // Non-matching field
        assert!(!match_query("title:ethereum", &event));

        // Field that doesn't exist
        assert!(!match_query("author:someone", &event));

        // Match summary tag (not in allowed tags, but accessible via field syntax)
        assert!(match_query("summary:guide", &event));
    }

    #[test]
    fn test_field_with_operators() {
        let keys = Keys::generate();
        let event = EventBuilder::text_note("content here")
            .tag(Tag::parse(["title", "Bitcoin and Nostr"]).unwrap())
            .tag(Tag::parse(["t", "crypto"]).unwrap())
            .sign_with_keys(&keys)
            .unwrap();
        let event: EventBorrow = (&event).into();

        // Must have title with bitcoin AND t tag with crypto
        assert!(match_query("+title:bitcoin +t:crypto", &event));

        // Must have title with bitcoin, must NOT have t tag with ethereum
        assert!(match_query("+title:bitcoin -t:ethereum", &event));

        // Must have title with ethereum - should not match
        assert!(!match_query("+title:ethereum", &event));
    }

    #[test]
    fn test_mixed_field_and_content() {
        let keys = Keys::generate();
        let event = EventBuilder::text_note("nostr is great")
            .tag(Tag::parse(["title", "Introduction to Bitcoin"]).unwrap())
            .sign_with_keys(&keys)
            .unwrap();
        let event: EventBorrow = (&event).into();

        // Content term + field term (both must match)
        assert!(match_query("+nostr +title:bitcoin", &event));
        assert!(match_query("nostr title:bitcoin", &event));

        // Content term present, field term not present - should not match
        assert!(!match_query("+nostr +title:ethereum", &event));
        assert!(!match_query("nostr title:ethereum", &event));
    }
}
