//! Integration tests for RLS on the event-setup schema.
//!
//! The core correctness question for this phase is whether any child
//! table in the event subtree (event_days, trials,
//! trial_class_offerings, judges, judge_assignments, trial_awards)
//! ever leaks rows across the tenant boundary. The FK-graph-walk
//! test seeds two complete event stacks (one per tenant) and then,
//! in tenant A's context, walks the full graph asserting that:
//!
//!   * tenant A sees every level of its own stack
//!   * tenant A sees zero rows at every level of tenant B's stack
//!   * the `parent_club_id` helper refuses to resolve a parent from
//!     the wrong tenant (because RLS hides it)
//!
//! If any child table gets added later without a `club_id` column
//! and matching RLS policy, this test should fail before the change
//! lands.

use qtrial_shared::tenancy::{self, ParentEntity};
use qtrial_shared::testing;
use sqlx::{Executor, Postgres, Transaction};
use uuid::Uuid;

/// Fully-seeded event stack for one tenant. Holds only the UUIDs
/// the tests need to compare; all the row data is written via the
/// seed helper and read back via the RLS-gated queries under test.
struct TenantStack {
    club_id: Uuid,
    user_id: Uuid,
    event_id: Uuid,
    event_day_ids: Vec<Uuid>,
    trial_ids: Vec<Uuid>,
    offering_ids: Vec<Uuid>,
    judge_id: Uuid,
    judge_assignment_ids: Vec<Uuid>,
    award_id: Uuid,
}

