| | loguru | attrs | click | requests | dateutil | pyflakes | toolz | six | schedule | all |
|---|---|---|---|---|---|---|---|---|---|---|
| test functions | 720 | 667 | 619 | 347 | 1107 | 564 | 163 | 70 | 81 | 4338 |
| tests calling a helper of another test module (b) | 151 | 44 | 0 | 34 | 41 | 507 | 0 | 0 | 0 | 777 |
| – distinct b helpers they call | 11 | 3 | 0 | 5 | 4 | 1 | 0 | 0 | 0 | 24 |
| – of them, branching (own body, or a test helper it calls) | 3 | 2 | 0 | 5 | 2 | 1 | 0 | 0 | 0 | 13 |
| – of them, driven by contextlib or pytest (free in any scope) | 4 | 0 | 0 | 1 | 1 | 0 | 0 | 0 | 0 | 6 |
| – of them, asserting | 1 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 2 |
| ESCAPE under module scope: tests whose act calls a branching b helper | 12 | 4 | 0 | 28 | 1 | 0 | 0 | 0 | 0 | 45 |
| – counting driven helpers too | 17 | 4 | 0 | 34 | 20 | 0 | 0 | 0 | 0 | 75 |
| tests calling an asserting b helper before their own check | 17 | 0 | 0 | 0 | 0 | 19 | 0 | 0 | 0 | 36 |
| tests whose branching b helper sits outside the folder scope too | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 | 1 |
| tests calling a branching helper of their own module (refused today) | 9 | 8 | 2 | 1 | 61 | 1 | 10 | 1 | 68 | 161 |
| tests taking a branching conftest fixture (not a call; not counted) | 0 | 0 | 0 | 0 | 12 | 0 | 0 | 0 | 0 | 12 |
| FOLDER SCOPE: modules of code under test in a test file's folder | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 1 | 2 |
| – their functions | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 69 | 57 | 126 |
| – of them, branching | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 20 | 34 | 54 |
| tests whose act calls that code, resolved statically | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 18 | 58 | 76 |
| – calling a branching function of it | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 7 | 4 | 11 |
| calls in acts that the analyser could not resolve | 0.286 | 0.16 | 0.195 | 0.31 | 0.06 | 0.028 | 0.147 | 0.162 | 0.347 | 0.188 |
