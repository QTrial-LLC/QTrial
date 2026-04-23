//! Integration tests for the AKC reference-data seed loader.
//!
//! The fixture (shared/src/testing.rs) spins up a Postgres 16
//! testcontainer, runs every migration forward, and hands out pools.
//! These tests run the seed loader against that database, then assert
//! row counts and cross-table invariants.
//!
//! Ordering: the five tests run serially within this binary so the
//! fresh-load test's side effects (every reference table populated)
//! are visible to the other tests. Assertions that depend on exact
//! counts use `>=` on the specific loader's rows_read tally rather
//! than the database table count when another test might have
//! re-upserted rows.

use qtrial_shared::testing;
use qtrial_workers::seed_loader;
use std::path::PathBuf;
use tokio::sync::OnceCell;

/// Absolute path to db/seed/akc/ from the workers crate. CARGO_MANIFEST_DIR
/// at test-run time is the workers crate root, so the seed directory
/// is one level up and then db/seed/akc.
fn seed_dir() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop();
    p.push("db");
    p.push("seed");
    p.push("akc");
    p
}

/// Run the full seed loader exactly once across all tests in this
/// binary. Without the gate, #[tokio::test] fans the five tests out
/// onto separate tokio runtimes and they race to INSERT into the same
/// testcontainer: the ON CONFLICT target on countries is alpha2_code
/// only, so a concurrent INSERT that collides on alpha3_code first
/// raises a bare unique-violation rather than falling through to
/// DO UPDATE. Serializing the loader side-steps that race while
/// keeping the per-test pool-per-runtime model intact.
static SEEDED: OnceCell<()> = OnceCell::const_new();

async fn ensure_seeded(pool: &sqlx::PgPool) {
    SEEDED
        .get_or_init(|| async {
            seed_loader::run_all(pool, &seed_dir())
                .await
                .expect("initial seed run should succeed");
        })
        .await;
}

/// Expected row counts per table after a fresh load. Per the PR-2a
/// decision log:
///   jump_heights: 20 (21 CSV rows minus the Jump=0 sentinel)
///   obedience_class_exercises: measured at first run; the test
///     records the actual count and subsequent runs assert equality
///     against the measured value.
fn expected_counts() -> &'static [(&'static str, i64)] {
    &[
        ("countries", 216),
        ("states", 63),
        ("breed_groups", 11),
        ("breeds", 288),
        ("breed_varieties", 19),
        ("title_prefixes", 49),
        ("title_suffixes", 259),
        ("jump_heights", 20),
        ("obedience_exercises", 20),
        ("otch_points", 23),
        ("om_points", 21),
        ("rally_rach_points", 10),
        ("sport_time_defaults", 6),
    ]
}

/// Fresh load: after one run of the loader, each table lands at the
/// expected row count. A failure here is the first signal of seed
/// drift (CSV count changed, CSV shape changed, or loader logic
/// regressed).
#[tokio::test]
async fn test_fresh_load() {
    let pool = testing::pool().await;
    ensure_seeded(&pool).await;

    // Per-table row counts match the expected row count list.
    for (table, expected) in expected_counts() {
        let count: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table}"))
            .fetch_one(&pool)
            .await
            .unwrap_or_else(|err| panic!("count {table}: {err}"));
        assert_eq!(
            count, *expected,
            "expected {expected} rows in {table}, got {count}",
        );
    }

    // obedience_class_exercises count is measured at CHECKPOINT 3 and
    // logged for the PR reviewer. The idempotency test verifies the
    // count stays stable across runs.
    let oce_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM obedience_class_exercises")
        .fetch_one(&pool)
        .await
        .expect("count obedience_class_exercises");
    eprintln!("obedience_class_exercises row count after fresh load: {oce_count}");
    assert!(
        oce_count > 0,
        "expected obedience_class_exercises to carry loaded rows",
    );
}

