# A workflow names each action by its commit

## Context and Problem Statement

A `uses:` line can name an action by a tag, such as `jdx/mise-action@v5`.
The owner of an action can move a tag to other code after a workflow names it.
GitHub's [Secure use reference](https://docs.github.com/en/actions/reference/security/secure-use) says:

> Pinning an action to a full-length commit SHA is currently the only way to use an action as an immutable release.

These jobs run an action while they hold a credential:

| job | credential | action |
|---|---|---|
| `bump.yml` `bump` | `contents: write`, then the App key | `jdx/mise-action`, which sets the `PATH` that later steps run `git` and `gh` from |
| `publish.yml` `wheel` | `contents: read`, and it builds the file that `upload` sends to PyPI | `jdx/mise-action` |
| `publish.yml` `upload` | `id-token: write` | `pypa/gh-action-pypi-publish` |
| `records.yml` `decision-records` | `pull-requests: write` | `actions/checkout` |
| `release.yml` `release` | `contents: write` and `actions: write`, then the App key | `jdx/mise-action` |
| `sign.yml` `sign` | `contents: write` and `checks: write` | `actions/checkout` |

## Considered Options

- **Name each action by a tag.** Rejected. Whoever controls the repository of the action can move the tag. The next run then executes that code with the credential of the job.
- **Name each action by its commit, with no comment.** Rejected. A reader and a reviewer of a Dependabot pull request cannot see which version a commit is.
- **Name each action by its commit, with its version in a comment.** Chosen. The commit cannot move, and the comment tells the reader the version.

## Decision Outcome

**Reach:** `.github/workflows/*.yml`

**Each `uses:` names an action by its full commit, and a comment names the version tag on that commit.**

```yaml
- uses: jdx/mise-action@2d8d4cafcbd33be2ea37d2b6f5ad595363d1f1ca # v5.1.1
```

The comment holds the most specific version tag on the commit.
The comment ends the line.
A new action takes the commit that its tag names on the day a person adds it.
[Dependabot proposes each newer action](dependabot-proposes-each-newer-action.md) moves the commit and the comment together.

**Downside:**

- **Dependabot raises no alert for an action named by its commit.** [GitHub](https://docs.github.com/en/actions/reference/security/secure-use) says: "Dependabot only creates alerts for vulnerable actions that use semantic versioning and will not create alerts for actions pinned to SHA values." A fixed action arrives only with the next weekly version update.
- **The comment can be false.** Nothing checks that the comment names the version of its commit. A hand edit can move one and not the other.
- **A person who adds an action resolves its commit by hand.** `git ls-remote --tags` prints it. Where a tag has a peeled `^{}` line, that line holds the commit.

## Confirmation

| property | check |
|---|---|
| each `uses:` names a commit and a version | `grep -nE 'uses: [^ ]+@' .github/workflows/*.yml \| grep -vE '@[0-9a-f]{40} # v[0-9]+(\.[0-9]+)*$'` prints nothing. The gate is not wired: [No gate refuses a workflow that names an action by a tag](https://github.com/kalonji-tools/purba/issues/456) |
| Dependabot moves the comment with the commit | read from `updated_comment` in [`version_commenter.rb`](https://raw.githubusercontent.com/dependabot/dependabot-core/3b68008e805baffb205e066abc61522083cb3f8b/github_actions/lib/dependabot/github_actions/file_updater/workflow_updater/version_commenter.rb), not measured. It rewrites a comment that ends in the version of the old commit. The first Dependabot pull request after this record shows it |
| the comment names the version of its commit | nothing checks it |
