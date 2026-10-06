// PROTOTYPE for purba #308 — throwaway.
//
// The perfect-tense rule with harper-core as a library: walk back from a word
// the dictionary marks as a past participle (or a regular `-ed` past) to
// has/have/had/having, over any word, and stop where the variant says.
//
// usage: libwalk <variant> <file.md>...
//   det     stop at DET, NUM, a code span, `to`, a form of `be`   (the walk, by tag)
//   np      det, and also stop at NOUN and PROPN
//   det+v   det, and a word the tagger marks VERB also counts as a participle
//   np+v    np, with the same fallback
//   ing-*   the gerund rule; see `gerund()` below
//   <v>+d   the variant, with a dictionary of purba's own merged over Harper's
//
// Prints `file:line: have participle` for each sentence with a fault, one per
// sentence (the first), and only on lines the bash gate reads: a table row
// (`|`) and a heading (`#`) are skipped, and a link title is not read.

use harper_core::parsers::MarkdownOptions;
use harper_core::{Document, TokenKind, TokenStringExt};
use harper_core::spell::FstDictionary;
use harper_pos_utils::UPOS;

const HAVE: [&str; 4] = ["has", "have", "had", "having"];
const BE: [&str; 8] = ["is", "are", "was", "were", "be", "been", "being", "am"];

fn main() {
    let mut args = std::env::args().skip(1);
    let variant = args.next().expect("variant");
    // `+d` merges a dictionary of purba's own over Harper's, which gives a
    // word the dictionary lacks its verb form.
    let curated = FstDictionary::curated();
    let mut extra = harper_core::spell::MutableDictionary::new();
    if variant.ends_with("+d") {
        let mut meta = harper_core::DictWordMetadata::default();
        meta.verb = Some(harper_core::VerbData {
            verb_forms: Some(harper_core::VerbFormFlags::PAST_PARTICIPLE),
            ..Default::default()
        });
        extra.append_word_str("grown", meta);
    }
    let mut dict = harper_core::spell::MergedDictionary::new();
    dict.add_dictionary(curated);
    dict.add_dictionary(std::sync::Arc::new(extra));
    let variant = variant.trim_end_matches("+d").to_string();
    for path in args {
        let text = std::fs::read_to_string(&path).expect("read");
        let source: Vec<char> = text.chars().collect();
        let lines: Vec<&str> = text.lines().collect();
        let mut options = MarkdownOptions::default();
        options.ignore_link_title = true;
        let doc = Document::new_markdown(&text, options, &dict);
        let tokens = doc.get_tokens();
        let line_of = |idx: usize| source[..idx].iter().filter(|c| **c == '\n').count() + 1;
        let word = |i: usize| -> String {
            tokens[i].span.get_content(&source).iter().collect::<String>().to_lowercase()
        };

        for sentence in tokens.iter_sentences() {
            // index of the sentence's first token in `tokens`
            let start = (sentence.as_ptr() as usize - tokens.as_ptr() as usize)
                / std::mem::size_of::<harper_core::Token>();
            let end = start + sentence.len();
            if variant.starts_with("ing") {
                gerund(&variant, tokens, &source, &lines, start, end, &path);
                continue;
            }
            'tok: for i in start..end {
                let TokenKind::Word(Some(meta)) = &tokens[i].kind else { continue };
                let w = word(i);
                let participle = meta.is_verb_past_participle_form()
                    || meta.verb.is_some_and(|v| {
                        v.verb_forms.is_some_and(|vf| {
                            vf.contains(harper_core::VerbFormFlags::PAST)
                        })
                    })
                    || w == "been"
                    || (variant.ends_with("+v") && meta.pos_tag == Some(UPOS::VERB));
                if !participle {
                    continue;
                }
                let mut j = i;
                while j > start {
                    j -= 1;
                    match &tokens[j].kind {
                        TokenKind::Space(_) | TokenKind::Newline(_) => continue,
                        TokenKind::Punctuation(_) => continue,
                        TokenKind::Word(m) => {
                            let p = word(j);
                            if HAVE.contains(&p.as_str()) {
                                let line = line_of(tokens[j].span.start);
                                let l = lines.get(line - 1).unwrap_or(&"").trim_start();
                                if !(l.starts_with('|') || l.starts_with('#')) {
                                    println!("{}:{}: {} {}", path, line, p, w);
                                }
                                break 'tok;
                            }
                            if p == "to" || BE.contains(&p.as_str()) {
                                break;
                            }
                            let tag = m.as_ref().and_then(|m| m.pos_tag);
                            let stop = match variant.trim_end_matches("+v") {
                                "det" => matches!(tag, Some(UPOS::DET) | Some(UPOS::NUM)),
                                "np" => matches!(
                                    tag,
                                    Some(UPOS::DET)
                                        | Some(UPOS::NUM)
                                        | Some(UPOS::NOUN)
                                        | Some(UPOS::PROPN)
                                ),
                                _ => panic!("variant"),
                            };
                            if stop {
                                break;
                            }
                        }
                        // a code span, a number, a URL
                        _ => break,
                    }
                }
            }
        }
    }
}

// The gerund rule. An `-ing` word is a verb form when the tagger marks it VERB
// or AUX, and with `+u` also when it has no tag and the dictionary marks it
// progressive. No technical-noun list.
//
//   ing-lead   refused after a form of `be`, or a word tagged ADP or SCONJ,
//              skipping words tagged ADV (the scope of the bash rule)
//   ing-any    refused wherever it stands
fn gerund(
    variant: &str,
    tokens: &[harper_core::Token],
    source: &[char],
    lines: &[&str],
    start: usize,
    end: usize,
    path: &str,
) {
    let word = |i: usize| -> String {
        tokens[i].span.get_content(source).iter().collect::<String>().to_lowercase()
    };
    let line_of = |idx: usize| source[..idx].iter().filter(|c| **c == '\n').count() + 1;
    for i in start..end {
        let TokenKind::Word(Some(meta)) = &tokens[i].kind else { continue };
        let w = word(i);
        if !w.ends_with("ing") || w.chars().count() < 5 {
            continue;
        }
        // a hyphenated compound such as `load-bearing`
        if i > start && matches!(tokens[i - 1].kind, TokenKind::Punctuation(_)) {
            let prev: String = tokens[i - 1].span.get_content(source).iter().collect();
            if prev == "-" {
                continue;
            }
        }
        let verb = matches!(meta.pos_tag, Some(UPOS::VERB) | Some(UPOS::AUX))
            || (variant.ends_with("+u")
                && meta.pos_tag.is_none()
                && meta.is_verb_progressive_form());
        if !verb {
            continue;
        }
        let mut lead = !variant.starts_with("ing-lead");
        let mut j = i;
        while !lead && j > start {
            j -= 1;
            match &tokens[j].kind {
                TokenKind::Space(_) | TokenKind::Newline(_) => continue,
                TokenKind::Word(m) => {
                    let p = word(j);
                    let tag = m.as_ref().and_then(|m| m.pos_tag);
                    if matches!(tag, Some(UPOS::ADV) | Some(UPOS::PART)) && p != "to" {
                        continue;
                    }
                    lead = BE.contains(&p.as_str())
                        || matches!(tag, Some(UPOS::ADP) | Some(UPOS::SCONJ));
                    break;
                }
                _ => break,
            }
        }
        if !lead {
            continue;
        }
        let line = line_of(tokens[i].span.start);
        let l = lines.get(line - 1).unwrap_or(&"").trim_start();
        if !(l.starts_with('|') || l.starts_with('#')) {
            println!("{}:{}: {}", path, line, w);
        }
        return;
    }
}
