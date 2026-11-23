//! The suppressions of a module: each `# purba:` comment, its site, and its
//! faults.

use std::collections::{HashMap, HashSet};

use purba_parsing::{Facts, Module};
use ruff_python_ast::{Expr, Stmt, StmtFunctionDef};
use ruff_text_size::{Ranged, TextSize};

/// What a suppression of a strict check does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tier {
    /// It turns a violation into a warning, and the test runs.
    Practice,
    /// It does not count: an outcome would stand on what purba did not check.
    Outcome,
    /// It cannot help: purba reads no expectation of such a test.
    NoExpectation,
}

/// Where a breach of a strict check shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Seen {
    /// In the text of the module, which selection reads.
    Selection,
    /// While a case runs, where execution and assertion see it.
    Run,
}

/// A strict check: its name, its tier, and the part that sees it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Check {
    pub name: &'static str,
    pub tier: Tier,
    pub seen: Seen,
}

const fn check(name: &'static str, tier: Tier, seen: Seen) -> Check {
    Check { name, tier, seen }
}

pub const FOREIGN_DECORATOR: Check = check("foreign-decorator", Tier::Practice, Seen::Selection);
pub const FOREIGN_FRAMEWORK: Check = check("foreign-framework", Tier::Practice, Seen::Selection);
pub const NAME_BOUND_AGAIN: Check = check("name-bound-again", Tier::Practice, Seen::Selection);
pub const NOT_ONE_COMPARISON: Check = check("not-one-comparison", Tier::Practice, Seen::Selection);
pub const COMPUTED_EXPECTED: Check = check("computed-expected", Tier::Practice, Seen::Selection);
pub const PATCHED_NAME: Check = check("patched-name", Tier::Practice, Seen::Selection);
pub const MOCK_WITHOUT_SPEC: Check = check("mock-without-spec", Tier::Practice, Seen::Selection);
pub const TYPE_ALONE: Check = check("type-alone", Tier::Practice, Seen::Selection);
pub const DEF_UNDER_BLOCK: Check = check("def-under-block", Tier::NoExpectation, Seen::Selection);
pub const NAME_WITHOUT_DEF: Check = check("name-without-def", Tier::NoExpectation, Seen::Selection);
pub const RETURNED_VALUE: Check = check("returned-value", Tier::NoExpectation, Seen::Selection);

/// Each strict check, once.
const CHECKS: [Check; 20] = [
    FOREIGN_DECORATOR,
    FOREIGN_FRAMEWORK,
    NAME_BOUND_AGAIN,
    NOT_ONE_COMPARISON,
    COMPUTED_EXPECTED,
    PATCHED_NAME,
    MOCK_WITHOUT_SPEC,
    TYPE_ALONE,
    check("second-path", Tier::Practice, Seen::Run),
    check("assert-before-expectation", Tier::Practice, Seen::Run),
    check("swallowed-raise", Tier::Practice, Seen::Run),
    check("iterator-read", Tier::Practice, Seen::Run),
    check("no-expectation", Tier::Outcome, Seen::Run),
    check("readings-differ", Tier::Outcome, Seen::Run),
    check("assert-in-reading", Tier::Outcome, Seen::Run),
    check("hook-replaced", Tier::Outcome, Seen::Run),
    check("no-item", Tier::Outcome, Seen::Run),
    DEF_UNDER_BLOCK,
    NAME_WITHOUT_DEF,
    RETURNED_VALUE,
];

/// The strict check named `name`, or `None` when no check has that name.
fn by_name(name: &str) -> Option<&'static Check> {
    CHECKS.iter().find(|check| check.name == name)
}

/// The names of the checks of the practice tier whose breach shows while a
/// case runs. Execution writes the same names, and a test of dispatch compares
/// the two.
pub fn seen_by_the_run() -> impl Iterator<Item = &'static str> {
    CHECKS
        .iter()
        .filter(|check| check.tier == Tier::Practice && check.seen == Seen::Run)
        .map(|check| check.name)
}

/// A suppression that reaches a test: its check, its reason, its line, and the
/// test above it, or `None` for a module's suppression.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Suppression {
    pub check: &'static str,
    pub reason: String,
    pub line: usize,
    pub test: Option<String>,
}

impl Suppression {
    /// Whether execution, and not selection, sees its check.
    #[must_use]
    pub fn seen_by_execution(&self) -> bool {
        by_name(self.check).is_some_and(|check| check.seen == Seen::Run)
    }
}

