//! Per-table seed loaders.
//!
//! Each function reads one CSV file, opens a transaction, upserts every
//! row via INSERT ... ON CONFLICT DO UPDATE SET (omitting updated_at
//! from the SET list so an idempotent re-run with no source drift
//! leaves timestamps unchanged), and returns TableStats. Counts of
//! inserts versus updates come from the `(xmax = 0) AS inserted`
//! trick on the RETURNING clause: newly inserted rows have xmax = 0,
//! updates have a non-zero xmax.
//!
//! Per-row error handling: a parse failure on a specific row aborts
//! the current file's transaction with a structured LoaderError that
//! names the file and the 1-based row number. Row-scoped skips (e.g.
//! the Jump=0 sentinel or an unresolved canonical_class name) emit
//! an INFO tracing event and increment rows_skipped without failing.

use super::csv_rows::*;
use super::{LoaderError, TableStats, db_err};
use csv::ReaderBuilder;
use regex::Regex;
use sqlx::PgPool;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::sync::OnceLock;
use uuid::Uuid;

/// Compile the cell-parsing regex exactly once. DOTALL (?s) so the
/// newline between name and "(40 pts)" doesn't break the match. The
/// pattern is lazy on the name half so trailing "(N pts)" is
/// captured as the points half even when the name itself contains
/// parentheses elsewhere.
fn cell_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?s)^(.+?)\s*\(\s*(\d+)\s*pts?\s*\)\s*$").unwrap())
}

/// Regex that identifies a box cell that is purely a Random-Reward
/// placeholder like "#1", "#2", ..., "#6". Rows whose every scored
/// cell matches this are skipped: they represent the base Open B /
/// Utility B / Preferred Open / Preferred Utility layouts and carry
/// no concrete exercise content.
fn placeholder_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^#\d+$").unwrap())
}

