# PROTOTYPE: the experiment shape (#963). Never merge.

Throwaway. It shows how a tester would write and read tests in the shape agreed in
the grilling of "Which practices of testing qualify for a strict check?":
a case table, inputs built fresh per case, a reaction of ONE act, and assays that each hand purba a reading.

Run it from this directory: `python run.py` (any Python 3.12). `OUTPUT.txt` holds one run.

| file | what it shows |
|---|---|
| `ex1_slugify.py` | a pure function: the easy case, and a failure report |
| `ex2_money.py` | an expected raise and an expected warning in the table; an unexpected warning fails |
| `ex3_sign_up.py` | stand-ins as inputs from builders; assays that read an input after the act |
| `ex4_cart.py` | a chain of three steps; a failed step blocks the next |
| `ex5_refused.py` | what the allow list refuses before the run; a departure with a reason |
| `ex6_friction.py` | where the shape hurts: a rule, a branch in a builder, the cost of sealing by copy |
| `experiment.py` | the mini runner (not how purba would build it) |
