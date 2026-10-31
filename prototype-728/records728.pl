#!/usr/bin/env perl
# PROTOTYPE of #728's record edits, on main at 53387a2. Run from the tree root:
#   perl records728.pl
# Each edit names its line on 53387a2, and dies when that line is not as expected.
use strict;
use warnings;

my $i728 = 'https://github.com/kalonji-tools/purba/issues/728';
my $i729 = 'https://github.com/kalonji-tools/purba/issues/729';
my $i726 = 'https://github.com/kalonji-tools/purba/issues/726';
my $i714 = 'https://github.com/kalonji-tools/purba/issues/714#issuecomment-6521025709';
my $l729 = "[a panic inside a test prints that test's internal error, and tests/kinds.bats checks each kind]($i729)";
my $l726 = "[the count names each column only above 0, with project and internal errors apart]($i726)";

sub edit {
    my ($file, $edits) = @_;
    open my $in, '<', $file or die "$file: $!";
    my @lines = <$in>;
    close $in;
    # Highest line first, so each number still names its line on 53387a2.
    for my $edit (sort { $b->[0] <=> $a->[0] } @$edits) {
        my ($n, $start, $after) = @$edit;
        index($lines[$n - 1], $start) == 0 or die "$file:$n does not start with: $start\n";
        splice @lines, $n - 1, 1, map { "$_\n" } @$after;
    }
    open my $out, '>', $file or die "$file: $!";
    print $out @lines;
    close $out;
}

my $d = 'docs/decisions';

edit("$d/a-run-exits-0-only-when-it-checked-something-and-every-test-passed.md", [
    [81, '- **purba does not exit 125 yet.**', []],
    [98, '`mise run test:wheel` checks that the installed `purba` exits 1', [
        '`mise run test:wheel` checks that the installed `purba` exits 1 on `test_a.py`, 3 on a usage error, and 125 on a panic in the `fault` wheel.',
    ]],
    [102, '[A panic that stops the run exits 125', [
        "[A panic inside a test prints that test's internal error, and tests/kinds.bats checks each kind]($i729) builds 125 for a run that a panic inside a test does not stop.",
    ]],
    [103, 'Each adds its check.', ['It adds its check.']],
]);

edit("$d/a-line-names-its-kind-in-the-glossarys-words.md", [
    [30, '- **purba keeps Rust\'s own panic hook.** Chosen.', [
        '- **purba keeps Rust\'s own panic hook.** Chosen. It writes the location and the way to a backtrace, and purba writes none of its text.',
        "- **The line names the Python by its version alone, as [the spec of this record]($i714) drew it.** Rejected. A free-threaded build reads as the build with a GIL.",
        '- **The Python carries its ABI flags.** Chosen. A report names the build that met the error.',
    ]],
    [50, '- **A run that met an internal error ends with two lines on stderr, once.**', [
        '- **A run that met an internal error ends with two lines on stderr, once.** The first names a bug in purba, purba\'s version, the Python, the OS and the architecture. The Python carries its ABI flags, so a free-threaded 3.14.0 reads `3.14.0t`. The second links the defect template. They follow the count, or the line that stopped the run.',
    ]],
    [62, '- **Of the lines this record adds, purba prints only those', [
        '- **purba prints no line of an internal error inside a test yet, nor the count this record decides.**',
    ]],
    [71, '`mise run test:wheel` checks the line of a project error', [
        '`mise run test:wheel` checks the line of a project error and the line of an invocation error on the installed `purba`.',
        'On the `fault` wheel, it checks the line of an internal error that stops the run, and the two lines that follow it.',
    ]],
    [73, 'Each slice that builds the rest', [
        "Each slice that builds the rest of what this record decides adds its check: $l726 and $l729.",
    ]],
]);

edit("$d/a-panic-inside-a-test-gives-that-test-an-internal-error.md", [
    [22, '- **One error carries a project error or an internal error.** Chosen.', [
        '- **One error carries a project error or an internal error.** Chosen. What dispatch hands keeps its shape.',
        '- **A part of its own holds the wrap and its record.** Rejected. `lint:wiring` and `lint:order` refuse it until the landscape gains the part, two arrows and a place in the order of design.',
        '- **Execution holds the wrap and its record.** Chosen. Dispatch reads the record through an arrow that the parts diagram already draws.',
        "- **A panic outside the source of a part names the part that catches it.** Chosen. Rust's hook prints the place of the panic above the line.",
    ]],
    [43, '- **The error that execution hands is a project error or an internal error.**', [
        '- **The error that execution hands is a project error or an internal error.** An internal error carries the part that found it, and its message. Both are data.',
        '- **Execution holds the wrap of Rust\'s panic hook and its record, and `_main` installs the wrap.**',
        '- **A panic in the source of a part names that part.** Any other panic names the part that catches it.',
    ]],
    [51, '- **stderr holds PyO3\'s banner', [
        '- **stderr holds PyO3\'s banner and a Python traceback beside Rust\'s hook.** A panic that crosses a test\'s call prints both.',
        "- **Until $l729 lands, a panic outside the source of a part names `command`.** Only `_main` catches a panic.",
    ]],
    [55, 'No command checks this record yet.', [
        '`mise run test:rust` checks that the wrap records the part and the message of a panic, and calls the hook it replaces.',
        "`mise run test:wheel` checks that Rust's hook still prints on the `fault` wheel.",
    ]],
    [56, '[A panic that stops the run exits 125', [
        "[A panic inside a test prints that test's internal error, and tests/kinds.bats checks each kind]($i729) builds the rest.",
    ]],
    [57, 'Both use the switch', [
        'It uses the switch that [a test causes an internal error through a switch that only a test build carries](a-test-causes-an-internal-error-through-a-switch-that-only-a-test-build-carries.md#decision-outcome) decides.',
    ]],
]);

edit("$d/a-test-causes-an-internal-error-through-a-switch-that-only-a-test-build-carries.md", [
    [41, 'It checks that the `fault` wheel panics under `PURBA_FAULT=assert`', [
        'It checks that the `fault` wheel panics under `PURBA_FAULT=assert` and under `PURBA_FAULT=command`, and that it passes without the variable.',
        'It checks that the wheel without `fault` panics under neither.',
    ]],
]);

edit("$d/a-case-of-each-kind-checks-that-no-failure-prints-as-another-kind.md", [
    [35, '- **`tests/kinds.bats` matches only the last two lines', [
        '- **`tests/kinds.bats` matches only the last two lines of the panic case\'s stderr.** It builds them from purba\'s version, the Python version and its ABI flags, the OS and the architecture of the run. Rust\'s hook and the Python traceback above them change with each edit and each Python version.',
    ]],
]);
