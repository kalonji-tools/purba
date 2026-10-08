#!/usr/bin/env -S cargo -Zscript
---cargo
[package]
edition = "2024"

[dependencies]
harper-core = { git = "https://github.com/Automattic/harper", tag = "v2.3.0" }
harper-pos-utils = { git = "https://github.com/Automattic/harper", tag = "v2.3.0" }
pulldown-cmark = "0.13.4"

[lints.rust]
warnings = "deny"

[lints.clippy]
pedantic = { level = "deny", priority = -1 }
nursery = { level = "deny", priority = -1 }
# Neither `clippy::cargo` nor `[lints.cargo]` reads this manifest. The first
# reads the Cargo.toml above the script, and the second passed a crate this
# script never used.
---

//! The prose rules of the records, read once through CommonMark.
//!
//!   the decisions:
//!     docs/decisions/purba-borrows-from-simplified-technical-english-rather-than-adopting-it.md
//!     docs/decisions/an-artifact-holds-the-minimum-that-conveys-its-point.md
//!     docs/decisions/a-part-of-speech-tagger-decides-the-tense-rule.md
//!   the caller:  scripts/check-records.sh
//!
//!   check-prose.rs <participles> <record>...
//!
//! Prints one line for each finding, tagged with the rule it breaks, then the
//! tallies and the evidence density that the caller reports. Exits 0 either
//! way, and 2 when a file cannot be read. The caller decides the refusal.

use std::collections::{BTreeSet, HashSet};
use std::ops::Range;
use std::sync::Arc;

use harper_core::parsers::{Parser, PlainEnglish};
use harper_core::spell::{FstDictionary, MergedDictionary, MutableDictionary};
use harper_core::{DictWordMetadata, Document, Token, TokenKind, VerbData, VerbFormFlags};
use harper_pos_utils::UPOS;
use pulldown_cmark::{Event, Options, Tag, TagEnd};

const LIMIT: usize = 25;
const PARAGRAPH: usize = 6;

// A code span, a link or an image, which a reader reads as one thing.
const ATOM: char = '\u{1}';

// An `-ing` word the gerund rule admits, because it is a technical noun rather
// than a verb form.
#[rustfmt::skip]
const NOUNS: &[&str] = &[
    "setting", "settings", "heading", "headings", "listing", "listings", "mapping", "mappings",
    "warning", "warnings", "wording", "tooling", "tracking", "logging", "string", "strings",
    "thing", "things", "nothing", "something", "anything", "everything", "during", "according",
    "including", "being", "morning", "evening", "spring", "ceiling", "meaning", "rating",
    "ratings", "casing", "padding", "wrapping",
];

const BE: [&str; 8] = ["is", "are", "was", "were", "be", "been", "being", "am"];

#[rustfmt::skip]
const PREPOSITIONS: &[&str] = &[
    "by", "for", "of", "without", "before", "after", "while", "when", "on", "in", "about", "from",
    "than", "through", "against", "into", "onto", "over", "under", "as", "because",
];

#[rustfmt::skip]
const ADVERBS: &[&str] = &[
    "not", "never", "already", "also", "still", "now", "then", "always", "often", "again", "just",
    "therefore", "once", "since", "ever", "twice", "yet",
];

// A participle that does not end in `-ed`.
#[rustfmt::skip]
const IRREGULAR: &[&str] = &[
    "been", "begun", "bound", "bought", "broken", "brought", "built", "burnt", "caught", "chosen",
    "cost", "cut", "dealt", "done", "drawn", "driven", "eaten", "fallen", "felt", "fought",
    "found", "forgotten", "given", "gone", "grown", "heard", "held", "kept", "laid", "led", "left",
    "lent", "lost", "made", "meant", "met", "paid", "put", "run", "said", "seen", "sold", "sent",
    "set", "shown", "shut", "sung", "slept", "spoken", "spent", "stood", "taken", "taught", "told",
    "thought", "thrown", "understood", "won", "written", "forbidden", "hidden", "risen", "torn",
    "worn", "proven", "shaken", "stuck", "struck", "sworn", "read", "beaten", "frozen",
];

// A formal word the standard replaces with a plain one. The list holds only
// words with no second sense here: `required` and `per` are left out, because
// GitHub names a required check and a rate per hour.
#[rustfmt::skip]
const FORMAL: &[&str] = &[
    "utilize", "utilise", "utilization", "commence", "commences", "commenced", "terminate",
    "terminates", "terminated", "endeavour", "endeavor", "ascertain", "aforementioned",
    "notwithstanding", "whilst", "amongst", "heretofore", "thereof", "herein", "pursuant",
    "facilitate", "facilitates", "expedite",
];

