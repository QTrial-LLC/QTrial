//! Integration tests for cross-tenant FK target validation.
//!
//! The helper under test, `verify_fk_targets_in_tenant`, closes the
//! gap left by Postgres's FK integrity checks bypassing RLS: a
//! tenant-context INSERT carrying a UUID that points at another
//! tenant's row would otherwise succeed at the DB layer. These
//! tests seed two distinct tenants and exercise the helper against
//! mixed target lists.

use qtrial_shared::fk_validation::{
    verify_fk_targets_in_tenant, FkTarget, FkValidationError, TenantTable,
};
use qtrial_shared::testing;
use sqlx::{Executor, Postgres, Transaction};
use uuid::Uuid;

/// A pair of fully-seeded tenants. The struct holds the UUIDs tests
/// need to reference across tenants; cross-tenant targets are just
/// the other tenant's equivalents.
struct CrossTenantPair {
    a: TenantStack,
    b: TenantStack,
}

// Several TenantStack fields are not referenced by every test; they
// exist so future tests against more FK target types can reuse the
// fixture without reshaping the struct.
#[allow(dead_code)]
struct TenantStack {
    club_id: Uuid,
    admin_user_id: Uuid,
    unrelated_user_id: Uuid,
    event_id: Uuid,
    event_day_id: Uuid,
    trial_id: Uuid,
    offering_id: Uuid,
    judge_id: Uuid,
    owner_id: Uuid,
    dog_id: Uuid,
    entry_id: Uuid,
    entry_line_id: Uuid,
}

