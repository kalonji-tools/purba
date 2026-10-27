# Results of the prototype for #664

Measured 2026-10-27 on Linux 6.18.47 with CPython 3.12.14, on this branch: `main` at d369fee plus the prototype.
Each block is the output of its script. The run of `run.sh` drops the shell's own
`Aborted (core dumped)` and `Segmentation fault (core dumped)` notices.
Two earlier runs of `run.sh` gave the same shape. Each gave today's `vec` exit 3 once in 20, in the slow row once and in the fast row once.

## `mise exec -- bash proto-664/run.sh`

```

A. test 3 of 5 crashes the process: which lines reach the reader?

print     crash  to    exit  lines the reader got
vec       abort  file  134   (none)
vec       abort  pipe  134   (none)
vec       segv   file  139   (none)
vec       segv   pipe  139   (none)
buffered  abort  file  134   (none)
buffered  abort  pipe  134   (none)
buffered  segv   file  139   (none)
buffered  segv   pipe  139   (none)
flushed   abort  file  134   test_1,test_2
flushed   abort  pipe  134   test_1,test_2
flushed   segv   file  139   test_1,test_2
flushed   segv   pipe  139   test_1,test_2

B. purba test_<speed>.py | head -1, 20 runs each: slow = 200 tests of 10 ms, fast = 200 tests, big = 3000 tests (over 64 KiB of lines)

speed answer         tests that ran         exit codes         stderr
slow  vec (today)    200×20                0×20              (none)
slow  stop           2×20                  3×20              purba: cannot write to stdout: Broken pipe (os error 32)
slow  quiet          200×20                3×20              purba: cannot write to stdout: Broken pipe (os error 32)
slow  ignore         200×20                0×20              (none)
fast  vec (today)    200×20                0×19 3×1         (none)|purba: cannot write to stdout: Broken pipe (os error 32)
fast  stop           2×4 3×16             3×20              purba: cannot write to stdout: Broken pipe (os error 32)
fast  quiet          200×20                3×20              purba: cannot write to stdout: Broken pipe (os error 32)
fast  ignore         200×20                0×20              (none)
big   vec (today)    3000×20               3×20              purba: cannot write to stdout: Broken pipe (os error 32)
big   stop           2×13 3×7             3×20              purba: cannot write to stdout: Broken pipe (os error 32)
big   quiet          3000×20               3×20              purba: cannot write to stdout: Broken pipe (os error 32)
big   ignore         3000×20               0×20              (none)
```

## `mise exec -- bash proto-664/probes.sh`

```
shape           cargo      lint:wiring
callback        builds     passes
iterator        refused    passes
                  rustc: error: lifetime may not live long enough
trait+dep       builds     refuses: A part may use only a part that the parts diagram draws an arrow to, because the diagram is the only wiring. Draw the arrow, or remove the dependency.
trait+reexport  builds     passes
reporting-sink  builds     refuses: A part may use only a part that the parts diagram draws an arrow to, because the diagram is the only wiring. Draw the arrow, or remove the dependency.
```

The two refusals of `lint:wiring` name `command uses execution, in crates/command/Cargo.toml` and `dispatch uses reporting, in crates/dispatch/Cargo.toml`.
rustc refuses the iterator because the closure that `attach` takes returns a value that borrows `&mut Worker<'_>`.
