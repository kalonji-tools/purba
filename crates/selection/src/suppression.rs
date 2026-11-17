//! The suppressions of a module: each `# purba:` comment, its site, and its
//! faults.

use std::collections::{HashMap, HashSet};

use purba_parsing::Module;
use ruff_python_ast::{Expr, Stmt, StmtFunctionDef};
use ruff_text_size::{Ranged, TextSize};

/// The checks that a suppression turns into a warning.
pub const PRACTICE: [&str; 9] = [
    "foreign-decorator",
    "name-bound-again",
    "not-one-comparison",
    "computed-expected",
    "second-path",
    "assert-before-expectation",
    "swallowed-raise",
    "patched-name",
    "mock-without-spec",
];

/// The checks that execution sees, of those that a suppression turns into a
/// warning.
const OF_EXECUTION: [&str; 3] = [
    "second-path",
    "assert-before-expectation",
    "swallowed-raise",
];

/// The checks without which an outcome would stand on what purba did not
/// check.
const OUTCOME: [&str; 4] = [
    "no-expectation",
    "readings-differ",
    "assert-in-reading",
    "hook-replaced",
];

/// The checks of a test whose expectations the run never reads, so that a
/// suppressed one still reaches no outcome.
const NO_EXPECTATION: [&str; 3] = ["def-under-block", "name-without-def", "returned-value"];

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
        OF_EXECUTION.contains(&self.check)
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

/// Reads each `# purba:` comment of `module`. `tests` names each `def` that is
/// a test, by the offset where it starts.
pub fn sites(module: &Module, tests: &[TextSize]) -> Sites {
    let body = &module.syntax().body;
    let first = body
        .get(usize::from(body.first().is_some_and(is_docstring)))
        .map(|statement| module.position(start(statement)).line());
    let defs: Vec<Def<'_>> = defs(body)
        .into_iter()
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
                let check = PRACTICE
                    .iter()
                    .copied()
                    .find(|named| *named == check)
                    .unwrap_or_default();
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
    if OUTCOME.contains(&check) {
        Some(format!(
            "line {line} suppresses {check}, and an outcome cannot stand without that check"
        ))
    } else if NO_EXPECTATION.contains(&check) {
        Some(format!(
            "line {line} suppresses {check}, and a suppression cannot help: \
             purba reads no expectation of such a test"
        ))
    } else if !PRACTICE.contains(&check) {
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

/// Each `def` in `body`, at any depth: in a block, a class or a `def`.
fn defs(body: &[Stmt]) -> Vec<&StmtFunctionDef> {
    let mut found = Vec::new();
    // A walk that recurses once for each level of nesting overflows the stack
    // on statements that nest deeper than the stack.
    let mut pending: Vec<&Stmt> = body.iter().rev().collect();
    while let Some(statement) = pending.pop() {
        match statement {
            Stmt::FunctionDef(function) => {
                found.push(function);
                pending.extend(function.body.iter().rev());
            }
            Stmt::ClassDef(class) => pending.extend(class.body.iter().rev()),
            other => pending.extend(crate::blocks(other).into_iter().flatten().rev()),
        }
    }
    found
}
