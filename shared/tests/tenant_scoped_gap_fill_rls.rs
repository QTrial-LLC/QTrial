//! Integration tests for RLS and related policies on the eight
//! tenant-scoped tables added in PR 2b plus the platform_admins
//! no-grant gate.
//!
//! Three behaviors per tenant-scoped table (mirrors the CHECKPOINT 3
//! smoke test that exercised the same shape ad hoc):
//!   1. qtrial_tenant + no app.current_club_id: SELECT returns 0 rows.
//!   2. qtrial_tenant + app.current_club_id=A: SELECT returns A's
//!      seeded rows and zero of B's. Cross-tenant INSERT is rejected
//!      by WITH CHECK.
//!   3. The audit_log pair: null actor_user_id INSERT succeeds when
//!      tenant context is set (system-action path); null actor_user_id
//!      INSERT fails WITH CHECK when tenant context is absent.
//!
//! platform_admins has no qtrial_tenant grant and no RLS; a tenant
//! SELECT must return a permission-denied error, not an empty result
//! set. This is the gate-layer defense that the absence of a read
//! policy alone would not guarantee.

use qtrial_shared::testing;
use sqlx::{Executor, Postgres, Transaction};
use uuid::Uuid;

/// One tenant's seed stack for the PR 2b tables plus the minimum
/// parent rows they need (club, user, owner, dog, event, day,
/// trial, entry, payment). Created under owner-role context so RLS
/// does not interfere with setup.
#[allow(dead_code)]
struct GapFillStack {
    club_id: Uuid,
    user_id: Uuid,
    owner_id: Uuid,
    dog_id: Uuid,
    event_id: Uuid,
    event_day_id: Uuid,
    trial_id: Uuid,
    entry_id: Uuid,
    payment_id: Uuid,
    dog_ownership_id: Uuid,
    dog_trial_jump_height_id: Uuid,
    armband_assignment_id: Uuid,
    email_template_id: Uuid,
    submission_record_id: Uuid,
    refund_id: Uuid,
    audit_log_id: Uuid,
}