async fn seed_tenant(tx: &mut Transaction<'_, Postgres>, label: &str) -> TenantStack {
    let club_id: Uuid = sqlx::query_scalar(
        "INSERT INTO clubs (display_name, abbreviation) VALUES ($1, $2) RETURNING id",
    )
    .bind(testing::unique_name(label))
    .bind(&label[..2])
    .fetch_one(&mut **tx)
    .await
    .expect("insert club");

    let admin_user_id: Uuid = sqlx::query_scalar(
        "INSERT INTO users (email, display_name) VALUES ($1, $2) RETURNING id",
    )
    .bind(testing::unique_name("admin") + "@example.test")
    .bind(format!("{label} admin"))
    .fetch_one(&mut **tx)
    .await
    .expect("insert admin user");

    // A user who exists globally but has no role at this club. Used
    // in the "user without a role at current club" test.
    let unrelated_user_id: Uuid = sqlx::query_scalar(
        "INSERT INTO users (email, display_name) VALUES ($1, $2) RETURNING id",
    )
    .bind(testing::unique_name("unrelated") + "@example.test")
    .bind("Unrelated User")
    .fetch_one(&mut **tx)
    .await
    .expect("insert unrelated user");

    sqlx::query(
        "INSERT INTO user_club_roles (club_id, user_id, role) \
         VALUES ($1, $2, 'trial_secretary')",
    )
    .bind(club_id)
    .bind(admin_user_id)
    .execute(&mut **tx)
    .await
    .expect("grant role to admin");

    let registry_id: Uuid =
        sqlx::query_scalar("SELECT id FROM registries WHERE code = 'AKC'")
            .fetch_one(&mut **tx)
            .await
            .expect("AKC registry id");

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

    let canonical_class_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM canonical_classes WHERE code = 'akc_rally_novice_a'",
    )
    .fetch_one(&mut **tx)
    .await
    .expect("rally novice a canonical id");

    let offering_id: Uuid = sqlx::query_scalar(
        "INSERT INTO trial_class_offerings (club_id, trial_id, canonical_class_id) \
         VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(club_id)
    .bind(trial_id)
    .bind(canonical_class_id)
    .fetch_one(&mut **tx)
    .await
    .expect("insert offering");

    let judge_id: Uuid = sqlx::query_scalar(
        "INSERT INTO judges (club_id, last_name, first_name) \
         VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(club_id)
    .bind(testing::unique_name(&format!("{label}-Judge")))
    .bind("Pat")
    .fetch_one(&mut **tx)
    .await
    .expect("insert judge");

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

    let entry_id: Uuid = sqlx::query_scalar(
        "INSERT INTO entries (club_id, event_id, dog_id, exhibitor_user_id, owner_id) \
         VALUES ($1, $2, $3, $4, $5) RETURNING id",
    )
    .bind(club_id)
    .bind(event_id)
    .bind(dog_id)
    .bind(admin_user_id)
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

    TenantStack {
        club_id,
        admin_user_id,
        unrelated_user_id,
        event_id,
        event_day_id,
        trial_id,
        offering_id,
        judge_id,
        owner_id,
        dog_id,
        entry_id,
        entry_line_id,
    }
}

async fn seed_cross_tenant_pair(tx: &mut Transaction<'_, Postgres>) -> CrossTenantPair {
    let a = seed_tenant(tx, "Tenant A").await;
    let b = seed_tenant(tx, "Tenant B").await;
    CrossTenantPair { a, b }
}

async fn enter_tenant_context(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    club_id: Uuid,
) {
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

fn assert_invalid(
    result: Result<(), FkValidationError>,
    expected_table: TenantTable,
    expected_id: Uuid,
) {
    match result {
        Err(FkValidationError::InvalidOrCrossTenant { table, id })
            if table == expected_table && id == expected_id => {}
        other => panic!(
            "expected InvalidOrCrossTenant({expected_table:?}, {expected_id}), got {other:?}"
        ),
    }
}

#[tokio::test]
async fn happy_path_accepts_all_targets_from_current_tenant() {
    let pool = testing::pool().await;
    let mut tx = pool.begin().await.expect("begin");

    let pair = seed_cross_tenant_pair(&mut tx).await;
    enter_tenant_context(&mut tx, pair.a.admin_user_id, pair.a.club_id).await;

    let targets = vec![
        FkTarget { table: TenantTable::Event, id: pair.a.event_id },
        FkTarget { table: TenantTable::Dog, id: pair.a.dog_id },
        FkTarget { table: TenantTable::Owner, id: pair.a.owner_id },
        FkTarget { table: TenantTable::TrialClassOffering, id: pair.a.offering_id },
        FkTarget { table: TenantTable::Judge, id: pair.a.judge_id },
    ];

    verify_fk_targets_in_tenant(&mut *tx, &targets)
        .await
        .expect("all targets belong to tenant A");
}

#[tokio::test]
async fn cross_tenant_owner_is_rejected() {
    let pool = testing::pool().await;
    let mut tx = pool.begin().await.expect("begin");

    let pair = seed_cross_tenant_pair(&mut tx).await;
    enter_tenant_context(&mut tx, pair.a.admin_user_id, pair.a.club_id).await;

    let targets = vec![FkTarget {
        table: TenantTable::Owner,
        id: pair.b.owner_id,
    }];

    assert_invalid(
        verify_fk_targets_in_tenant(&mut *tx, &targets).await,
        TenantTable::Owner,
        pair.b.owner_id,
    );
}

#[tokio::test]
async fn cross_tenant_dog_is_rejected() {
    let pool = testing::pool().await;
    let mut tx = pool.begin().await.expect("begin");

    let pair = seed_cross_tenant_pair(&mut tx).await;
    enter_tenant_context(&mut tx, pair.a.admin_user_id, pair.a.club_id).await;

    let targets = vec![FkTarget {
        table: TenantTable::Dog,
        id: pair.b.dog_id,
    }];

    assert_invalid(
        verify_fk_targets_in_tenant(&mut *tx, &targets).await,
        TenantTable::Dog,
        pair.b.dog_id,
    );
}

#[tokio::test]
async fn cross_tenant_trial_class_offering_is_rejected() {
    // This is the realistic attack shape: an exhibitor-submitted
    // entry_line carries a trial_class_offering_id. If the API
    // doesn't validate, a malicious client could supply another
    // tenant's offering id and get a cross-tenant entry_line.
    let pool = testing::pool().await;
    let mut tx = pool.begin().await.expect("begin");

    let pair = seed_cross_tenant_pair(&mut tx).await;
    enter_tenant_context(&mut tx, pair.a.admin_user_id, pair.a.club_id).await;

    let targets = vec![FkTarget {
        table: TenantTable::TrialClassOffering,
        id: pair.b.offering_id,
    }];

    assert_invalid(
        verify_fk_targets_in_tenant(&mut *tx, &targets).await,
        TenantTable::TrialClassOffering,
        pair.b.offering_id,
    );
}

#[tokio::test]
async fn nonexistent_uuid_is_rejected_without_distinct_error() {
    // Same rejection as cross-tenant. No information leak about
    // whether the UUID exists somewhere in the database.
    let pool = testing::pool().await;
    let mut tx = pool.begin().await.expect("begin");

    let pair = seed_cross_tenant_pair(&mut tx).await;
    enter_tenant_context(&mut tx, pair.a.admin_user_id, pair.a.club_id).await;

    let phantom = Uuid::new_v4();
    let targets = vec![FkTarget {
        table: TenantTable::Event,
        id: phantom,
    }];

    assert_invalid(
        verify_fk_targets_in_tenant(&mut *tx, &targets).await,
        TenantTable::Event,
        phantom,
    );
}

#[tokio::test]
async fn user_with_role_at_current_club_is_accepted() {
    let pool = testing::pool().await;
    let mut tx = pool.begin().await.expect("begin");

    let pair = seed_cross_tenant_pair(&mut tx).await;
    enter_tenant_context(&mut tx, pair.a.admin_user_id, pair.a.club_id).await;

    // admin_user_id has an active trial_secretary grant at club_a.
    let targets = vec![FkTarget {
        table: TenantTable::User,
        id: pair.a.admin_user_id,
    }];

    verify_fk_targets_in_tenant(&mut *tx, &targets)
        .await
        .expect("admin user has a role at current club");
}

#[tokio::test]
async fn user_without_role_at_current_club_is_rejected() {
    let pool = testing::pool().await;
    let mut tx = pool.begin().await.expect("begin");

    let pair = seed_cross_tenant_pair(&mut tx).await;
    enter_tenant_context(&mut tx, pair.a.admin_user_id, pair.a.club_id).await;

    // unrelated_user exists globally but has no user_club_roles row
    // for any club. A FK reference from this tenant's scope would
    // not be a correctness disaster in itself (users are global),
    // but the application's write paths (exhibitor_user_id on
    // entries, etc.) expect the user to be connected to this club.
    let targets = vec![FkTarget {
        table: TenantTable::User,
        id: pair.a.unrelated_user_id,
    }];

    assert_invalid(
        verify_fk_targets_in_tenant(&mut *tx, &targets).await,
        TenantTable::User,
        pair.a.unrelated_user_id,
    );
}

#[tokio::test]
async fn user_with_role_only_at_other_club_is_rejected() {
    // Tenant B's admin has a role at tenant B. From tenant A's
    // context, that user is NOT a valid FK target because their
    // only active grant is at tenant B, not at current_club_id.
    let pool = testing::pool().await;
    let mut tx = pool.begin().await.expect("begin");

    let pair = seed_cross_tenant_pair(&mut tx).await;
    enter_tenant_context(&mut tx, pair.a.admin_user_id, pair.a.club_id).await;

    let targets = vec![FkTarget {
        table: TenantTable::User,
        id: pair.b.admin_user_id,
    }];

    assert_invalid(
        verify_fk_targets_in_tenant(&mut *tx, &targets).await,
        TenantTable::User,
        pair.b.admin_user_id,
    );
}

#[tokio::test]
async fn batch_validation_identifies_the_specific_failing_target() {
    let pool = testing::pool().await;
    let mut tx = pool.begin().await.expect("begin");

    let pair = seed_cross_tenant_pair(&mut tx).await;
    enter_tenant_context(&mut tx, pair.a.admin_user_id, pair.a.club_id).await;

    // Four targets: three valid for tenant A, one cross-tenant.
    // The helper should reject with the exact (table, id) of the
    // failing target, not a generic batch-failure message.
    let targets = vec![
        FkTarget { table: TenantTable::Event, id: pair.a.event_id },
        FkTarget { table: TenantTable::Dog, id: pair.b.dog_id },
        FkTarget { table: TenantTable::Owner, id: pair.a.owner_id },
        FkTarget { table: TenantTable::TrialClassOffering, id: pair.a.offering_id },
    ];

    assert_invalid(
        verify_fk_targets_in_tenant(&mut *tx, &targets).await,
        TenantTable::Dog,
        pair.b.dog_id,
    );
}

#[tokio::test]
async fn empty_targets_list_is_accepted() {
    // Calling the helper with an empty list is a no-op and must
    // not round-trip to the database. Guards against regression
    // where a caller with nothing to validate pays a query cost.
    let pool = testing::pool().await;
    let mut tx = pool.begin().await.expect("begin");

    let pair = seed_cross_tenant_pair(&mut tx).await;
    enter_tenant_context(&mut tx, pair.a.admin_user_id, pair.a.club_id).await;

    verify_fk_targets_in_tenant(&mut *tx, &[])
        .await
        .expect("empty batch is a no-op");
}
