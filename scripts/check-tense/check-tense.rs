#!/usr/bin/env -S cargo -Zscript
---cargo
[package]
edition = "2024"

[dependencies]
harper-core = { git = "https://github.com/Automattic/harper", tag = "v2.3.0" }
harper-pos-utils = { git = "https://github.com/Automattic/harper", tag = "v2.3.0" }

[lints.rust]
warnings = "deny"

[lints.clippy]
pedantic = { level = "deny", priority = -1 }
nursery = { level = "deny", priority = -1 }
# Neither `clippy::cargo` nor `[lints.cargo]` reads this manifest. The first
# reads the Cargo.toml above the script, and the second passed a crate this
# script never used.
---

//! The perfect tenses in the records, decided by Harper's tagger.
//!
//!   the decision:  docs/decisions/a-part-of-speech-tagger-decides-the-tense-rule.md
//!   the caller:    scripts/check-records.sh
//!
//!   check-tense.rs <participles> <record>...
//!
//! Prints `<record>:<line>: <have> <participle>` for each sentence that holds
//! one, and exits 0 either way. The caller decides the refusal.

use std::sync::Arc;

use harper_core::parsers::MarkdownOptions;
use harper_core::spell::{FstDictionary, MergedDictionary, MutableDictionary};
use harper_core::{
    DictWordMetadata, Document, Token, TokenKind, TokenStringExt, VerbData, VerbFormFlags,
};
use harper_pos_utils::UPOS;

const HAVE: [&str; 4] = ["has", "have", "had", "having"];
const BE: [&str; 8] = ["is", "are", "was", "were", "be", "been", "being", "am"];
const ASK: [&str; 9] = [
    "what", "which", "who", "whom", "whose", "why", "where", "when", "how",
];

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(participles) = args.next() else {
        eprintln!("usage: check-tense.rs <participles> <record>...");
        std::process::exit(2);
    };
    let dictionary = dictionary(&participles);
    // A link counts as one word in the gate, so its title is not prose.
    let mut options = MarkdownOptions::default();
    options.ignore_link_title = true;

    for path in args {
        let Ok(text) = std::fs::read_to_string(&path) else {
            eprintln!("{path} cannot be read.");
            std::process::exit(2);
        };
        let document = Document::new_markdown(&text, options, &dictionary);
        for (line, pair) in faults(&text, document.get_tokens()) {
            println!("{path}:{line}: {pair}");
        }
    }
}

// Harper's dictionary, with each word of purba's list as a past participle.
fn dictionary(participles: &str) -> MergedDictionary {
    let Ok(list) = std::fs::read_to_string(participles) else {
        eprintln!("{participles} cannot be read.");
        std::process::exit(2);
    };
    let mut own = MutableDictionary::new();
    for word in list.lines().map(str::trim).filter(|word| !word.is_empty()) {
        let metadata = DictWordMetadata {
            verb: Some(VerbData {
                verb_forms: Some(VerbFormFlags::PAST_PARTICIPLE),
                ..VerbData::default()
            }),
            ..DictWordMetadata::default()
        };
        own.append_word_str(word, metadata);
    }
    let mut merged = MergedDictionary::new();
    merged.add_dictionary(FstDictionary::curated());
    merged.add_dictionary(Arc::new(own));
    merged
}

// The first perfect tense in each sentence, by the line its `have` stands on.
// The gate reads no table row and no heading, so neither does this.
fn faults(text: &str, tokens: &[Token]) -> Vec<(usize, String)> {
    let source: Vec<char> = text.chars().collect();
    let lines: Vec<&str> = text.lines().collect();
    let word = |token: &Token| -> String {
        token
            .span
            .get_content(&source)
            .iter()
            .collect::<String>()
            .to_lowercase()
    };
    let line_of = |index: usize| source[..index].iter().filter(|c| **c == '\n').count() + 1;

    let mut found = Vec::new();
    for sentence in tokens.iter_sentences() {
        let Some(have) = perfect(sentence, &word) else {
            continue;
        };
        let line = line_of(sentence[have.0].span.start);
        let opens = lines.get(line - 1).map_or("", |l| l.trim_start());
        if !(opens.starts_with('|') || opens.starts_with('#')) {
            found.push((line, have.1));
        }
    }
    found
}

fn asks(sentence: &[Token], word: &impl Fn(&Token) -> String) -> bool {
    let first = sentence
        .iter()
        .find(|t| matches!(t.kind, TokenKind::Word(_)));
    let last = sentence
        .iter()
        .rfind(|t| !matches!(t.kind, TokenKind::Space(_) | TokenKind::Newline(_)));
    first.is_some_and(|t| {
        let w = word(t);
        HAVE.contains(&w.as_str()) || ASK.contains(&w.as_str())
    }) && last.is_some_and(|t| word(t) == "?")
}

// The index of `have` and the pair it forms, for the first participle that a
// walk back over any word reaches `have` from. A determiner, a number or a noun
// opens a noun phrase, so `has a fixed span` is a possession. `to` and a form
// of `be` end the walk, so `has nothing to run` and `has records which were
// refused` are not tenses. In a question the subject follows `have`, so the
// walk passes a noun phrase there, unless no noun or pronoun stands in it.
fn perfect(sentence: &[Token], word: &impl Fn(&Token) -> String) -> Option<(usize, String)> {
    let question = asks(sentence, word);
    for (i, token) in sentence.iter().enumerate() {
        let TokenKind::Word(Some(metadata)) = &token.kind else {
            continue;
        };
        let participle = word(token);
        let past = metadata.is_verb_past_participle_form()
            || metadata.verb.is_some_and(|v| {
                v.verb_forms
                    .is_some_and(|f| f.contains(VerbFormFlags::PAST))
            })
            || participle == "been";
        if !past {
            continue;
        }
        let mut opened = false;
        let mut subject = false;
        for j in (0..i).rev() {
            match &sentence[j].kind {
                TokenKind::Space(_) | TokenKind::Newline(_) | TokenKind::Punctuation(_) => {}
                TokenKind::Word(before) => {
                    let w = word(&sentence[j]);
                    if HAVE.contains(&w.as_str()) {
                        if opened && !subject {
                            break;
                        }
                        return Some((j, format!("{w} {participle}")));
                    }
                    let tag = before.as_ref().and_then(|m| m.pos_tag);
                    if w == "to" || BE.contains(&w.as_str()) {
                        break;
                    }
                    match tag {
                        Some(UPOS::NOUN | UPOS::PROPN) if question => subject = true,
                        Some(UPOS::PRON) => subject = true,
                        Some(UPOS::DET | UPOS::NUM) if question => opened = true,
                        Some(UPOS::DET | UPOS::NUM | UPOS::NOUN | UPOS::PROPN) => break,
                        _ => {}
                    }
                }
                TokenKind::Number(_) if question => opened = true,
                // A code span, a number or an address.
                _ => break,
            }
        }
    }
    None
}
