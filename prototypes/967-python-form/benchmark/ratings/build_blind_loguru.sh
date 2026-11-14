#!/usr/bin/env bash
set -euo pipefail
E="$1"; B="$E/blind"
scrub() { sed -E -e 's/record_strict/experiments/g' -e 's/form_record_strict/form_experiment/g' -e 's/\(#96[0-9][^)]*\)//g' -e 's/PROTOTYPE \(#967\)\. ?//g' -e 's/Form RECORD_STRICT: the form agreed on 2026-11-14\. Never merge\./Form./' -e 's/ Never merge[.;]?//g' -e 's/from the answers of #963-#966/from earlier design answers/' -e 's/PROTOTYPE \. //' "$@"; }
build() {  # $1 rater, $2 letter for oxitest, $3 letter for the prototype
  local R="$B/rater$1"; mkdir -p "$R/ORIGINAL_pytest" "$R/PORT_$2" "$R/PORT_$3/experiments"
  cp "$E"/pytest_orig/{test_add_option_format.py,conftest.py,LICENSE.loguru,OUTPUT.txt,CHECKERS.txt,pyrightconfig.json} "$R/ORIGINAL_pytest/"
  printf 'Run from this folder:\n\n    %s/.venv/bin/pytest -v -p no:cacheprovider\n' "$E" > "$R/ORIGINAL_pytest/RUN.md"
  (cd "$E/oxitest" && tar --exclude=__pycache__ -cf - .) | (cd "$R/PORT_$2" && tar -xf -)
  local P="$R/PORT_$3"
  scrub "$E/proto/engine.py" > "$P/engine.py"
  scrub "$E/proto/form_record_strict.py" > "$P/form_experiments.py"
  scrub "$E/proto/run.py" | sed -E -e 's/^FORMS = .*/FORMS = ("experiments",)/' -e 's/^EXAMPLES = .*/EXAMPLES = ("ex_loguru_format",)/' > "$P/run.py"
  scrub "$E/proto/record_strict/ex_loguru_format.py" > "$P/experiments/ex_loguru_format.py"
  scrub "$E/proto/record_strict/loguru_lab.py" > "$P/experiments/loguru_lab.py"
  scrub "$E/proto/app.py" > "$P/app.py"; scrub "$E/proto/lab.py" > "$P/lab.py"
  scrub "$E/proto/LOGURU_NOTES.md" > "$P/NOTES.md"; scrub "$E/proto/LOGURU_OUTPUT.txt" > "$P/OUTPUT.txt"; scrub "$E/proto/LOGURU_CHECKERS.txt" > "$P/CHECKERS.txt"
  cp "$E/FORM_rules.md" "$P/FORM.md"
  printf '{"venvPath": "%s", "venv": ".venv", "extraPaths": ["%s"]}\n' "$E" "$P" > "$P/pyrightconfig.json"
  printf 'Run from this folder:\n\n    %s/.venv/bin/python run.py experiments\n\nRules of the form: FORM.md\n' "$E" > "$P/RUN.md"
}
build 1 K M; build 2 M K; build 3 K M