const HAVE: [&str; 4] = ["has", "have", "had", "having"];
const ASK: [&str; 9] = [
    "what", "which", "who", "whom", "whose", "why", "where", "when", "how",
];

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(participles) = args.next() else {
        eprintln!("usage: check-prose.rs <participles> <record>...");
        std::process::exit(2);
    };
    let Ok(list) = std::fs::read_to_string(&participles) else {
        eprintln!("{participles} cannot be read.");
        std::process::exit(2);
    };
    let dictionary = dictionary(&list);

    let mut tally = Tally::default();
    let mut density = Vec::new();
    for path in args {
        let Ok(text) = std::fs::read_to_string(&path) else {
            eprintln!("{path} cannot be read.");
            std::process::exit(2);
        };
        let record = Record::read(&path, &text, &dictionary);
        for finding in &record.findings {
            println!("{finding}");
        }
        tally.add(&record.tally);
        density.push((path.clone(), record.bare, record.prose));
    }

    let formal: Vec<_> = tally.formal.iter().map(String::as_str).collect();
    println!(
        "tally\t{}\t{}\t{}\t{}\t{}\t{}",
        tally.sentences,
        tally.passives,
        tally.articles,
        tally.formals,
        tally.words,
        formal.join(" ")
    );
    for line in report_density(&density) {
        println!("density\t{line}");
    }
}

// Evidence density, one line for each record that holds a prose line, then one
// for all of them.
fn report_density(density: &[(String, u32, u32)]) -> Vec<String> {
    let percent = |bare: u32, prose: u32| 100.0 * f64::from(bare) / f64::from(prose);
    let mut lines: Vec<_> = density
        .iter()
        .filter(|(_, _, prose)| *prose > 0)
        .map(|(path, bare, prose)| {
            format!(
                "{:5.1}%  {bare:3} of {prose:3}  {path}",
                percent(*bare, *prose)
            )
        })
        .collect();
    let bare: u32 = density.iter().map(|(_, bare, _)| bare).sum();
    let prose: u32 = density.iter().map(|(_, _, prose)| prose).sum();
    // A corpus with no prose line is refused already, for having no Downside
    // list, and dividing by its zero would write an error into the report.
    lines.push(if prose == 0 {
        "all records  no prose line to read".to_string()
    } else {
        format!(
            "all records  {:.1}%  {bare} of {prose}",
            percent(bare, prose)
        )
    });
    lines
}