async fn seed_gap_fill_stack(tx: &mut Transaction<'_, Postgres>, label: &str) -> GapFillStack {
    // Parent rows come first; every PR 2b table FKs into at least
    // one of these. Seeding happens as the owner role so RLS does
    // not gate the fixture itself; the tests then switch to tenant
    // role to exercise policy behavior.
    let club_id: Uuid = sqlx::query_scalar(
        "INSERT INTO clubs (display_name, abbreviation) VALUES ($1, $2) RETURNING id",
    )
    .bind(testing::unique_name(label))
    .bind(&label[..2])
    .fetch_one(&mut **tx)
    .await
    .expect("insert club");

    let user_id: Uuid =
        sqlx::query_scalar("INSERT INTO users (email, display_name) VALUES ($1, $2) RETURNING id")
            .bind(testing::unique_name("secretary") + "@example.test")
            .bind(format!("{label} secretary"))
            .fetch_one(&mut **tx)
            .await
            .expect("insert user");

    sqlx::query(
        "INSERT INTO user_club_roles (club_id, user_id, role) \
         VALUES ($1, $2, 'trial_secretary')",
    )
    .bind(club_id)
    .bind(user_id)
    .execute(&mut **tx)
    .await
    .expect("grant role");

    let registry_id: Uuid = sqlx::query_scalar("SELECT id FROM registries WHERE code = 'AKC'")
        .fetch_one(&mut **tx)
        .await
        .expect("AKC registry id");

    let owner_id: Uuid = sqlx::query_scalar(
        "INSERT INTO owners (club_id, last_name, first_name) \
         VALUES ($1, 'Owner', 'Jamie') RETURNING id",
    )
    .bind(club_id)
    .fetch_one(&mut **tx)
    .await
    .expect("insert owner");

    let dog_id: Uuid = sqlx::query_scalar(
        "INSERT INTO dogs (club_id, owner_id, registry_id, sex) \
         VALUES ($1, $2, $3, 'female_spayed') RETURNING id",
    )
    .bind(club_id)
    .bind(owner_id)
    .bind(registry_id)
    .fetch_one(&mut **tx)
    .await
    .expect("insert dog");

    let event_id: Uuid = sqlx::query_scalar(
        "INSERT INTO events (club_id, registry_id, name) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(club_id)
    .bind(registry_id)
    .bind(testing::unique_name(&format!("{label} Trial")))
    .fetch_one(&mut **tx)
    .await
    .expect("insert event");

    let event_day_id: Uuid = sqlx::query_scalar(
        "INSERT INTO event_days (club_id, event_id, day_number, date) \
         VALUES ($1, $2, 1, $3::date) RETURNING id",
    )
    .bind(club_id)
    .bind(event_id)
    .bind(format!("2026-11-{:02}", (label.len() % 27) + 1))
    .fetch_one(&mut **tx)
    .await
    .expect("insert event_day");

    let trial_id: Uuid = sqlx::query_scalar(
        "INSERT INTO trials (club_id, event_day_id, trial_number, sport) \
         VALUES ($1, $2, 1, 'rally') RETURNING id",
    )
    .bind(club_id)
    .bind(event_day_id)
    .fetch_one(&mut **tx)
    .await
    .expect("insert trial");

    let entry_id: Uuid = sqlx::query_scalar(
        "INSERT INTO entries (club_id, event_id, dog_id, exhibitor_user_id, owner_id) \
         VALUES ($1, $2, $3, $4, $5) RETURNING id",
    )
    .bind(club_id)
    .bind(event_id)
    .bind(dog_id)
    .bind(user_id)
    .bind(owner_id)
    .fetch_one(&mut **tx)
    .await
    .expect("insert entry");

    // PR 2b tables: one row each, all carrying the same club_id so
    // tenant-A context sees A's rows and tenant-B context sees B's.
    let dog_ownership_id: Uuid = sqlx::query_scalar(
        "INSERT INTO dog_ownerships (club_id, dog_id, owner_contact_id, is_primary) \
         VALUES ($1, $2, $3, TRUE) RETURNING id",
    )
    .bind(club_id)
    .bind(dog_id)
    .bind(owner_id)
    .fetch_one(&mut **tx)
    .await
    .expect("insert dog_ownership");

    let dog_trial_jump_height_id: Uuid = sqlx::query_scalar(
        "INSERT INTO dog_trial_jump_heights (club_id, dog_id, trial_id, jump_height_inches) \
         VALUES ($1, $2, $3, 20) RETURNING id",
    )
    .bind(club_id)
    .bind(dog_id)
    .bind(trial_id)
    .fetch_one(&mut **tx)
    .await
    .expect("insert dog_trial_jump_height");

    let armband_assignment_id: Uuid = sqlx::query_scalar(
        "INSERT INTO armband_assignments \
           (club_id, dog_id, trial_id, armband_series, armband_number) \
         VALUES ($1, $2, $3, '500', 509) RETURNING id",
    )
    .bind(club_id)
    .bind(dog_id)
    .bind(trial_id)
    .fetch_one(&mut **tx)
    .await
    .expect("insert armband_assignment");

    let email_template_id: Uuid = sqlx::query_scalar(
        "INSERT INTO email_templates (club_id, template_key, subject_template, body_template) \
         VALUES ($1, 'entry_confirmation', 'subj', 'body') RETURNING id",
    )
    .bind(club_id)
    .fetch_one(&mut **tx)
    .await
    .expect("insert email_template");

    let submission_record_id: Uuid = sqlx::query_scalar(
        "INSERT INTO submission_records \
           (club_id, trial_id, submission_type, akc_destination_email, fee_total) \
         VALUES ($1, $2, 'pdf_package', 'rallyresults@akc.org', 100.00) RETURNING id",
    )
    .bind(club_id)
    .bind(trial_id)
    .fetch_one(&mut **tx)
    .await
    .expect("insert submission_record");

    let payment_id: Uuid = sqlx::query_scalar(
        "INSERT INTO payments (club_id, entry_id, amount, method) \
         VALUES ($1, $2, 35.00, 'check') RETURNING id",
    )
    .bind(club_id)
    .bind(entry_id)
    .fetch_one(&mut **tx)
    .await
    .expect("insert payment");

    let refund_id: Uuid = sqlx::query_scalar(
        "INSERT INTO refunds (club_id, payment_id, amount, reason) \
         VALUES ($1, $2, 5.00, 'other') RETURNING id",
    )
    .bind(club_id)
    .bind(payment_id)
    .fetch_one(&mut **tx)
    .await
    .expect("insert refund");

    let audit_log_id: Uuid = sqlx::query_scalar(
        "INSERT INTO audit_log (club_id, actor_user_id, entity_type, entity_id, action) \
         VALUES ($1, $2, 'entry', $3, 'status_changed') RETURNING id",
    )
    .bind(club_id)
    .bind(user_id)
    .bind(entry_id)
    .fetch_one(&mut **tx)
    .await
    .expect("insert audit_log");

    GapFillStack {
        club_id,
        user_id,
        owner_id,
        dog_id,
        event_id,
        event_day_id,
        trial_id,
        entry_id,
        payment_id,
        dog_ownership_id,
        dog_trial_jump_height_id,
        armband_assignment_id,
        email_template_id,
        submission_record_id,
        refund_id,
        audit_log_id,
    }
}

async fn enter_tenant_context(tx: &mut Transaction<'_, Postgres>, user_id: Uuid, club_id: Uuid) {
    sqlx::query("SELECT set_config('app.current_user_id', $1, true)")
        .bind(user_id.to_string())
        .execute(&mut **tx)
        .await
        .expect("set current_user_id");
    sqlx::query("SELECT set_config('app.current_club_id', $1, true)")
        .bind(club_id.to_string())
        .execute(&mut **tx)
        .await
        .expect("set current_club_id");
    tx.execute("SET LOCAL ROLE qtrial_tenant")
        .await
        .expect("set local role qtrial_tenant");
}

/// Switch to qtrial_tenant WITHOUT setting app.current_club_id. Used
/// to prove that policies fail closed when the session variable is
/// absent (NULLIF returns NULL, the policy filter compares club_id
/// against NULL, which excludes all rows).
async fn enter_tenant_role_without_context(tx: &mut Transaction<'_, Postgres>) {
    tx.execute("SET LOCAL ROLE qtrial_tenant")
        .await
        .expect("set local role qtrial_tenant");
}

async fn count_visible<'c, E>(executor: E, table: &str, id: Uuid) -> i64
where
    E: Executor<'c, Database = Postgres>,
{
    let sql = format!("SELECT COUNT(*) FROM {table} WHERE id = $1");
    sqlx::query_scalar::<_, i64>(&sql)
        .bind(id)
        .fetch_one(executor)
        .await
        .unwrap_or_else(|err| panic!("count {table}: {err}"))
}

/// Each of the eight tenant-scoped PR 2b tables: under tenant-A's
/// context, A's seeded row is visible (count = 1), and B's is not
/// (count = 0). Walks every table in one test so a regression at
/// any layer surfaces together rather than across eight failures.
#[tokio::test]
async fn tenant_sees_own_gap_fill_rows_and_not_others() {
    let pool = testing::pool().await;
    let mut tx = pool.begin().await.expect("begin");

    let a = seed_gap_fill_stack(&mut tx, "GapA").await;
    let b = seed_gap_fill_stack(&mut tx, "GapB").await;

    enter_tenant_context(&mut tx, a.user_id, a.club_id).await;

    // A's own rows: visible.
    assert_eq!(
        count_visible(&mut *tx, "dog_ownerships", a.dog_ownership_id).await,
        1
    );
    assert_eq!(
        count_visible(
            &mut *tx,
            "dog_trial_jump_heights",
            a.dog_trial_jump_height_id
        )
        .await,
        1
    );
    assert_eq!(
        count_visible(&mut *tx, "armband_assignments", a.armband_assignment_id).await,
        1
    );
    assert_eq!(
        count_visible(&mut *tx, "email_templates", a.email_template_id).await,
        1
    );
    assert_eq!(
        count_visible(&mut *tx, "submission_records", a.submission_record_id).await,
        1
    );
    assert_eq!(count_visible(&mut *tx, "payments", a.payment_id).await, 1);
    assert_eq!(count_visible(&mut *tx, "refunds", a.refund_id).await, 1);
    assert_eq!(
        count_visible(&mut *tx, "audit_log", a.audit_log_id).await,
        1
    );

    // B's rows: invisible from A's context.
    assert_eq!(
        count_visible(&mut *tx, "dog_ownerships", b.dog_ownership_id).await,
        0
    );
    assert_eq!(
        count_visible(
            &mut *tx,
            "dog_trial_jump_heights",
            b.dog_trial_jump_height_id
        )
        .await,
        0
    );
    assert_eq!(
        count_visible(&mut *tx, "armband_assignments", b.armband_assignment_id).await,
        0
    );
    assert_eq!(
        count_visible(&mut *tx, "email_templates", b.email_template_id).await,
        0
    );
    assert_eq!(
        count_visible(&mut *tx, "submission_records", b.submission_record_id).await,
        0
    );
    assert_eq!(count_visible(&mut *tx, "payments", b.payment_id).await, 0);
    assert_eq!(count_visible(&mut *tx, "refunds", b.refund_id).await, 0);
    assert_eq!(
        count_visible(&mut *tx, "audit_log", b.audit_log_id).await,
        0
    );
}

/// Without app.current_club_id set, a tenant-role SELECT on every
/// PR 2b table returns 0. NULLIF of the unset-or-empty setting
/// returns NULL; the policy filter club_id = NULL fails for every
/// row. Defense-in-depth that forgetting to enter tenant context
/// fails closed rather than leaking everything.
#[tokio::test]
async fn tenant_role_without_context_sees_no_rows() {
    let pool = testing::pool().await;
    let mut tx = pool.begin().await.expect("begin");

    // Seed something so "zero rows" is a policy outcome, not an
    // "empty database" tautology.
    let _a = seed_gap_fill_stack(&mut tx, "NoCtxA").await;

    enter_tenant_role_without_context(&mut tx).await;

    for table in [
        "dog_ownerships",
        "dog_trial_jump_heights",
        "armband_assignments",
        "email_templates",
        "submission_records",
        "payments",
        "refunds",
        "audit_log",
    ] {
        let n: i64 = sqlx::query_scalar::<_, i64>(&format!("SELECT COUNT(*) FROM {table}"))
            .fetch_one(&mut *tx)
            .await
            .unwrap_or_else(|err| panic!("count {table}: {err}"));
        assert_eq!(n, 0, "table {table} should be empty without tenant context");
    }
}

/// Under A's context, an INSERT carrying club_id=B must be rejected
/// by the table's WITH CHECK. Walks every PR 2b tenant table; each
/// failure mode is "new row violates row-level security policy".
#[tokio::test]
async fn cross_club_insert_is_rejected_by_with_check() {
    let pool = testing::pool().await;
    let mut tx = pool.begin().await.expect("begin");

    let a = seed_gap_fill_stack(&mut tx, "XClubA").await;
    let b = seed_gap_fill_stack(&mut tx, "XClubB").await;

    enter_tenant_context(&mut tx, a.user_id, a.club_id).await;

    // Helper: run INSERT with club_id=b.club_id under A's context;
    // assert the error payload names a RLS policy violation (SQLSTATE
    // 42501 "insufficient_privilege" is Postgres's mapping for this).
    async fn assert_rls_reject(
        tx: &mut Transaction<'_, Postgres>,
        table: &str,
        sql: &str,
        binds: impl FnOnce(
            sqlx::query::Query<'_, Postgres, sqlx::postgres::PgArguments>,
        ) -> sqlx::query::Query<'_, Postgres, sqlx::postgres::PgArguments>,
    ) {
        // Wrap in a savepoint so the test can continue past the
        // expected failure; Postgres otherwise aborts the outer
        // transaction on the RLS error.
        sqlx::query("SAVEPOINT cross_club_attempt")
            .execute(&mut **tx)
            .await
            .expect("savepoint");

        let result = binds(sqlx::query(sql)).execute(&mut **tx).await;
        match result {
            Err(sqlx::Error::Database(dberr)) if dberr.code().as_deref() == Some("42501") => {
                // Expected: row-level security policy violation.
            }
            Err(other) => panic!("{table}: expected RLS reject (42501), got {other:?}"),
            Ok(_) => panic!("{table}: cross-club INSERT unexpectedly succeeded"),
        }

        sqlx::query("ROLLBACK TO SAVEPOINT cross_club_attempt")
            .execute(&mut **tx)
            .await
            .expect("rollback savepoint");
    }

    // dog_ownerships: club B's dog/owner would also violate FK-via-RLS,
    // but the WITH CHECK on club_id fires first.
    assert_rls_reject(
        &mut tx,
        "dog_ownerships",
        "INSERT INTO dog_ownerships (club_id, dog_id, owner_contact_id, is_primary) \
         VALUES ($1, $2, $3, FALSE)",
        |q| q.bind(b.club_id).bind(a.dog_id).bind(a.owner_id),
    )
    .await;

    assert_rls_reject(
        &mut tx,
        "dog_trial_jump_heights",
        "INSERT INTO dog_trial_jump_heights \
           (club_id, dog_id, trial_id, jump_height_inches) \
         VALUES ($1, $2, $3, 16)",
        |q| q.bind(b.club_id).bind(a.dog_id).bind(a.trial_id),
    )
    .await;

    assert_rls_reject(
        &mut tx,
        "armband_assignments",
        "INSERT INTO armband_assignments \
           (club_id, dog_id, trial_id, armband_series, armband_number) \
         VALUES ($1, $2, $3, '500', 701)",
        |q| q.bind(b.club_id).bind(a.dog_id).bind(a.trial_id),
    )
    .await;

    assert_rls_reject(
        &mut tx,
        "email_templates",
        "INSERT INTO email_templates \
           (club_id, template_key, subject_template, body_template) \
         VALUES ($1, 'post_closing_reminder', 's', 'b')",
        |q| q.bind(b.club_id),
    )
    .await;

    assert_rls_reject(
        &mut tx,
        "submission_records",
        "INSERT INTO submission_records \
           (club_id, trial_id, submission_type, akc_destination_email, fee_total) \
         VALUES ($1, $2, 'pdf_package', 'r@akc.org', 0.00)",
        |q| q.bind(b.club_id).bind(a.trial_id),
    )
    .await;

    assert_rls_reject(
        &mut tx,
        "payments",
        "INSERT INTO payments (club_id, entry_id, amount, method) \
         VALUES ($1, $2, 1.00, 'cash')",
        |q| q.bind(b.club_id).bind(a.entry_id),
    )
    .await;

    assert_rls_reject(
        &mut tx,
        "refunds",
        "INSERT INTO refunds (club_id, payment_id, amount, reason) \
         VALUES ($1, $2, 1.00, 'other')",
        |q| q.bind(b.club_id).bind(a.payment_id),
    )
    .await;

    assert_rls_reject(
        &mut tx,
        "audit_log",
        "INSERT INTO audit_log (club_id, entity_type, entity_id, action) \
         VALUES ($1, $2, $3, $4)",
        |q| {
            q.bind(b.club_id)
                .bind("entry")
                .bind(a.entry_id)
                .bind("cross_club_leak_attempt")
        },
    )
    .await;
}

/// audit_log system-action path: actor_user_id = NULL with tenant
/// context set must succeed. The WITH CHECK only constrains club_id
/// against current_club_id; the nullable actor_user_id is unrelated
/// to the policy. Positive half of the pair.
#[tokio::test]
async fn audit_log_null_actor_insert_succeeds_with_tenant_context() {
    let pool = testing::pool().await;
    let mut tx = pool.begin().await.expect("begin");

    let a = seed_gap_fill_stack(&mut tx, "AuditPos").await;
    enter_tenant_context(&mut tx, a.user_id, a.club_id).await;

    let inserted: Uuid = sqlx::query_scalar(
        "INSERT INTO audit_log (club_id, entity_type, entity_id, action) \
         VALUES ($1, 'submission_record', $2, 'system_retry') RETURNING id",
    )
    .bind(a.club_id)
    .bind(a.entry_id)
    .fetch_one(&mut *tx)
    .await
    .expect("null-actor insert with tenant context should succeed");

    let null_actor_visible: bool = sqlx::query_scalar(
        "SELECT COUNT(*) = 1 FROM audit_log WHERE id = $1 AND actor_user_id IS NULL",
    )
    .bind(inserted)
    .fetch_one(&mut *tx)
    .await
    .expect("count null-actor row");
    assert!(
        null_actor_visible,
        "inserted null-actor audit_log row should be visible to its own tenant"
    );
}

/// audit_log negative: under qtrial_tenant role with NO
/// app.current_club_id set, a null-actor INSERT must fail WITH CHECK.
/// Defense-in-depth: even a system-action writer must carry tenant
/// context; otherwise the write path uses the qtrial owner role.
#[tokio::test]
async fn audit_log_null_actor_insert_without_tenant_context_is_rejected() {
    let pool = testing::pool().await;
    let mut tx = pool.begin().await.expect("begin");

    let a = seed_gap_fill_stack(&mut tx, "AuditNeg").await;

    // Deliberately do NOT call enter_tenant_context; current_club_id
    // is never set. Switch to qtrial_tenant only.
    enter_tenant_role_without_context(&mut tx).await;

    let result = sqlx::query(
        "INSERT INTO audit_log (club_id, entity_type, entity_id, action) \
         VALUES ($1, 'submission_record', $2, 'system_retry_no_ctx')",
    )
    .bind(a.club_id)
    .bind(a.entry_id)
    .execute(&mut *tx)
    .await;

    match result {
        Err(sqlx::Error::Database(dberr)) if dberr.code().as_deref() == Some("42501") => {
            // Expected: RLS policy violation. SQLSTATE 42501.
        }
        Err(other) => panic!("expected RLS reject (42501), got {other:?}"),
        Ok(_) => {
            panic!("null-actor audit_log insert without tenant context unexpectedly succeeded")
        }
    }
}

/// platform_admins has no qtrial_tenant grant and no RLS. A tenant-
/// role SELECT must raise permission denied (SQLSTATE 42501),
/// not return an empty result. The grant layer is the gate, not a
/// read policy.
#[tokio::test]
async fn platform_admins_select_is_permission_denied_for_tenant_role() {
    let pool = testing::pool().await;
    let mut tx = pool.begin().await.expect("begin");

    // Seed a gap-fill stack so tenant context is valid; platform_admins
    // visibility is independent of context anyway, but a valid role
    // grant ensures we're exercising exactly the right failure path.
    let a = seed_gap_fill_stack(&mut tx, "PAdmin").await;
    enter_tenant_context(&mut tx, a.user_id, a.club_id).await;

    let result = sqlx::query("SELECT 1 FROM platform_admins LIMIT 1")
        .execute(&mut *tx)
        .await;

    match result {
        Err(sqlx::Error::Database(dberr)) if dberr.code().as_deref() == Some("42501") => {
            // Expected: permission denied for table platform_admins.
        }
        Err(other) => panic!("expected permission denied (42501), got {other:?}"),
        Ok(_) => panic!("qtrial_tenant unexpectedly could SELECT platform_admins"),
    }
}

/// payments read-side gate: a fixture with one payment per tenant;
/// tenant A's session sees exactly one row (A's own), not two.
/// The total row count is pre-verified against the owner role so
/// the assertion is not a "nothing was inserted" tautology.
#[tokio::test]
async fn payments_tenant_read_filters_cross_club_rows() {
    let pool = testing::pool().await;
    let mut tx = pool.begin().await.expect("begin");

    let a = seed_gap_fill_stack(&mut tx, "PayA").await;
    let _b = seed_gap_fill_stack(&mut tx, "PayB").await;

    // Owner role sees both payments (2 distinct club_ids).
    let owner_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM payments WHERE club_id IN ($1, $2)")
            .bind(a.club_id)
            .bind(_b.club_id)
            .fetch_one(&mut *tx)
            .await
            .expect("owner-role payments count");
    assert_eq!(owner_count, 2, "fixture should produce two payments rows");

    enter_tenant_context(&mut tx, a.user_id, a.club_id).await;

    let tenant_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM payments WHERE club_id IN ($1, $2)")
            .bind(a.club_id)
            .bind(_b.club_id)
            .fetch_one(&mut *tx)
            .await
            .expect("tenant-role payments count");
    assert_eq!(
        tenant_count, 1,
        "tenant A should see only A's payment row, not B's"
    );
}
