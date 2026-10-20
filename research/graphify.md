# How Graphify builds a graph of a Markdown corpus

This file answers [#601](https://github.com/kalonji-tools/purba/issues/601) for the prototype in [#602](https://github.com/kalonji-tools/purba/issues/602).
It is never merged.

## Answer

Graphify builds two different graphs from Markdown.
`graphify update` parses each `.md` file with no model: one node for each file and heading, one edge for each link.
Over a copy of the 68 records, that build took 0.6 seconds and no tokens.
Concepts and rationale come only from a model pass.
The CLI runs that pass in `graphify extract` when it finds a backend: an API key, or the local `claude` binary on the person's subscription.
The `/graphify` skill runs it as Claude Code subagents.
A build or a query wrote nothing outside the scratch directory, once `HOME`, `PATH` and three environment variables were set.
`graphify install` and `graphify hook install` write global or shared files.
`graphifyy` is Apache-2.0, and each of its 29 default dependencies has a permissive licence.

| item | value |
|---|---|
| package read and run | `graphifyy` 0.9.93, uploaded to PyPI 2026-10-20 16:30 UTC |
| sdist | [`graphifyy-0.9.93.tar.gz`](https://files.pythonhosted.org/packages/c5/81/29c097991b81f600ee3716fe303c4d943d6e814ad807137fefb0fc3f619c/graphifyy-0.9.93.tar.gz), sha256 `666c8e59f59d1ff832c7357fada26a9722b6b3084b713f39777da550f91b99dd` |
| repository | [Graphify-Labs/graphify](https://github.com/Graphify-Labs/graphify); `github.com/safishamsi/graphify` answers 301 to it |
| releases on PyPI | 261 |
| interpreter | CPython 3.13.15 from nixpkgs |

A path such as `graphify/cli.py:4041` is a file and line in that sdist.
Each claim carries one evidence class.

| class | meaning |
|---|---|
| source | read in the sdist, its README or its metadata |
| measured | run here, on a copy of `docs/decisions/` in a scratch directory |
| inferred | derived from source, not run |

## 1. Who extracts concepts from Markdown

| path | what it takes from a `.md` file | model | needs | evidence |
|---|---|---|---|---|
| `graphify update <dir>` | a page node, a node for each heading, `contains` edges, a `references` edge for each link | none | nothing | source: `graphify/watch.py:1828-1833`, `graphify/extractors/markdown.py:369-418`; measured |
| `graphify extract <dir>` | concepts, `rationale`, `references`, `cites` and `conceptually_related_to` edges, hyperedges | the first backend it finds | an environment variable from the table below, or `--backend claude-cli` | source: `graphify/cli.py:3644-3650`, `graphify/cli.py:4120-4185`, `graphify/llm.py:3398-3443` |
| `/graphify` skill | the same schema as `extract` | the host session's subagents, or Gemini when its key is set | Claude Code, the skill installed, `graphify` importable | source: `graphify/skill.md:163-172`, `graphify/skill.md:207-226` |
| `cluster-only`, `label` | community names only | the detected backend, else `claude -p` when `claude` is on `PATH` | nothing | source: `graphify/llm.py:3795-3824` |

| backend | what selects it | default model | extra to install |
|---|---|---|---|
| `claude` | `ANTHROPIC_API_KEY` | `claude-sonnet-4-6`, or `ANTHROPIC_MODEL` | `graphifyy[anthropic]` |
| `claude-cli` | `--backend claude-cli`; runs `claude -p --no-session-persistence` | the CLI's default, or `GRAPHIFY_CLAUDE_CLI_MODEL` | none |
| `gemini` | `GEMINI_API_KEY` or `GOOGLE_API_KEY` | `gemini-3-flash-preview` | `graphifyy[gemini]` |
| `openai` | `OPENAI_API_KEY` | `gpt-4.1-mini`, or `OPENAI_MODEL` | `graphifyy[openai]` |
| `kimi`, `deepseek`, `azure`, `bedrock`, `ollama` | `MOONSHOT_API_KEY`, `DEEPSEEK_API_KEY`, `AZURE_OPENAI_API_KEY`, `AWS_PROFILE`, `OLLAMA_BASE_URL` | per backend | per backend |

Source: `graphify/llm.py:125-245`, `graphify/llm.py:1812-1900`.

- Auto-detection never picks `claude-cli`. Only the labeller falls back to it, when `claude` is on `PATH` (source: `graphify/llm.py:3808-3824`).
- `GRAPHIFY_BACKEND=none` turns off every model call, the `claude -p` fallback too (source: `graphify/llm.py:3379-3387`, `graphify/llm.py:3795-3804`).
- With it set, `extract` skipped all 69 non-code files and built nothing (measured; source: `graphify/cli.py:4130-4146`).
- The model-free graph holds 68 page nodes, 340 heading nodes, 340 `contains` edges and 131 `references` edges. Every edge is `EXTRACTED`, and no node is a concept (measured).
- `update` leaves out `purba-ranks-an-honest-outcome-first.md`. It meets three "paper" patterns, and `update` scans only documents (measured; source: `graphify/detect.py:258-330`).
- `update` scans `.template.md` as a record. The 68 page nodes are 67 records and the template (measured).
- A model pass would still read the paper-classed record, because `extract` sends papers to the model (source: `graphify/cli.py:4036`).

## 2. What each step writes

| step | what it writes | scope | evidence |
|---|---|---|---|
| `pip install graphifyy` | the package and the `graphify` and `graphify-mcp` scripts in the venv; a wheel runs no code at install | the venv | measured (the wheel's `RECORD`) |
| `graphify install` | `~/.claude/skills/graphify/SKILL.md`, its `references/` and `.graphify_version`; a block in `~/.claude/CLAUDE.md`, or under `CLAUDE_CONFIG_DIR` | global | source: `graphify/install.py:608-613`, `graphify/install.py:917-968` |
| `graphify install --project` | the same skill under `./.claude/skills/graphify/`; a block in `./.claude/CLAUDE.md` | project | source: `graphify/install.py:955-968`, `README.md:206-215` |
| `graphify claude install` | a `## graphify` section in `./CLAUDE.md`; `PreToolUse` hooks in `./.claude/settings.json` | project | source: `graphify/install.py:2136-2183` |
| `graphify hook install` | `post-commit`, `post-checkout`, `post-merge` and `post-rewrite` in the hooks directory, appended to an existing hook; `merge.graphify.*` in git config; a line in `.gitattributes` | the git directory and a tracked file | source: `graphify/hooks.py:857-886`, `graphify/hooks.py:980-1025`, `graphify/hooks.py:1093-1131` |
| a hook run | appends to `~/.cache/graphify-rebuild.log` | global | source: `graphify/hooks.py:365` |
| `/graphify` | `graphify-out/` in the working directory: `graph.json`, `GRAPH_REPORT.md`, `graph.html`, `manifest.json`, `cost.json`, `cache/semantic/`, `.graphify_*` files | project | source: `graphify/skill.md:113-155`, `graphify/skill.md:629-700` |
| `/graphify`, step 1 | `uv tool install --upgrade graphifyy` or `pip install graphifyy`, when `graphify` does not import | global | source: `graphify/skill.md:65-111` |
| `graphify update <dir>` | `<dir>/graphify-out/`, or `$GRAPHIFY_OUT`: `graph.json`, `GRAPH_REPORT.md`, `graph.html`, `manifest.json`, `cache/ast/`, `.graphify_*` files; 1.4 MB here | the scan folder | measured |
| `graphify query`, `explain`, `god-nodes` | nothing | none | measured |
| `graphify-mcp`, the skill's `--mcp` | nothing; it serves stdio | none | measured; source: `graphify/skills/claude/references/exports.md:63-82` |
| each CLI run but `--version` and `--help` | a fresh copy of each user-scope graphify skill whose version stamp is older | global, and only where a skill exists | source: `graphify/__main__.py:265-336`, `graphify/__main__.py:839-848` |

- `GRAPHIFY_NO_AUTO_REFRESH=1` stops the skill refresh. No graphify skill exists under `~/.claude/skills/` on this machine (measured).
- `README.md:422` says each query is logged by default. `graphify/querylog.py:15-31` logs only when `GRAPHIFY_QUERY_LOG` or `GRAPHIFY_QUERY_LOG_ENABLE` is set. The source wins.
- `claude` is on `PATH` here, at `/etc/profiles/per-user/snregales/bin/claude`. So `cluster-only` and `label` would call `claude -p` without `GRAPHIFY_BACKEND=none` (measured; source as above).
- A linked worktree reads git's hooks from the common git directory. So `hook install` in any worktree changes the hooks of `main` too (inferred from git's layout; source: `graphify/hooks.py:396-401`).

## 3. Running with no global install

| way | command | left outside the scratch directory | evidence |
|---|---|---|---|
| scratch venv | `uv venv <scratch>/venv`, then `uv pip install --python <scratch>/venv/bin/python graphifyy==0.9.93`, with `UV_CACHE_DIR` in scratch | nothing | measured |
| `uvx` | `uvx --from graphifyy==0.9.93 graphify <command>`, with `UV_CACHE_DIR` in scratch | `~/.local/share/uv/tools/.lock` and `.gitignore` | measured |
| `uvx`, `UV_TOOL_DIR` in scratch | the same | nothing | measured |

Each run here used this environment, and nothing else.

```sh
env -i HOME=<scratch>/home PATH=<scratch>/venv/bin:/run/current-system/sw/bin \
  GRAPHIFY_BACKEND=none GRAPHIFY_NO_AUTO_REFRESH=1 GRAPHIFY_OUT=<absolute path> \
  <scratch>/venv/bin/graphify update <copy of docs/decisions>
```

- After each run, the fake `HOME` and the working directory were still empty (measured).
- With no `GRAPHIFY_OUT`, `update` writes into the scanned folder: `docs/decisions/graphify-out/` in a worktree (measured).
- On this host the numpy wheel needs `libstdc++.so.6`, which `env -i` hides. `update` and `query` ran, and `extract` stopped with exit 1 on that import (measured).

## 4. Licences

Each distribution that `graphifyy` installs has a permissive licence.
Two grammar packages have inconsistent metadata, and neither is restrictive.

| distribution | version | licence | evidence |
|---|---|---|---|
| `graphifyy` | 0.9.93 | Apache-2.0; `NOTICE` keeps MIT terms for code contributed before the relicensing | source: `pyproject.toml:10-11`, `NOTICE`, `LICENSE`, `LICENSE-MIT` |
| `networkx` | 3.7 | BSD-3-Clause | measured |
| `numpy` | 2.5.4 | BSD-3-Clause AND 0BSD AND MIT AND Zlib AND CC0-1.0 | measured |
| `RapidFuzz` | 3.14.6 | MIT | measured |
| `tree-sitter` | 0.25.2 | MIT | measured |
| 23 `tree-sitter-*` grammars | various | MIT | measured |
| `tree-sitter-elixir` | 0.3.5 | the `License` field and the file say Apache-2.0; the classifier says MIT | measured |
| `tree-sitter-groovy` | 0.1.2 | MIT in metadata; the wheel ships no licence file | measured |
| `graphifyy[mcp]`, 28 more | various | MIT, BSD-3-Clause, Apache-2.0, MIT-0, PSF-2.0, and Apache-2.0 OR BSD-3-Clause | measured |

"Measured" here means read from each installed `METADATA` and licence file.
The `[anthropic]` extra was not installed, so its tree is not listed.

## 5. Time and tokens of a build

| build | wall clock | tokens | evidence |
|---|---:|---|---|
| `update`, 68 records, first run | 1.16 s | 0 | measured |
| `update`, 68 records, 3 more runs | 0.59 s each | 0 | measured |
| `update`, nothing changed | 0.44 s | 0 | measured |
| `update`, one record changed, 3 runs | 0.66 s each | 0 | measured |
| `extract`, Claude API, 68 records | not run | about 95,500 input in 2 calls; at most 16,384 output per call | inferred |
| `extract`, one record changed | not run | 1 call: 2,329 input for the median record, 5,554 for the largest | inferred |
| `/graphify`, 68 records | not run | 4 subagents of 22, 22, 22 and 2 records; the skill guesses 45 s for each parallel batch | inferred |
| `/graphify --update`, one record changed | not run | 1 subagent | inferred |

How the inferred rows were derived:

- A default install has no `tiktoken`. So a file costs (bytes + 160) / 4 tokens, capped at 20,000 bytes (source: `graphify/llm.py:31-41`, `graphify/llm.py:2228-2283`).
- Files pack greedily into chunks of 60,000 tokens: 59,648 and 33,786 here. Each call adds the 1,012-token system prompt (source: `graphify/llm.py:2285-2343`, `graphify/llm.py:502-542`).
- The cache keys a Markdown file on its body below the frontmatter. So one changed record is one call (source: `graphify/cache.py:511-528`).
- The skill chunks at most 22 files and 6,000 lines (source: `graphify/skill.md:223`, `graphify/skill.md:266`).
- Its own price table puts the Claude API at USD 3 in and 15 out per million tokens. That makes a full build about USD 0.29 in and at most 0.49 out (source: `graphify/llm.py:133`).
- A subagent's tokens depend on its turns, so section 5 gives no token figure for the skill.

One record changed moved the community count from 39 to 45, in each of 3 runs (measured).
So the community list in `GRAPH_REPORT.md` changes widely after a small edit.

## 6. How an agent queries with no skill

| way in | needs | gives | evidence |
|---|---|---|---|
| `graphify query "<question>" --graph <abs>/graph.json` | the scratch venv | a breadth-first subgraph as text, capped at 2,000 tokens unless `--budget`; exit 0, 3 when ambiguous, 4 when no node matches; 0.18 s | measured; source: `graphify/__main__.py:917-925` |
| `graphify explain`, `path`, `god-nodes`, `affected` | the same | one node's edges, a path, the hubs, a reverse traversal; 0.16 s for `explain` | measured for `explain` and `god-nodes` |
| `GRAPH_REPORT.md` | a file read | 14,466 bytes, 2,133 words for the records | measured |
| `graph.json` | a file read or `jq` | 323,637 bytes of node-link JSON: `nodes`, `links`, `hyperedges` | measured |
| `graphify-mcp <graph.json>` | `graphifyy[mcp]`, and an MCP registration in the agent host | 10 tools: `query_graph`, `get_node`, `get_neighbors`, `get_community`, `god_nodes`, `graph_stats`, `shortest_path`, `list_prs`, `get_pr_impact`, `triage_prs` | measured |

- `triage_prs` picks a backend, and falls back to `claude -p` when `claude` is on `PATH` (source: `graphify/prs.py:620-630`).
- `explain` on one record printed this, cut to four of its 11 edges (measured):

```text
Node: a-gate-owns-the-mechanical.md
  Degree:    11
  --> a-standard-a-gate-cannot-decide-does-not-become-a-gate.md [references] [EXTRACTED] a-gate-owns-the-mechanical.md:L44
  <-- a-gate-refuses-each-invariant-a-tool-can-see.md [references] [EXTRACTED] a-gate-refuses-each-invariant-a-tool-can-see.md:L7
  --> A gate owns the mechanical standard [contains] [EXTRACTED] a-gate-owns-the-mechanical.md:L1
  --> purba-is-always-strict.md [references] [EXTRACTED] a-gate-owns-the-mechanical.md:L12
```

## What this means for the prototype

- Choose the graph before the run. The free one is a link map of 67 records and the template. A concept graph needs a model pass whose tokens count against the saving.
- A concept graph needs no skill: `extract --backend claude-cli`, or `ANTHROPIC_API_KEY` with the `[anthropic]` extra. The person picks one. Whether `claude -p --no-session-persistence` writes under `~/.claude` is not settled here.
- Run each build in the environment of section 3, pin `graphifyy==0.9.93`, and point `GRAPHIFY_OUT` outside `docs/decisions/`.
- The graph arm gets in through the absolute path of the scratch `graphify` and of `GRAPH_REPORT.md`. Neither installs anything. MCP adds 28 packages and host configuration, and no capability the CLI lacks.
- With `update`, the graph cannot point at `purba-ranks-an-honest-outcome-first.md`, and it points at `.template.md`. A pull request whose breach cites that record tests the gap.
