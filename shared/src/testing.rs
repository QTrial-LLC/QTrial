//! Reusable testcontainers fixture for integration tests.
//!
//! Every downstream test file that needs a real migrated Postgres
//! calls [`pool`] to obtain a fresh connection pool against a shared
//! container. The container itself is initialized lazily on first
//! call and kept alive for the process lifetime; subsequent tests
//! reuse the running container.
//!
//! Why per-test pools rather than a static shared pool: `#[tokio::test]`
//! creates a fresh runtime for each test, and a `sqlx::PgPool` pins
//! its background connection-management tasks to whichever runtime
//! created it. A static pool therefore becomes unusable as soon as
//! the first test's runtime shuts down. Keeping only the container's
//! host and port in the static sidesteps the cross-runtime issue
//! entirely; opening a pool per test is cheap against a local
//! container (sub-millisecond handshakes) and every test gets a
//! cleanly-scoped resource.
//!
//! Isolation between tests is transaction-per-test with rollback on
//! drop. Call [`begin_as_tenant`] (or `PgPool::begin` directly) inside
//! a `#[tokio::test]`, do work, let the transaction drop at scope
//! end. No cleanup code needed; no test can see another test's data.
//!
//! The fixture intentionally uses the same `db/docker-init/01-create-
//! databases.sql` bootstrap script as dev Compose so every test
//! exercises the same role and database topology the real app runs
//! against. If dev bootstrap drifts from tests, dev is what breaks
//! first — that's the point.
//!
//! Errors during fixture init (Docker not running, migration syntax
//! error, port collision) panic with a descriptive message rather than
//! returning a `Result`, because a failed fixture means no test can
//! run anyway and the test harness surfaces the panic cleanly.
//!
//! Container logs are suppressed by default. Set
//! `QTRIAL_TEST_CONTAINER_LOGS=1` in the environment to see them when
//! debugging a boot failure.

use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::sync::OnceLock;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::testcontainers::ImageExt;
use testcontainers_modules::testcontainers::runners::AsyncRunner;
use tokio::sync::OnceCell;
use uuid::Uuid;

use crate::tenancy;

/// The same bootstrap script dev Compose feeds into its Postgres
/// container's `/docker-entrypoint-initdb.d/` directory. Compiled in so
/// the fixture cannot drift from dev without CI noticing.
const BOOTSTRAP_SQL: &str =
    include_str!("../../db/docker-init/01-create-databases.sql");

/// Postgres image tag. When the dev `postgres:16` in `docker-compose.yml`
/// changes, update this tag in lockstep so tests exercise the same
/// server version the app runs against.
const POSTGRES_IMAGE_TAG: &str = "16-alpine";

/// Cached host and port of the running test Postgres container. The
/// container itself is leaked on first init so the Drop that would
/// require a tokio runtime context never runs; testcontainers' ryuk
/// sidecar cleans up the container when the test process exits.
static CONTAINER_ENDPOINT: OnceCell<(String, u16)> = OnceCell::const_new();

/// Open a fresh connection pool against the shared test database.
/// Returns an owned `PgPool` that the caller is responsible for
/// dropping when the test completes. The first call through this
/// function boots the container and applies every migration; later
/// calls reuse the running container.
pub async fn pool() -> PgPool {
    let (host, port) = CONTAINER_ENDPOINT
        .get_or_init(|| async {
            init_container().await.unwrap_or_else(|err| {
                panic!(
                    "testcontainers fixture failed to initialize: {err}.\n\
                     Most likely causes, in rough order:\n\
                       * Docker is not running (try: docker info)\n\
                       * Port conflict while launching Postgres (try: docker ps)\n\
                       * A migration failed to apply; rerun with\n\
                         QTRIAL_TEST_CONTAINER_LOGS=1 cargo test ... -- --nocapture"
                )
            })
        })
        .await;

    let url = format!("postgres://qtrial:qtrial@{host}:{port}/qtrial");
    PgPoolOptions::new()
        .max_connections(4)
        .connect(&url)
        .await
        .expect("open per-test pool against shared container")
}