// Harper's dictionary, with each word of purba's list as a past participle.
fn dictionary(list: &str) -> MergedDictionary {
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

// The three borrowed rules a command cannot decide, counted for the report.
#[derive(Default)]
struct Tally {
    sentences: u32,
    passives: u32,
    articles: u32,
    formals: u32,
    words: u32,
    formal: BTreeSet<String>,
}

impl Tally {
    fn add(&mut self, other: &Self) {
        self.sentences += other.sentences;
        self.passives += other.passives;
        self.articles += other.articles;
        self.formals += other.formals;
        self.words += other.words;
        self.formal.extend(other.formal.iter().cloned());
    }

    // Counts what the report reads in one sentence, and says whether it is in
    // the passive voice.
    fn count(&mut self, words: &[String]) -> bool {
        let mut passive = false;
        for (i, word) in words.iter().enumerate() {
            let lower = word.to_lowercase();
            if ["a", "an", "the"].contains(&lower.as_str()) {
                self.articles += 1;
            }
            if FORMAL.contains(&lower.as_str()) {
                self.formals += 1;
                self.formal.insert(lower.clone());
            }
            if lower.contains(|c: char| c.is_ascii_alphabetic()) {
                self.words += 1;
            }
            if !passive && participle(&lower) {
                passive = BE.contains(&prior(words, i).as_str());
            }
        }
        passive
    }
}

// One character a rule reads, and the byte of the record it stands on.
#[derive(Clone, Copy)]
struct Char {
    c: char,
    at: usize,
}

// The prose of a paragraph or a list item, or the text of a table cell.
struct Block {
    chars: Vec<Char>,
    cell: bool,
    quoted: bool,
    top: bool,
}

impl Block {
    const fn new(cell: bool, quoted: bool, top: bool) -> Self {
        Self {
            chars: Vec::new(),
            cell,
            quoted,
            top,
        }
    }

    fn push(&mut self, text: &str, at: usize) {
        self.chars.extend(text.chars().map(|c| Char { c, at }));
    }

    // A sentence starts at the head of a block, or after a boundary. Bold opens
    // it when no word stands before the bold in that sentence.
    fn opens(&self) -> bool {
        let start = boundaries(&self.chars).last().copied().unwrap_or(0);
        !self.chars[start..]
            .iter()
            .any(|c| c.c == ATOM || c.c.is_ascii_alphanumeric())
    }
}

// Whether a line closes at a boundary, which bold or a quotation mark may follow.
fn ends(line: &str) -> bool {
    line.trim_end_matches(|c: char| c.is_ascii_whitespace())
        .trim_end_matches(['*', '"', ')', ']'])
        .ends_with(['.', '!', '?'])
}

// What one record holds for the rules.
struct Record<'a> {
    path: &'a str,
    text: &'a str,
    lines: Vec<&'a str>,
    starts: Vec<usize>,
    dictionary: &'a MergedDictionary,
    findings: Vec<String>,
    seen: HashSet<(&'static str, usize)>,
    tally: Tally,
    prose: u32,
    bare: u32,
}

impl<'a> Record<'a> {
    fn read(path: &'a str, text: &'a str, dictionary: &'a MergedDictionary) -> Self {
        let starts = std::iter::once(0)
            .chain(text.match_indices('\n').map(|(at, _)| at + 1))
            .collect();
        let mut record = Self {
            path,
            text,
            lines: text.lines().collect(),
            starts,
            dictionary,
            findings: Vec::new(),
            seen: HashSet::new(),
            tally: Tally::default(),
            prose: 0,
            bare: 0,
        };
        record.walk();
        record
    }

    fn line(&self, at: usize) -> usize {
        self.starts.partition_point(|start| *start <= at)
    }

    // A rule reports a line once, however many times it breaks there.
    fn find(&mut self, rule: &'static str, line: usize, detail: &str) {
        if self.seen.insert((rule, line)) {
            self.findings
                .push(format!("{rule}\t{}:{line}{detail}", self.path));
        }
    }

    // The options Harper reads Markdown with, so `[[a title]]` is still a link.
    fn walk(&mut self) {
        let options = Options::all().difference(Options::ENABLE_SMART_PUNCTUATION);
        let mut walk = Walk::default();
        for (event, range) in pulldown_cmark::Parser::new_ext(self.text, options).into_offset_iter()
        {
            match event {
                Event::Start(tag) => self.open(&mut walk, &tag, &range),
                Event::End(tag) => self.close(&mut walk, tag),
                Event::Text(text) if walk.hidden == 0 => {
                    // The em-dash rule reads a link title, because the writer
                    // chose it.
                    if walk.quoted == 0 && text.contains('—') {
                        self.find("emdash", self.line(range.start), "");
                    }
                    if walk.linked == 0 {
                        walk.block().push(&text, range.start);
                    }
                }
                Event::Code(_)
                | Event::InlineMath(_)
                | Event::DisplayMath(_)
                | Event::FootnoteReference(_)
                    if walk.hidden == 0 && walk.linked == 0 =>
                {
                    walk.block().push(&ATOM.to_string(), range.start);
                }
                Event::SoftBreak | Event::HardBreak if walk.hidden == 0 => {
                    self.wrap(range.start);
                    walk.block().push("\n", range.start);
                }
                _ => {}
            }
        }
        let block = walk.current.take();
        self.flush(block);
    }

    fn open(&mut self, walk: &mut Walk, tag: &Tag, range: &Range<usize>) {
        match tag {
            Tag::Paragraph => {
                self.flush(walk.current.take());
                let top = walk.quoted == 0 && walk.nested == 0;
                if top {
                    self.count(range);
                }
                walk.current = Some(Block::new(false, walk.quoted > 0, top));
            }
            Tag::TableCell => {
                self.flush(walk.current.take());
                walk.current = Some(Block::new(true, walk.quoted > 0, false));
            }
            Tag::BlockQuote(_) => {
                self.flush(walk.current.take());
                walk.quoted += 1;
            }
            Tag::Item
            | Tag::FootnoteDefinition(_)
            | Tag::DefinitionListTitle
            | Tag::DefinitionListDefinition => {
                self.flush(walk.current.take());
                walk.nested += 1;
            }
            Tag::Heading { .. } | Tag::CodeBlock(_) | Tag::MetadataBlock(_) => {
                self.flush(walk.current.take());
                walk.hidden += 1;
            }
            Tag::List(_) | Tag::Table(_) | Tag::DefinitionList | Tag::HtmlBlock => {
                self.flush(walk.current.take());
            }
            Tag::Strong if walk.hidden == 0 && walk.quoted == 0 => {
                if !walk.block().opens() {
                    self.find("bold", self.line(range.start), "");
                }
            }
            Tag::Link { .. } | Tag::Image { .. } => {
                if walk.linked == 0 {
                    walk.link = range.start;
                }
                walk.linked += 1;
            }
            _ => {}
        }
    }

    fn close(&mut self, walk: &mut Walk, tag: TagEnd) {
        match tag {
            TagEnd::Paragraph | TagEnd::TableCell | TagEnd::List(_) | TagEnd::Table => {
                self.flush(walk.current.take());
            }
            TagEnd::BlockQuote(_) => {
                self.flush(walk.current.take());
                walk.quoted -= 1;
            }
            TagEnd::Item
            | TagEnd::FootnoteDefinition
            | TagEnd::DefinitionListTitle
            | TagEnd::DefinitionListDefinition => {
                self.flush(walk.current.take());
                walk.nested -= 1;
            }
            TagEnd::Heading(_) | TagEnd::CodeBlock | TagEnd::MetadataBlock(_) => {
                walk.hidden -= 1;
            }
            TagEnd::Link | TagEnd::Image => {
                walk.linked -= 1;
                if walk.linked == 0 && walk.hidden == 0 {
                    let at = walk.link;
                    walk.block().push(&ATOM.to_string(), at);
                }
            }
            _ => {}
        }
    }

    // `docs/decisions/.template.md` puts one sentence on one line, so a line
    // that ends mid-sentence and goes on is wrapped. A quote is read here,
    // because the writer chose where its lines break.
    fn wrap(&mut self, at: usize) {
        let line = self.line(at);
        let text = self.lines.get(line - 1).copied().unwrap_or_default();
        if !ends(text) {
            self.find("wrap", line, "");
        }
    }

    // Evidence density reads a paragraph outside a list item and a quote, line
    // by line. A line is bare when it holds no digit, no code span and no link.
    fn count(&mut self, range: &Range<usize>) {
        let first = self.line(range.start);
        let last = self.line(range.end.saturating_sub(1).max(range.start));
        for line in first..=last {
            let text = self.lines.get(line - 1).copied().unwrap_or_default();
            self.prose += 1;
            if !text.contains(|c: char| c.is_ascii_digit() || c == '`') && !text.contains("](") {
                self.bare += 1;
            }
        }
    }

    fn flush(&mut self, block: Option<Block>) {
        let Some(block) = block.filter(|block| !block.cell && !block.quoted) else {
            return;
        };
        let mut opened = Vec::new();
        for sentence in sentences(&block.chars) {
            let Some(first) = sentence.iter().find(|c| !c.c.is_ascii_whitespace()) else {
                continue;
            };
            let line = self.line(first.at);
            opened.push(line);
            let text: String = sentence.iter().map(|c| c.c).collect();

            let length = length(&text);
            if length > LIMIT {
                self.find("long", line, &format!(": {length} words"));
            }
            let (words, starts): (Vec<String>, Vec<usize>) = words(sentence).into_iter().unzip();
            // The line the gerund stands on, which a wrapped sentence moves.
            if let Some(i) = gerund(&words) {
                let at = self.line(sentence[starts[i]].at);
                self.find("ing", at, &format!(": {}", words[i]));
            }
            self.tally.sentences += 1;
            if self.tally.count(&words) {
                self.tally.passives += 1;
            }
            if let Some((line, pair)) = self.tense(sentence) {
                self.findings
                    .push(format!("tense\t{}:{line}: {pair}", self.path));
            }
        }
        // A paragraph is a paragraph outside a list item and a quote.
        if block.top && opened.len() > PARAGRAPH {
            let line = opened[PARAGRAPH];
            let count = opened.iter().filter(|opens| **opens <= line).count();
            self.find("para", line, &format!(": {count} sentences"));
        }
    }

    // Harper tags the sentence, and each code span and link is one token that
    // ends the walk back to `have`.
    fn tense(&self, sentence: &[Char]) -> Option<(usize, String)> {
        let mut text = String::new();
        let mut lines = Vec::new();
        let mut atoms = Vec::new();
        for c in sentence {
            let line = self.line(c.at);
            if c.c == ATOM {
                let start = lines.len();
                text.push_str("codespan");
                lines.resize(start + "codespan".len(), line);
                atoms.push(start..lines.len());
            } else {
                text.push(c.c);
                lines.push(line);
            }
        }
        let document = Document::new(&text, &Atoms(atoms), self.dictionary);
        let source: Vec<char> = text.chars().collect();
        let word = |token: &Token| -> String {
            token
                .span
                .get_content(&source)
                .iter()
                .collect::<String>()
                .to_lowercase()
        };
        let tokens = document.get_tokens();
        perfect(tokens, &word).map(|(have, pair)| (lines[tokens[have].span.start], pair))
    }
}

// Where the walk over the events of a record stands.
#[derive(Default)]
struct Walk {
    current: Option<Block>,
    quoted: usize,
    nested: usize,
    hidden: usize,
    linked: usize,
    link: usize,
}

impl Walk {
    // A tight list item holds its text with no paragraph around it.
    fn block(&mut self) -> &mut Block {
        let quoted = self.quoted > 0;
        self.current
            .get_or_insert_with(|| Block::new(false, quoted, false))
    }
}

// A boundary is `.`, `!` or `?`, then whitespace, as a closing quote, bracket
// or bold may follow the mark. A version number and a bare filename keep their
// full stop, because neither puts a space after it.
//
// A lowercase word is not a continuation, because a record opens a sentence
// with a lowercase name such as `purba` or `mise`.
fn boundaries(chars: &[Char]) -> Vec<usize> {
    let mut found = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let mut j = i + 1;
        if matches!(chars[i].c, '.' | '!' | '?') {
            while j < chars.len() && matches!(chars[j].c, '*' | '"' | ')' | ']') {
                j += 1;
            }
            if j < chars.len() && chars[j].c.is_ascii_whitespace() {
                while j < chars.len() && chars[j].c.is_ascii_whitespace() {
                    j += 1;
                }
                found.push(j);
            } else {
                j = i + 1;
            }
        }
        i = j;
    }
    found
}

