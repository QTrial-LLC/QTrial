#!/usr/bin/env bash
# Run sqlx migrations against the local OffLeash dev database.
#
# Thin wrapper so developers never have to remember the DATABASE_URL
# shape or where the migrations directory lives. Reads the same `.env`
# file docker-compose uses, so port overrides are honored automatically.
#
# Usage:
#   scripts/db-migrate.sh              # apply pending migrations
#   scripts/db-migrate.sh info         # show applied vs pending
#   scripts/db-migrate.sh revert       # revert the last applied migration
#   scripts/db-migrate.sh add <name>   # create a new reversible migration

set -u

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Share port overrides with docker-compose and scripts/smoke-compose.sh.
if [[ -f "$repo_root/.env" ]]; then
    set -a
    # shellcheck disable=SC1091
    source "$repo_root/.env"
    set +a
fi

postgres_port="${OFFLEASH_POSTGRES_PORT:-5432}"
database_url="${DATABASE_URL:-postgres://offleash:offleash@localhost:${postgres_port}/offleash}"

export DATABASE_URL="$database_url"

# Default action is `migrate run` so the bare command does the most
# common thing.
if [[ $# -eq 0 ]]; then
    exec sqlx migrate run --source "$repo_root/db/migrations"
fi

subcommand="$1"
shift

case "$subcommand" in
    run|info|revert)
        exec sqlx migrate "$subcommand" --source "$repo_root/db/migrations" "$@"
        ;;
    add)
        exec sqlx migrate add -r --source "$repo_root/db/migrations" "$@"
        ;;
    *)
        # Forward anything else directly to sqlx for power users.
        exec sqlx "$subcommand" "$@"
        ;;
esac
