//! Where a name that a test reads comes from.

use std::collections::{HashMap, HashSet};

use ruff_python_ast::Stmt;

use super::{blocks, bound, origins};

/// Where a name that a test reads comes from.
#[derive(Debug, PartialEq, Eq)]
pub enum Origin<'l> {
    /// An import in the test's own body, and where it comes from.
    Own(&'l str),
    /// A name that the test's body binds, other than by an import.
    Local,
    /// An import of the module, and where it comes from.
    Imported(&'l str),
    /// A name that the module binds, other than by an import.
    Module,
    /// A name that nothing binds.
    Builtin,
}

/// The names that one test can read: the imports in its body, the names its
/// body binds, the imports of its module, and the names its module binds. A
/// body's import is also one of its locals, and a module's import is also one
/// of its names.
pub struct Lookup<'a, 'b> {
    own: HashMap<&'a str, String>,
    locals: &'b HashSet<&'a str>,
    imported: &'b HashMap<&'a str, String>,
    bound: &'b HashSet<&'a str>,
}

impl<'a, 'b> Lookup<'a, 'b> {
    /// The names that the test whose body is `body` can read.
    pub fn new(
        body: &'a [Stmt],
        locals: &'b HashSet<&'a str>,
        imported: &'b HashMap<&'a str, String>,
        bound: &'b HashSet<&'a str>,
    ) -> Self {
        Self {
            own: own_imports(body),
            locals,
            imported,
            bound,
        }
    }

    /// Where `name` comes from.
    ///
    /// A name that an import in the test's body binds comes from that import.
    /// Else, a name that the test's body binds is a local.
    /// Else, a name that an import of the module binds comes from that import.
    /// Else, a name that the module binds is the module's own.
    /// Else, the name is a builtin.
    pub fn origin(&self, name: &str) -> Origin<'_> {
        self.own.get(name).map_or_else(
            || {
                if self.locals.contains(name) {
                    Origin::Local
                } else if let Some(from) = self.imported.get(name) {
                    Origin::Imported(from)
                } else if self.bound.contains(name) {
                    Origin::Module
                } else {
                    Origin::Builtin
                }
            },
            |from| Origin::Own(from),
        )
    }
}

/// Where each name that an `import` in `body` binds comes from, at any depth.
fn own_imports(body: &[Stmt]) -> HashMap<&str, String> {
    let mut own = HashMap::new();
    let mut pending: Vec<&Stmt> = body.iter().collect();
    while let Some(statement) = pending.pop() {
        for (name, origin) in origins(statement) {
            own.insert(bound(name), origin);
        }
        pending.extend(blocks(statement).into_iter().flatten());
        if let Stmt::FunctionDef(function) = statement {
            pending.extend(&function.body);
        } else if let Stmt::ClassDef(class) = statement {
            pending.extend(&class.body);
        }
    }
    own
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use super::{Lookup, Origin};

    /// Checks that `name` comes from `expected`. The test's body imports `own`
    /// and binds `locals`, and its module imports `imported` and binds `bound`.
    /// Each pair names an import and where it comes from.
    fn row(
        own: &[(&str, &str)],
        locals: &[&str],
        imported: &[(&str, &str)],
        bound: &[&str],
        name: &str,
        expected: &Origin<'_>,
    ) {
        let pairs = |pairs: &[(&str, &str)]| -> HashMap<String, String> {
            pairs
                .iter()
                .map(|(name, from)| ((*name).to_owned(), (*from).to_owned()))
                .collect()
        };
        let (own, imported) = (pairs(own), pairs(imported));
        let locals: HashSet<&str> = locals.iter().copied().collect();
        let imported: HashMap<&str, String> = imported
            .iter()
            .map(|(name, from)| (name.as_str(), from.clone()))
            .collect();
        let bound: HashSet<&str> = bound.iter().copied().collect();
        let lookup = Lookup {
            own: own
                .iter()
                .map(|(name, from)| (name.as_str(), from.clone()))
                .collect(),
            locals: &locals,
            imported: &imported,
            bound: &bound,
        };

        assert_eq!(&lookup.origin(name), expected, "where {name} comes from");
    }

    #[test]
    fn each_rule_of_origin_decides_where_a_name_comes_from() {
        // A test that imports `mock` reads its own import, though that import
        // also binds a local, and the module imports a `mock` too.
        // fails when: the check of the body's locals moves above the check of
        // the body's imports.
        row(
            &[("mock", "unittest.mock")],
            &["mock"],
            &[("mock", "mock")],
            &["mock"],
            "mock",
            &Origin::Own("unittest.mock"),
        );

        // A test that binds `lending` itself hides the module's import of it.
        // fails when: the check of the module's imports moves above the check
        // of the body's locals.
        row(
            &[],
            &["lending"],
            &[("lending", "lending")],
            &["lending"],
            "lending",
            &Origin::Local,
        );

        // A module that imports `patch` from `unittest.mock` also binds `patch`.
        // fails when: the check of the module's names moves above the check of
        // the module's imports.
        row(
            &[],
            &[],
            &[("patch", "unittest.mock.patch")],
            &["patch"],
            "patch",
            &Origin::Imported("unittest.mock.patch"),
        );

        // A module that defines `setattr` binds it, so the test calls the
        // module's own `setattr`, not the builtin.
        // fails when: the check of the module's names goes, so the name falls
        // through to the builtin.
        row(&[], &[], &[], &["setattr"], "setattr", &Origin::Module);

        // A name that nothing binds is a builtin.
        // fails when: the last `else` gives `Origin::Module`.
        row(&[], &[], &[], &[], "setattr", &Origin::Builtin);
    }
}
