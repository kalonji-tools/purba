# The pain of a crate, calculated from its levels, and the relief it needs to join.
#
#   the decision:  docs/decisions/a-crate-joins-when-its-relief-exceeds-its-pain.md
#
#   include "crate-pain";  with jq -L scripts

def others: "upkeep", "maintainers", "upstream-review", "tests", "unsafe", "api-churn",
  "owner-in-purba";
# The levels the formula calculates pain from.
def levels: "blast-area", others;

# Pain is a fourteenth of this sum: half the sum of blast area and the mean of the others.
def fourteenths($entry): 7 * $entry["blast-area"] + ([$entry[others]] | add);
def hundredths($sum): ($sum * 100 / 14 | round)
  | "\(. / 100 | floor).\(. % 100 | tostring | if length < 2 then "0" + . else . end)";

def joins($relief; $sum): 14 * $relief > $sum;

# The lowest relief that lets a crate join, or "never".
def needs($sum): [range(1; 4) | select(joins(.; $sum))] | first // "never";
