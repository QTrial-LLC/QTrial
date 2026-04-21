#!/usr/bin/env bash
# Verify the local QTrial development stack is not just up, but healthy.
#
# "Running" and "accepting requests" are different standards. This script
# exercises each service at the protocol layer: Postgres accepts auth,
# Keycloak serves its master realm OIDC discovery document, NATS reports
# healthy on its monitoring port, Valkey responds to PING. Any failure
# exits non-zero with a pointer at the failing service.
#
# Usage:
#   docker compose up -d
#   scripts/smoke-compose.sh

set -u

# Docker Compose auto-loads `.env`; bash does not. Source it ourselves so
# the smoke check reads the same port overrides Compose used to start
# the stack.
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
if [[ -f "$repo_root/.env" ]]; then
    set -a
    # shellcheck disable=SC1091
    source "$repo_root/.env"
    set +a
fi

# Port overrides use the same names as docker-compose.yml so a single
# `.env` file configures both the stack and the smoke check.
readonly POSTGRES_PORT="${QTRIAL_POSTGRES_PORT:-5432}"
readonly KEYCLOAK_PORT="${QTRIAL_KEYCLOAK_PORT:-8080}"
readonly NATS_HTTP_PORT="${QTRIAL_NATS_HTTP_PORT:-8222}"
readonly VALKEY_PORT="${QTRIAL_VALKEY_PORT:-6379}"

readonly DEFAULT_TIMEOUT_SECONDS=120
readonly KEYCLOAK_TIMEOUT_SECONDS=240

pass=0
fail=0

log_ok() { printf "  [ok] %s\n" "$1"; pass=$((pass + 1)); }
log_fail() { printf "  [fail] %s\n" "$1" 1>&2; fail=$((fail + 1)); }

# retry_until runs a shell snippet every 2 seconds until it exits 0 or the
# deadline is hit. Used instead of fixed sleeps so fast services return
# quickly and slow ones (Keycloak on first boot) still get a chance.
retry_until() {
    local timeout="$1"
    local description="$2"
    shift 2
    local deadline=$(( $(date +%s) + timeout ))
    while true; do
        if "$@" >/dev/null 2>&1; then
            log_ok "$description"
            return 0
        fi
        if [[ $(date +%s) -ge $deadline ]]; then
            log_fail "$description (timed out after ${timeout}s)"
            return 1
        fi
        sleep 2
    done
}

check_postgres() {
    retry_until "$DEFAULT_TIMEOUT_SECONDS" \
        "postgres accepts connections on :${POSTGRES_PORT}" \
        docker compose exec -T postgres pg_isready -U postgres -d postgres
}

# Keycloak's OIDC discovery document is the most honest readiness signal
# we can hit without extra configuration: it is served only after the DB
# schema is initialized and the master realm is wired up.
check_keycloak() {
    retry_until "$KEYCLOAK_TIMEOUT_SECONDS" \
        "keycloak master realm reachable on :${KEYCLOAK_PORT}" \
        curl -fsS "http://localhost:${KEYCLOAK_PORT}/realms/master/.well-known/openid-configuration"
}

check_nats() {
    retry_until "$DEFAULT_TIMEOUT_SECONDS" \
        "nats monitoring endpoint healthy on :${NATS_HTTP_PORT}" \
        curl -fsS "http://localhost:${NATS_HTTP_PORT}/healthz"
}

valkey_ping() {
    local response
    response="$(docker compose exec -T valkey valkey-cli ping 2>/dev/null)" || return 1
    [[ "$response" == "PONG" ]]
}

check_valkey() {
    retry_until "$DEFAULT_TIMEOUT_SECONDS" \
        "valkey responds to PING on :${VALKEY_PORT}" \
        valkey_ping
}

printf "smoke-compose: checking QTrial local stack...\n"

check_postgres || true
check_keycloak || true
check_nats || true
check_valkey || true

printf "\n%d passed, %d failed.\n" "$pass" "$fail"

if [[ "$fail" -gt 0 ]]; then
    exit 1
fi
