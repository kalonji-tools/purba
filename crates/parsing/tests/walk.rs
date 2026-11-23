use std::thread;

use purba_parsing::{Arms, Enter, Module, Node, walk};
use ruff_python_ast::{Alias, Comprehension, ExceptHandler, Expr, Pattern, Stmt};

const SMALL_STACK: usize = 256 * 1024;

fn parsed(text: &str) -> Module {
    purba_parsing::parse_source(text.to_owned()).expect("the module parses")
}

fn statements(module: &Module) -> impl Iterator<Item = Node<'_>> {
    module.syntax().body.iter().map(Node::Stmt)
}

/// Each name that the walk meets, and each `def`, `class`, `lambda` and
/// comprehension entered as `enter` says.
struct Names {
    enter: Enter,
    seen: Vec<String>,
}

impl Names {
    fn of<'a>(roots: impl IntoIterator<Item = Node<'a>>, enter: Enter) -> Vec<String> {
        let mut names = Self {
            enter,
            seen: Vec::new(),
        };
        walk(roots, &mut names);
        names.seen
    }
}

impl<'a> Arms<'a> for Names {
    fn stmt(&mut self, statement: &'a Stmt) -> Enter {
        match statement {
            Stmt::FunctionDef(_) | Stmt::ClassDef(_) => self.enter,
            _ => Enter::All,
        }
    }

    fn expr(&mut self, expr: &'a Expr) -> Enter {
        match expr {
            Expr::Name(name) => {
                self.seen.push(name.id.to_string());
                Enter::All
            }
            Expr::Lambda(_) => self.enter,
            _ => Enter::All,
        }
    }

    fn comprehension(&mut self, _comprehension: &'a Comprehension) -> Enter {
        self.enter
    }
}

#[test]
fn the_walk_meets_each_node_in_source_order() {
    let module = parsed(concat!(
        "a = b\n",
        "if c:\n",
        "    d(e)\n",
        "else:\n",
        "    for f in g:\n",
        "        h[i] = j\n",
        "k = f\"{l}\"\n",
        "try:\n",
        "    m\n",
        "except n as error:\n",
        "    o\n",
        "else:\n",
        "    p\n",
        "finally:\n",
        "    q\n",
        "with r as s:\n",
        "    t\n",
        "match u:\n",
        "    case [*_] if v:\n",
        "        w\n",
        "x = {y: z for aa in bb for cc in dd if ee}\n",
        "if ff:\n",
        "    gg\n",
        "elif hh:\n",
        "    ii\n",
    ));

    assert_eq!(
        Names::of(statements(&module), Enter::All),
        [
            "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q",
            "r", "s", "t", "u", "v", "w", "x", "y", "z", "aa", "bb", "cc", "dd", "ee", "ff", "gg",
            "hh", "ii",
        ],
        "each name, in the order the source writes it: in each block, a format string, each \
         part of a try, a with, a case and its guard, a comprehension of two parts, and the \
         test of an elif"
    );

    let Stmt::Assign(assign) = &module.syntax().body[0] else {
        panic!("the first statement is an assignment");
    };
    assert_eq!(
        Names::of([Node::Expr(&assign.value)], Enter::All),
        ["b"],
        "a walk from an expression meets only that expression"
    );
}

#[test]
fn the_walk_enters_a_def_a_class_a_lambda_and_a_comprehension_as_far_as_the_arm_says() {
    let module = parsed(concat!(
        "@deco\n",
        "def inner(a=default, *args: rest, b: note = kw, **kwargs: more) -> back:\n",
        "    body\n",
        "@class_deco\n",
        "class Box(base, metaclass=meta):\n",
        "    inside\n",
        "f = lambda c=dflt: lam_body\n",
        "items = [elt for target in source if cond]\n",
        "def typed[T: bound]():\n",
        "    typed_body\n",
        "def positional(a=pdefault, /):\n",
        "    positional_body\n",
        "class Typed[U: cbound]:\n",
        "    class_body\n",
    ));

    assert_eq!(
        Names::of(statements(&module), Enter::All),
        [
            "deco",
            "default",
            "rest",
            "note",
            "kw",
            "more",
            "back",
            "body",
            "class_deco",
            "base",
            "meta",
            "inside",
            "f",
            "dflt",
            "lam_body",
            "items",
            "elt",
            "target",
            "source",
            "cond",
            "bound",
            "typed_body",
            "pdefault",
            "positional_body",
            "cbound",
            "class_body",
        ],
        "All enters each node that a def, a class, a lambda and a comprehension hold"
    );
    assert_eq!(
        Names::of(statements(&module), Enter::Head),
        [
            "deco",
            "default",
            "rest",
            "note",
            "kw",
            "more",
            "back",
            "class_deco",
            "base",
            "meta",
            "f",
            "dflt",
            "items",
            "elt",
            "source",
            "cond",
            "pdefault",
        ],
        "Head enters what runs where each stands: decorators, defaults of each kind of \
         parameter, annotations and bases, and a comprehension's iterables and conditions; no \
         body, no target, and no bound of a type parameter of a def or a class, which CPython \
         reads later"
    );
    assert_eq!(
        Names::of(statements(&module), Enter::Nothing),
        ["f", "items", "elt"],
        "Nothing enters no node that a def, a class, a lambda or a comprehension holds"
    );
}

