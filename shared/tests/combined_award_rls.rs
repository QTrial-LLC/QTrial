//! Integration tests for RLS on the combined-award reference tables.
//!
//! `combined_award_groups` and `combined_award_group_classes` are
//! registry-scoped reference data shared across tenants. Per the
//! pattern established for the 14 PR 2a reference tables (see
//! `db/migrations/README.md` §RLS conventions and the
//! `20260423120700_enable_rls_on_reference_data_foundation.up.sql`
//! migration), the contract is:
//!
//!   * Permissive-read RLS via a `*_read_all` policy with
//!     `USING (TRUE)` and a SELECT-only GRANT to qtrial_tenant.
//!   * No INSERT / UPDATE / DELETE policy and no INSERT / UPDATE /
//!     DELETE GRANT means tenant-role writes are blocked at the
//!     grant layer (SQLSTATE 42501 permission denied), not through
//!     a policy denial.
//!   * The `qtrial` table-owner role bypasses RLS by Postgres
//!     convention; this is how the seed loader and admin tooling
//!     populate the rows.
//!
//! These tests cover both directions: the read path returns the
//! expected row counts (so the permissive policy is functional, not
//! just declared), and every write attempt as the tenant role is
//! rejected with the grant-layer error.
//!
//! The tests seed the rows directly via the owner role inside each
//! test transaction rather than running the seed loader. The seed
//! loader lives in the workers crate and would create a circular
//! dep if pulled in here; manual seeding plus row-count assertions
//! against the same row set the loader would produce keeps the test
//! self-contained.
//!
//! Cross-binary contamination: the testcontainer fixture is shared
//! across every test binary in a `cargo test --workspace` run. The
//! seed-loader integration tests in `workers/tests/seed_loader.rs`
//! commit the same 5 + 12 rows this file's seed helper inserts. So
//! the seed helper uses `ON CONFLICT DO NOTHING` for both tables;
//! whether the row already exists from a prior committed seed run
//! or is being inserted fresh by this test, the post-seed count is
//! the same 5 + 12.

use qtrial_shared::testing;
use sqlx::{Executor, Postgres, Transaction};
use uuid::Uuid;

/// Seed the parent + junction tables to mirror the loader's output:
/// 5 parent rows (Obedience HC plus Rally RHC, RHTQ, RAE, RACH) and
/// 12 junction rows. Returns nothing; the tests query counts back
/// after seeding to confirm the inserts landed under the owner role
/// before any tenant-context switch.
async fn seed_combined_awards(tx: &mut Transaction<'_, Postgres>) {
    let registry_id: Uuid = sqlx::query_scalar("SELECT id FROM registries WHERE code = 'AKC'")
        .fetch_one(&mut **tx)
        .await
        .expect("AKC registry must exist; ran migrations before this test");

    // Parent rows. award_type binds as Option<&str> and the SQL
    // casts to the ENUM at parse time; NULL on RAE / RACH for the
    // title-progression paths that have no per-trial ribbon.
    type Group<'a> = (&'a str, &'a str, &'a str, Option<&'a str>);
    let groups: [Group<'_>; 5] = [
        (
            "akc_obedience_hc",
            "obedience",
            "Highest Combined Score in Open B and Utility B",
            Some("hc"),
        ),
        (
            "akc_rally_rhc",
            "rally",
            "Highest Combined Score in the Advanced B and Excellent B Classes",
            Some("rhc"),
        ),
        (
            "akc_rally_rhtq",
            "rally",
            "Highest Scoring Triple Qualifying Score",
            Some("rhtq"),
        ),
        ("akc_rally_rae", "rally", "Rally Advanced Excellent", None),
        ("akc_rally_rach", "rally", "Rally Champion", None),
    ];
    for (code, sport, display_name, award_type) in groups {
        sqlx::query(
            "INSERT INTO combined_award_groups
                 (registry_id, sport, code, display_name,
                  award_type, is_discount_eligible)
             VALUES ($1, $2::sport, $3, $4, $5::award_type, TRUE)
             ON CONFLICT (registry_id, sport, code) DO NOTHING",
        )
        .bind(registry_id)
        .bind(sport)
        .bind(code)
        .bind(display_name)
        .bind(award_type)
        .execute(&mut **tx)
        .await
        .expect("insert combined_award_groups parent row");
    }

    // Junction rows. The membership matches the AKC-recognized
    // combined entries verified against the regulation PDFs in
    // CHECKPOINT 1; every row is is_required_for_award = TRUE for
    // the CHECKPOINT 2 seed.
    let memberships: &[(&str, &[&str])] = &[
        (
            "akc_obedience_hc",
            &["akc_obed_open_b", "akc_obed_utility_b"],
        ),
        (
            "akc_rally_rhc",
            &["akc_rally_advanced_b", "akc_rally_excellent_b"],
        ),
        (
            "akc_rally_rhtq",
            &[
                "akc_rally_advanced_b",
                "akc_rally_excellent_b",
                "akc_rally_master",
            ],
        ),
        (
            "akc_rally_rae",
            &["akc_rally_advanced_b", "akc_rally_excellent_b"],
        ),
        (
            "akc_rally_rach",
            &[
                "akc_rally_advanced_b",
                "akc_rally_excellent_b",
                "akc_rally_master",
            ],
        ),
    ];
    for (group_code, members) in memberships {
        for class_code in *members {
            sqlx::query(
                "INSERT INTO combined_award_group_classes
                     (combined_award_group_id, canonical_class_id,
                      is_required_for_award)
                 SELECT g.id, c.id, TRUE
                 FROM combined_award_groups g, canonical_classes c
                 WHERE g.code = $1 AND c.code = $2
                 ON CONFLICT (combined_award_group_id, canonical_class_id)
                     DO NOTHING",
            )
            .bind(group_code)
            .bind(class_code)
            .execute(&mut **tx)
            .await
            .expect("insert combined_award_group_classes junction row");
        }
    }
}