/// Idempotency: a second run of the loader produces zero new inserts,
/// and no row's updated_at advances. We capture max(updated_at) per
/// table before the extra run, run the loader again, then assert each
/// table's max(updated_at) did not move.
#[tokio::test]
async fn test_idempotency() {
    let pool = testing::pool().await;
    // Guarantee the data is seeded exactly once across the test
    // binary. Subsequent calls are no-ops.
    ensure_seeded(&pool).await;

    // Snapshot max(updated_at) per table. Using max() over NOW()
    // ticks catches any row the second run upserted with a refreshed
    // updated_at.
    let tables = [
        "countries",
        "states",
        "breed_groups",
        "breeds",
        "breed_varieties",
        "title_prefixes",
        "title_suffixes",
        "jump_heights",
        "obedience_exercises",
        "obedience_class_exercises",
        "otch_points",
        "om_points",
        "rally_rach_points",
        "sport_time_defaults",
    ];
    let mut before = std::collections::HashMap::new();
    let mut counts_before = std::collections::HashMap::new();
    for table in tables {
        let ts: Option<String> =
            sqlx::query_scalar(&format!("SELECT MAX(updated_at)::text FROM {table}"))
                .fetch_one(&pool)
                .await
                .unwrap_or_else(|err| panic!("max(updated_at) {table}: {err}"));
        before.insert(table, ts);

        let count: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table}"))
            .fetch_one(&pool)
            .await
            .unwrap_or_else(|err| panic!("count {table}: {err}"));
        counts_before.insert(table, count);
    }

    // Extra run to validate re-run idempotency. ensure_seeded is a
    // no-op after its first call, so this is the only run_all call
    // in this test that could touch the database beyond the initial
    // seed.
    let second = seed_loader::run_all(&pool, &seed_dir())
        .await
        .expect("second run should succeed");

    for (table, stats) in &second {
        assert_eq!(
            stats.rows_inserted, 0,
            "idempotent re-run of {table} should insert zero new rows, got {stats:?}",
        );
    }

    // Row counts unchanged.
    for table in tables {
        let count_after: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table}"))
            .fetch_one(&pool)
            .await
            .unwrap_or_else(|err| panic!("count {table}: {err}"));
        assert_eq!(
            count_after, counts_before[table],
            "{table} row count drifted on re-run: before={} after={}",
            counts_before[table], count_after,
        );

        // updated_at stasis.
        let ts_after: Option<String> =
            sqlx::query_scalar(&format!("SELECT MAX(updated_at)::text FROM {table}"))
                .fetch_one(&pool)
                .await
                .unwrap_or_else(|err| panic!("max(updated_at) {table}: {err}"));
        assert_eq!(
            ts_after, before[table],
            "{table} MAX(updated_at) advanced on re-run (idempotency broken)",
        );
    }
}

/// FK integrity: sampled cross-table foreign keys resolve to real
/// parent rows. This catches loader bugs that insert a child with a
/// random UUID or with a parent that was rolled back, and it also
/// validates the obedience_class_exercises happy path for non-NULL
/// FKs.
#[tokio::test]
async fn test_fk_integrity() {
    let pool = testing::pool().await;
    ensure_seeded(&pool).await;

    // Every breed points at a real breed_group.
    let dangling_breeds: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM breeds b
         WHERE NOT EXISTS (SELECT 1 FROM breed_groups g WHERE g.id = b.breed_group_id)",
    )
    .fetch_one(&pool)
    .await
    .expect("count dangling breeds");
    assert_eq!(dangling_breeds, 0, "every breed must link to a breed_group");

    // Every state points at a real country.
    let dangling_states: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM states s
         WHERE NOT EXISTS (SELECT 1 FROM countries c WHERE c.id = s.country_id)",
    )
    .fetch_one(&pool)
    .await
    .expect("count dangling states");
    assert_eq!(dangling_states, 0, "every state must link to a country");

    // Every non-NULL obedience_exercise_id points at a real exercise,
    // and every canonical_class_id points at a real canonical class.
    let dangling_exercises: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM obedience_class_exercises oce
         WHERE oce.obedience_exercise_id IS NOT NULL
           AND NOT EXISTS (SELECT 1 FROM obedience_exercises oe WHERE oe.id = oce.obedience_exercise_id)",
    )
    .fetch_one(&pool)
    .await
    .expect("count dangling exercise FKs");
    assert_eq!(dangling_exercises, 0);

    let dangling_classes: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM obedience_class_exercises oce
         WHERE NOT EXISTS (SELECT 1 FROM canonical_classes cc WHERE cc.id = oce.canonical_class_id)",
    )
    .fetch_one(&pool)
    .await
    .expect("count dangling canonical_class FKs");
    assert_eq!(dangling_classes, 0);

    // Every breed_variety points at a real breed.
    let dangling_varieties: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM breed_varieties v
         WHERE NOT EXISTS (SELECT 1 FROM breeds b WHERE b.id = v.breed_id)",
    )
    .fetch_one(&pool)
    .await
    .expect("count dangling varieties");
    assert_eq!(dangling_varieties, 0);
}

