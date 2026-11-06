# A workflow names each action by its commit

## Context and Problem Statement

A `uses:` line can name an action by a tag, such as `jdx/mise-action@v5`.
The owner of an action can move a tag to other code after a workflow names it.
GitHub's [Secure use reference](https://docs.github.com/en/actions/reference/security/secure-use) says:

> Pinning an action to a full-length commit SHA is currently the only way to use an action as an immutable release.

These jobs run an action while they hold a credential:

| job | credential | action |
|---|---|---|
| `bump.yml` `bump` | `contents: write` and `issues: write`, then the App key | `actions/checkout`, `actions/create-github-app-token`, and `jdx/mise-action`, which sets the `PATH` that later steps run `git` and `gh` from |
| `publish.yml` `report` | `issues: write` | `actions/checkout` |
| `publish.yml` `upload` | `id-token: write` | `actions/download-artifact`, `pypa/gh-action-pypi-publish` |
| `publish.yml` `wheel` | `contents: read`, and it builds the file that `upload` sends to PyPI | `actions/checkout`, `actions/upload-artifact`, `jdx/mise-action` |
| `records.yml` `decision-records` | `pull-requests: write` | `actions/checkout` |
| `release.yml` `release` | `contents: write`, `issues: write` and `actions: write`, then the App key | `actions/checkout`, `actions/create-github-app-token`, `jdx/mise-action` |
| `sign.yml` `sign` | `contents: write` and `checks: write` | `actions/checkout` |

## Considered Options

- **Name each action by a tag.** Rejected. Whoever controls the repository of the action can move the tag. The next run then executes that code with the credential of the job.
- **Name each action by its commit, with no comment.** Rejected. A reader and a reviewer of a Dependabot pull request cannot see which version a commit is.
- **Name each action by its commit, with its version in a comment.** Chosen. The commit cannot move, and the comment tells the reader the version.

## Decision Outcome

**Reach:** `.github/workflows/*.yml` `/.config/tasks.toml` `/prek.toml` `/.pinact.yaml` `/.pinact.yml` `.github/pinact.yaml` `.github/pinact.yml`

**Each `uses:` names an action by its full commit, and a comment names the version tag on that commit.**

```yaml
- uses: jdx/mise-action@2d8d4cafcbd33be2ea37d2b6f5ad595363d1f1ca # v5.1.1
```

The comment holds the most specific version tag on the commit.
A new action takes the commit that its tag names on the day a person adds it.
Dependabot moves the commit and the comment together, as [Dependabot proposes each newer action](dependabot-proposes-each-newer-action.md) says.

**`mise run fmt:pins` writes the pin, and a hook runs it at commit.**
`mise run lint:pins` refuses a tag and a comment that names another version.
`Quality` runs it, because CI cannot write.

**Downside:**

- **Dependabot raises no alert for an action named by its commit.** [GitHub](https://docs.github.com/en/actions/reference/security/secure-use) says: "Dependabot only creates alerts for vulnerable actions that use semantic versioning and will not create alerts for actions pinned to SHA values." A fixed action arrives only with the next weekly version update.
- **`lint:pins` reads the GitHub API.** It needs the network, like `lint:links`. Without a token it reads anonymously, and a rate limit fails it.
- **A commit that adds an action by a tag needs the network.** The hook resolves the tag through the GitHub API.

## Confirmation

Each row ran on copies of `.github/workflows/`, with the pinact that `.config/mise.lock` holds.

| tree | `fmt:pins` | `lint:pins` |
|---|---|---|
| every action by a tag | writes the pins the workflows hold now | not run |
| every action by its commit | changes nothing | exits 0 |
| one `actions/checkout@v7` | writes its pin | exits 1, and changes no file |
| one `# v7` on the commit of `v7.0.1` | writes `# v7.0.1` | exits 1 |
| one `# v7.0.0` on the commit of `v7.0.1` | not run | exits 3 |
| every action by its commit, with a token GitHub refuses | changes nothing, so it calls no API | exits 3 |

`pinact run --fix=false --no-api` reads no API.
It misses the false comment, so `lint:pins` does not use it.

Whether Dependabot moves the comment with the commit is read from `updated_comment` in [`version_commenter.rb`](https://raw.githubusercontent.com/dependabot/dependabot-core/3b68008e805baffb205e066abc61522083cb3f8b/github_actions/lib/dependabot/github_actions/file_updater/workflow_updater/version_commenter.rb), not measured.
`updated_comment` rewrites a comment that ends in the version of the old commit.
The first Dependabot pull request after this record shows it.
