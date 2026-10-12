# A script calculates the order of design, and only a changed input moves it

## Context and Problem Statement

[A design in depth waits for what it assumes](a-design-in-depth-waits-for-what-it-assumes.md#decision-outcome) states the rule that orders each design in depth, and the score of each risk.
It says that a merged design removes its own risk and order lines from the landscape.
It does not say which other lines a design can make untrue, or when the order changes.
A design can remove an arrow, change the parts a concern crosses, or change the probability of another risk.
Each of these changes the order.

| fact | source |
|---|---|
| each line of the landscape states purba's shape now | [the register of the landscape](../../LANDSCAPE.md#what-belongs-in-this-file) |
| the research recommended that the landscape be revised after each design in depth, and that the risks be ranked again against the walking skeleton | [What do established design methods say about drawing a landscape before a deep design?](https://github.com/kalonji-tools/purba/issues/462#issuecomment-6099871177) |
| the walking skeleton that the research described runs one trivial test, with no fixture, no async and one worker | [What do established design methods say about drawing a landscape before a deep design?](https://github.com/kalonji-tools/purba/issues/462#issuecomment-6099871177) |
| each input of the order is a line of the landscape | [the landscape](../../LANDSCAPE.md) |
| a script that needs no Rust library is a bash script | [a script that needs a Rust library that bash cannot call is a cargo script](a-script-that-needs-a-rust-library-is-a-cargo-script.md#decision-outcome) |

## Considered Options

- **A design rewrites only its own risk and order lines, and an issue fixes each other line.** Rejected. `main` states a false shape between the two merges.
- **A design rewrites each line it makes untrue.** Chosen. `main` states purba's shape after each merge.
- **The order stays as the map charted it.** Rejected. A wrong judgement then never leaves the order.
- **Each design calculates the order again, whatever it changed.** Rejected. A design that changes no input gets the same order.
- **The walking skeleton ends with a new score for each risk.** Rejected. It scores again the risks that it cannot see.
- **The order changes in the pull request that changes an input.** Chosen. The order moves with its evidence, and with nothing else.
- **The reviewer calculates the order by hand.** Rejected. Two reviewers can get two orders from the same landscape.
- **A gate alone.** Rejected. It finds the error, and the author still repairs it by hand.
- **A task that writes the order, and a gate that refuses a different one.** Chosen. One script gives one order, and the task repairs what the gate refuses.

## Decision Outcome

**Reach:** `/LANDSCAPE.md` `docs/decisions/a-design-in-depth-waits-for-what-it-assumes.md` `scripts/order-of-design.sh` `scripts/test/order-of-design.bats` `/.config/tasks.toml`

**A script calculates the order of design, and only a changed input moves it.**

- **A pull request rewrites each line of the landscape that it makes untrue.** A design in depth is one such pull request.
- **The inputs of the order are lines of the landscape.** They are the probability and impact of each risk, where it lands, the arrows between the parts, and the parts each concern crosses.
  They also include which parts and concerns link their design.
- **The pull request that changes an input runs `mise run fmt:order`.** That task rewrites the order section, and prints each step the rule takes.
- **That pull request also rewrites the row of each risk it changed.** The row is in the risk table of [a design in depth waits for what it assumes](a-design-in-depth-waits-for-what-it-assumes.md#decision-outcome).
  The rewritten row links the new evidence.
- **A design in depth that merges in its turn, and changes no other input, leaves the rest of the order as it was.** The rule already placed that design first, so only its order line leaves.
  A design that merges out of its turn can move the rest of the order.
- **The walking skeleton is no exception.** It moves the order only through an input that one of its pull requests changes.
- **The rule skips each part or concern whose line links its design.** No part or concern waits for a skipped one.
- **A concern waits for nothing.** No arrow starts at a concern, so a concern uses no part.

**Downside:**

- **Nothing checks that a line became untrue.** The reviewer judges which lines a design made untrue.
- **The order is only as good as its probabilities.** The script calculates exactly, and a wrong judgement still moves the order.
- **The script reads each link in the line of a part or concern.** A link to anything except its design marks that part or concern as designed.
- **The script reads the prose of the landscape.** Each change to how the landscape writes a risk, an arrow or a crossing also changes the script.

## Confirmation

`mise run lint:order` calculates the order again, and refuses an order section that differs.
It also refuses a risk with no row in the record.
It refuses a risk whose row holds another probability, impact, part or concern.
`lint` runs it, and `Quality` runs `lint` on each pull request.
`scripts/test/order-of-design.bats` runs it green and red.
The reviewer decides whether a line became untrue, because [a standard a gate cannot decide does not become a gate](a-standard-a-gate-cannot-decide-does-not-become-a-gate.md) applies.
