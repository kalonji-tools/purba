# shellcheck shell=bash
# The keys that sign a gate's token in a test, and a key set like GitHub's.
#
#   the task:      mise run test:shell
#
#   make_gate_keys   the keys `gate`, `other` and `short`, and the key set that
#                    holds `gate` and `short`. `short` is of 1024 bits, and no
#                    set holds `other`. A file of tests calls it in `setup_file`
#   gate_b64url <file>
#                    the base64url of the bytes in <file>, with no padding
: "${BATS_FILE_TMPDIR:?set by bats}"

# `bats --jobs` runs the tests of one file at once, and a key of 2048 bits is
# slow to make. `setup_file` runs once, before any test of its file.
make_gate_keys() {
  local dir="${BATS_FILE_TMPDIR}/gate-keys" name bits gate short
  mkdir -p "${dir}"
  for name in gate:2048 other:2048 short:1024; do
    bits=${name#*:}
    name=${name%:*}
    openssl genpkey -algorithm RSA -pkeyopt "rsa_keygen_bits:${bits}" \
      -out "${dir}/${name}.pem" 2>/dev/null
  done
  gate=$(gate_modulus "${dir}/gate.pem")
  short=$(gate_modulus "${dir}/short.pem")
  jq -n --arg gate "${gate}" --arg short "${short}" '{keys: [
      {kid: "gate", kty: "RSA", alg: "RS256", use: "sig", e: "AQAB", n: $gate},
      {kid: "short", kty: "RSA", alg: "RS256", use: "sig", e: "AQAB", n: $short}]}' \
    >"${dir}/jwks.json"
}

# The modulus of a key, in base64url.
gate_modulus() {
  local hex escaped="" i
  hex=$(openssl rsa -in "$1" -noout -modulus)
  hex=${hex#Modulus=}
  for ((i = 0; i < ${#hex}; i += 2)); do escaped+="\\x${hex:i:2}"; done
  printf '%b' "${escaped}" >"$1.modulus"
  gate_b64url "$1.modulus"
}

gate_b64url() {
  local encoded
  encoded=$(base64 <"$1")
  encoded=${encoded//$'\n'/}
  encoded=${encoded//=/}
  encoded=${encoded//+/-}
  printf '%s' "${encoded//\//_}"
}
