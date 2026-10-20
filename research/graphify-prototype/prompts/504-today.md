You run step 3 of a post pass on the purba repository. This is a read-only replay for a measurement. Do not edit any file, do not commit, do not run any `gh` command that writes, and post nothing anywhere.

- Worktree: `<scratch>/wt/504`, detached at head `7beb77b`. Run every command there, with `git -C` or an absolute path.
- Base: `fbec038`. The change is `git diff fbec038 HEAD` and `git log fbec038..HEAD` in that worktree.
- Step 3's text is in `<scratch>/wt/504/docs/agents/post-pass.md`, under "The steps". Follow step 3 as written. Steps 1, 2 and 4 are not yours.
- Step 2's output, `mise run list:records` at this head, is already in `<scratch>/lists/504.md`. Do not run it again.

Your final message is the report, and nothing else:

1. Each breach: the file and line in the change, the record and line it breaks, and one sentence on why. Write "None." when there is none.
2. The third list: the number of lines you judged, and each line you judged stale.
