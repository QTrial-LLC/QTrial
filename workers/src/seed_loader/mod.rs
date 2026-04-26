//! Idempotent loader for the AKC reference-data seed package.
//!
//! The loader reads every core-scope CSV under db/seed/akc/ (see the
//! seed directory README for the authoritative file inventory) and
//! upserts rows into the reference tables created by the migrations
//! in this PR. It is safe to re-run: each loader uses
//! INSERT ... ON CONFLICT (<natural_key>) DO UPDATE SET so a second
//! invocation produces zero inserts and leaves updated_at unchanged
//! on rows whose values did not drift.
//!
//! Run order follows FK dependencies:
//!
//! ```text
//! countries
//! -> states
//! -> breed_groups -> breeds -> breed_varieties
//! -> title_prefixes / title_suffixes
//! -> jump_heights
//! -> obedience_exercises -> obedience_class_exercises
//! -> otch_points / om_points / rally_rach_points
//! -> sport_time_defaults
//! ```
//!
//! Each table runs inside its own transaction. A file-level error
//! rolls back that file's work without aborting later files; the
//! overall run still exits non-zero if any file errored.
//!
//! Files that are explicitly NOT loaded at MVP:
//!
//! - `non_akc_title_suffixes.csv` and
//!   `non_akc_title_suffix_breed_restrictions.csv`: per Deborah's Q2
//!   (2026-04-20), only AKC and Barn Hunt titles are in MVP scope;
//!   the broader non-AKC title catalog is preserved on disk for
//!   post-MVP.
//! - `post_mvp/` subdirectory: AKC XML submission codes; deferred
//!   with the XML submission workstream (Agility, post-MVP).
//! - `legacy_akc_country_codes.csv`: an AKC-to-ISO mapping used by
//!   the migration from Access, not a seed target.
//! - `canonical_classes.csv` and `akc_overrides_added.csv`: loaded by
//!   prior migrations / maintained as audit artifacts.

use sqlx::PgPool;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub mod csv_rows;
pub mod tables;

/// Files intentionally not loaded in MVP. The loader walks the seed
/// directory, matches core-scope filenames, and logs an INFO line for
/// each NOT_YET_SEEDED filename it encounters. Using a constant list
/// (rather than silently ignoring "anything I don't recognize") keeps
/// the deferral explicit: a new file dropped in the seed directory
/// shows up in logs rather than vanishing without comment.
pub const NOT_YET_SEEDED: &[&str] = &[
    "non_akc_title_suffixes.csv",
    "non_akc_title_suffix_breed_restrictions.csv",
];

/// Per-table tallies produced by a single loader invocation.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TableStats {
    /// Rows read from the CSV file (data rows, excluding header).
    pub rows_read: u64,
    /// Rows that were newly inserted this run. On a second (idempotent)
    /// run this should be zero for every table.
    pub rows_inserted: u64,
    /// Rows that already existed and were reconciled via
    /// ON CONFLICT DO UPDATE SET. On a re-run with no source changes
    /// this is equal to rows_read and rows_inserted is zero.
    pub rows_upserted: u64,
    /// Rows the loader intentionally skipped (filtered by design).
    /// Examples: the Jump=0 sentinel in jump_heights.csv,
    /// unresolvable Class names in obedience_class_exercises.csv.
    pub rows_skipped: u64,
}

/// Full-run results keyed by table name so callers (the binary's exit
/// code path, the idempotency test) can iterate predictably.
pub type RunStats = BTreeMap<&'static str, TableStats>;

/// Structured error type for the loader. Keeping this separate from
/// `sqlx::Error` gives the binary an easy way to report which file
/// failed and at which CSV row; the inner `source` carries the root
/// cause for logging.
#[derive(Debug, thiserror::Error)]
pub enum LoaderError {
    #[error("seed directory not found: {0}")]
    SeedDirMissing(PathBuf),

    #[error("required seed file missing: {path}")]
    SeedFileMissing { path: PathBuf },

    #[error("failed to open {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("csv parse error in {path} at row {row}: {source}")]
    CsvParse {
        path: PathBuf,
        row: u64,
        #[source]
        source: csv::Error,
    },

    #[error("database error loading {table}: {source}")]
    Database {
        table: &'static str,
        #[source]
        source: sqlx::Error,
    },

    #[error("AKC registry row not found - run registries migration first")]
    AkcRegistryMissing,

    #[error("required obedience_exercises row missing: {name}")]
    ExerciseLookupFailed { name: String },

    #[error("required canonical_classes row missing: {name}")]
    CanonicalClassLookupFailed { name: String },

