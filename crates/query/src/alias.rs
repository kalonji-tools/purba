//! The aliases of the project's file, and their expansion into a tree with no
//! call.

use std::collections::BTreeMap;

use crate::{DEPTH, Query, Refused, Why, parse};

/// Where the project's file declares the aliases, as its words name it.
const TABLE: &str = "[tool.purba.queries]";

/// The words that a term of a query takes, so that no alias may.
const TERMS: [&str; 3] = ["name", "path", "tag"];

/// The name nearest to a word among names, by edits, if any.
pub type Nearest = for<'a> fn(&str, &[&'a str]) -> Option<&'a str>;

/// The aliases that the project's file declares, each sound: each parses,
/// names itself through none, and nests at most 255 levels.
#[derive(Debug, Clone)]
pub struct Aliases {
    declared: BTreeMap<String, Alias>,
    nearest: Nearest,
}

#[derive(Debug, Clone)]
struct Alias {
    /// The key that declares it, such as `stable(x)`.
    form: String,
    parameter: Option<String>,
    body: Query,
}

impl Default for Aliases {
    /// No alias.
    fn default() -> Self {
        Self {
            declared: BTreeMap::new(),
            nearest: |_, _| None,
        }
    }
}

/// Why an expansion stops.
enum Stop {
    /// The aliases, in the order the expansion entered them, that lead back to
    /// the first.
    Loop(Vec<String>),
    /// The expansion nests deeper than [`DEPTH`] levels.
    Deep,
    /// A call or a bare word that the aliases refuse, and the alias whose body
    /// holds it, if any.
    Refused(Refused, Option<String>),
}

impl Aliases {
    /// The aliases that `declared` declares, each as its key and the text of
    /// its query. `nearest` names the alias nearest to a call that names none.
    ///
    /// # Errors
    ///
    /// The words of each alias that the project's file cannot hold, sorted by
    /// key.
    pub fn new<'t>(
        declared: impl IntoIterator<Item = (&'t str, &'t str)>,
        nearest: Nearest,
    ) -> Result<Self, Vec<String>> {
        let mut entries: Vec<(&str, &str)> = declared.into_iter().collect();
        entries.sort_unstable();
        let mut errors = Vec::new();
        let mut aliases = Self {
            declared: BTreeMap::new(),
            nearest,
        };
        for (key, text) in entries {
            let Some((name, parameter)) = declaration(key) else {
                errors.push(format!(
                    "{key} under {TABLE} is no name of an alias; \
                     a name holds letters, digits, _ and -, as fast or stable(x)"
                ));
                continue;
            };
            if TERMS.contains(&name) {
                errors.push(format!(
                    "{key} under {TABLE} is a term of a query; give the alias another name"
                ));
                continue;
            }
            if let Some(other) = aliases.declared.get(name) {
                errors.push(format!(
                    "{} and {key} under {TABLE} share a name; keep one",
                    other.form
                ));
                continue;
            }
            match parse(text) {
                Ok(body) => {
                    aliases.declared.insert(
                        name.to_owned(),
                        Alias {
                            form: key.to_owned(),
                            parameter: parameter.map(str::to_owned),
                            body,
                        },
                    );
                }
                Err(refusal) => errors.push(format!("{key} under {TABLE} is no query: {refusal}")),
            }
        }
        for (name, alias) in &aliases.declared {
            let argument = alias.parameter.as_ref().map(|it| Query::Id(it.clone()));
            let placeholder = argument.as_ref().map(|argument| (argument, 0));
            let Err(stop) = Walk::new(&aliases).enter(name, placeholder, 1) else {
                continue;
            };
            match stop {
                Stop::Loop(names) if names[0] == *name => {
                    let line = format!("{} under {TABLE} names itself", alias.form);
                    errors.push(match names.get(1..) {
                        Some(others) if !others.is_empty() => {
                            format!("{line} through {}", listed(others))
                        }
                        _ => line,
                    });
                }
                Stop::Deep => errors.push(format!(
                    "{} under {TABLE} nests deeper than {DEPTH} levels",
                    alias.form
                )),
                Stop::Refused(refusal, Some(owner)) if owner == *name => {
                    errors.push(format!("{} under {TABLE}: {refusal}", alias.form));
                }
                // The line of another alias names the fault.
                Stop::Loop(_) | Stop::Refused(..) => {}
            }
        }
        errors.sort();
        if errors.is_empty() {
            Ok(aliases)
        } else {
            Err(errors)
        }
    }

    /// The tree of `tree` with each alias expanded, so that it holds no call.
    ///
    /// # Errors
    ///
    /// A call names no alias, hands an alias the wrong count of arguments, or
    /// the expansion nests deeper than 255 levels.
    pub fn expand(&self, tree: &Query) -> Result<Query, Refused> {
        let mut walk = Walk::new(self);
        walk.node(tree, 0, false, None).map_err(|stop| match stop {
            Stop::Refused(refusal, _) => refusal,
            // `new` refuses each loop, so a loop here could only nest without end.
            Stop::Loop(_) | Stop::Deep => Refused {
                column: 0,
                why: Why::Through(walk.outermost),
            },
        })
    }

    /// The key that declares each alias, sorted by name.
    #[must_use]
    pub fn forms(&self) -> Vec<&str> {
        self.declared
            .values()
            .map(|alias| alias.form.as_str())
            .collect()
    }

    /// The name of each alias, sorted.
    #[must_use]
    pub fn names(&self) -> Vec<&str> {
        self.declared.keys().map(String::as_str).collect()
    }

    /// The key that declares the alias `name`, if any.
    #[must_use]
    pub fn form(&self, name: &str) -> Option<&str> {
        self.declared.get(name).map(|alias| alias.form.as_str())
    }
}

