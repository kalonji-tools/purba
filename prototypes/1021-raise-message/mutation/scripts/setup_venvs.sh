#!/usr/bin/env bash
# Create one venv per project under mut/venvs/<p>, install pytest 9.2.0 + test deps + project (editable, from the mut/src copy).
set -u
M=/tmp/claude-1000/-home-snregales-Projects-kalonji-tools/e9de2c2b-a74e-4d6e-b5d0-498e964f8ad4/scratchpad/mut
PY=/tmp/claude-1000/-home-snregales-Projects-kalonji-tools/4b30dbd8-035c-4ed5-8f24-6293c7c0a7b9/scratchpad/v449/bin/python
export SETUPTOOLS_SCM_PRETEND_VERSION=0.0.0
export UV_CACHE_DIR=$M/uvcache
mkdir -p $M/venvs $M/runs/setup
declare -A DEPS=(
  [attrs]="hypothesis pympler cloudpickle"
  [click]=""
  [dateutil]="freezegun hypothesis"
  [loguru]="freezegun colorama"
  [pyflakes]=""
  [requests]="pytest-httpbin==2.1.0 httpbin~=0.10.0 trustme PySocks"
  [schedule]=""
  [six]=""
  [toolz]=""
)
one() {
  p=$1
  V=$M/venvs/$p
  {
    uv venv --python "$PY" "$V" &&
      uv pip install --python "$V/bin/python" pytest==9.2.0 ${DEPS[$p]} &&
      uv pip install --python "$V/bin/python" -e "$M/src/$p"
  } >"$M/runs/setup/$p.log" 2>&1
  echo "$p exit=$?"
}
for p in "${!DEPS[@]}"; do one "$p" & done
wait