fn sentences(chars: &[Char]) -> Vec<&[Char]> {
    let mut found = Vec::new();
    let mut start = 0;
    for end in boundaries(chars) {
        found.push(&chars[start..end]);
        start = end;
    }
    if chars[start..].iter().any(|c| !c.c.is_ascii_whitespace()) {
        found.push(&chars[start..]);
    }
    found
}

// The words the length rule counts. A code span and a link count as one word,
// and a mark with no letter and no digit counts as none.
fn length(sentence: &str) -> usize {
    sentence
        .replace(ATOM, " codespan ")
        .split_ascii_whitespace()
        .filter(|word| word.contains(|c: char| c.is_ascii_alphanumeric()))
        .count()
}

// The words the gerund rule and the tallies read. A word keeps its apostrophe
// and its hyphen. A code span becomes a word and never a gap: dropping it
// makes the words on either side adjacent, and `over CODE, checking` then reads
// as a preposition with a gerund nobody wrote.
fn words(sentence: &[Char]) -> Vec<(String, usize)> {
    let mut found = Vec::new();
    let mut word = String::new();
    let mut start = 0;
    for (i, c) in sentence.iter().enumerate() {
        if c.c.is_ascii_alphabetic() || c.c == '\'' || c.c == '-' {
            if word.is_empty() {
                start = i;
            }
            word.push(c.c);
            continue;
        }
        if !word.is_empty() {
            found.push((std::mem::take(&mut word), start));
        }
        if c.c == ATOM {
            found.push(("codespan".to_string(), i));
        }
    }
    if !word.is_empty() {
        found.push((word, start));
    }
    found
}

