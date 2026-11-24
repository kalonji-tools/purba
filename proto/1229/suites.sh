#!/usr/bin/env bash
# Prototype for #1229. This branch is never merged.
#
#   proto/1229/suites.sh <tally> <suites dir> <out dir> <strategy>...
#
# Runs the tally example of the branch once per suite and strategy, and writes
# <out dir>/<strategy>.tsv.gz and <out dir>/<strategy>.counts.tsv. Under
# `list-and-check`, each suite's list is proto/1229/lists/<suite>.toml.
# The tally parses the suites. It imports and runs nothing in them.
set -euo pipefail

tally=$1
suites=$2
out=$3
shift 3
here=$(cd "$(dirname "$0")" && pwd)
libdir=$(mise exec -- python -I -c 'import sysconfig; print(sysconfig.get_config_var("LIBDIR"))')
functions=$(mktemp)
trap 'rm -f "$functions"' EXIT
zcat "$here/ground/functions.tsv.gz" >"$functions"
mkdir -p "$out"

for strategy in "$@"; do
  for suite in loguru attrs click requests dateutil pyflakes toolz six schedule; do
    PURBA_PROTO_MARKERS=$strategy PURBA_PROTO_LIST=$here/lists/$suite.toml \
      LD_LIBRARY_PATH=$libdir "$tally" "$suite=$suites/$suite"
  done | gzip -9n >"$out/$strategy.tsv.gz"
  mise exec -- python -I "$here/summarize.py" "$out/$strategy.tsv.gz" \
    "$here/ground/files.tsv" "$functions" "$here/judged.tsv" >"$out/$strategy.counts.tsv"
  echo "$strategy: $out/$strategy.counts.tsv"
done
