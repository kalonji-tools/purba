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
//
// Prints `file:line: have participle` for each sentence with a fault, one per
// sentence (the first), and only on lines the bash gate reads: a table row
// (`|`) and a heading (`#`) are skipped.

use harper_core::parsers::MarkdownOptions;
use harper_core::{Document, TokenKind, TokenStringExt};
use harper_core::spell::FstDictionary;
use harper_pos_utils::UPOS;

const HAVE: [&str; 4] = ["has", "have", "had", "having"];
const BE: [&str; 8] = ["is", "are", "was", "were", "be", "been", "being", "am"];

fn main() {
    let mut args = std::env::args().skip(1);
    let variant = args.next().expect("variant");
    let dict = FstDictionary::curated();
    for path in args {
        let text = std::fs::read_to_string(&path).expect("read");
        let source: Vec<char> = text.chars().collect();
        let lines: Vec<&str> = text.lines().collect();
        let doc = Document::new_markdown(&text, MarkdownOptions::default(), &dict);
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
