#!/usr/bin/env bash
# Prueba local de la lógica de ci-gate (jq): failure/cancelled/skipped ≠ success.
# No crea check runs ni estados en GitHub.
set -euo pipefail

jq_check() {
  local json="$1"
  local expect="$2" # ok|fail
  set +e
  echo "$json" | jq -e '
    to_entries
    | map(select(.value.result != "success"))
    | if length == 0 then true else
        error("ci-gate: jobs no success: " + (map(.key + "=" + .value.result) | join(", ")))
      end
  ' >/dev/null 2>&1
  local st=$?
  set -e
  if [[ "$expect" == "ok" && "$st" -eq 0 ]]; then
    return 0
  fi
  if [[ "$expect" == "fail" && "$st" -ne 0 ]]; then
    return 0
  fi
  echo "error: caso expect=${expect} status=${st}" >&2
  return 1
}

ALL_OK='{"fmt":{"result":"success"},"desktop-e2e":{"result":"success"}}'
jq_check "$ALL_OK" ok

FAIL_ONE='{"fmt":{"result":"success"},"desktop-e2e":{"result":"failure"}}'
jq_check "$FAIL_ONE" fail

SKIP_ONE='{"fmt":{"result":"success"},"desktop-e2e":{"result":"skipped"}}'
jq_check "$SKIP_ONE" fail

CANCEL_ONE='{"fmt":{"result":"cancelled"},"desktop-e2e":{"result":"success"}}'
jq_check "$CANCEL_ONE" fail

echo "OK check-ci-gate-logic (success-only; failure/skipped/cancelled rechazados)"