/// The name and the parameter, if any, that `key` declares: `fast` or
/// `stable(x)`.
fn declaration(key: &str) -> Option<(&str, Option<&str>)> {
    let is_name = |text: &str| {
        !text.is_empty()
            && text
                .chars()
                .all(|c| c.is_alphanumeric() || matches!(c, '_' | '-'))
    };
    let (name, parameter) = match key.strip_suffix(')') {
        Some(call) => {
            let (name, parameter) = call.split_once('(')?;
            (name, Some(parameter))
        }
        None => (key, None),
    };
    (is_name(name) && parameter.is_none_or(is_name)).then_some((name, parameter))
}

/// The words that `items` hold, joined by `, ` and a last ` and `.
fn listed(items: &[String]) -> String {
    match items {
        [] => String::new(),
        [one] => one.clone(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}

/// The parameter of the alias whose body the walk is in, its argument, already
/// expanded, and the levels that the argument opened as it expanded.
type Bound<'t> = Option<(&'t str, &'t Query, usize)>;

/// One expansion, with the aliases it is inside, the last alias it entered
/// from outside every alias, and the deepest level it reached.
struct Walk<'a> {
    aliases: &'a Aliases,
    inside: Vec<String>,
    outermost: String,
    deepest: usize,
}

impl<'a> Walk<'a> {
    const fn new(aliases: &'a Aliases) -> Self {
        Self {
            aliases,
            inside: Vec::new(),
            outermost: String::new(),
            deepest: 0,
        }
    }

    /// Notes that the expansion reached `level`.
    ///
    /// # Errors
    ///
    /// `level` is deeper than [`DEPTH`].
    fn reach(&mut self, level: usize) -> Result<(), Stop> {
        if level > DEPTH {
            return Err(Stop::Deep);
        }
        self.deepest = self.deepest.max(level);
        Ok(())
    }

    /// The refusal `why` at `column`, from the body of the alias the walk is
    /// in.
    fn refused(&self, column: usize, why: Why) -> Stop {
        Stop::Refused(Refused { column, why }, self.inside.last().cloned())
    }

    /// `tree`, expanded, at `level`. `in_operator` says whether it sits under
    /// `!`, `&` or `|`, where an `&` or a `|` needs a group. `bound` is the
    /// parameter of the alias whose body holds `tree`.
    fn node(
        &mut self,
        tree: &Query,
        level: usize,
        in_operator: bool,
        bound: Bound<'_>,
    ) -> Result<Query, Stop> {
        match tree {
            Query::Id(word) => {
                if let Some((parameter, argument, levels)) = bound
                    && parameter == word
                {
                    let group = in_operator && matches!(argument, Query::And(_) | Query::Or(_));
                    self.reach(level + levels + usize::from(group))?;
                    return Ok(argument.clone());
                }
                match self.aliases.declared.get(word) {
                    None => Ok(tree.clone()),
                    Some(alias) if alias.parameter.is_some() => {
                        Err(self.refused(0, Why::OneArgument(word.clone(), alias.form.clone())))
                    }
                    Some(_) => self.enter(word, None, level + 1),
                }
            }
            Query::Name(_) | Query::Path(_) => Ok(tree.clone()),
            Query::Not(term) => {
                self.reach(level + 1)?;
                Ok(Query::Not(Box::new(self.node(
                    term,
                    level + 1,
                    true,
                    bound,
                )?)))
            }
            Query::And(terms) | Query::Or(terms) => {
                let level = level + usize::from(in_operator);
                self.reach(level)?;
                let terms = terms
                    .iter()
                    .map(|term| self.node(term, level, true, bound))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(if matches!(tree, Query::And(_)) {
                    Query::And(terms)
                } else {
                    Query::Or(terms)
                })
            }
            Query::Call {
                name,
                argument,
                column,
            } => match self.aliases.declared.get(name) {
                None => {
                    let names = self.aliases.names();
                    let nearest = (self.aliases.nearest)(name, &names)
                        .and_then(|nearest| self.aliases.form(nearest))
                        .map(str::to_owned);
                    Err(self.refused(*column, Why::Call(name.clone(), nearest)))
                }
                Some(alias) if alias.parameter.is_none() => {
                    Err(self.refused(*column, Why::NoArgument(name.clone())))
                }
                Some(_) => {
                    // The argument expands once, from level 0, so that each
                    // place of the parameter adds the levels it opened.
                    let outer = std::mem::take(&mut self.deepest);
                    let argument = self.node(argument, 0, false, bound)?;
                    let levels = std::mem::replace(&mut self.deepest, outer);
                    self.enter(name, Some((&argument, levels)), level + 1)
                }
            },
        }
    }

    /// The body of the alias `name`, expanded at `level`, with `argument` in
    /// place of its parameter, and the levels that the argument opened.
    fn enter(
        &mut self,
        name: &str,
        argument: Option<(&Query, usize)>,
        level: usize,
    ) -> Result<Query, Stop> {
        self.reach(level)?;
        if let Some(at) = self.inside.iter().position(|inside| inside == name) {
            return Err(Stop::Loop(self.inside[at..].to_vec()));
        }
        let Some(alias) = self.aliases.declared.get(name) else {
            return Ok(Query::Id(name.to_owned()));
        };
        if self.inside.is_empty() {
            name.clone_into(&mut self.outermost);
        }
        self.inside.push(name.to_owned());
        let bound = alias
            .parameter
            .as_deref()
            .zip(argument)
            .map(|(parameter, (argument, levels))| (parameter, argument, levels));
        let expanded = self.node(&alias.body, level, false, bound);
        self.inside.pop();
        expanded
    }
}