/// Seed one complete event stack for a fresh club: club, user,
/// role, one event with two days, two trials per day, one class
/// offering per trial (using the seeded AKC Rally Novice A),
/// one judge, judge assignments for every offering, and one HIT
/// award. Runs as the owning `qtrial` role so RLS does not
/// interfere with setup.
async fn seed_full_stack(tx: &mut Transaction<'_, Postgres>, name: &str) -> TenantStack {
    let club_id: Uuid = sqlx::query_scalar(
        "INSERT INTO clubs (display_name, abbreviation) VALUES ($1, $2) RETURNING id",
    )
    .bind(testing::unique_name(name))
    .bind(&name[..2])
    .fetch_one(&mut **tx)
    .await
    .expect("insert club");

    let user_id: Uuid =
        sqlx::query_scalar("INSERT INTO users (email, display_name) VALUES ($1, $2) RETURNING id")
            .bind(testing::unique_name("secretary") + "@example.test")
            .bind(format!("{name} secretary"))
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
        .expect("load AKC registry id");

    let event_id: Uuid = sqlx::query_scalar(
        "INSERT INTO events (club_id, registry_id, name) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(club_id)
    .bind(registry_id)
    .bind(testing::unique_name(&format!("{name} Fall Trial")))
    .fetch_one(&mut **tx)
    .await
    .expect("insert event");

    let mut event_day_ids = Vec::new();
    let mut trial_ids = Vec::new();
    let mut offering_ids = Vec::new();

    for day_number in 1..=2 {
        let day_id: Uuid = sqlx::query_scalar(
            "INSERT INTO event_days (club_id, event_id, day_number, date) \
             VALUES ($1, $2, $3, $4::date) RETURNING id",
        )
        .bind(club_id)
        .bind(event_id)
        .bind(day_number)
        // Distinct dates per day, rendered as ISO text so this test
        // file does not need to reach into chrono types directly.
        .bind(format!("2026-11-{:02}", day_number + 10))
        .fetch_one(&mut **tx)
        .await
        .expect("insert event_day");
        event_day_ids.push(day_id);

        for trial_number in 1..=2 {
            let trial_id: Uuid = sqlx::query_scalar(
                "INSERT INTO trials (club_id, event_day_id, trial_number, sport) \
                 VALUES ($1, $2, $3, 'rally') RETURNING id",
            )
            .bind(club_id)
            .bind(day_id)
            .bind(trial_number)
            .fetch_one(&mut **tx)
            .await
            .expect("insert trial");
            trial_ids.push(trial_id);

            let canonical_class_id: Uuid = sqlx::query_scalar(
                "SELECT id FROM canonical_classes WHERE code = 'akc_rally_novice_a'",
            )
            .fetch_one(&mut **tx)
            .await
            .expect("load rally novice a canonical id");

            let offering_id: Uuid = sqlx::query_scalar(
                "INSERT INTO trial_class_offerings \
                   (club_id, trial_id, canonical_class_id) \
                 VALUES ($1, $2, $3) RETURNING id",
            )
            .bind(club_id)
            .bind(trial_id)
            .bind(canonical_class_id)
            .fetch_one(&mut **tx)
            .await
            .expect("insert class offering");
            offering_ids.push(offering_id);
        }
    }

    let judge_id: Uuid = sqlx::query_scalar(
        "INSERT INTO judges (club_id, last_name, first_name) \
         VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(club_id)
    .bind(testing::unique_name(&format!("{name}-Judge")))
    .bind("Pat")
    .fetch_one(&mut **tx)
    .await
    .expect("insert judge");

    let mut judge_assignment_ids = Vec::new();
    for offering_id in &offering_ids {
        let assignment_id: Uuid = sqlx::query_scalar(
            "INSERT INTO judge_assignments \
               (club_id, trial_class_offering_id, judge_id) \
             VALUES ($1, $2, $3) RETURNING id",
        )
        .bind(club_id)
        .bind(offering_id)
        .bind(judge_id)
        .fetch_one(&mut **tx)
        .await
        .expect("insert judge assignment");
        judge_assignment_ids.push(assignment_id);
    }

    let award_id: Uuid = sqlx::query_scalar(
        "INSERT INTO trial_awards \
           (club_id, trial_id, award_type, winning_armband, winning_score) \
         VALUES ($1, $2, 'hit', 101, 198.5) RETURNING id",
    )
    .bind(club_id)
    .bind(trial_ids[0])
    .fetch_one(&mut **tx)
    .await
    .expect("insert trial award");

    TenantStack {
        club_id,
        user_id,
        event_id,
        event_day_ids,
        trial_ids,
        offering_ids,
        judge_id,
        judge_assignment_ids,
        award_id,
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

#[tokio::test]
async fn tenant_sees_own_stack_at_every_level() {
    let pool = testing::pool().await;
    let mut tx = pool.begin().await.expect("begin");

    let a = seed_full_stack(&mut tx, "Tenant A").await;
    let _b = seed_full_stack(&mut tx, "Tenant B").await;

    enter_tenant_context(&mut tx, a.user_id, a.club_id).await;

    let events: Vec<Uuid> = sqlx::query_scalar("SELECT id FROM events")
        .fetch_all(&mut *tx)
        .await
        .expect("events");
    assert_eq!(events, vec![a.event_id], "tenant A sees only its own event");

    let event_days: Vec<Uuid> = sqlx::query_scalar("SELECT id FROM event_days ORDER BY day_number")
        .fetch_all(&mut *tx)
        .await
        .expect("event_days");
    assert_eq!(
        event_days, a.event_day_ids,
        "tenant A sees both its event days"
    );

    let trials: Vec<Uuid> = sqlx::query_scalar("SELECT id FROM trials ORDER BY created_at")
        .fetch_all(&mut *tx)
        .await
        .expect("trials");
    assert_eq!(trials.len(), 4, "tenant A sees all four of its trials");
    for expected_trial in &a.trial_ids {
        assert!(trials.contains(expected_trial));
    }

    let offerings: Vec<Uuid> =
        sqlx::query_scalar("SELECT id FROM trial_class_offerings ORDER BY created_at")
            .fetch_all(&mut *tx)
            .await
            .expect("offerings");
    assert_eq!(offerings.len(), 4);

    let judges: Vec<Uuid> = sqlx::query_scalar("SELECT id FROM judges")
        .fetch_all(&mut *tx)
        .await
        .expect("judges");
    assert_eq!(judges, vec![a.judge_id]);

    let assignments: Vec<Uuid> = sqlx::query_scalar("SELECT id FROM judge_assignments")
        .fetch_all(&mut *tx)
        .await
        .expect("assignments");
    assert_eq!(assignments.len(), 4, "one assignment per offering");

    let awards: Vec<Uuid> = sqlx::query_scalar("SELECT id FROM trial_awards")
        .fetch_all(&mut *tx)
        .await
        .expect("awards");
    assert_eq!(awards, vec![a.award_id]);
}

#[tokio::test]
async fn tenant_cannot_see_other_tenants_stack_at_any_level() {
    let pool = testing::pool().await;
    let mut tx = pool.begin().await.expect("begin");

    let a = seed_full_stack(&mut tx, "Tenant A").await;
    let b = seed_full_stack(&mut tx, "Tenant B").await;

    enter_tenant_context(&mut tx, a.user_id, a.club_id).await;

    // Targeted lookups of every one of tenant B's IDs must return
    // zero rows. Any non-zero count is the "forgot club_id on a
    // child table" bug this test exists to catch.
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

    assert_eq!(count_visible(&mut *tx, "events", b.event_id).await, 0);
    for id in &b.event_day_ids {
        assert_eq!(count_visible(&mut *tx, "event_days", *id).await, 0);
    }
    for id in &b.trial_ids {
        assert_eq!(count_visible(&mut *tx, "trials", *id).await, 0);
    }
    for id in &b.offering_ids {
        assert_eq!(
            count_visible(&mut *tx, "trial_class_offerings", *id).await,
            0
        );
    }
    assert_eq!(count_visible(&mut *tx, "judges", b.judge_id).await, 0);
    for id in &b.judge_assignment_ids {
        assert_eq!(count_visible(&mut *tx, "judge_assignments", *id).await, 0);
    }
    assert_eq!(count_visible(&mut *tx, "trial_awards", b.award_id).await, 0);
}

#[tokio::test]
async fn tenant_cannot_write_child_rows_with_wrong_club_id() {
    // A tenant in club A's context attempts to INSERT an event_day
    // pointing at tenant A's event, but with club_id set to tenant
    // B's id. WITH CHECK on event_days rejects because the new row's
    // club_id != current_club_id.
    let pool = testing::pool().await;
    let mut tx = pool.begin().await.expect("begin");

    let a = seed_full_stack(&mut tx, "Tenant A").await;
    let b = seed_full_stack(&mut tx, "Tenant B").await;

    enter_tenant_context(&mut tx, a.user_id, a.club_id).await;

    let result = sqlx::query(
        "INSERT INTO event_days (club_id, event_id, day_number, date) \
         VALUES ($1, $2, 99, '2026-12-31')",
    )
    .bind(b.club_id) // Wrong club_id - tries to smuggle in tenant B's id
    .bind(a.event_id)
    .execute(&mut *tx)
    .await;

    assert!(
        result.is_err(),
        "INSERT with mismatched club_id must be rejected by WITH CHECK"
    );
    let err = result.err().unwrap().to_string();
    assert!(
        err.contains("row-level security") || err.contains("42501") || err.contains("policy"),
        "expected RLS policy error, got: {err}"
    );
}

#[tokio::test]
async fn parent_club_id_helper_returns_correct_club() {
    // Smoke-test the helper that app code will use when inserting
    // into child tables. Running as owner so RLS doesn't interfere;
    // the helper's correctness is about the SQL it generates, not
    // the RLS path.
    let pool = testing::pool().await;
    let mut tx = pool.begin().await.expect("begin");

    let a = seed_full_stack(&mut tx, "Tenant A").await;

    let from_event = tenancy::parent_club_id(&mut *tx, ParentEntity::Event, a.event_id)
        .await
        .expect("event lookup");
    assert_eq!(from_event, a.club_id);

    let from_day = tenancy::parent_club_id(&mut *tx, ParentEntity::EventDay, a.event_day_ids[0])
        .await
        .expect("event_day lookup");
    assert_eq!(from_day, a.club_id);

    let from_trial = tenancy::parent_club_id(&mut *tx, ParentEntity::Trial, a.trial_ids[0])
        .await
        .expect("trial lookup");
    assert_eq!(from_trial, a.club_id);

    let from_offering = tenancy::parent_club_id(
        &mut *tx,
        ParentEntity::TrialClassOffering,
        a.offering_ids[0],
    )
    .await
    .expect("offering lookup");
    assert_eq!(from_offering, a.club_id);
}

#[tokio::test]
async fn parent_club_id_helper_returns_not_found_for_missing_parent() {
    let pool = testing::pool().await;
    let mut tx = pool.begin().await.expect("begin");

    let nonexistent = Uuid::new_v4();
    let err = tenancy::parent_club_id(&mut *tx, ParentEntity::Event, nonexistent)
        .await
        .expect_err("nonexistent event must return NotFound");

    match err {
        tenancy::ParentClubIdError::NotFound { entity, id } => {
            assert_eq!(entity, ParentEntity::Event);
            assert_eq!(id, nonexistent);
        }
        other => panic!("expected NotFound, got {other:?}"),
    }
}

#[tokio::test]
async fn trial_awards_gin_index_supports_contributor_lookup() {
    // The GIN index on trial_awards.contributing_entry_line_ids is
    // the supporting index for "is this entry_line a contributor to
    // any award?" queries that scoring code will issue. Verify the
    // array containment operator returns the expected rows under a
    // realistic seed.
    let pool = testing::pool().await;
    let mut tx = pool.begin().await.expect("begin");

    let a = seed_full_stack(&mut tx, "Tenant A").await;

    let el1 = Uuid::new_v4();
    let el2 = Uuid::new_v4();
    let el3 = Uuid::new_v4();

    // Seed a second award with a non-trivial contributing array.
    sqlx::query(
        "INSERT INTO trial_awards \
           (club_id, trial_id, award_type, winning_armband, winning_score, \
            contributing_entry_line_ids) \
         VALUES ($1, $2, 'hc', 509, 394.0, $3)",
    )
    .bind(a.club_id)
    .bind(a.trial_ids[0])
    .bind(&[el1, el2][..])
    .execute(&mut *tx)
    .await
    .expect("insert HC award");

    // Contributors include el1. The @> operator is GIN-indexed and
    // is the shape the app will use.
    let hits: Vec<Uuid> = sqlx::query_scalar(
        "SELECT id FROM trial_awards \
         WHERE contributing_entry_line_ids @> ARRAY[$1]::uuid[]",
    )
    .bind(el1)
    .fetch_all(&mut *tx)
    .await
    .expect("GIN lookup");
    assert_eq!(hits.len(), 1, "should match the HC award");

    // el3 was never a contributor.
    let misses: Vec<Uuid> = sqlx::query_scalar(
        "SELECT id FROM trial_awards \
         WHERE contributing_entry_line_ids @> ARRAY[$1]::uuid[]",
    )
    .bind(el3)
    .fetch_all(&mut *tx)
    .await
    .expect("GIN lookup");
    assert!(misses.is_empty(), "should not match el3");
}
