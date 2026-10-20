You run step 3 of a post pass on the purba repository. This is a read-only replay for a measurement. Do not edit any file, do not commit, do not run any `gh` command that writes, and post nothing anywhere.

- Worktree: `<scratch>/wt/594`, detached at head `1f9d948`. Run every command there, with `git -C` or an absolute path.
- Base: `bfbf718`. The change is `git diff bfbf718 HEAD` and `git log bfbf718..HEAD` in that worktree.
- Step 3's text is in `<scratch>/wt/594/docs/agents/post-pass.md`, under "The steps". Follow step 3 as written. Steps 1, 2 and 4 are not yours.
- Step 2's output, `mise run list:records` at this head, is already in `<scratch>/lists/594.md`. Do not run it again.
- A Graphify graph of `docs/decisions/` at this head is in `<scratch>/graphs/594/concept/`. A model built it: it holds a node for each record, concept and rationale, and edges between them. You may read `GRAPH_REPORT.md` there, read `graph.json` with `jq`, or query it with `<scratch>/bin/graphify query "<question>" --graph <scratch>/graphs/594/concept/graph.json`, `<scratch>/bin/graphify explain <node> --graph …` or `<scratch>/bin/graphify path <a> <b> --graph …`. Use it where it helps your judgement. You still read what step 3 names.

Your final message is the report, and nothing else:

1. Each breach: the file and line in the change, the record and line it breaks, and one sentence on why. Write "None." when there is none.
2. The third list: the number of lines you judged, and each line you judged stale.
