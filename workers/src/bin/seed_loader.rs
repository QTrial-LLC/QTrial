//! qtrial-seed-loader CLI binary.
//!
//! Thin wrapper around `qtrial_workers::seed_loader::run_all`. Reads
//! DATABASE_URL from the environment and an optional seed directory
//! from argv[1] (default: db/seed/akc/). Migrations must already be
//! applied before this runs.
//!
//! Exit codes:
//!   0   all files loaded successfully
//!   1   loader returned a structured error (see logs for details)
//!   2   environment or argument parse error

use qtrial_workers::seed_loader;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

#[tokio::main]
async fn main() -> ExitCode {
    qtrial_shared::tracing_init::init("qtrial-seed-loader");

    let database_url = match env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            tracing::error!(
                "DATABASE_URL environment variable is required; typical local value is \
                 postgres://qtrial:qtrial@localhost:5432/qtrial"
            );
            return ExitCode::from(2);
        }
    };

    let seed_dir = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("db/seed/akc"));

    tracing::info!(
        seed_dir = %seed_dir.display(),
        "starting qtrial-seed-loader"
    );

    let pool = match PgPoolOptions::new()
        .max_connections(4)
        .acquire_timeout(Duration::from_secs(10))
        .connect(&database_url)
        .await
    {
        Ok(pool) => pool,
        Err(err) => {
            tracing::error!(error = %err, "failed to connect to DATABASE_URL");
            return ExitCode::from(1);
        }
    };

    match seed_loader::run_all(&pool, &seed_dir).await {
        Ok(stats) => {
            for (table, s) in &stats {
                tracing::info!(
                    table = table,
                    rows_read = s.rows_read,
                    rows_inserted = s.rows_inserted,
                    rows_upserted = s.rows_upserted,
                    rows_skipped = s.rows_skipped,
                    "loader complete"
                );
            }
            // Also emit a human-readable summary table to stdout for
            // direct invocation. Tracing JSON logs cover the machine
            // readable case.
            println!();
            println!(
                "{:<30} {:>10} {:>10} {:>10} {:>10}",
                "table", "read", "inserted", "upserted", "skipped"
            );
            for (table, s) in &stats {
                println!(
                    "{:<30} {:>10} {:>10} {:>10} {:>10}",
                    table, s.rows_read, s.rows_inserted, s.rows_upserted, s.rows_skipped
                );
            }
            ExitCode::SUCCESS
        }
        Err(err) => {
            tracing::error!(error = %err, "seed loader failed");
            ExitCode::from(1)
        }
    }
}