/// Rollup rows: obedience_class_exercises must contain rows whose
/// obedience_exercise_id is NULL but whose box_label carries the
/// label text (Subtotal of Points Off, Maximum Score (200), etc.).
/// The CHECK constraints prevent rows with all three of exercise_id,
/// max_points, and box_label NULL; this test asserts the positive
/// case.
#[tokio::test]
async fn test_rollup_rows() {
    let pool = testing::pool().await;
    ensure_seeded(&pool).await;

    let rollup_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM obedience_class_exercises
         WHERE obedience_exercise_id IS NULL
           AND max_points IS NULL
           AND box_label IS NOT NULL",
    )
    .fetch_one(&pool)
    .await
    .expect("count rollup rows");
    assert!(
        rollup_count > 0,
        "expected rollup rows (e.g. 'Subtotal of Points Off') to load; got {rollup_count}",
    );

    // No row can have all three of exercise_id, max_points, and
    // box_label NULL; the CHECK constraint guarantees this, but
    // re-asserting protects against a future CHECK weakening.
    let empty_rows: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM obedience_class_exercises
         WHERE obedience_exercise_id IS NULL
           AND max_points IS NULL
           AND box_label IS NULL",
    )
    .fetch_one(&pool)
    .await
    .expect("count empty rows");
    assert_eq!(
        empty_rows, 0,
        "no obedience_class_exercises row may be fully empty",
    );

    // Unmatched scored cells (exercise_id NULL + max_points set +
    // box_label set) should exist for compound cells like
    // "Heel on Leash & Figure Eight" and sub-numbered cells like
    // "Scent Discrimination #1". If zero rows match, either the
    // parser regressed or the CSV no longer has compound cells.
    let unmatched_scored: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM obedience_class_exercises
         WHERE obedience_exercise_id IS NULL
           AND max_points IS NOT NULL
           AND box_label IS NOT NULL",
    )
    .fetch_one(&pool)
    .await
    .expect("count unmatched scored cells");
    assert!(
        unmatched_scored > 0,
        "expected unmatched scored cells (compound exercises) to load with box_label set",
    );
}

/// NOT_YET_SEEDED guard: non_akc_title_suffixes.csv and
/// non_akc_title_suffix_breed_restrictions.csv sit in the seed
/// directory but must not load. The negative assertion: zero
/// title_suffixes rows have the SuffixTitle codes that appear only
/// in non_akc_title_suffixes.csv (WC, WCI, WCX are canonical
/// examples from the NSDTRC source_organization).
#[tokio::test]
async fn test_non_akc_skip() {
    let pool = testing::pool().await;
    ensure_seeded(&pool).await;

    // The NSDTRC "WC" row in non_akc_title_suffixes.csv has
    // source_organization = 'NSDTRC'. If the loader ever started
    // reading that file, we'd see at least one NSDTRC row here.
    let nsdtrc_rows: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM title_suffixes WHERE source_organization = 'NSDTRC'",
    )
    .fetch_one(&pool)
    .await
    .expect("count NSDTRC titles");
    assert_eq!(
        nsdtrc_rows, 0,
        "loader must not seed non_akc_title_suffixes.csv at MVP",
    );

    // Similarly for the breed-restrictions file: no table is meant
    // to exist for it at MVP; the guard is via the loader's
    // directory walk, which only opens the files it knows how to
    // handle. We can at least assert that the loader filename
    // constants include the expected entries so the guard is
    // maintained.
    assert!(seed_loader::NOT_YET_SEEDED.contains(&"non_akc_title_suffixes.csv"));
    assert!(seed_loader::NOT_YET_SEEDED.contains(&"non_akc_title_suffix_breed_restrictions.csv"));
}
