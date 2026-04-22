//! Integration tests proving Postgres row-level security actually
//! isolates tenants end-to-end.
//!
//! Every test opens its own transaction via the shared testcontainers
//! fixture, seeds the minimum clubs/users/role grants it needs, then
//! asserts RLS behavior. Transactions roll back on drop so tests do
//! not see each other's data.

use qtrial_shared::testing;
use sqlx::{Executor, Postgres, Transaction};
use uuid::Uuid;

/// Minimum fixture: two clubs and two users with one role grant each,
/// user A scoped to club A and user B scoped to club B. Returns the
/// handles the tests assert against.
struct TwoTenants {
    user_a: Uuid,
    club_a: Uuid,
    user_b: Uuid,
    club_b: Uuid,
}

async fn seed_two_tenants(tx: &mut Transaction<'_, Postgres>) -> TwoTenants {
    let club_a: Uuid = sqlx::query_scalar(
        "INSERT INTO clubs (display_name, abbreviation) VALUES ($1, $2) RETURNING id",
    )
    .bind(testing::unique_name("Club A"))
    .bind("CA")
    .fetch_one(&mut **tx)
    .await
    .expect("insert club A");

    let club_b: Uuid = sqlx::query_scalar(
        "INSERT INTO clubs (display_name, abbreviation) VALUES ($1, $2) RETURNING id",
    )
    .bind(testing::unique_name("Club B"))
    .bind("CB")
    .fetch_one(&mut **tx)
    .await
    .expect("insert club B");

    let user_a: Uuid =
        sqlx::query_scalar("INSERT INTO users (email, display_name) VALUES ($1, $2) RETURNING id")
            .bind(testing::unique_name("user.a") + "@example.test")
            .bind("User A")
            .fetch_one(&mut **tx)
            .await
            .expect("insert user A");

    let user_b: Uuid =
        sqlx::query_scalar("INSERT INTO users (email, display_name) VALUES ($1, $2) RETURNING id")
            .bind(testing::unique_name("user.b") + "@example.test")
            .bind("User B")
            .fetch_one(&mut **tx)
            .await
            .expect("insert user B");

    sqlx::query(
        "INSERT INTO user_club_roles (club_id, user_id, role) VALUES ($1, $2, 'club_admin')",
    )
    .bind(club_a)
    .bind(user_a)
    .execute(&mut **tx)
    .await
    .expect("grant role at club A");

    sqlx::query(
        "INSERT INTO user_club_roles (club_id, user_id, role) VALUES ($1, $2, 'club_admin')",
    )
    .bind(club_b)
    .bind(user_b)
    .execute(&mut **tx)
    .await
    .expect("grant role at club B");

    TwoTenants {
        user_a,
        club_a,
        user_b,
        club_b,
    }
}