fn adverb(word: &str) -> bool {
    ADVERBS.contains(&word) || word.ends_with("ly")
}

// Three letters or fewer is not a participle, which is what keeps `red` out.
fn participle(word: &str) -> bool {
    IRREGULAR.contains(&word) || (word.ends_with("ed") && word.len() > 3)
}

// The word before, skipping every adverb.
fn prior(words: &[String], i: usize) -> String {
    words[..i]
        .iter()
        .rev()
        .map(|word| word.to_lowercase())
        .find(|word| !adverb(word))
        .unwrap_or_default()
}

// An `-ing` form after a form of be or a preposition. A hyphenated word is a
// compound adjective and never a verb form, so `load-bearing` after `is` is not
// the progressive it looks like.
fn gerund(words: &[String]) -> Option<usize> {
    (0..words.len()).find(|i| {
        let lower = words[*i].to_lowercase();
        let verb = !lower.contains('-')
            && lower.ends_with("ing")
            && lower.len() >= 5
            && !NOUNS.contains(&lower.as_str());
        let before = prior(words, *i);
        verb && (BE.contains(&before.as_str()) || PREPOSITIONS.contains(&before.as_str()))
    })
}

// Harper's plain English tokens, with each code span and link as one
// unlintable token, as Harper's Markdown parser makes a code span.
struct Atoms(Vec<Range<usize>>);