/// Open a CSV reader from a path, wrapping IO/open errors with the
/// file path so the error message points at the real culprit.
fn open_csv(path: &Path) -> Result<csv::Reader<File>, LoaderError> {
    let file = File::open(path).map_err(|source| LoaderError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    Ok(ReaderBuilder::new().has_headers(true).from_reader(file))
}

/// Convert one CSV deserialization result into a typed row, tagging
/// parse errors with the file path and 1-based data-row number.
fn decode<T: serde::de::DeserializeOwned>(
    path: &Path,
    row_idx: u64,
    result: Result<T, csv::Error>,
) -> Result<T, LoaderError> {
    result.map_err(|source| LoaderError::CsvParse {
        path: path.to_path_buf(),
        row: row_idx,
        source,
    })
}

pub async fn load_countries(pool: &PgPool, seed_dir: &Path) -> Result<TableStats, LoaderError> {
    let table = "countries";
    let path = seed_dir.join("countries.csv");
    let mut reader = open_csv(&path)?;
    let mut stats = TableStats::default();

    let mut tx = pool.begin().await.map_err(db_err(table))?;
    for (idx, result) in reader.deserialize::<CountryRow>().enumerate() {
        let row = decode(&path, idx as u64 + 1, result)?;
        stats.rows_read += 1;

        let inserted: bool = sqlx::query_scalar(
            "INSERT INTO countries (alpha2_code, alpha3_code, display_name, display_order)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (alpha2_code) DO UPDATE SET
                 alpha3_code = EXCLUDED.alpha3_code,
                 display_name = EXCLUDED.display_name,
                 display_order = EXCLUDED.display_order
             RETURNING (xmax = 0) AS inserted",
        )
        .bind(&row.alpha2_code)
        .bind(&row.alpha3_code)
        .bind(&row.display_name)
        .bind(row.display_order)
        .fetch_one(&mut *tx)
        .await
        .map_err(db_err(table))?;

        stats.rows_upserted += 1;
        if inserted {
            stats.rows_inserted += 1;
        }
    }
    tx.commit().await.map_err(db_err(table))?;
    Ok(stats)
}

pub async fn load_states(pool: &PgPool, seed_dir: &Path) -> Result<TableStats, LoaderError> {
    let table = "states";
    let path = seed_dir.join("states.csv");
    let mut reader = open_csv(&path)?;
    let mut stats = TableStats::default();

    // Pre-fetch countries keyed by alpha2_code so the loader resolves
    // country_id without per-row SELECTs against the database.
    let country_lookup: HashMap<String, Uuid> =
        sqlx::query_as::<_, (String, Uuid)>("SELECT alpha2_code, id FROM countries")
            .fetch_all(pool)
            .await
            .map_err(db_err(table))?
            .into_iter()
            .collect();

    let mut tx = pool.begin().await.map_err(db_err(table))?;
    for (idx, result) in reader.deserialize::<StateRow>().enumerate() {
        let row = decode(&path, idx as u64 + 1, result)?;
        stats.rows_read += 1;

        let country_id = match country_lookup.get(&row.country_alpha2_code) {
            Some(id) => *id,
            None => {
                tracing::warn!(
                    csv_row = idx as u64 + 1,
                    country_alpha2_code = %row.country_alpha2_code,
                    code = %row.code,
                    "skipping state: country alpha2_code not found",
                );
                stats.rows_skipped += 1;
                continue;
            }
        };

        let inserted: bool = sqlx::query_scalar(
            "INSERT INTO states (country_id, code, legacy_id)
             VALUES ($1, $2, $3)
             ON CONFLICT (country_id, code) DO UPDATE SET
                 legacy_id = EXCLUDED.legacy_id
             RETURNING (xmax = 0) AS inserted",
        )
        .bind(country_id)
        .bind(&row.code)
        .bind(row.legacy_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(db_err(table))?;

        stats.rows_upserted += 1;
        if inserted {
            stats.rows_inserted += 1;
        }
    }
    tx.commit().await.map_err(db_err(table))?;
    Ok(stats)
}

pub async fn load_breed_groups(
    pool: &PgPool,
    seed_dir: &Path,
    akc_registry_id: Uuid,
) -> Result<TableStats, LoaderError> {
    let table = "breed_groups";
    let path = seed_dir.join("breed_groups.csv");
    let mut reader = open_csv(&path)?;
    let mut stats = TableStats::default();

    let mut tx = pool.begin().await.map_err(db_err(table))?;
    for (idx, result) in reader.deserialize::<BreedGroupRow>().enumerate() {
        let row = decode(&path, idx as u64 + 1, result)?;
        stats.rows_read += 1;

        // Split the pipe-delimited "SN|SR|SS" into a TEXT[] bind.
        let prefixes: Vec<String> = row
            .registration_prefixes
            .split('|')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let inserted: bool = sqlx::query_scalar(
            "INSERT INTO breed_groups
                 (registry_id, legacy_id, group_number, display_name, registration_prefixes)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (registry_id, group_number) DO UPDATE SET
                 legacy_id = EXCLUDED.legacy_id,
                 display_name = EXCLUDED.display_name,
                 registration_prefixes = EXCLUDED.registration_prefixes
             RETURNING (xmax = 0) AS inserted",
        )
        .bind(akc_registry_id)
        .bind(row.legacy_id)
        .bind(row.group_number)
        .bind(&row.display_name)
        .bind(&prefixes)
        .fetch_one(&mut *tx)
        .await
        .map_err(db_err(table))?;

        stats.rows_upserted += 1;
        if inserted {
            stats.rows_inserted += 1;
        }
    }
    tx.commit().await.map_err(db_err(table))?;
    Ok(stats)
}

pub async fn load_breeds(
    pool: &PgPool,
    seed_dir: &Path,
    akc_registry_id: Uuid,
) -> Result<TableStats, LoaderError> {
    let table = "breeds";
    let path = seed_dir.join("breeds.csv");
    let mut reader = open_csv(&path)?;
    let mut stats = TableStats::default();

    // Pre-fetch breed_groups keyed by legacy_id so the loader resolves
    // breed_group_id without per-row SELECTs.
    let group_lookup: HashMap<i32, Uuid> = sqlx::query_as::<_, (Option<i32>, Uuid)>(
        "SELECT legacy_id, id FROM breed_groups WHERE registry_id = $1",
    )
    .bind(akc_registry_id)
    .fetch_all(pool)
    .await
    .map_err(db_err(table))?
    .into_iter()
    .filter_map(|(legacy, id)| legacy.map(|l| (l, id)))
    .collect();

    let mut tx = pool.begin().await.map_err(db_err(table))?;
    for (idx, result) in reader.deserialize::<BreedRow>().enumerate() {
        let row = decode(&path, idx as u64 + 1, result)?;
        stats.rows_read += 1;

        let breed_group_id = match group_lookup.get(&row.group_legacy_id) {
            Some(id) => *id,
            None => {
                tracing::warn!(
                    csv_row = idx as u64 + 1,
                    group_legacy_id = row.group_legacy_id,
                    breed = %row.name,
                    "skipping breed: breed_group legacy_id not found",
                );
                stats.rows_skipped += 1;
                continue;
            }
        };

        let inserted: bool = sqlx::query_scalar(
            "INSERT INTO breeds
                 (registry_id, breed_group_id, legacy_id, display_name, abbreviation,
                  default_height_inches, is_giant, is_three_quarters,
                  has_variety, has_division, display_order)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
             ON CONFLICT (registry_id, display_name) DO UPDATE SET
                 breed_group_id = EXCLUDED.breed_group_id,
                 legacy_id = EXCLUDED.legacy_id,
                 abbreviation = EXCLUDED.abbreviation,
                 default_height_inches = EXCLUDED.default_height_inches,
                 is_giant = EXCLUDED.is_giant,
                 is_three_quarters = EXCLUDED.is_three_quarters,
                 has_variety = EXCLUDED.has_variety,
                 has_division = EXCLUDED.has_division,
                 display_order = EXCLUDED.display_order
             RETURNING (xmax = 0) AS inserted",
        )
        .bind(akc_registry_id)
        .bind(breed_group_id)
        .bind(row.legacy_id)
        .bind(&row.name)
        .bind(row.abbreviation.as_deref())
        .bind(row.default_height_inches)
        .bind(row.is_giant)
        .bind(row.is_three_quarters)
        .bind(row.has_variety)
        .bind(row.has_division)
        .bind(row.display_order)
        .fetch_one(&mut *tx)
        .await
        .map_err(db_err(table))?;

        stats.rows_upserted += 1;
        if inserted {
            stats.rows_inserted += 1;
        }
    }
    tx.commit().await.map_err(db_err(table))?;
    Ok(stats)
}

pub async fn load_breed_varieties(
    pool: &PgPool,
    seed_dir: &Path,
) -> Result<TableStats, LoaderError> {
    let table = "breed_varieties";
    let path = seed_dir.join("breed_varieties.csv");
    let mut reader = open_csv(&path)?;
    let mut stats = TableStats::default();

    let breed_lookup: HashMap<i32, Uuid> =
        sqlx::query_as::<_, (Option<i32>, Uuid)>("SELECT legacy_id, id FROM breeds")
            .fetch_all(pool)
            .await
            .map_err(db_err(table))?
            .into_iter()
            .filter_map(|(legacy, id)| legacy.map(|l| (l, id)))
            .collect();

    let mut tx = pool.begin().await.map_err(db_err(table))?;
    for (idx, result) in reader.deserialize::<BreedVarietyRow>().enumerate() {
        let row = decode(&path, idx as u64 + 1, result)?;
        stats.rows_read += 1;

        let breed_id = match breed_lookup.get(&row.breed_legacy_id) {
            Some(id) => *id,
            None => {
                tracing::warn!(
                    csv_row = idx as u64 + 1,
                    breed_legacy_id = row.breed_legacy_id,
                    variety = %row.name,
                    "skipping breed_variety: breed legacy_id not found",
                );
                stats.rows_skipped += 1;
                continue;
            }
        };

        let inserted: bool = sqlx::query_scalar(
            "INSERT INTO breed_varieties (breed_id, legacy_id, display_name, display_order)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (breed_id, display_name) DO UPDATE SET
                 legacy_id = EXCLUDED.legacy_id,
                 display_order = EXCLUDED.display_order
             RETURNING (xmax = 0) AS inserted",
        )
        .bind(breed_id)
        .bind(row.legacy_id)
        .bind(&row.name)
        .bind(row.display_order)
        .fetch_one(&mut *tx)
        .await
        .map_err(db_err(table))?;

        stats.rows_upserted += 1;
        if inserted {
            stats.rows_inserted += 1;
        }
    }
    tx.commit().await.map_err(db_err(table))?;
    Ok(stats)
}

/// Pick the registry_id for a title row based on source_organization.
/// AKC-sourced titles get the AKC registry UUID; everything else
/// (Barn Hunt Association and future non-AKC organizations) gets
/// NULL, matching the nullable FK on the title tables.
fn title_registry_for(source_organization: &str, akc_registry_id: Uuid) -> Option<Uuid> {
    if source_organization.eq_ignore_ascii_case("AKC") {
        Some(akc_registry_id)
    } else {
        None
    }
}

pub async fn load_title_prefixes(
    pool: &PgPool,
    seed_dir: &Path,
    akc_registry_id: Uuid,
) -> Result<TableStats, LoaderError> {
    let table = "title_prefixes";
    let path = seed_dir.join("title_prefixes.csv");
    let mut reader = open_csv(&path)?;
    let mut stats = TableStats::default();

    let mut tx = pool.begin().await.map_err(db_err(table))?;
    for (idx, result) in reader.deserialize::<TitlePrefixRow>().enumerate() {
        let row = decode(&path, idx as u64 + 1, result)?;
        stats.rows_read += 1;
        let registry_id = title_registry_for(&row.source_organization, akc_registry_id);

        let inserted: bool = sqlx::query_scalar(
            "INSERT INTO title_prefixes
                 (registry_id, legacy_id, code, long_name, sport_scope_code,
                  sport_scope_description, source_organization, display_order, earning_rules)
             VALUES ($1, $2, $3, $4, $5, $6, $7, NULL, NULL)
             ON CONFLICT (source_organization, code) DO UPDATE SET
                 registry_id = EXCLUDED.registry_id,
                 legacy_id = EXCLUDED.legacy_id,
                 long_name = EXCLUDED.long_name,
                 sport_scope_code = EXCLUDED.sport_scope_code,
                 sport_scope_description = EXCLUDED.sport_scope_description
             RETURNING (xmax = 0) AS inserted",
        )
        .bind(registry_id)
        .bind(row.legacy_id)
        .bind(&row.code)
        .bind(row.long_name.as_deref())
        .bind(row.sport_scope_code.as_deref())
        .bind(row.sport_scope_description.as_deref())
        .bind(&row.source_organization)
        .fetch_one(&mut *tx)
        .await
        .map_err(db_err(table))?;

        stats.rows_upserted += 1;
        if inserted {
            stats.rows_inserted += 1;
        }
    }
    tx.commit().await.map_err(db_err(table))?;
    Ok(stats)
}

pub async fn load_title_suffixes(
    pool: &PgPool,
    seed_dir: &Path,
    akc_registry_id: Uuid,
) -> Result<TableStats, LoaderError> {
    let table = "title_suffixes";
    let path = seed_dir.join("title_suffixes.csv");
    let mut reader = open_csv(&path)?;
    let mut stats = TableStats::default();

    let mut tx = pool.begin().await.map_err(db_err(table))?;
    for (idx, result) in reader.deserialize::<TitleSuffixRow>().enumerate() {
        let row = decode(&path, idx as u64 + 1, result)?;
        stats.rows_read += 1;
        let registry_id = title_registry_for(&row.source_organization, akc_registry_id);

        let inserted: bool = sqlx::query_scalar(
            "INSERT INTO title_suffixes
                 (registry_id, legacy_id, code, long_name, sport_scope_code,
                  sport_scope_description, source_organization, display_order, earning_rules)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NULL)
             ON CONFLICT (source_organization, code) DO UPDATE SET
                 registry_id = EXCLUDED.registry_id,
                 legacy_id = EXCLUDED.legacy_id,
                 long_name = EXCLUDED.long_name,
                 sport_scope_code = EXCLUDED.sport_scope_code,
                 sport_scope_description = EXCLUDED.sport_scope_description,
                 display_order = EXCLUDED.display_order
             RETURNING (xmax = 0) AS inserted",
        )
        .bind(registry_id)
        .bind(row.legacy_id)
        .bind(&row.code)
        .bind(row.long_name.as_deref())
        .bind(row.sport_scope_code.as_deref())
        .bind(row.sport_scope_description.as_deref())
        .bind(&row.source_organization)
        .bind(row.display_order)
        .fetch_one(&mut *tx)
        .await
        .map_err(db_err(table))?;

        stats.rows_upserted += 1;
        if inserted {
            stats.rows_inserted += 1;
        }
    }
    tx.commit().await.map_err(db_err(table))?;
    Ok(stats)
}

pub async fn load_jump_heights(
    pool: &PgPool,
    seed_dir: &Path,
    akc_registry_id: Uuid,
) -> Result<TableStats, LoaderError> {
    let table = "jump_heights";
    let path = seed_dir.join("jump_heights.csv");
    let mut reader = open_csv(&path)?;
    let mut stats = TableStats::default();

    let mut tx = pool.begin().await.map_err(db_err(table))?;
    for (idx, result) in reader.deserialize::<JumpHeightRow>().enumerate() {
        let row = decode(&path, idx as u64 + 1, result)?;
        stats.rows_read += 1;

        // Jump=0 is the "no jump" sentinel used by on-leash Obedience
        // classes. On-leash classes do not consult jump_heights, so
        // loading a 0-inch row would pollute UI dropdowns and height
        // queries with a meaningless value.
        if row.jump == 0 {
            tracing::info!(
                csv_row = idx as u64 + 1,
                jump_id = row.jump_id,
                "skipping jump_heights row: Jump=0 sentinel for on-leash classes",
            );
            stats.rows_skipped += 1;
            continue;
        }

        // The CSV capitalizes sport names ("Obedience", "Rally"); the
        // Postgres ENUM values are lowercase.
        let sport_lower = row.event.to_lowercase();

        let inserted: bool = sqlx::query_scalar(
            "INSERT INTO jump_heights
                 (registry_id, legacy_id, sport, height_inches,
                  akc_secondary_class_code, display_order)
             VALUES ($1, $2, $3::sport, $4::numeric(4,1), NULL, $5)
             ON CONFLICT (registry_id, sport, height_inches) DO UPDATE SET
                 legacy_id = EXCLUDED.legacy_id,
                 display_order = EXCLUDED.display_order
             RETURNING (xmax = 0) AS inserted",
        )
        .bind(akc_registry_id)
        .bind(row.jump_id)
        .bind(&sport_lower)
        .bind(row.jump.to_string())
        .bind(row.new_order)
        .fetch_one(&mut *tx)
        .await
        .map_err(db_err(table))?;

        stats.rows_upserted += 1;
        if inserted {
            stats.rows_inserted += 1;
        }
    }
    tx.commit().await.map_err(db_err(table))?;
    Ok(stats)
}

pub async fn load_obedience_exercises(
    pool: &PgPool,
    seed_dir: &Path,
) -> Result<TableStats, LoaderError> {
    let table = "obedience_exercises";
    let path = seed_dir.join("obedience_exercises.csv");
    let mut reader = open_csv(&path)?;
    let mut stats = TableStats::default();

    let mut tx = pool.begin().await.map_err(db_err(table))?;
    for (idx, result) in reader.deserialize::<ObedienceExerciseRow>().enumerate() {
        let row = decode(&path, idx as u64 + 1, result)?;
        stats.rows_read += 1;

        let inserted: bool = sqlx::query_scalar(
            "INSERT INTO obedience_exercises (legacy_id, display_name)
             VALUES ($1, $2)
             ON CONFLICT (display_name) DO UPDATE SET
                 legacy_id = EXCLUDED.legacy_id
             RETURNING (xmax = 0) AS inserted",
        )
        .bind(row.exercise_id)
        .bind(&row.exercise)
        .fetch_one(&mut *tx)
        .await
        .map_err(db_err(table))?;

        stats.rows_upserted += 1;
        if inserted {
            stats.rows_inserted += 1;
        }
    }
    tx.commit().await.map_err(db_err(table))?;
    Ok(stats)
}

/// Parse the Roman-numeral variant suffix on class names like
/// "Open B I" or "Utility B VI". Returns (base_name, variant) when
/// the class name ends with " I".."VI"; returns (full_name, 1) for
/// non-variant class names. Only the six variants AKC publishes are
/// recognized; anything beyond VI returns None so the loader skips.
fn parse_class_variant(class_name: &str) -> Option<(String, i32)> {
    const VARIANTS: &[(&str, i32)] = &[
        (" I", 1),
        (" II", 2),
        (" III", 3),
        (" IV", 4),
        (" V", 5),
        (" VI", 6),
    ];
    for (suffix, variant) in VARIANTS.iter().rev() {
        if let Some(base) = class_name.strip_suffix(suffix) {
            return Some((base.to_string(), *variant));
        }
    }
    Some((class_name.to_string(), 1))
}

pub async fn load_obedience_class_exercises(
    pool: &PgPool,
    seed_dir: &Path,
) -> Result<TableStats, LoaderError> {
    let table = "obedience_class_exercises";
    let path = seed_dir.join("obedience_class_exercises.csv");
    let mut reader = open_csv(&path)?;
    let mut stats = TableStats::default();

    // Resolve canonical Obedience class UUIDs by display_name. Scoping
    // the SELECT to sport='obedience' prevents a cross-sport name
    // collision from mis-resolving (e.g. if a Rally class were ever
    // named "Utility B").
    let class_lookup: HashMap<String, Uuid> = sqlx::query_as::<_, (String, Uuid)>(
        "SELECT display_name, id FROM canonical_classes WHERE sport = 'obedience'",
    )
    .fetch_all(pool)
    .await
    .map_err(db_err(table))?
    .into_iter()
    .collect();

    let exercise_lookup: HashMap<String, Uuid> =
        sqlx::query_as::<_, (String, Uuid)>("SELECT display_name, id FROM obedience_exercises")
            .fetch_all(pool)
            .await
            .map_err(db_err(table))?
            .into_iter()
            .collect();

    // Guard against a mis-seeded database: the loader needs the
    // canonical_classes and obedience_exercises seeds to be in place
    // already. Early-return with a structured error so the caller can
    // tell whether the run failed because of seed data or because of
    // a database error.
    if class_lookup.is_empty() {
        return Err(LoaderError::CanonicalClassLookupFailed {
            name: "(no obedience classes seeded)".to_string(),
        });
    }
    if exercise_lookup.is_empty() {
        return Err(LoaderError::ExerciseLookupFailed {
            name: "(no obedience exercises seeded)".to_string(),
        });
    }

    let mut tx = pool.begin().await.map_err(db_err(table))?;
    for (idx, result) in reader
        .deserialize::<ObedienceClassExerciseRow>()
        .enumerate()
    {
        let row = decode(&path, idx as u64 + 1, result)?;
        stats.rows_read += 1;

        let (base_name, variant) = match parse_class_variant(&row.class_name) {
            Some(pair) => pair,
            None => {
                tracing::info!(
                    csv_row = idx as u64 + 1,
                    class = %row.class_name,
                    "skipping obedience_class_exercises row: unrecognized variant suffix",
                );
                stats.rows_skipped += 1;
                continue;
            }
        };

        let canonical_class_id = match class_lookup.get(&base_name) {
            Some(id) => *id,
            None => {
                tracing::info!(
                    csv_row = idx as u64 + 1,
                    class_code = row.class_code,
                    class = %row.class_name,
                    "skipping obedience_class_exercises row: canonical class not seeded",
                );
                stats.rows_skipped += 1;
                continue;
            }
        };

        let boxes = row.boxes();
        let cell_regex = cell_regex();
        let placeholder_regex = placeholder_regex();

        // Rows with any `#N` placeholder cell are Random-Reward base
        // layouts (base Open B / Utility B / Preferred Open /
        // Preferred Utility). Skip them in favor of:
        //   * Open B I-VI and Utility B I-VI rows (legacy_ids 901-912)
        //     which carry the real per-variant exercise lists and
        //     load as pattern_variants 1-6 of the base canonical class
        //   * Preferred Open / Preferred Utility, which the seed CSV
        //     has no concrete exercise data for. A later PR will add
        //     those classes per published AKC sources.
        // The `^#\d+$` match is exact (whole-cell) so sub-numbered
        // exercise cells like "Scent Discrimination #1" are not
        // caught by this filter.
        let has_placeholder_box = boxes
            .iter()
            .any(|(_, text)| placeholder_regex.is_match(text.trim()));
        if has_placeholder_box {
            tracing::info!(
                csv_row = idx as u64 + 1,
                class = %row.class_name,
                class_code = row.class_code,
                "skipping obedience_class_exercises row: Random-Reward base layout \
                 (loaded per-variant via Open B I-VI / Utility B I-VI rows)",
            );
            stats.rows_skipped += 1;
            continue;
        }

        for (display_order, cell_text) in boxes {
            let (exercise_id, max_points, box_label) =
                classify_cell(&cell_text, &exercise_lookup, cell_regex);

            let inserted: bool = sqlx::query_scalar(
                "INSERT INTO obedience_class_exercises
                     (canonical_class_id, obedience_exercise_id, pattern_variant,
                      display_order, max_points, box_label)
                 VALUES ($1, $2, $3, $4, $5, $6)
                 ON CONFLICT (canonical_class_id, pattern_variant, display_order) DO UPDATE SET
                     obedience_exercise_id = EXCLUDED.obedience_exercise_id,
                     max_points = EXCLUDED.max_points,
                     box_label = EXCLUDED.box_label
                 RETURNING (xmax = 0) AS inserted",
            )
            .bind(canonical_class_id)
            .bind(exercise_id)
            .bind(variant)
            .bind(display_order)
            .bind(max_points)
            .bind(box_label.as_deref())
            .fetch_one(&mut *tx)
            .await
            .map_err(db_err(table))?;

            stats.rows_upserted += 1;
            if inserted {
                stats.rows_inserted += 1;
            }
        }
    }
    tx.commit().await.map_err(db_err(table))?;
    Ok(stats)
}

/// Decide how to store a single box cell. Returns
/// (exercise_id, max_points, box_label):
///
/// * Matched scored cell: (Some(uuid), Some(pts), None)
/// * Unmatched scored cell (compound / sub-numbered / placeholder):
///   (None, Some(pts), Some(raw_text))
/// * Rollup / header cell (no "(N pts)" suffix): (None, None, Some(text))
fn classify_cell(
    cell_text: &str,
    exercise_lookup: &HashMap<String, Uuid>,
    cell_regex: &Regex,
) -> (Option<Uuid>, Option<i32>, Option<String>) {
    let trimmed = cell_text.trim();
    if let Some(captures) = cell_regex.captures(trimmed) {
        let name = captures.get(1).unwrap().as_str().trim();
        // The regex captures a pure decimal int so parse cannot fail
        // on the seed data; the unwrap is deliberate.
        let points: i32 = captures.get(2).unwrap().as_str().parse().unwrap();
        if let Some(&uuid) = exercise_lookup.get(name) {
            (Some(uuid), Some(points), None)
        } else {
            (None, Some(points), Some(cell_text.to_string()))
        }
    } else {
        (None, None, Some(cell_text.to_string()))
    }
}

pub async fn load_otch_points(pool: &PgPool, seed_dir: &Path) -> Result<TableStats, LoaderError> {
    let table = "otch_points";
    let path = seed_dir.join("otch_points.csv");
    let mut reader = open_csv(&path)?;
    let mut stats = TableStats::default();

    let mut tx = pool.begin().await.map_err(db_err(table))?;
    for (idx, result) in reader.deserialize::<OtchPointsRow>().enumerate() {
        let row = decode(&path, idx as u64 + 1, result)?;
        stats.rows_read += 1;

        let inserted: bool = sqlx::query_scalar(
            "INSERT INTO otch_points
                 (legacy_id, class_name, entries_min, entries_max,
                  first_place_points, second_place_points,
                  third_place_points, fourth_place_points)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             ON CONFLICT (class_name, entries_min, entries_max) DO UPDATE SET
                 legacy_id = EXCLUDED.legacy_id,
                 first_place_points = EXCLUDED.first_place_points,
                 second_place_points = EXCLUDED.second_place_points,
                 third_place_points = EXCLUDED.third_place_points,
                 fourth_place_points = EXCLUDED.fourth_place_points
             RETURNING (xmax = 0) AS inserted",
        )
        .bind(row.legacy_id)
        .bind(&row.class_name)
        .bind(row.entries_min)
        .bind(row.entries_max)
        .bind(row.first_place_points)
        .bind(row.second_place_points)
        .bind(row.third_place_points)
        .bind(row.fourth_place_points)
        .fetch_one(&mut *tx)
        .await
        .map_err(db_err(table))?;

        stats.rows_upserted += 1;
        if inserted {
            stats.rows_inserted += 1;
        }
    }
    tx.commit().await.map_err(db_err(table))?;
    Ok(stats)
}

pub async fn load_om_points(pool: &PgPool, seed_dir: &Path) -> Result<TableStats, LoaderError> {
    let table = "om_points";
    let path = seed_dir.join("om_points.csv");
    let mut reader = open_csv(&path)?;
    let mut stats = TableStats::default();

    let mut tx = pool.begin().await.map_err(db_err(table))?;
    for (idx, result) in reader.deserialize::<OmPointsRow>().enumerate() {
        let row = decode(&path, idx as u64 + 1, result)?;
        stats.rows_read += 1;

        let inserted: bool = sqlx::query_scalar(
            "INSERT INTO om_points (legacy_id, score, om_points)
             VALUES ($1, $2::numeric(4,1), $3)
             ON CONFLICT (score) DO UPDATE SET
                 legacy_id = EXCLUDED.legacy_id,
                 om_points = EXCLUDED.om_points
             RETURNING (xmax = 0) AS inserted",
        )
        .bind(row.legacy_id)
        .bind(&row.score)
        .bind(row.om_points)
        .fetch_one(&mut *tx)
        .await
        .map_err(db_err(table))?;

        stats.rows_upserted += 1;
        if inserted {
            stats.rows_inserted += 1;
        }
    }
    tx.commit().await.map_err(db_err(table))?;
    Ok(stats)
}

pub async fn load_rally_rach_points(
    pool: &PgPool,
    seed_dir: &Path,
) -> Result<TableStats, LoaderError> {
    let table = "rally_rach_points";
    let path = seed_dir.join("rally_rach_points.csv");
    let mut reader = open_csv(&path)?;
    let mut stats = TableStats::default();

    let mut tx = pool.begin().await.map_err(db_err(table))?;
    for (idx, result) in reader.deserialize::<RallyRachPointsRow>().enumerate() {
        let row = decode(&path, idx as u64 + 1, result)?;
        stats.rows_read += 1;

        let inserted: bool = sqlx::query_scalar(
            "INSERT INTO rally_rach_points (legacy_id, score, rach_points)
             VALUES ($1, $2::numeric(4,1), $3)
             ON CONFLICT (score) DO UPDATE SET
                 legacy_id = EXCLUDED.legacy_id,
                 rach_points = EXCLUDED.rach_points
             RETURNING (xmax = 0) AS inserted",
        )
        .bind(row.legacy_id)
        .bind(&row.score)
        .bind(row.rach_points)
        .fetch_one(&mut *tx)
        .await
        .map_err(db_err(table))?;

        stats.rows_upserted += 1;
        if inserted {
            stats.rows_inserted += 1;
        }
    }
    tx.commit().await.map_err(db_err(table))?;
    Ok(stats)
}

pub async fn load_sport_time_defaults(
    pool: &PgPool,
    seed_dir: &Path,
) -> Result<TableStats, LoaderError> {
    let table = "sport_time_defaults";
    let path = seed_dir.join("trial_time_calculations.csv");
    let mut reader = open_csv(&path)?;
    let mut stats = TableStats::default();

    let mut tx = pool.begin().await.map_err(db_err(table))?;
    for (idx, result) in reader.deserialize::<SportTimeDefaultRow>().enumerate() {
        let row = decode(&path, idx as u64 + 1, result)?;
        stats.rows_read += 1;

        let inserted: bool = sqlx::query_scalar(
            "INSERT INTO sport_time_defaults
                 (legacy_id, sport_or_event, minutes_per_dog,
                  class_change_seconds, event_change_seconds)
             VALUES ($1, $2, $3::numeric(3,1), $4, $5)
             ON CONFLICT (sport_or_event) DO UPDATE SET
                 legacy_id = EXCLUDED.legacy_id,
                 minutes_per_dog = EXCLUDED.minutes_per_dog,
                 class_change_seconds = EXCLUDED.class_change_seconds,
                 event_change_seconds = EXCLUDED.event_change_seconds
             RETURNING (xmax = 0) AS inserted",
        )
        .bind(row.legacy_id)
        .bind(&row.sport_or_event)
        .bind(&row.minutes_per_dog)
        .bind(row.class_change_seconds)
        .bind(row.event_change_seconds)
        .fetch_one(&mut *tx)
        .await
        .map_err(db_err(table))?;

        stats.rows_upserted += 1;
        if inserted {
            stats.rows_inserted += 1;
        }
    }
    tx.commit().await.map_err(db_err(table))?;
    Ok(stats)
}