/// The suppressions of a module, by site, and the faults of each.
#[derive(Default)]
pub struct Sites {
    /// The suppressions above each `def` of a test, by the offset where it starts.
    tests: HashMap<TextSize, Vec<Suppression>>,
    /// The faults of the suppressions above each `def` of a test, by the offset
    /// where it starts: the line, and the words.
    test_faults: HashMap<TextSize, Vec<(usize, String)>>,
    /// The suppressions of the module.
    module: Vec<Suppression>,
    /// The faults of the module's suppressions, and of each `# purba:` comment
    /// at no site: the line, and the words.
    module_faults: Vec<(usize, String)>,
}

impl Sites {
    /// The suppressions that reach the test whose `def`s start at `defs`: its
    /// own, then the module's of a check that it does not name.
    pub fn of(&self, defs: &[TextSize]) -> Vec<Suppression> {
        let defs = unique(defs);
        let mut own: Vec<Suppression> = Vec::new();
        for suppression in defs.iter().filter_map(|at| self.tests.get(at)).flatten() {
            if !own.iter().any(|named| named.check == suppression.check) {
                own.push(suppression.clone());
            }
        }
        let module = self
            .module
            .iter()
            .filter(|suppression| !own.iter().any(|named| named.check == suppression.check))
            .cloned()
            .collect::<Vec<_>>();
        own.into_iter().chain(module).collect()
    }

    /// The words of each fault of the suppressions of the test whose `def`s
    /// start at `defs`, by its line.
    pub fn faults_of(&self, defs: &[TextSize]) -> Vec<String> {
        let defs = unique(defs);
        let mut faults: Vec<(usize, String)> = defs
            .iter()
            .filter_map(|at| self.test_faults.get(at))
            .flatten()
            .cloned()
            .collect();
        faults.sort_by_key(|(line, _)| *line);
        faults.into_iter().map(|(_, words)| words).collect()
    }

    /// Each suppression in the module, above a test or of the module, in source
    /// order.
    pub fn all(&self) -> Vec<Suppression> {
        let mut all: Vec<Suppression> = self
            .tests
            .values()
            .flatten()
            .chain(&self.module)
            .cloned()
            .collect();
        all.sort_by_key(|suppression| suppression.line);
        all
    }

    /// The faults of the module's suppressions and of each `# purba:` comment
    /// at no site, by line, joined by `; `, or `None` when there is none.
    pub fn module_fault(&self) -> Option<String> {
        let mut faults = self.module_faults.clone();
        faults.sort_by_key(|(line, _)| *line);
        (!faults.is_empty()).then(|| {
            faults
                .into_iter()
                .map(|(_, words)| words)
                .collect::<Vec<_>>()
                .join("; ")
        })
    }
}

/// `defs` with each offset once, in order.
fn unique(defs: &[TextSize]) -> Vec<TextSize> {
    let mut defs = defs.to_vec();
    defs.sort_unstable();
    defs.dedup();
    defs
}

/// What a `# purba:` comment reads as.
enum Read<'a> {
    /// A suppression: of the module or of a test, the name of its check, and
    /// its reason, which can be empty.
    Suppress {
        module: bool,
        check: &'a str,
        reason: &'a str,
    },
    /// No suppression.
    Unread,
}

/// What `text`, a comment, reads as, or `None` when it is not purba's.
fn read(text: &str) -> Option<Read<'_>> {
    let rest = text
        .strip_prefix('#')?
        .trim_start()
        .strip_prefix("purba:")?;
    let rest = rest.trim_start();
    // Prototype: `# purba: set[...]` belongs to the command's configuration.
    if rest.starts_with("set[") {
        return None;
    }
    let (module, rest) = if let Some(rest) = rest.strip_prefix("suppress-module[") {
        (true, rest)
    } else if let Some(rest) = rest.strip_prefix("suppress[") {
        (false, rest)
    } else {
        return Some(Read::Unread);
    };
    let Some((check, after)) = rest.split_once(']') else {
        return Some(Read::Unread);
    };
    if check.is_empty() || check.contains(char::is_whitespace) {
        return Some(Read::Unread);
    }
    if !after.is_empty() && !after.starts_with(char::is_whitespace) {
        return Some(Read::Unread);
    }
    Some(Read::Suppress {
        module,
        check,
        reason: after.trim(),
    })
}

/// A `def` of the module, at any depth: its name, the line of its first
/// decorator or of its `def`, and whether it is a test.
struct Def<'a> {
    name: &'a str,
    at: TextSize,
    top: usize,
    test: bool,
}

/// The site of a `# purba:` comment.
enum Site {
    Test(TextSize),
    Module,
}

