# What purba's process holds

Written by `purba_process.py`.

## `state`

What the module recorded:

```text
{
 "pytest_in_sys_modules": false,
 "_pytest_in_sys_modules": false,
 "modules": 61,
 "sys_path_0": "v/bin",
 "meta_path": [
  "BuiltinImporter",
  "FrozenImporter",
  "PathFinder"
 ],
 "package": null,
 "spec": "None",
 "monitoring_tools": [
  null,
  null,
  null,
  null,
  null,
  null
 ],
 "import_function_type": "builtin_function_or_method"
}
```

purba's output, exit 0:

```text
test_state.py::test_one passed
1 case: 1 passed
```

## `sibling`

What the module recorded:

```text
None
```

purba's output, exit 4:

```text
test_sibling.py project error: ModuleNotFoundError: No module named 'helper_next_to_me'
test_sibling.py::test_one blocked: test_sibling.py did not load
1 case: 1 blocked; 1 project error
```

## `preloaded`

What the module recorded:

```text
{
 "pytest_in_sys_modules": true,
 "_pytest_in_sys_modules": true,
 "modules": 292,
 "sys_path_0": "v/bin",
 "meta_path": [
  "BuiltinImporter",
  "FrozenImporter",
  "PathFinder"
 ],
 "package": null,
 "spec": "None",
 "monitoring_tools": [
  null,
  null,
  null,
  null,
  null,
  null
 ],
 "import_function_type": "builtin_function_or_method"
}
```

purba's output, exit 0:

```text
test_preloaded.py::test_one passed
1 case: 1 passed
```

## `per_test`

What the module recorded:

```text
{
 "pytest_in_sys_modules": false,
 "monitoring_tools": [
  null,
  null,
  null,
  "purba",
  null,
  null
 ]
}
```

purba's output, exit 0:

```text
test_per_test.py::test_imports passed
test_per_test.py::test_after passed
2 cases: 2 passed
```

## `relative`

What the module recorded:

```text
ImportError: attempted relative import with no known parent package
```

purba's output, exit 0:

```text
test_relative.py::test_one passed
1 case: 1 passed
```

## `imports`

What the module recorded:

```text
None
```

purba's output, exit 0:

```text
test_imports.py::test_one passed
1 case: 1 passed
```
