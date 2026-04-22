//! Integration tests for RLS and CHECK constraints on the entry
//! layer.
//!
//! The FK-graph-walk pair (positive and negative) is the primary
//! regression guard: it seeds two complete tenant stacks including
//! an entry, an entry_line, and an entry_line_result, then asserts
//! that a tenant-context query sees its own subtree in full and
//! sees zero rows of the other tenant's subtree at every table.
//!
//! The remaining tests cover the CHECK constraints and the armband
//! partial unique that enforce domain invariants in SQL: armband
//! reuse across live entries is forbidden; a result row with a
//! placement must be qualifying; a waitlist position cannot attach
//! to a non-waitlisted line.

use qtrial_shared::tenancy::{self, ParentEntity};
use qtrial_shared::testing;
use sqlx::{Executor, Postgres, Transaction};
use uuid::Uuid;

/// Full entry-layer stack for one tenant. Holds the UUIDs the tests
/// need to compare. The seed helper creates every row in owner-
/// role context so RLS does not interfere with setup.
struct EntryStack {
    club_id: Uuid,
    user_id: Uuid,
    event_id: Uuid,
    event_day_id: Uuid,
    trial_id: Uuid,
    offering_id: Uuid,
    judge_id: Uuid,
    judge_assignment_id: Uuid,
    owner_id: Uuid,
    dog_id: Uuid,
    entry_id: Uuid,
    entry_line_id: Uuid,
    entry_line_result_id: Uuid,
}

