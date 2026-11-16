import csv, sys, collections

rows = list(csv.reader(open(sys.argv[1]), delimiter="\t"))
hdr = rows[0][1:-1]
A = collections.defaultdict(lambda: [0] * len(hdr))
for r in rows[1:]:
    base = r[0].rsplit("|", 1)[0]
    for i, v in enumerate(r[1:-1]):
        A[base][i] += int(v)
keys = [
    "T_tests",
    "assert",
    "S_self_assert",
    "A1_rt2",
    "A1_rt2_strict",
    "A1_rt1",
    "A1x_rt_deep",
    "A1x_rt_one",
    "S1_rt2",
    "S1x_rt_deep",
    "S1x_rt_one",
    "A2_rel_eq",
    "A2_rel_eq_call_both_sides",
    "A2_rel_other_op",
    "S2_rel_eq",
    "A3_allany",
    "A4_in_loop",
    "A4_loop_is_last_top_stmt",
    "T_A4_in_loop",
    "T_given",
    "files_import_hypothesis",
    "T_tests_in_hypothesis_file",
    "S7_in_for",
    "T_no_assert_or_self_assert",
]
print("| row | " + " | ".join(hdr) + " | total |")
print("|---" * (len(hdr) + 2) + "|")
for k in keys:
    print(f"| {k} | " + " | ".join(map(str, A[k])) + f" | {sum(A[k])} |")
