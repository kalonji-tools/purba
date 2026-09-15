#!/usr/bin/env bash
# PROTOTYPE probe. Records what a git hook can actually see.
# Every hook in the frozen prototype is `language = "system"`, so prek installs
# nothing and the hook sees whatever the invoking environment put on PATH.
log="${HOOKPROBE_LOG:-/tmp/hookprobe.log}"
{
  printf '=== %s ===\n' "${HOOKPROBE_ARM:-unlabelled}"
  for tool in prek cargo rustc rustfmt maturin just; do
    printf '  %-9s %s\n' "$tool" "$(command -v "$tool" 2>/dev/null || echo ABSENT)"
  done
  printf '  %-9s %s\n' "rustc-ver" "$(rustc --version 2>&1 | head -1 || echo ABSENT)"
  printf '  %-9s %s\n' "RUSTUP_TC" "${RUSTUP_TOOLCHAIN:-unset}"
} >> "$log"