async fn seed_entry_stack(tx: &mut Transaction<'_, Postgres>, name: &str) -> EntryStack {
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

    let event_day_id: Uuid = sqlx::query_scalar(
        "INSERT INTO event_days (club_id, event_id, day_number, date) \
         VALUES ($1, $2, 1, $3::date) RETURNING id",
    )
    .bind(club_id)
    .bind(event_id)
    .bind(format!("2026-11-{:02}", (name.len() % 27) + 1))
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

    let canonical_class_id: Uuid =
        sqlx::query_scalar("SELECT id FROM canonical_classes WHERE code = 'akc_rally_novice_a'")
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

    let judge_assignment_id: Uuid = sqlx::query_scalar(
        "INSERT INTO judge_assignments \
           (club_id, trial_class_offering_id, judge_id) \
         VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(club_id)
    .bind(offering_id)
    .bind(judge_id)
    .fetch_one(&mut **tx)
    .await
    .expect("insert judge_assignment");

    let owner_id: Uuid = sqlx::query_scalar(
        "INSERT INTO owners (club_id, last_name, first_name, email) \
         VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind(club_id)
    .bind(format!("{name}-Owner"))
    .bind("Jamie")
    .bind(testing::unique_name("owner") + "@example.test")
    .fetch_one(&mut **tx)
    .await
    .expect("insert owner");

    let dog_id: Uuid = sqlx::query_scalar(
        "INSERT INTO dogs (club_id, owner_id, registry_id, call_name, registered_name, sex) \
         VALUES ($1, $2, $3, $4, $5, 'female_spayed') RETURNING id",
    )
    .bind(club_id)
    .bind(owner_id)
    .bind(registry_id)
    .bind(format!("{name}-CallName"))
    .bind(testing::unique_name(&format!("{name} Registered Name")))
    .fetch_one(&mut **tx)
    .await
    .expect("insert dog");

    let entry_id: Uuid = sqlx::query_scalar(
        "INSERT INTO entries (club_id, event_id, dog_id, exhibitor_user_id, owner_id, armband) \
         VALUES ($1, $2, $3, $4, $5, 101) RETURNING id",
    )
    .bind(club_id)
    .bind(event_id)
    .bind(dog_id)
    .bind(user_id)
    .bind(owner_id)
    .fetch_one(&mut **tx)
    .await
    .expect("insert entry");

    let entry_line_id: Uuid = sqlx::query_scalar(
        "INSERT INTO entry_lines (club_id, entry_id, trial_class_offering_id, status) \
         VALUES ($1, $2, $3, 'active') RETURNING id",
    )
    .bind(club_id)
    .bind(entry_id)
    .bind(offering_id)
    .fetch_one(&mut **tx)
    .await
    .expect("insert entry_line");

    let entry_line_result_id: Uuid = sqlx::query_scalar(
        "INSERT INTO entry_line_results \
           (club_id, entry_line_id, score, qualifying, placement) \
         VALUES ($1, $2, 98.0, 'q', 1) RETURNING id",
    )
    .bind(club_id)
    .bind(entry_line_id)
    .fetch_one(&mut **tx)
    .await
    .expect("insert entry_line_result");

    EntryStack {
        club_id,
        user_id,
        event_id,
        event_day_id,
        trial_id,
        offering_id,
        judge_id,
        judge_assignment_id,
        owner_id,
        dog_id,
        entry_id,
        entry_line_id,
        entry_line_result_id,
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

#[tokio::test]
async fn tenant_sees_full_entry_stack_in_own_context() {
    let pool = testing::pool().await;
    let mut tx = pool.begin().await.expect("begin");

    let a = seed_entry_stack(&mut tx, "Tenant A").await;
    let _b = seed_entry_stack(&mut tx, "Tenant B").await;

    enter_tenant_context(&mut tx, a.user_id, a.club_id).await;

    // Walk every level of tenant A's stack. Each lookup targets a
    // single known id; if RLS is correctly scoped, the count is 1.
    assert_eq!(count_visible(&mut *tx, "events", a.event_id).await, 1);
    assert_eq!(
        count_visible(&mut *tx, "event_days", a.event_day_id).await,
        1
    );
    assert_eq!(count_visible(&mut *tx, "trials", a.trial_id).await, 1);
    assert_eq!(
        count_visible(&mut *tx, "trial_class_offerings", a.offering_id).await,
        1
    );
    assert_eq!(count_visible(&mut *tx, "judges", a.judge_id).await, 1);
    assert_eq!(
        count_visible(&mut *tx, "judge_assignments", a.judge_assignment_id).await,
        1
    );
    assert_eq!(count_visible(&mut *tx, "owners", a.owner_id).await, 1);
    assert_eq!(count_visible(&mut *tx, "dogs", a.dog_id).await, 1);
    assert_eq!(count_visible(&mut *tx, "entries", a.entry_id).await, 1);
    assert_eq!(
        count_visible(&mut *tx, "entry_lines", a.entry_line_id).await,
        1
    );
    assert_eq!(
        count_visible(&mut *tx, "entry_line_results", a.entry_line_result_id).await,
        1
    );
}

#[tokio::test]
async fn tenant_cannot_see_other_tenants_entry_stack_at_any_level() {
    let pool = testing::pool().await;
    let mut tx = pool.begin().await.expect("begin");

    let a = seed_entry_stack(&mut tx, "Tenant A").await;
    let b = seed_entry_stack(&mut tx, "Tenant B").await;

    enter_tenant_context(&mut tx, a.user_id, a.club_id).await;

    // Any non-zero count at any level is the "forgot club_id on a
    // child table" bug this test exists to catch.
    assert_eq!(count_visible(&mut *tx, "events", b.event_id).await, 0);
    assert_eq!(
        count_visible(&mut *tx, "event_days", b.event_day_id).await,
        0
    );
    assert_eq!(count_visible(&mut *tx, "trials", b.trial_id).await, 0);
    assert_eq!(
        count_visible(&mut *tx, "trial_class_offerings", b.offering_id).await,
        0
    );
    assert_eq!(count_visible(&mut *tx, "judges", b.judge_id).await, 0);
    assert_eq!(
        count_visible(&mut *tx, "judge_assignments", b.judge_assignment_id).await,
        0
    );
    assert_eq!(count_visible(&mut *tx, "owners", b.owner_id).await, 0);
    assert_eq!(count_visible(&mut *tx, "dogs", b.dog_id).await, 0);
    assert_eq!(count_visible(&mut *tx, "entries", b.entry_id).await, 0);
    assert_eq!(
        count_visible(&mut *tx, "entry_lines", b.entry_line_id).await,
        0
    );
    assert_eq!(
        count_visible(&mut *tx, "entry_line_results", b.entry_line_result_id).await,
        0
    );
}

#[tokio::test]
async fn tenant_cannot_discover_other_tenants_dog_ids_for_cross_tenant_entries() {
    // The cross-tenant entry attack this test guards against is:
    // tenant A constructs an entries INSERT pointing at tenant B's
    // dog_id. The actual protection is defense-in-depth via RLS on
    // dogs: tenant A cannot see B's dog in any tenant-context
    // query, so cannot discover the UUID required to craft such an
    // INSERT.
    //
    // This test asserts the SELECT-hiding behavior rather than
    // testing the INSERT itself, because Postgres FK integrity
    // checks bypass RLS on the referenced table. If a UUID were
    // leaked outside the normal query path and supplied as a
    // literal to an INSERT, the FK check would succeed. That gap is
    // flagged in the session-5 check-in and can be closed at the
    // app layer by verifying the FK target's club_id before insert,
    // or at the DB layer by adding a trigger. Neither lands in
    // this session.
    let pool = testing::pool().await;
    let mut tx = pool.begin().await.expect("begin");

    let a = seed_entry_stack(&mut tx, "Tenant A").await;
    let b = seed_entry_stack(&mut tx, "Tenant B").await;

    enter_tenant_context(&mut tx, a.user_id, a.club_id).await;

    // Tenant A under tenant role: can see A's dog, cannot see B's.
    let visible_a: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM dogs WHERE id = $1")
        .bind(a.dog_id)
        .fetch_one(&mut *tx)
        .await
        .expect("count A dog");
    let visible_b: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM dogs WHERE id = $1")
        .bind(b.dog_id)
        .fetch_one(&mut *tx)
        .await
        .expect("count B dog");

    assert_eq!(visible_a, 1, "tenant A must see its own dog");
    assert_eq!(
        visible_b, 0,
        "tenant A must not see tenant B's dog, which is the real \
         barrier to constructing a cross-tenant entries INSERT"
    );
}

#[tokio::test]
async fn tenant_cannot_discover_other_tenants_offering_ids_for_cross_tenant_lines() {
    // Mirror of the dog test at the trial_class_offerings level.
    // Same reasoning: FK integrity checks bypass RLS, so a supplied
    // UUID referencing another tenant's offering would technically
    // succeed at the DB level. The practical defense is that the
    // UUID cannot be discovered via any tenant-context query.
    let pool = testing::pool().await;
    let mut tx = pool.begin().await.expect("begin");

    let a = seed_entry_stack(&mut tx, "Tenant A").await;
    let b = seed_entry_stack(&mut tx, "Tenant B").await;

    enter_tenant_context(&mut tx, a.user_id, a.club_id).await;

    let visible_a: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM trial_class_offerings WHERE id = $1")
            .bind(a.offering_id)
            .fetch_one(&mut *tx)
            .await
            .expect("count A offering");
    let visible_b: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM trial_class_offerings WHERE id = $1")
            .bind(b.offering_id)
            .fetch_one(&mut *tx)
            .await
            .expect("count B offering");

    assert_eq!(visible_a, 1);
    assert_eq!(
        visible_b, 0,
        "tenant A must not discover tenant B's offering UUID"
    );
}

#[tokio::test]
async fn armband_is_unique_among_live_entries_in_an_event() {
    // Two entries in the same event cannot both hold armband 42 if
    // both are live (neither soft-deleted). Soft-deleting the first
    // one must release the armband for reuse.
    //
    // The seed already created one entry (a.entry_id) against
    // (a.event_id, a.dog_id). Re-use that entry as "the first
    // armband-42 entry" by UPDATE-ing its armband, which avoids
    // colliding with the entries_event_dog_uk partial unique on
    // (event_id, dog_id).
    let pool = testing::pool().await;
    let mut tx = pool.begin().await.expect("begin");

    let a = seed_entry_stack(&mut tx, "Tenant A").await;

    // Seed a second owner and dog for the second entry.
    let second_owner_id: Uuid = sqlx::query_scalar(
        "INSERT INTO owners (club_id, last_name, first_name) \
         VALUES ($1, 'Second', 'Owner') RETURNING id",
    )
    .bind(a.club_id)
    .fetch_one(&mut *tx)
    .await
    .expect("second owner");

    let a_registry: Uuid = sqlx::query_scalar("SELECT id FROM registries WHERE code = 'AKC'")
        .fetch_one(&mut *tx)
        .await
        .expect("registry");

    let second_dog_id: Uuid = sqlx::query_scalar(
        "INSERT INTO dogs (club_id, owner_id, registry_id, call_name, sex) \
         VALUES ($1, $2, $3, 'Second', 'male_neutered') RETURNING id",
    )
    .bind(a.club_id)
    .bind(second_owner_id)
    .bind(a_registry)
    .fetch_one(&mut *tx)
    .await
    .expect("second dog");

    // Re-use the seed entry as the first armband-42 holder.
    sqlx::query("UPDATE entries SET armband = 42 WHERE id = $1")
        .bind(a.entry_id)
        .execute(&mut *tx)
        .await
        .expect("claim armband 42 on seed entry");

    // Savepoint so the expected-fail INSERT does not poison the tx.
    sqlx::query("SAVEPOINT before_duplicate")
        .execute(&mut *tx)
        .await
        .expect("savepoint");

    let duplicate_attempt = sqlx::query(
        "INSERT INTO entries (club_id, event_id, dog_id, exhibitor_user_id, owner_id, armband) \
         VALUES ($1, $2, $3, $4, $5, 42)",
    )
    .bind(a.club_id)
    .bind(a.event_id)
    .bind(second_dog_id)
    .bind(a.user_id)
    .bind(second_owner_id)
    .execute(&mut *tx)
    .await;

    assert!(
        duplicate_attempt.is_err(),
        "duplicate armband on live entries must be rejected"
    );

    sqlx::query("ROLLBACK TO SAVEPOINT before_duplicate")
        .execute(&mut *tx)
        .await
        .expect("rollback savepoint");

    // Soft-delete the seed entry. Armband should be freed.
    sqlx::query("UPDATE entries SET deleted_at = NOW() WHERE id = $1")
        .bind(a.entry_id)
        .execute(&mut *tx)
        .await
        .expect("soft-delete seed entry");

    let reuse_attempt = sqlx::query(
        "INSERT INTO entries (club_id, event_id, dog_id, exhibitor_user_id, owner_id, armband) \
         VALUES ($1, $2, $3, $4, $5, 42)",
    )
    .bind(a.club_id)
    .bind(a.event_id)
    .bind(second_dog_id)
    .bind(a.user_id)
    .bind(second_owner_id)
    .execute(&mut *tx)
    .await;

    assert!(
        reuse_attempt.is_ok(),
        "after soft-deleting the first entry the partial unique index \
         must allow armband 42 to be reassigned, got: {reuse_attempt:?}"
    );
}

#[tokio::test]
async fn waitlist_position_only_attaches_to_waitlisted_lines() {
    let pool = testing::pool().await;
    let mut tx = pool.begin().await.expect("begin");

    let a = seed_entry_stack(&mut tx, "Tenant A").await;

    // Need a second trial_class_offering so we can insert a new
    // entry_line (the offering from seed is already taken by
    // a.entry_line_id).
    let second_canonical_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM canonical_classes WHERE code = 'akc_rally_intermediate'",
    )
    .fetch_one(&mut *tx)
    .await
    .expect("rally intermediate class id");

    let second_offering_id: Uuid = sqlx::query_scalar(
        "INSERT INTO trial_class_offerings (club_id, trial_id, canonical_class_id) \
         VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(a.club_id)
    .bind(a.trial_id)
    .bind(second_canonical_id)
    .fetch_one(&mut *tx)
    .await
    .expect("second offering");

    // Savepoint so the expected-fail INSERT does not abort the
    // outer transaction.
    sqlx::query("SAVEPOINT before_bad")
        .execute(&mut *tx)
        .await
        .expect("savepoint");

    let bad = sqlx::query(
        "INSERT INTO entry_lines \
           (club_id, entry_id, trial_class_offering_id, status, waitlist_position) \
         VALUES ($1, $2, $3, 'active', 5)",
    )
    .bind(a.club_id)
    .bind(a.entry_id)
    .bind(second_offering_id)
    .execute(&mut *tx)
    .await;

    assert!(
        bad.is_err(),
        "waitlist_position on a non-waitlisted line must be rejected"
    );

    sqlx::query("ROLLBACK TO SAVEPOINT before_bad")
        .execute(&mut *tx)
        .await
        .expect("rollback savepoint");

    let good = sqlx::query(
        "INSERT INTO entry_lines \
           (club_id, entry_id, trial_class_offering_id, status, waitlist_position) \
         VALUES ($1, $2, $3, 'waitlist', 5)",
    )
    .bind(a.club_id)
    .bind(a.entry_id)
    .bind(second_offering_id)
    .execute(&mut *tx)
    .await;

    assert!(
        good.is_ok(),
        "waitlist status with waitlist_position must succeed, got: {good:?}"
    );
}

#[tokio::test]
async fn parent_club_id_helper_covers_new_entry_layer_variants() {
    let pool = testing::pool().await;
    let mut tx = pool.begin().await.expect("begin");

    let a = seed_entry_stack(&mut tx, "Tenant A").await;

    let from_owner = tenancy::parent_club_id(&mut *tx, ParentEntity::Owner, a.owner_id)
        .await
        .expect("owner lookup");
    assert_eq!(from_owner, a.club_id);

    let from_dog = tenancy::parent_club_id(&mut *tx, ParentEntity::Dog, a.dog_id)
        .await
        .expect("dog lookup");
    assert_eq!(from_dog, a.club_id);

    let from_entry = tenancy::parent_club_id(&mut *tx, ParentEntity::Entry, a.entry_id)
        .await
        .expect("entry lookup");
    assert_eq!(from_entry, a.club_id);

    let from_line = tenancy::parent_club_id(&mut *tx, ParentEntity::EntryLine, a.entry_line_id)
        .await
        .expect("entry_line lookup");
    assert_eq!(from_line, a.club_id);

    // NotFound path for each new variant.
    let missing = Uuid::new_v4();
    for entity in [
        ParentEntity::Owner,
        ParentEntity::Dog,
        ParentEntity::Entry,
        ParentEntity::EntryLine,
    ] {
        let err = tenancy::parent_club_id(&mut *tx, entity, missing)
            .await
            .expect_err("missing id must return NotFound");
        match err {
            tenancy::ParentClubIdError::NotFound { entity: got, id }
                if got == entity && id == missing => {}
            other => panic!("expected NotFound for {entity:?}, got {other:?}"),
        }
    }
}

#[tokio::test]
async fn placement_requires_qualifying_score() {
    let pool = testing::pool().await;
    let mut tx = pool.begin().await.expect("begin");

    let a = seed_entry_stack(&mut tx, "Tenant A").await;

    // The seed helper already inserted one result against
    // a.entry_line_id. Create a second entry_line we can attach
    // additional results to for this test.
    let second_canonical_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM canonical_classes WHERE code = 'akc_rally_intermediate'",
    )
    .fetch_one(&mut *tx)
    .await
    .expect("rally intermediate class id");

    let second_offering_id: Uuid = sqlx::query_scalar(
        "INSERT INTO trial_class_offerings (club_id, trial_id, canonical_class_id) \
         VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(a.club_id)
    .bind(a.trial_id)
    .bind(second_canonical_id)
    .fetch_one(&mut *tx)
    .await
    .expect("second offering");

    let second_line_id: Uuid = sqlx::query_scalar(
        "INSERT INTO entry_lines (club_id, entry_id, trial_class_offering_id, status) \
         VALUES ($1, $2, $3, 'active') RETURNING id",
    )
    .bind(a.club_id)
    .bind(a.entry_id)
    .bind(second_offering_id)
    .fetch_one(&mut *tx)
    .await
    .expect("second entry_line");

    sqlx::query("SAVEPOINT before_bad")
        .execute(&mut *tx)
        .await
        .expect("savepoint");

    let bad = sqlx::query(
        "INSERT INTO entry_line_results \
           (club_id, entry_line_id, qualifying, placement) \
         VALUES ($1, $2, 'nq', 1)",
    )
    .bind(a.club_id)
    .bind(second_line_id)
    .execute(&mut *tx)
    .await;

    assert!(
        bad.is_err(),
        "placement with non-qualifying score must be rejected"
    );

    sqlx::query("ROLLBACK TO SAVEPOINT before_bad")
        .execute(&mut *tx)
        .await
        .expect("rollback savepoint");

    let good = sqlx::query(
        "INSERT INTO entry_line_results \
           (club_id, entry_line_id, score, qualifying, placement) \
         VALUES ($1, $2, 97.0, 'q', 1)",
    )
    .bind(a.club_id)
    .bind(second_line_id)
    .execute(&mut *tx)
    .await;

    assert!(
        good.is_ok(),
        "placement with qualifying score must succeed, got: {good:?}"
    );
}
