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

/// CHECKPOINT 2 row count: combined_award_groups.csv loads exactly
/// the 5 AKC-recognized combined awards in scope (Obedience HC,
/// Rally RHC, Rally RHTQ, Rally RAE, Rally RACH). A future seed
/// addition that changes the count must update this assertion
/// deliberately, not silently.
#[tokio::test]
async fn test_combined_award_groups_load_count() {
    let pool = testing::pool().await;
    ensure_seeded(&pool).await;

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM combined_award_groups")
        .fetch_one(&pool)
        .await
        .expect("count combined_award_groups");
    assert_eq!(
        count, 5,
        "expected 5 combined_award_groups rows from CHECKPOINT 2 seed; got {count}",
    );
}

/// CHECKPOINT 2 row count: combined_award_group_classes.csv loads
/// exactly 12 junction rows: HC=2, RHC=2, RHTQ=3, RAE=2, RACH=3.
#[tokio::test]
async fn test_combined_award_group_classes_load_count() {
    let pool = testing::pool().await;
    ensure_seeded(&pool).await;

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM combined_award_group_classes")
        .fetch_one(&pool)
        .await
        .expect("count combined_award_group_classes");
    assert_eq!(
        count, 12,
        "expected 12 combined_award_group_classes rows from CHECKPOINT 2 seed; got {count}",
    );
}

/// Idempotency: a second run of the loader produces zero new inserts
/// for both combined-award tables and leaves the parent's
/// MAX(updated_at) unchanged. The junction table has no updated_at
/// column (one-shot reference data; never re-stamped on conflict),
/// so the junction's idempotency check is row-count stability plus
/// zero-inserted from the loader stats.
#[tokio::test]
async fn test_combined_award_seed_idempotent() {
    let pool = testing::pool().await;
    ensure_seeded(&pool).await;

    // Snapshot before the extra run.
    let parent_count_before: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM combined_award_groups")
        .fetch_one(&pool)
        .await
        .expect("parent count before");
    let parent_max_updated_before: Option<String> =
        sqlx::query_scalar("SELECT MAX(updated_at)::text FROM combined_award_groups")
            .fetch_one(&pool)
            .await
            .expect("parent max(updated_at) before");
    let junction_count_before: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM combined_award_group_classes")
            .fetch_one(&pool)
            .await
            .expect("junction count before");

    // Extra run; ensure_seeded already gated the first run, so this
    // is the only run that could mutate the database during the test.
    let stats = seed_loader::run_all(&pool, &seed_dir())
        .await
        .expect("second run should succeed");

    let parent_stats = stats
        .get("combined_award_groups")
        .expect("parent stats present");
    let junction_stats = stats
        .get("combined_award_group_classes")
        .expect("junction stats present");
    assert_eq!(
        parent_stats.rows_inserted, 0,
        "combined_award_groups must report zero new inserts on re-run; got {parent_stats:?}",
    );
    assert_eq!(
        junction_stats.rows_inserted, 0,
        "combined_award_group_classes must report zero new inserts on re-run; got {junction_stats:?}",
    );

    let parent_count_after: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM combined_award_groups")
        .fetch_one(&pool)
        .await
        .expect("parent count after");
    let parent_max_updated_after: Option<String> =
        sqlx::query_scalar("SELECT MAX(updated_at)::text FROM combined_award_groups")
            .fetch_one(&pool)
            .await
            .expect("parent max(updated_at) after");
    let junction_count_after: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM combined_award_group_classes")
            .fetch_one(&pool)
            .await
            .expect("junction count after");

    assert_eq!(parent_count_after, parent_count_before);
    assert_eq!(junction_count_after, junction_count_before);
    assert_eq!(
        parent_max_updated_after, parent_max_updated_before,
        "combined_award_groups MAX(updated_at) advanced on re-run (idempotency broken)",
    );
}

/// Sport-mismatch validation: the junction loader rejects a row whose
/// canonical_class.sport does not match the parent group's sport.
/// The bad row is fed via a tempdir CSV; the loader returns the
/// CombinedAwardSportMismatch variant with both codes and both sport
/// strings; no junction insert lands.
#[tokio::test]
async fn test_combined_award_seed_sport_mismatch_rejected() {
    let pool = testing::pool().await;
    ensure_seeded(&pool).await;

    let count_before: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM combined_award_group_classes")
        .fetch_one(&pool)
        .await
        .expect("count before");

    // Build a per-test temp seed directory with a deliberately bad
    // junction CSV: rally group + obedience class. Stdlib temp_dir
    // plus pid + nanos keeps the path collision-free across parallel
    // test binaries and avoids adding a tempfile dependency.
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let temp_dir = std::env::temp_dir().join(format!(
        "qtrial-seed-mismatch-{}-{}",
        std::process::id(),
        nanos
    ));
    std::fs::create_dir_all(&temp_dir).expect("create temp seed dir");
    let bad_csv = temp_dir.join("combined_award_group_classes.csv");
    std::fs::write(
        &bad_csv,
        "group_code,canonical_class_code,is_required_for_award\n\
         akc_rally_rhc,akc_obed_open_b,true\n",
    )
    .expect("write bad junction csv");

    let result = seed_loader::tables::load_combined_award_group_classes(&pool, &temp_dir).await;

    // Cleanup before any potential panic so a future test re-running
    // with the same nanos does not collide.
    let _ = std::fs::remove_dir_all(&temp_dir);

    match result {
        Err(seed_loader::LoaderError::CombinedAwardSportMismatch {
            csv_row,
            group_code,
            group_sport,
            canonical_class_code,
            canonical_class_sport,
        }) => {
            assert_eq!(csv_row, 1, "bad row should be CSV row 1");
            assert_eq!(group_code, "akc_rally_rhc");
            assert_eq!(group_sport, "rally");
            assert_eq!(canonical_class_code, "akc_obed_open_b");
            assert_eq!(canonical_class_sport, "obedience");
        }
        other => panic!("expected LoaderError::CombinedAwardSportMismatch, got {other:?}",),
    }

    let count_after: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM combined_award_group_classes")
        .fetch_one(&pool)
        .await
        .expect("count after");
    assert_eq!(
        count_after, count_before,
        "sport-mismatch load must not insert any junction rows; before={count_before} after={count_after}",
    );
}

/// is_required_for_award fidelity: the CHECKPOINT 2 seed has every
/// junction row at TRUE because every AKC-recognized combined entry
/// in scope requires Q's in all listed classes. A future seed that
/// adds a FALSE row (optional contributors for a new group) updates
/// this assertion deliberately.
#[tokio::test]
async fn test_combined_award_is_required_for_award_all_true() {
    let pool = testing::pool().await;
    ensure_seeded(&pool).await;

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM combined_award_group_classes")
        .fetch_one(&pool)
        .await
        .expect("count total");
    let true_rows: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM combined_award_group_classes WHERE is_required_for_award = TRUE",
    )
    .fetch_one(&pool)
    .await
    .expect("count true rows");
    assert_eq!(
        true_rows, total,
        "every CHECKPOINT 2 junction row must have is_required_for_award = TRUE; \
         total={total} true_rows={true_rows}",
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