/// Each alias, handler and pattern that the walk meets, beside each name.
#[derive(Default)]
struct Events {
    seen: Vec<String>,
}

impl<'a> Arms<'a> for Events {
    fn expr(&mut self, expr: &'a Expr) -> Enter {
        if let Expr::Name(name) = expr {
            self.seen.push(format!("name {}", name.id));
        }
        Enter::All
    }

    fn pattern(&mut self, pattern: &'a Pattern) {
        let kind = match pattern {
            Pattern::MatchSequence(_) => "sequence",
            Pattern::MatchAs(_) => "as",
            Pattern::MatchStar(_) => "star",
            _ => "other",
        };
        self.seen.push(format!("pattern {kind}"));
    }

    fn except_handler(&mut self, _handler: &'a ExceptHandler) {
        self.seen.push("except".to_owned());
    }

    fn alias(&mut self, alias: &'a Alias) {
        self.seen.push(format!("alias {}", alias.name));
    }
}

#[test]
fn the_walk_gives_each_alias_handler_and_pattern_to_its_arm() {
    let module = parsed(concat!(
        "import os as system\n",
        "try:\n",
        "    pass\n",
        "except ValueError as error:\n",
        "    pass\n",
        "match point:\n",
        "    case [x, *rest]:\n",
        "        pass\n",
    ));
    let mut events = Events::default();

    walk(statements(&module), &mut events);

    assert_eq!(
        events.seen,
        [
            "alias os",
            "except",
            "name ValueError",
            "name point",
            "pattern sequence",
            "pattern as",
            "pattern star",
        ],
        "each alias, handler and pattern reaches its arm, before the nodes it holds"
    );
}

/// How many statements, expressions and patterns the walk meets.
#[derive(Default)]
struct Count {
    statements: usize,
    exprs: usize,
    patterns: usize,
}

impl<'a> Arms<'a> for Count {
    fn stmt(&mut self, _statement: &'a Stmt) -> Enter {
        self.statements += 1;
        Enter::All
    }

    fn expr(&mut self, _expr: &'a Expr) -> Enter {
        self.exprs += 1;
        Enter::All
    }

    fn pattern(&mut self, _pattern: &'a Pattern) {
        self.patterns += 1;
    }
}

#[test]
fn a_walk_over_nodes_that_nest_deeper_than_the_stack_returns() {
    const DEPTH: usize = 3_000;
    let mut statements_text = String::new();
    for level in 0..DEPTH {
        statements_text.push_str(&" ".repeat(level));
        statements_text.push_str("if c:\n");
    }
    statements_text.push_str(&" ".repeat(DEPTH));
    statements_text.push_str("pass\n");
    for (name, text, expected) in [
        ("statements", statements_text, [DEPTH + 1, DEPTH, 0]),
        (
            "power",
            format!("x = {}1\n", "a ** -".repeat(20_000)),
            [1, 60_002, 0],
        ),
        (
            "subscript",
            format!("x = y{}\n", "[0]".repeat(20_000)),
            [1, 40_002, 0],
        ),
    ] {
        let counted = thread::Builder::new()
            .name(name.to_owned())
            .stack_size(SMALL_STACK)
            .spawn(move || {
                let module = parsed(&text);
                let mut count = Count::default();
                walk(statements(&module), &mut count);
                [count.statements, count.exprs, count.patterns]
            })
            .expect("the thread starts")
            .join()
            .expect("the walk returns on a small stack");
        assert_eq!(
            counted, expected,
            "{name}: the walk meets each statement, expression and pattern once"
        );
    }
}