/// Switch the transaction into the tenant role. Permissive-read
/// reference tables do not consult `app.current_club_id`, so we
/// only need the role switch; the session variable is unset.
async fn enter_tenant_role(tx: &mut Transaction<'_, Postgres>) {
    tx.execute("SET LOCAL ROLE qtrial_tenant")
        .await
        .expect("set local role qtrial_tenant");
}

/// Read path: tenant SELECT on both tables succeeds and returns the
/// 5 parent rows + 12 junction rows seeded above. This proves the
/// permissive-read policy is functional (not just declared) AND
/// that tenant queries see every reference-data row regardless of
/// `app.current_club_id`.
#[tokio::test]
async fn tenant_can_select_combined_award_rows() {
    let pool = testing::pool().await;
    let mut tx = pool.begin().await.expect("begin");
    seed_combined_awards(&mut tx).await;

    enter_tenant_role(&mut tx).await;

    let parent_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM combined_award_groups")
        .fetch_one(&mut *tx)
        .await
        .expect("count combined_award_groups as tenant");
    assert_eq!(
        parent_count, 5,
        "tenant SELECT must return all 5 seeded parent rows; got {parent_count}",
    );

    let junction_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM combined_award_group_classes")
            .fetch_one(&mut *tx)
            .await
            .expect("count combined_award_group_classes as tenant");
    assert_eq!(
        junction_count, 12,
        "tenant SELECT must return all 12 seeded junction rows; got {junction_count}",
    );
}

/// Run one statement inside its own SAVEPOINT and roll the savepoint
/// back, so a failed statement (which would otherwise abort the
/// outer transaction with SQLSTATE 25P02) does not poison the
/// subsequent assertions in the same test. Postgres's
/// "current transaction is aborted" state is per-transaction, but
/// SAVEPOINTs are independent: ROLLBACK TO SAVEPOINT clears the
/// abort flag without touching the outer transaction.
async fn run_in_savepoint(
    tx: &mut Transaction<'_, Postgres>,
    name: &str,
    sql: &str,
) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    tx.execute(format!("SAVEPOINT {name}").as_str())
        .await
        .expect("savepoint");
    let result = sqlx::query(sql).execute(&mut **tx).await;
    tx.execute(format!("ROLLBACK TO SAVEPOINT {name}").as_str())
        .await
        .expect("rollback to savepoint");
    tx.execute(format!("RELEASE SAVEPOINT {name}").as_str())
        .await
        .expect("release savepoint");
    result
}