/// Reads each `# purba:` comment of the module of `facts`. `tests` names each
/// `def` that is a test, by the offset where it starts.
pub fn sites(facts: &Facts<'_>, tests: &[TextSize]) -> Sites {
    let module = facts.module();
    let body = &module.syntax().body;
    let first = body
        .get(usize::from(body.first().is_some_and(is_docstring)))
        .map(|statement| module.position(start(statement)).line());
    let defs: Vec<Def<'_>> = facts
        .defs()
        .iter()
        .map(|function| Def {
            name: function.name.as_str(),
            at: function.range.start(),
            top: module.position(start_of_def(function)).line(),
            test: tests.contains(&function.range.start()),
        })
        .collect();
    let comments = comments(module);
    let own_lines: HashSet<usize> = comments
        .iter()
        .filter(|(_, own_line, _)| *own_line)
        .map(|(line, _, _)| *line)
        .collect();
    let attached = |line: usize| -> Option<&Def<'_>> {
        defs.iter().find(|def| {
            def.top > line && (line + 1..def.top).all(|between| own_lines.contains(&between))
        })
    };
    let mut sites = Sites::default();
    let mut found: Vec<(Site, Option<&str>, usize, &str, &str)> = Vec::new();
    for (line, own_line, text) in comments {
        let Some(read) = read(text) else {
            continue;
        };
        let def = if own_line { attached(line) } else { None };
        let test = def.filter(|def| def.test).map(|def| def.at);
        let in_header = first.is_none_or(|first| line < first);
        match (read, test) {
            (Read::Unread, Some(test)) => push(&mut sites.test_faults, test, line, unread(line)),
            (Read::Unread, None) => sites.module_faults.push((line, unread(line))),
            (
                Read::Suppress {
                    module: true,
                    check,
                    reason,
                },
                _,
            ) if in_header => {
                found.push((Site::Module, None, line, check, reason));
            }
            (Read::Suppress { module: true, .. }, Some(test)) => {
                push(&mut sites.test_faults, test, line, below(line));
            }
            (Read::Suppress { module: true, .. }, None) => {
                sites.module_faults.push((line, below(line)));
            }
            (
                Read::Suppress {
                    module: false,
                    check,
                    reason,
                },
                Some(test),
            ) => {
                found.push((
                    Site::Test(test),
                    def.map(|def| def.name),
                    line,
                    check,
                    reason,
                ));
            }
            (Read::Suppress { module: false, .. }, None) => {
                let words = def.map_or_else(
                    || format!("line {line} suppresses above no test"),
                    |def| {
                        format!(
                            "line {line} suppresses above {}, which is no test",
                            def.name
                        )
                    },
                );
                sites.module_faults.push((line, words));
            }
        }
    }
    for (site, name, line, check, reason) in found {
        sites.settle(site, name, line, check, reason);
    }
    sites
}

impl Sites {
    /// Keeps the suppression of `check` with `reason` at `line` of `site`, above
    /// the test `name`, or the module's when `name` is `None`, or its fault.
    fn settle(&mut self, site: Site, name: Option<&str>, line: usize, check: &str, reason: &str) {
        let named = match &site {
            Site::Test(test) => self.tests.get(test).map_or(&[][..], Vec::as_slice),
            Site::Module => self.module.as_slice(),
        };
        match (site, fault(line, check, reason, named)) {
            (Site::Test(test), Some(words)) => push(&mut self.test_faults, test, line, words),
            (Site::Module, Some(words)) => self.module_faults.push((line, words)),
            (site, None) => {
                let check = by_name(check)
                    .filter(|check| check.tier == Tier::Practice)
                    .map_or("", |check| check.name);
                let suppression = Suppression {
                    check,
                    reason: reason.to_owned(),
                    line,
                    test: name.map(str::to_owned),
                };
                match site {
                    Site::Test(test) => self.tests.entry(test).or_default().push(suppression),
                    Site::Module => self.module.push(suppression),
                }
            }
        }
    }
}

/// Prototype: a `# purba: set[<key>] <value>` comment above a `def`.
#[derive(Clone, Debug)]
pub struct Setting {
    pub function: String,
    pub key: String,
    pub value: String,
    pub line: usize,
}

/// Prototype: each `# purba: set[...]` comment that sits above a `def`, and
/// the line of each that sits above none. A comment attaches to a `def` as a
/// suppression does: on lines of their own, with only comments between.
#[must_use]
pub fn settings(facts: &Facts<'_>) -> (Vec<Setting>, Vec<usize>) {
    let module = facts.module();
    let defs: Vec<(&str, usize)> = facts
        .defs()
        .iter()
        .map(|function| {
            (
                function.name.as_str(),
                module.position(start_of_def(function)).line(),
            )
        })
        .collect();
    let comments = comments(module);
    let own_lines: HashSet<usize> = comments
        .iter()
        .filter(|(_, own_line, _)| *own_line)
        .map(|(line, _, _)| *line)
        .collect();
    let mut found = Vec::new();
    let mut stray = Vec::new();
    for (line, own_line, text) in comments {
        let Some((key, value)) = set(text) else {
            continue;
        };
        let def = defs.iter().find(|(_, top)| {
            own_line && *top > line && (line + 1..*top).all(|between| own_lines.contains(&between))
        });
        match def {
            Some((name, _)) => found.push(Setting {
                function: (*name).to_owned(),
                key: key.to_owned(),
                value: value.trim().to_owned(),
                line,
            }),
            None => stray.push(line),
        }
    }
    (found, stray)
}

