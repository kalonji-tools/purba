# versions.jq: upkeep and API churn from a crates.io crate record.
# Arguments: $until (ISO time), $v (the version measured).
# Only versions that are not yanked, not a prerelease, and published before $until count.
# Prints "<upkeep> <churn>".
def core: split("+")[0];
def key:
  (.num | core | split(".") | map(tonumber)) as $p
  | if $p[0] >= 1 then "\($p[0])"
    elif $p[1] >= 1 then "0.\($p[1])"
    else "0.0.\($p[2])" end;
# The day $n months before $until, compared as text: 2026-05-31 less 3 months is 2026-02-31.
def before($n):
  (($until[0:4] | tonumber) * 12 + ($until[5:7] | tonumber) - 1 - $n) as $month
  | "\($month / 12 | floor)-\($month % 12 + 1 | tostring | if length < 2 then "0" + . else . end)"
    + $until[7:10];
before(3) as $m3 | before(12) as $m12 | before(24) as $y2
| [ .versions[]
  | select(.yanked | not)
  | select(.num | core | test("-") | not)
  | select(.created_at < $until) ] as $vs
| ($vs | max_by(.created_at).created_at[0:10]) as $d
| (if $d >= $m12 then 1 elif $d >= $y2 then 2 else 3 end) as $up
# the first release of each compatibility key is a breaking release
| ($vs | group_by(key) | map(min_by(.created_at) | {k: key, d: .created_at[0:10]})) as $brk
| ($v | core | split(".") | map(tonumber)) as $vp
| (if $vp[0] >= 1 then
     ($brk | map(select(.k | test("^[1-9]")))) as $maj
     | (if any($maj[]; .d >= $m3) then 3 elif any($maj[]; .d >= $m12) then 2 else 1 end)
   else
     (if any($brk[]; .d >= $m3) then 3 else 2 end)
   end) as $ch
| "\($up) \($ch)"