/// Write-denial on the parent table: INSERT, UPDATE, and DELETE all
/// fail with SQLSTATE 42501 permission denied. The grant layer is
/// the gate; no INSERT / UPDATE / DELETE policy was created so the
/// failure mode is at the GRANT layer, not RLS policy denial.
///
/// Each statement runs inside its own SAVEPOINT so a failed
/// statement does not abort the outer transaction (SQLSTATE 25P02);
/// see `run_in_savepoint`.
#[tokio::test]
async fn tenant_cannot_write_combined_award_groups() {
    let pool = testing::pool().await;
    let mut tx = pool.begin().await.expect("begin");
    seed_combined_awards(&mut tx).await;

    let registry_id: Uuid = sqlx::query_scalar("SELECT id FROM registries WHERE code = 'AKC'")
        .fetch_one(&mut *tx)
        .await
        .expect("AKC registry");

    enter_tenant_role(&mut tx).await;

    // INSERT denied. The registry_id is bound by stringification
    // because the SAVEPOINT helper takes a static SQL string; using
    // a stable test-only registry-id stringification is fine for
    // the negative path, since the GRANT layer rejects before
    // parameter values are consulted.
    let insert_sql = format!(
        "INSERT INTO combined_award_groups
             (registry_id, sport, code, display_name,
              award_type, is_discount_eligible)
         VALUES ('{registry_id}', 'rally', 'tenant_must_not_insert', 'X',
                 NULL::award_type, TRUE)"
    );
    let insert = run_in_savepoint(&mut tx, "ins_parent", &insert_sql).await;
    assert_permission_denied(insert, "INSERT combined_award_groups");

    // UPDATE denied.
    let update = run_in_savepoint(
        &mut tx,
        "upd_parent",
        "UPDATE combined_award_groups
             SET display_name = 'should not happen'
             WHERE code = 'akc_obedience_hc'",
    )
    .await;
    assert_permission_denied(update, "UPDATE combined_award_groups");

    // DELETE denied.
    let delete = run_in_savepoint(
        &mut tx,
        "del_parent",
        "DELETE FROM combined_award_groups WHERE code = 'akc_obedience_hc'",
    )
    .await;
    assert_permission_denied(delete, "DELETE combined_award_groups");
}

/// Write-denial on the junction table: INSERT, UPDATE, and DELETE
/// all fail with SQLSTATE 42501 permission denied, same reasoning
/// as the parent. Junction is a separate table with its own GRANT,
/// so the denial path is the same but the assertion has to run
/// against this table specifically to catch a future GRANT-drift
/// where the parent stays locked but the junction accidentally
/// gains write grants.
#[tokio::test]
async fn tenant_cannot_write_combined_award_group_classes() {
    let pool = testing::pool().await;
    let mut tx = pool.begin().await.expect("begin");
    seed_combined_awards(&mut tx).await;

    enter_tenant_role(&mut tx).await;

    // INSERT denied. The SELECT-into-INSERT shape is what a
    // misconfigured loader-as-tenant would actually attempt; the
    // failure happens at the INSERT permission check, not the
    // SELECT.
    let insert = run_in_savepoint(
        &mut tx,
        "ins_junction",
        "INSERT INTO combined_award_group_classes
             (combined_award_group_id, canonical_class_id,
              is_required_for_award)
         SELECT g.id, c.id, TRUE
         FROM combined_award_groups g, canonical_classes c
         WHERE g.code = 'akc_obedience_hc' AND c.code = 'akc_obed_novice_a'",
    )
    .await;
    assert_permission_denied(insert, "INSERT combined_award_group_classes");

    // UPDATE denied.
    let update = run_in_savepoint(
        &mut tx,
        "upd_junction",
        "UPDATE combined_award_group_classes SET is_required_for_award = FALSE",
    )
    .await;
    assert_permission_denied(update, "UPDATE combined_award_group_classes");

    // DELETE denied.
    let delete = run_in_savepoint(
        &mut tx,
        "del_junction",
        "DELETE FROM combined_award_group_classes",
    )
    .await;
    assert_permission_denied(delete, "DELETE combined_award_group_classes");
}

/// Helper: assert that a sqlx execute result is the GRANT-layer
/// permission-denied error (SQLSTATE 42501) and not something else
/// (a different policy violation, a syntax error, an unexpected
/// success). Mirrors the
/// `platform_admins_select_is_permission_denied_for_tenant_role`
/// pattern in `tenant_scoped_gap_fill_rls.rs`.
fn assert_permission_denied<T>(result: Result<T, sqlx::Error>, op: &str) {
    match result {
        Err(sqlx::Error::Database(dberr)) if dberr.code().as_deref() == Some("42501") => {
            // Expected: permission denied for the table.
        }
        Err(other) => panic!("{op}: expected SQLSTATE 42501 permission denied, got {other:?}"),
        Ok(_) => {
            panic!("{op}: tenant role unexpectedly succeeded; the GRANT layer should have rejected")
        }
    }
}