/// Open a transaction against the test database with tenant context
/// applied. The returned `Transaction<'static, Postgres>` carries its
/// own connection from a single-use pool; dropping the transaction
/// rolls it back, which is the rollback-per-test discipline integration
/// tests rely on.
///
/// Under the hood we leak the per-test pool so the transaction carries
/// the `'static` lifetime that sqlx needs. The leak is bounded: each
/// test creates one pool, process exit frees everything (including the
/// testcontainer, via ryuk).
pub async fn begin_as_tenant(
    user_id: Uuid,
    club_id: Uuid,
) -> Result<sqlx::Transaction<'static, sqlx::Postgres>, sqlx::Error> {
    let pool = pool().await;
    let pool_ref: &'static PgPool = Box::leak(Box::new(pool));
    tenancy::begin_as_tenant(pool_ref, user_id, club_id).await
}

async fn init_container() -> Result<(String, u16), Box<dyn std::error::Error + Send + Sync>> {
    // Order matters: image-config methods (with_db_name, with_user,
    // with_password) are inherent on Postgres. `with_tag` is the
    // ImageExt trait method that converts the Image into a
    // ContainerRequest, so it has to come last.
    let image = Postgres::default()
        .with_db_name("postgres")
        .with_user("postgres")
        .with_password("postgres")
        .with_tag(POSTGRES_IMAGE_TAG);

    let container = image
        .start()
        .await
        .map_err(|err| format!("container start failed: {err}"))?;

    let host = container
        .get_host()
        .await
        .map_err(|err| format!("container host lookup failed: {err}"))?
        .to_string();
    let port = container
        .get_host_port_ipv4(5432)
        .await
        .map_err(|err| format!("container port lookup failed: {err}"))?;

    // Run the dev bootstrap SQL as the superuser. Each statement
    // executes as its own command because `CREATE DATABASE` cannot
    // run inside a transaction and sqlx's multi-statement path wraps
    // in one. Our bootstrap script is intentionally simple (no dollar
    // quotes, no DO blocks), so splitting on ';' after stripping line
    // comments is safe.
    let bootstrap_url = format!("postgres://postgres:postgres@{host}:{port}/postgres");
    let bootstrap_pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&bootstrap_url)
        .await
        .map_err(|err| {
            format!("bootstrap connection as postgres superuser failed: {err}")
        })?;
    for statement in split_bootstrap_statements(BOOTSTRAP_SQL) {
        sqlx::query(&statement)
            .execute(&bootstrap_pool)
            .await
            .map_err(|err| format!("bootstrap statement {statement:?} failed: {err}"))?;
    }
    bootstrap_pool.close().await;

    // Connect as the owning `qtrial` user and apply every migration.
    // Pool is capped low; migrations only need one connection and
    // smaller pools make connection leaks easier to spot.
    let app_url = format!("postgres://qtrial:qtrial@{host}:{port}/qtrial");
    let migration_pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&app_url)
        .await
        .map_err(|err| format!("app connection as qtrial failed: {err}"))?;

    sqlx::migrate!("../db/migrations")
        .run(&migration_pool)
        .await
        .map_err(|err| format!("sqlx migrate run failed: {err}"))?;

    migration_pool.close().await;

    // Leak the container so Drop (which would try to spawn cleanup
    // tasks on a tokio runtime that may have already shut down) never
    // runs. testcontainers' ryuk sidecar kills the container when the
    // test process exits, so this is not an actual leak.
    Box::leak(Box::new(container));

    Ok((host, port))
}

/// Strip line comments and split a simple SQL script into individual
/// statements. Used only on the dev bootstrap script, which has no
/// dollar-quoted blocks, so plain comment stripping and semicolon
/// splitting is sufficient.
fn split_bootstrap_statements(sql: &str) -> Vec<String> {
    sql.lines()
        .map(|line| match line.find("--") {
            Some(idx) => &line[..idx],
            None => line,
        })
        .collect::<Vec<_>>()
        .join(" ")
        .split(';')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Tracker for test-binary-unique identifiers. Integration tests
/// share a database; names like "test club" collide if every test
/// inserts the same literal. Using [`unique_name`] keeps inserts
/// independent without requiring a full rollback-per-test regime
/// (though transactions still give us that for RLS-sensitive data).
static SEQ: OnceLock<std::sync::atomic::AtomicU64> = OnceLock::new();

/// Produce a name that is unique within this test run.
pub fn unique_name(prefix: &str) -> String {
    let counter = SEQ.get_or_init(|| std::sync::atomic::AtomicU64::new(0));
    let n = counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    format!("{prefix}-{n}")
}