impl Parser for Atoms {
    fn parse(&self, source: &[char]) -> Vec<Token> {
        let mut tokens: Vec<Token> = Vec::new();
        for token in PlainEnglish.parse(source) {
            let Some(atom) = self.0.iter().find(|atom| atom.contains(&token.span.start)) else {
                tokens.push(token);
                continue;
            };
            match tokens.last_mut() {
                Some(last) if last.kind.is_unlintable() && atom.contains(&last.span.start) => {
                    last.span.end = token.span.end;
                }
                _ => tokens.push(Token::new(token.span, TokenKind::Unlintable)),
            }
        }
        tokens
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    // The findings a record holding `text` gives, with the path left out.
    fn findings(text: &str) -> Vec<String> {
        findings_with("", text)
    }

    fn findings_with(participles: &str, text: &str) -> Vec<String> {
        let dictionary = dictionary(participles);
        Record::read("r.md", text, &dictionary)
            .findings
            .into_iter()
            .map(|finding| finding.replace("r.md:", ""))
            .collect()
    }

    fn passives(text: &str) -> u32 {
        let dictionary = dictionary("");
        Record::read("r.md", text, &dictionary).tally.passives
    }

    // The bare lines and the prose lines that evidence density counts.
    fn density(text: &str) -> (u32, u32) {
        let dictionary = dictionary("");
        let record = Record::read("r.md", text, &dictionary);
        (record.bare, record.prose)
    }

    fn filler(words: usize) -> String {
        "word ".repeat(words)
    }

    #[test]
    fn an_em_dash_in_a_table_cell_is_read() {
        let found = findings("| a | b |\n|---|---|\n| one — two | three |\n");
        assert_eq!(found, ["emdash\t3"]);
    }

    #[test]
    fn an_em_dash_in_a_code_span_or_a_fenced_block_is_not_read() {
        let found = findings("It quotes `a — b` here.\n\n```\na — b\n```\n");
        assert_eq!(found, [] as [&str; 0]);
    }

    #[test]
    fn two_globs_on_one_line_are_not_bold() {
        assert_eq!(
            findings("It names .github/** and /scripts/** alike.\n"),
            [] as [&str; 0]
        );
    }

    #[test]
    fn bold_after_a_code_span_sits_inside_the_sentence() {
        assert_eq!(findings("`mise` **really** runs it.\n"), ["bold\t1"]);
    }

    #[test]
    fn bold_that_opens_a_table_cell_opens_its_sentence() {
        let found = findings("| a | b |\n|---|---|\n| one | **two** |\n");
        assert_eq!(found, [] as [&str; 0]);
    }

    #[test]
    fn a_line_that_breaks_a_sentence_in_a_list_item_or_a_quote_is_wrapped() {
        for text in [
            "- The gate runs\n  on each record.\n",
            "> The gate runs\n> on each record.\n",
        ] {
            assert_eq!(findings(text), ["wrap\t1"], "{text}");
        }
    }

    #[test]
    fn a_line_before_a_list_a_table_a_fence_or_a_heading_is_not_wrapped() {
        for text in [
            "The rules are these:\n- One.\n",
            "The rules are these:\n| a |\n|---|\n",
            "The rules are these:\n```\nrule\n```\nThat is all.\n",
            "The rules are these:\n### Rules\nOne.\n",
            "- The rules are these:\n  - One.\n",
            "> The rules are these:\n> - One.\n",
        ] {
            assert_eq!(findings(text), [] as [&str; 0], "{text}");
        }
    }

    #[test]
    fn a_sentence_that_ends_inside_bold_is_not_wrapped() {
        assert_eq!(
            findings("**The rule is decided.**\nIt holds.\n"),
            [] as [&str; 0]
        );
    }

    #[test]
    fn a_link_is_one_word_whatever_its_title_holds() {
        let title = "[a title that is ten words long all on its own](https://example.invalid)";
        let found = findings(&format!("{}{title} end.\n", filler(23)));
        assert_eq!(found, [] as [&str; 0]);
    }

    #[test]
    fn a_code_span_is_one_word_and_never_none() {
        let span = "`one two three four five`";
        assert_eq!(
            findings(&format!("{}{span} end.\n", filler(23))),
            [] as [&str; 0]
        );
        let found = findings(&format!("{}{span} end.\n", filler(24)));
        assert_eq!(found, ["long\t1: 26 words"]);
    }

    #[test]
    fn a_mark_with_no_letter_and_no_digit_is_not_a_word() {
        assert_eq!(
            findings(&format!("{}/ end.\n", filler(24))),
            [] as [&str; 0]
        );
    }

    #[test]
    fn a_sentence_in_a_list_item_is_prose() {
        let found = findings(&format!("- {}end.\n", filler(25)));
        assert_eq!(found, ["long\t1: 26 words"]);
    }

    #[test]
    fn a_table_cell_is_not_prose() {
        let found = findings(&format!("| a |\n|---|\n| {}end |\n", filler(29)));
        assert_eq!(found, [] as [&str; 0]);
    }

    #[test]
    fn a_block_quote_is_not_prose() {
        let text = format!("> {}end — and **so** on.\n", filler(25));
        assert_eq!(findings(&text), [] as [&str; 0]);
        assert_eq!(density(&text), (0, 0));
    }

    #[test]
    fn an_html_block_is_not_prose() {
        let found = findings(&format!("<div>\n{}end.\n</div>\n", filler(25)));
        assert_eq!(found, [] as [&str; 0]);
    }

    #[test]
    fn an_exclamation_mark_and_a_question_mark_each_end_a_sentence() {
        let words = filler(13);
        let found = findings(&format!("{words}end! {words}end? {words}end.\n"));
        assert_eq!(found, [] as [&str; 0]);
    }

    #[test]
    fn a_blank_line_splits_a_paragraph() {
        let found = findings("One.\nTwo.\nThree.\nFour.\n\nFive.\nSix.\nSeven.\nEight.\n");
        assert_eq!(found, [] as [&str; 0]);
    }

    #[test]
    fn seven_list_items_are_not_a_paragraph() {
        let found = findings("- One.\n- Two.\n- Three.\n- Four.\n- Five.\n- Six.\n- Seven.\n");
        assert_eq!(found, [] as [&str; 0]);
    }

    #[test]
    fn a_list_item_is_not_a_paragraph() {
        let found = findings("- One. Two. Three. Four. Five. Six. Seven.\n\n- Eight.\n");
        assert_eq!(found, [] as [&str; 0]);
    }

    #[test]
    fn each_block_that_is_not_prose_closes_the_paragraph_above_it() {
        for closer in [
            "### A heading",
            "> Quoted.",
            "1. Numbered.",
            "* Starred.",
            "+ Plus.",
        ] {
            let text =
                format!("One.\nTwo.\nThree.\nFour.\n{closer}\nFive.\nSix.\nSeven.\nEight.\n");
            assert_eq!(findings(&text), [] as [&str; 0], "{closer}");
        }
    }

    #[test]
    fn an_indented_line_goes_on_with_the_paragraph_above_it() {
        let found = findings("One.\nTwo.\nThree.\nFour.\n  Five.\nSix.\nSeven.\n");
        assert_eq!(found, ["para\t7: 7 sentences"]);
    }

    #[test]
    fn a_code_span_keeps_a_preposition_and_a_gerund_apart() {
        let found = findings("It reads over `the tree`, checking each line.\n");
        assert_eq!(found, [] as [&str; 0]);
    }

    #[test]
    fn a_tense_wrapped_across_two_lines_is_found_at_its_first_line() {
        let found = findings("The gate has\nrefused the record.\n");
        assert_eq!(found, ["wrap\t1", "tense\t1: has refused"]);
    }

    #[test]
    fn a_table_row_a_heading_and_a_link_title_hold_no_tense() {
        for text in [
            "| a | The gate has refused the record. |\n|---|---|\n",
            "### The gate has refused the record\n",
            "See [the gate has refused the record](https://example.com) here.\n",
            "[**The gate has refused the record**](https://example.com) is the case.\n",
            "[*The gate has refused the record*](https://example.com) is the case.\n",
            "[~~The gate has refused the record~~](https://example.com) is the case.\n",
            "See [[the gate has refused the record]] here.\n",
        ] {
            assert_eq!(findings(text), [] as [&str; 0], "{text}");
        }
    }

    #[test]
    fn a_paragraph_that_ends_in_a_link_still_ends_there() {
        let found = findings("See [the list](https://example.com)\n\nHas the gate refused it?\n");
        assert_eq!(found, ["tense\t3: has refused"]);
    }

    #[test]
    fn a_fence_nested_under_a_list_item_opens_a_block() {
        let found = findings("- **This.** Chosen.\n\n  ```\n  a — b\n  ```\n");
        assert_eq!(found, [] as [&str; 0]);
    }

    #[test]
    fn a_fence_one_record_leaves_open_does_not_hide_the_next_record() {
        let dictionary = dictionary("");
        let first = Record::read("a.md", "One.\n\n```\nunclosed\n", &dictionary);
        let second = Record::read("b.md", "Something — else.\n", &dictionary);
        assert_eq!(first.findings, [] as [&str; 0]);
        assert_eq!(second.findings, ["emdash\tb.md:1"]);
    }

    #[test]
    fn a_line_that_holds_a_link_is_not_bare() {
        assert_eq!(
            density("Something needs [a decision](https://example.invalid).\n"),
            (0, 1)
        );
    }

    #[test]
    fn a_list_item_opened_by_a_plus_holds_no_prose_line() {
        assert_eq!(density("+ **This.** Chosen.\n"), (0, 0));
    }

    #[test]
    fn bold_that_opens_a_second_sentence_passes() {
        let found = findings("Something needs a decision. **This one.** It is taken.\n");
        assert_eq!(found, [] as [&str; 0]);
    }

    #[test]
    fn one_bold_letter_inside_a_sentence_is_bold() {
        assert_eq!(findings("Something needs **a** decision.\n"), ["bold\t1"]);
    }

    #[test]
    fn a_paragraph_of_six_sentences_passes_and_a_longer_one_is_found_once() {
        let six = "One.\nTwo.\nThree.\nFour.\nFive.\nSix.\n";
        assert_eq!(findings(six), [] as [&str; 0]);
        let found = findings(&format!("{six}Seven.\nEight.\n"));
        assert_eq!(found, ["para\t7: 7 sentences"]);
    }

    #[test]
    fn a_gerund_after_a_form_of_be_or_a_preposition_is_found() {
        for (text, gerund) in [
            ("The gate is running the checks.", "running"),
            ("A writer finds it by running the gate.", "running"),
            ("The record names it as owing work.", "owing"),
            (
                "The gate stops, because refusing would close the issue.",
                "refusing",
            ),
        ] {
            assert_eq!(findings(text), [format!("ing\t1: {gerund}")], "{text}");
        }
    }

    #[test]
    fn an_adverb_between_the_two_does_not_hide_the_gerund() {
        for text in [
            "The gate is already running the checks.",
            "The gate is quickly running.",
            "The gate is not always running the checks.",
        ] {
            assert_eq!(findings(text), ["ing\t1: running"], "{text}");
        }
    }

    #[test]
    fn a_technical_noun_a_compound_and_a_word_of_four_letters_are_no_gerund() {
        for text in [
            "A rule is nothing without tooling.",
            "The setting is load-bearing.",
            "A host answers by ping.",
        ] {
            assert_eq!(findings(text), [] as [&str; 0], "{text}");
        }
    }

    #[test]
    fn a_perfect_tense_is_found_in_each_of_its_forms() {
        for (text, pair) in [
            ("They have refused it.", "have refused"),
            ("It had refused it.", "had refused"),
            ("It has written it.", "has written"),
            ("Having refused it, the gate stops.", "having refused"),
        ] {
            assert_eq!(findings(text), [format!("tense\t1: {pair}")], "{text}");
        }
    }

    #[test]
    fn an_adverb_does_not_hide_a_perfect_tense() {
        for (text, pair) in [
            ("The gate has not yet refused a record.", "has refused"),
            (
                "Having not yet refused it, the gate waits.",
                "having refused",
            ),
            ("A person has recently moved the date.", "has moved"),
            ("A person has since moved the date.", "has moved"),
            ("No person has ever moved the date.", "has moved"),
            ("A person has twice moved the date.", "has moved"),
        ] {
            assert_eq!(findings(text), [format!("tense\t1: {pair}")], "{text}");
        }
    }

    #[test]
    fn a_word_between_have_and_its_participle_does_not_hide_the_tense() {
        for (text, pair) in [
            ("A person has even moved the date.", "has moved"),
            ("The gate has itself refused the record.", "has refused"),
            ("Has anyone moved the date?", "has moved"),
            ("Has the gate refused the record?", "has refused"),
            ("Have purba's own 92 issues been consistent?", "have been"),
            ("Which records has the gate refused?", "has refused"),
            ("Why has the gate refused the record?", "has refused"),
            ("Have they all refused it?", "have refused"),
            (
                "Having itself refused it, the gate waits.",
                "having refused",
            ),
        ] {
            assert_eq!(findings(text), [format!("tense\t1: {pair}")], "{text}");
        }
    }

    #[test]
    fn a_participle_that_no_list_names_is_a_tense() {
        for (text, pair) in [
            ("This project has never had a contributor.", "has had"),
            ("The person has rewritten the record.", "has rewritten"),
        ] {
            assert_eq!(findings(text), [format!("tense\t1: {pair}")], "{text}");
        }
    }

    #[test]
    fn a_participle_is_a_tense_only_when_the_list_names_it() {
        let text = "It has grown one workflow at a time.\n";
        assert_eq!(findings(text), [] as [&str; 0]);
        assert_eq!(findings_with("grown\n", text), ["tense\t1: has grown"]);
    }

    #[test]
    fn a_modal_a_possession_an_obligation_a_passive_and_a_code_span_are_no_tense() {
        for text in [
            "The gate must be run, and that is a bound worth having.",
            "Not yet refused, the record stands.",
            "The gate has a fixed span.",
            "A check has nothing to run against.",
            "The gate has records which were refused.",
            "The lock has `rust` pinned.",
            "Does the record have a fixed span?",
            "Having a record refused is rare.",
            "What has a fixed span?",
            "Which record has 3 fixed spans?",
            "The record has its scope narrowed.",
        ] {
            assert_eq!(findings(text), [] as [&str; 0], "{text}");
        }
    }

    #[test]
    fn an_adverb_does_not_hide_the_passive_voice() {
        assert_eq!(passives("The gate is not yet refused by nothing.\n"), 1);
    }

    #[test]
    fn a_participle_is_irregular_or_ends_in_ed_and_runs_past_three_letters() {
        for (text, passive) in [
            ("The gate is used.", 1),
            ("The record is written.", 1),
            ("The light was red.", 0),
            ("The gate is open.", 0),
        ] {
            assert_eq!(passives(text), passive, "{text}");
        }
    }
}