/// The key and the value of a `# purba: set[<key>] <value>` comment.
fn set(text: &str) -> Option<(&str, &str)> {
    text.strip_prefix('#')?
        .trim_start()
        .strip_prefix("purba:")?
        .trim_start()
        .strip_prefix("set[")?
        .split_once(']')
}

/// Each comment of `module`: its line, whether it sits on a line of its own,
/// and its text.
fn comments(module: &Module) -> Vec<(usize, bool, &str)> {
    let source = module.source();
    module
        .trivia()
        .iter()
        .filter(|range| source[**range].starts_with('#'))
        .map(|range| {
            let line_start = source[..range.start().to_usize()]
                .rfind('\n')
                .map_or(0, |at| at + 1);
            let own_line = source[line_start..range.start().to_usize()]
                .trim()
                .is_empty();
            (
                module.position(range.start()).line(),
                own_line,
                &source[*range],
            )
        })
        .collect()
}

/// The words of the fault of a suppression of `check` with `reason` at
/// `line`, at a site that already holds `named`, or `None` when it has none.
fn fault(line: usize, check: &str, reason: &str, named: &[Suppression]) -> Option<String> {
    let tier = by_name(check).map(|check| check.tier);
    if tier == Some(Tier::Outcome) {
        Some(format!(
            "line {line} suppresses {check}, and an outcome cannot stand without that check"
        ))
    } else if tier == Some(Tier::NoExpectation) {
        Some(format!(
            "line {line} suppresses {check}, and a suppression cannot help: \
             purba reads no expectation of such a test"
        ))
    } else if tier.is_none() {
        Some(format!(
            "line {line} suppresses {check}, which is no strict check"
        ))
    } else if reason.is_empty() {
        Some(format!(
            "line {line} suppresses {check} with no reason, and a suppression gives one"
        ))
    } else if named.iter().any(|suppression| suppression.check == check) {
        Some(format!(
            "line {line} suppresses {check} again, and a site names each check once"
        ))
    } else {
        None
    }
}

/// The words of a `# purba:` comment at `line` that reads as no suppression.
fn unread(line: usize) -> String {
    format!(
        "line {line} is a purba comment that reads as no suppression, \
         and a suppression reads # purba: suppress[<check>] <reason>"
    )
}

/// The words of a suppression of the module at `line`, below its first
/// statement.
fn below(line: usize) -> String {
    format!("line {line} suppresses the module below its first statement")
}

fn push(
    faults: &mut HashMap<TextSize, Vec<(usize, String)>>,
    test: TextSize,
    line: usize,
    words: String,
) {
    faults.entry(test).or_default().push((line, words));
}

/// Whether `statement` is a docstring.
fn is_docstring(statement: &Stmt) -> bool {
    matches!(statement, Stmt::Expr(expr) if matches!(*expr.value, Expr::StringLiteral(_)))
}

/// Where `statement` starts, at its first decorator if it has one.
fn start(statement: &Stmt) -> TextSize {
    match statement {
        Stmt::FunctionDef(function) => start_of_def(function),
        Stmt::ClassDef(class) => class
            .decorator_list
            .iter()
            .map(Ranged::start)
            .chain([class.range.start()])
            .min()
            .unwrap_or_default(),
        other => other.start(),
    }
}

/// Where `function` starts, at its first decorator if it has one.
fn start_of_def(function: &StmtFunctionDef) -> TextSize {
    function
        .decorator_list
        .iter()
        .map(Ranged::start)
        .chain([function.range.start()])
        .min()
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::CHECKS;

    #[test]
    fn each_strict_check_has_a_case_in_the_file_of_strictness() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/strictness.bats");
        let cases = std::fs::read_to_string(path).unwrap_or_else(|error| panic!("{path}: {error}"));

        let missing: Vec<&str> = CHECKS
            .iter()
            .map(|check| check.name)
            .filter(|check| !cases.contains(&format!("@test \"[{check}]")))
            .collect();

        assert!(missing.is_empty(), "{path} holds no case of {missing:?}");
    }
}
