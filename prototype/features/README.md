# The feature battery

Everything in this directory is runnable. `justfile` and `mise.toml` express
the same nine capabilities twice, so the matrix in the parent `README.md` is
measured rather than read off two documentation sites.

```console
$ just vars ; just cond ; just plat ; just outer ; just many a b c ; just sub hello
$ mise run vars ; mise run cond ; mise run plat ; mise run outer ; mise run many a b c
$ mise run rich --verbose --jobs 4 widget     # no just equivalent
$ mise run bandreport                          # a file task, a real .py-shaped file
```

`just py` needs mise activated to find a Python. `mise run py` does not. That
is the same finding as the parent file's §1 and §3, arriving through a feature
test rather than a hook.