/// After SET LOCAL ROLE qtrial_tenant + session vars, apply the
/// tenant context to an already-open transaction. Used when a test
/// wants to seed as owner first, then switch to tenant mode for
/// assertions.
async fn set_tenant_context(tx: &mut Transaction<'_, Postgres>, user_id: Uuid, club_id: Uuid) {
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
async fn owner_role_sees_all_clubs_before_context_switch() {
    // Baseline: confirm the seed helper inserted both clubs and that
    // the owner (qtrial) connection can see them. If this fails the
    // RLS tests below cannot be trusted.
    let pool = testing::pool().await;
    let mut tx = pool.begin().await.expect("begin");

    let seed = seed_two_tenants(&mut tx).await;

    let visible: Vec<Uuid> = sqlx::query_scalar("SELECT id FROM clubs WHERE id IN ($1, $2)")
        .bind(seed.club_a)
        .bind(seed.club_b)
        .fetch_all(&mut *tx)
        .await
        .expect("owner select");

    assert_eq!(visible.len(), 2, "owner should see both seeded clubs");
}

#[tokio::test]
async fn tenant_sees_only_clubs_with_matching_role_or_session_var() {
    let pool = testing::pool().await;
    let mut tx = pool.begin().await.expect("begin");
    let seed = seed_two_tenants(&mut tx).await;

    // User A, in A's club context, should see A but not B.
    set_tenant_context(&mut tx, seed.user_a, seed.club_a).await;

    let visible_a: Vec<Uuid> = sqlx::query_scalar("SELECT id FROM clubs WHERE id IN ($1, $2)")
        .bind(seed.club_a)
        .bind(seed.club_b)
        .fetch_all(&mut *tx)
        .await
        .expect("tenant select");

    assert!(
        visible_a.contains(&seed.club_a),
        "user A at club A should see club A"
    );
    assert!(
        !visible_a.contains(&seed.club_b),
        "user A at club A must NOT see club B"
    );
}

#[tokio::test]
async fn tenant_insert_into_other_clubs_user_roles_is_rejected() {
    let pool = testing::pool().await;
    let mut tx = pool.begin().await.expect("begin");
    let seed = seed_two_tenants(&mut tx).await;

    set_tenant_context(&mut tx, seed.user_a, seed.club_a).await;

    // Attempt: as user A in club A's context, grant a role at club B.
    // WITH CHECK on user_club_roles_tenant should reject because the
    // inserted club_id does not match app.current_club_id.
    let result = sqlx::query(
        "INSERT INTO user_club_roles (club_id, user_id, role) VALUES ($1, $2, 'judge')",
    )
    .bind(seed.club_b)
    .bind(seed.user_a)
    .execute(&mut *tx)
    .await;

    assert!(
        result.is_err(),
        "insert at another club must fail RLS WITH CHECK"
    );
    // Postgres returns sqlstate 42501 (insufficient privilege) for
    // RLS-rejected rows. Assert loosely by inspecting the error text
    // so we do not over-couple the test to the driver's struct shape.
    let err = result.err().unwrap().to_string();
    assert!(
        err.contains("row-level security") || err.contains("42501") || err.contains("policy"),
        "expected RLS policy error, got: {err}"
    );
}

#[tokio::test]
async fn tenant_insert_into_own_club_user_roles_succeeds() {
    let pool = testing::pool().await;
    let mut tx = pool.begin().await.expect("begin");
    let seed = seed_two_tenants(&mut tx).await;

    // Seed an additional user who will receive the new role grant,
    // since user_club_roles_active_uk prevents duplicate active grants
    // of the same role for the same user at the same club.
    let user_c: Uuid =
        sqlx::query_scalar("INSERT INTO users (email, display_name) VALUES ($1, $2) RETURNING id")
            .bind(testing::unique_name("user.c") + "@example.test")
            .bind("User C")
            .fetch_one(&mut *tx)
            .await
            .expect("insert user C");

    set_tenant_context(&mut tx, seed.user_a, seed.club_a).await;

    let result = sqlx::query(
        "INSERT INTO user_club_roles (club_id, user_id, role) VALUES ($1, $2, 'judge')",
    )
    .bind(seed.club_a)
    .bind(user_c)
    .execute(&mut *tx)
    .await;

    assert!(
        result.is_ok(),
        "insert at current_club_id must pass RLS, got: {result:?}"
    );
}

#[tokio::test]
async fn clubs_with_check_rejects_writes_outside_current_club_context() {
    // Critical test for the asymmetric WITH CHECK on clubs. User A
    // holds role grants at BOTH club A and club B. With
    // app.current_club_id set to A, an UPDATE targeting club B must
    // be rejected because id != current_club_id, even though the row
    // is reachable for reads via the OR arm. Under the old symmetric
    // policy this update succeeded; the tightened policy is what
    // prevents cross-tenant writes by a multi-club user.
    let pool = testing::pool().await;
    let mut tx = pool.begin().await.expect("begin");
    let seed = seed_two_tenants(&mut tx).await;

    // Give user A a second role grant at club B so the OR arm of the
    // USING clause does not exclude B from reads.
    sqlx::query(
        "INSERT INTO user_club_roles (club_id, user_id, role) VALUES ($1, $2, 'exhibitor')",
    )
    .bind(seed.club_b)
    .bind(seed.user_a)
    .execute(&mut *tx)
    .await
    .expect("extra role grant at club B");

    set_tenant_context(&mut tx, seed.user_a, seed.club_a).await;

    let result = sqlx::query("UPDATE clubs SET display_name = $1 WHERE id = $2")
        .bind("hacked by wrong context")
        .bind(seed.club_b)
        .execute(&mut *tx)
        .await;

    assert!(
        result.is_err(),
        "UPDATE of club B in club A's context must be rejected by WITH CHECK"
    );
    let err = result.err().unwrap().to_string();
    assert!(
        err.contains("row-level security") || err.contains("42501") || err.contains("policy"),
        "expected RLS policy error, got: {err}"
    );
}

#[tokio::test]
async fn clubs_with_check_rejects_tenant_insert_regardless_of_role() {
    // Defense-in-depth test: even with zero pre-existing role grants
    // elsewhere, a tenant-context INSERT into `clubs` must be rejected
    // because the new row's id will not match the session's
    // current_club_id. Creating clubs is a platform-admin action, not
    // a tenant-role action. This test guards against a future
    // regression that loosens WITH CHECK to accept writes for any
    // reason other than tenant-scope match.
    let pool = testing::pool().await;
    let mut tx = pool.begin().await.expect("begin");

    let club_a: Uuid = sqlx::query_scalar(
        "INSERT INTO clubs (display_name, abbreviation) VALUES ($1, $2) RETURNING id",
    )
    .bind(testing::unique_name("Club A"))
    .bind("CA")
    .fetch_one(&mut *tx)
    .await
    .expect("insert club A");

    let user_a: Uuid = sqlx::query_scalar(
        "INSERT INTO users (email, display_name) VALUES ($1, $2) RETURNING id",
    )
    .bind(testing::unique_name("lone.user") + "@example.test")
    .bind("Lone User")
    .fetch_one(&mut *tx)
    .await
    .expect("insert lone user");

    sqlx::query(
        "INSERT INTO user_club_roles (club_id, user_id, role) VALUES ($1, $2, 'club_admin')",
    )
    .bind(club_a)
    .bind(user_a)
    .execute(&mut *tx)
    .await
    .expect("grant role at club A");

    set_tenant_context(&mut tx, user_a, club_a).await;

    // Tenant tries to create a new club. The new row's id is a fresh
    // UUID that cannot possibly equal current_club_id, so WITH CHECK
    // rejects.
    let result = sqlx::query("INSERT INTO clubs (display_name) VALUES ($1)")
        .bind(testing::unique_name("Club C"))
        .execute(&mut *tx)
        .await;

    assert!(
        result.is_err(),
        "tenant-role INSERT into clubs must be rejected by WITH CHECK"
    );
    let err = result.err().unwrap().to_string();
    assert!(
        err.contains("row-level security") || err.contains("42501") || err.contains("policy"),
        "expected RLS policy error, got: {err}"
    );
}

#[tokio::test]
async fn my_clubs_picker_returns_all_user_role_grants_without_club_context() {
    // Regression guard for a session-3 bug the commit-1 tightening
    // fixed. The "my clubs" picker flow is: user logs in, session
    // has current_user_id but no current_club_id yet, query
    // `SELECT * FROM clubs` to populate the picker. Under the
    // session-3 user_club_roles policy (club_id = current_club_id
    // only), the subquery in clubs USING returned zero rows because
    // user_club_roles RLS filtered out every grant not at the current
    // club, which was NULL. The picker therefore returned zero clubs.
    //
    // Session-4 widens user_club_roles USING to include
    // `user_id = current_user_id`, restoring the intended behavior:
    // with NULL current_club_id, a user sees all clubs they hold a
    // role at.
    let pool = testing::pool().await;
    let mut tx = pool.begin().await.expect("begin");
    let seed = seed_two_tenants(&mut tx).await;

    // Second role grant for user A at club B so the picker should
    // return two clubs for this user.
    sqlx::query(
        "INSERT INTO user_club_roles (club_id, user_id, role) VALUES ($1, $2, 'exhibitor')",
    )
    .bind(seed.club_b)
    .bind(seed.user_a)
    .execute(&mut *tx)
    .await
    .expect("extra role grant at club B");

    // Set only current_user_id; leave current_club_id unset (the
    // pre-picker state). Switch to tenant role directly rather than
    // via set_tenant_context, since set_tenant_context requires a
    // club id.
    sqlx::query("SELECT set_config('app.current_user_id', $1, true)")
        .bind(seed.user_a.to_string())
        .execute(&mut *tx)
        .await
        .expect("set current_user_id");
    sqlx::query("SELECT set_config('app.current_club_id', '', true)")
        .execute(&mut *tx)
        .await
        .expect("clear current_club_id");
    sqlx::query("SET LOCAL ROLE qtrial_tenant")
        .execute(&mut *tx)
        .await
        .expect("set tenant role");

    let visible: Vec<Uuid> = sqlx::query_scalar("SELECT id FROM clubs ORDER BY id")
        .fetch_all(&mut *tx)
        .await
        .expect("my clubs picker query");

    assert!(
        visible.contains(&seed.club_a),
        "my clubs picker must include club A"
    );
    assert!(
        visible.contains(&seed.club_b),
        "my clubs picker must include club B"
    );
    assert_eq!(
        visible.len(),
        2,
        "picker must return exactly the user's two role-granted clubs, got {visible:?}"
    );
}

#[tokio::test]
async fn users_self_policy_hides_other_users() {
    let pool = testing::pool().await;
    let mut tx = pool.begin().await.expect("begin");
    let seed = seed_two_tenants(&mut tx).await;

    set_tenant_context(&mut tx, seed.user_a, seed.club_a).await;

    let visible: Vec<Uuid> = sqlx::query_scalar("SELECT id FROM users WHERE id IN ($1, $2)")
        .bind(seed.user_a)
        .bind(seed.user_b)
        .fetch_all(&mut *tx)
        .await
        .expect("tenant select users");

    assert_eq!(
        visible,
        vec![seed.user_a],
        "users self-RLS must return only the authenticated user's row"
    );
}
