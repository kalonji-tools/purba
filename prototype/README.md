# Throwaway prototype: actor bindings in gitattributes syntax

**This branch is never merged.** It is captured as the primary source behind
[A location inherits its readers](https://github.com/kalonji-tools/purba/blob/main/docs/decisions/a-location-inherits-its-readers.md)
and [Build the reader-binding gate](https://github.com/kalonji-tools/purba/issues/85).

Open either file directly in a browser. No build, no server, no dependencies.

| file | tree it drives |
|---|---|
| `01-purba-19-files.html` | purba, 19 tracked files |
| `02-oxitest-1109-files.html` | oxitest, 1,109 tracked files, 288 directories, 8 levels deep |

## The question it answered

Does the established gitattributes language fit for binding a reader to a
location, written to a purba-owned filename so git never reads it?

**Yes.** Three findings came out of driving it that argument alone had not produced.

1. **Globs carry the model.** Four extension patterns reached 956 of oxitest's
   1,109 files without naming a directory. A directory-prefix format cannot do this.
2. **The roster was missing an actor.** Laying the whole agent-steering surface in
   one table showed 9 files and 993 lines with no reader on the roster. That is the
   `handler`.
3. **`contributor` and `user` are not actors.** Making them bindable quietly made
   the roster thirteen while record A said eleven. They came back as `[attr]` macros,
   which is the borrowed language's own feature.

It also demonstrated, rather than argued, that a binding at the project root takes
coverage to 100% and makes the gate vacuously green.

## What it is not

The matcher is JavaScript and implements a **subset** of gitattributes globbing:
`**`, `*`, `?` and literals. No character classes, no escapes, no `key=value`.
It validates the model, never the crate. The resolver core measures **62 lines**
with no dependencies, which is the number #85 should be costed against.