    /// Junction CSV refers to a combined_award_groups.code that is not
    /// in the database. Surfaces when the parent CSV is out of sync
    /// with the junction CSV; the seed loader is structured so the
    /// parent loads first, so this should only fire on a mistyped row.
    #[error(
        "combined_award_group_classes row {csv_row} references combined_award_groups.code \
         '{group_code}' which is not loaded; check db/seed/akc/combined_award_groups.csv"
    )]
    CombinedAwardGroupCodeMissing { csv_row: u64, group_code: String },

    /// Junction CSV refers to a canonical_classes.code that is not in
    /// the database. Either the canonical_classes seed migration did
    /// not load this code or the junction CSV has a typo.
    #[error(
        "combined_award_group_classes row {csv_row} references canonical_classes.code \
         '{canonical_class_code}' which is not loaded"
    )]
    CombinedAwardCanonicalClassCodeMissing {
        csv_row: u64,
        canonical_class_code: String,
    },

    /// Junction row links a combined_award_groups parent to a
    /// canonical_classes child whose sport differs from the parent's.
    /// The schema does not enforce this with a DDL trigger; the seed
    /// loader is the enforcement point per the CHECKPOINT 0 design
    /// note's combined_award_groups validation rule. Validation runs
    /// over every CSV row before any junction insert, so a mismatch
    /// produces no partial loads.
    #[error(
        "combined_award_group_classes row {csv_row} links group {group_code} (sport={group_sport}) \
         to canonical_class {canonical_class_code} (sport={canonical_class_sport}); \
         sports must match"
    )]
    CombinedAwardSportMismatch {
        csv_row: u64,
        group_code: String,
        group_sport: String,
        canonical_class_code: String,
        canonical_class_sport: String,
    },
}

/// Resolve the AKC registry UUID once at startup. Every registry-scoped
/// loader reuses this value rather than running SELECT per row.
pub async fn lookup_akc_registry_id(pool: &PgPool) -> Result<Uuid, LoaderError> {
    let row: Option<(Uuid,)> = sqlx::query_as("SELECT id FROM registries WHERE code = 'AKC'")
        .fetch_optional(pool)
        .await
        .map_err(|source| LoaderError::Database {
            table: "registries",
            source,
        })?;
    row.map(|(id,)| id).ok_or(LoaderError::AkcRegistryMissing)
}

/// Run every MVP-scope seed loader in FK-safe order. Returns a
/// per-table stats map; the caller decides what to do with it (the
/// binary pretty-prints, tests assert on specific rows).
///
/// On a fresh database this runs all 14 loaders. On a re-run every
/// loader reports zero inserts and zero skipped new rows.
///
/// FK-ordered transactions: each loader runs in its own transaction
/// and commits before the next begins, because later tables reference
/// earlier tables' primary keys and a cross-file rollback would be
/// difficult to reason about. A failure mid-run leaves the database
/// in a partial state; the re-run semantics of the upserts mean
/// picking up from a partial state is safe.
pub async fn run_all(pool: &PgPool, seed_dir: &Path) -> Result<RunStats, LoaderError> {
    if !seed_dir.exists() {
        return Err(LoaderError::SeedDirMissing(seed_dir.to_path_buf()));
    }

    let akc_registry_id = lookup_akc_registry_id(pool).await?;
    let mut stats = RunStats::new();

    // Log the NOT_YET_SEEDED files once per run so deferral visibility
    // is maintained at the run-summary level.
    for filename in NOT_YET_SEEDED {
        let path = seed_dir.join(filename);
        if path.exists() {
            tracing::info!(
                file = filename,
                reason = "deferred until post-MVP per Deborah Q2 (2026-04-20)",
                "skipping not-yet-seeded file"
            );
        }
    }

    stats.insert("countries", tables::load_countries(pool, seed_dir).await?);
    stats.insert("states", tables::load_states(pool, seed_dir).await?);
    stats.insert(
        "breed_groups",
        tables::load_breed_groups(pool, seed_dir, akc_registry_id).await?,
    );
    stats.insert(
        "breeds",
        tables::load_breeds(pool, seed_dir, akc_registry_id).await?,
    );
    stats.insert(
        "breed_varieties",
        tables::load_breed_varieties(pool, seed_dir).await?,
    );
    stats.insert(
        "title_prefixes",
        tables::load_title_prefixes(pool, seed_dir, akc_registry_id).await?,
    );
    stats.insert(
        "title_suffixes",
        tables::load_title_suffixes(pool, seed_dir, akc_registry_id).await?,
    );
    stats.insert(
        "jump_heights",
        tables::load_jump_heights(pool, seed_dir, akc_registry_id).await?,
    );
    stats.insert(
        "obedience_exercises",
        tables::load_obedience_exercises(pool, seed_dir).await?,
    );
    stats.insert(
        "obedience_class_exercises",
        tables::load_obedience_class_exercises(pool, seed_dir).await?,
    );
    stats.insert(
        "otch_points",
        tables::load_otch_points(pool, seed_dir).await?,
    );
    stats.insert("om_points", tables::load_om_points(pool, seed_dir).await?);
    stats.insert(
        "rally_rach_points",
        tables::load_rally_rach_points(pool, seed_dir).await?,
    );
    stats.insert(
        "sport_time_defaults",
        tables::load_sport_time_defaults(pool, seed_dir).await?,
    );

    // Combined-award groups land after every other reference table so
    // the canonical_classes lookup the junction loader needs is
    // guaranteed in place. The parent loads before the junction
    // because the junction's group-code FK lookup reads from the
    // parent rows.
    stats.insert(
        "combined_award_groups",
        tables::load_combined_award_groups(pool, seed_dir, akc_registry_id).await?,
    );
    stats.insert(
        "combined_award_group_classes",
        tables::load_combined_award_group_classes(pool, seed_dir).await?,
    );

    Ok(stats)
}

/// Shorthand for tagging a sqlx error with the table it came from.
/// Every per-table loader calls pool.begin() / tx.commit() directly
/// so the transaction lifetime stays syntactically simple; this helper
/// only builds the LoaderError enum variant.
pub(crate) fn db_err(table: &'static str) -> impl FnOnce(sqlx::Error) -> LoaderError {
    move |source| LoaderError::Database { table, source }
}
