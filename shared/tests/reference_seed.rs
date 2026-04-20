//! Verify the Phase 0 reference-data seeds loaded correctly.
//!
//! These tests are a regression net for session 2's seed migrations.
//! If a future seed migration accidentally deletes or reshapes any of
//! the canonical rows, one of these tests should fail before the
//! change lands. The tests also exercise the permissive-read policies
//! on reference tables from a tenant context, confirming that a
//! seeded-and-locked-down tenant can still read the full catalog.

use offleash_shared::testing;
use uuid::Uuid;

#[tokio::test]
async fn registries_seed_loaded_exactly_akc() {
    let pool = testing::pool().await;
    let pool = &pool;

    let codes: Vec<String> = sqlx::query_scalar("SELECT code FROM registries ORDER BY code")
        .fetch_all(pool)
        .await
        .expect("select registries");

    assert_eq!(codes, vec!["AKC".to_string()]);
}

#[tokio::test]
async fn akc_fee_schedules_have_2025_and_2026_rows_per_sport() {
    let pool = testing::pool().await;
    let pool = &pool;

    // Fetch NUMERIC as text via SQL cast to avoid a decimal-crate
    // dependency just for the sake of an integration test assertion.
    let rows: Vec<(String, i32, String, String)> = sqlx::query_as(
        "SELECT sport::text,
                effective_year,
                service_fee_first_entry::text,
                service_fee_additional::text
         FROM akc_fee_schedules
         ORDER BY sport, effective_year",
    )
    .fetch_all(pool)
    .await
    .expect("select fee schedules");

    assert_eq!(rows.len(), 4, "expected 4 fee-schedule rows, got {rows:?}");

    let find = |sport: &str, year: i32| {
        rows.iter()
            .find(|(s, y, _, _)| s == sport && *y == year)
            .unwrap_or_else(|| panic!("missing row: {sport} {year}"))
    };

    // Obedience/Conformation 2025: $3.00 first / $3.50 additional.
    let (_, _, first, additional) = find("obedience_conformation", 2025);
    assert_eq!(first, "3.00");
    assert_eq!(additional, "3.50");

    // Obedience/Conformation 2026: $4.00 first / $4.50 additional.
    let (_, _, first, additional) = find("obedience_conformation", 2026);
    assert_eq!(first, "4.00");
    assert_eq!(additional, "4.50");

    // Rally 2025: flat $3.50.
    let (_, _, first, additional) = find("rally", 2025);
    assert_eq!(first, "3.50");
    assert_eq!(additional, "3.50");

    // Rally 2026: flat $4.50.
    let (_, _, first, additional) = find("rally", 2026);
    assert_eq!(first, "4.50");
    assert_eq!(additional, "4.50");
}

#[tokio::test]
async fn canonical_classes_counts_match_session_two_scope() {
    let pool = testing::pool().await;
    let pool = &pool;

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM canonical_classes")
        .fetch_one(pool)
        .await
        .expect("count canonical_classes");
    assert_eq!(total, 23, "session 2 scope is exactly 23 classes");

    let obedience: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM canonical_classes WHERE sport = 'obedience'",
    )
    .fetch_one(pool)
    .await
    .expect("count obedience");
    assert_eq!(obedience, 14, "14 Obedience classes: 6 Regular + 5 OT + 3 Preferred");

    let rally: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM canonical_classes WHERE sport = 'rally'")
            .fetch_one(pool)
            .await
            .expect("count rally");
    assert_eq!(rally, 9, "9 Rally Regular classes");
}

#[tokio::test]
async fn rally_choice_is_seeded_with_ten_leg_title_rule() {
    let pool = testing::pool().await;
    let pool = &pool;

    let row: Option<(String, Option<i32>, String, bool)> = sqlx::query_as(
        "SELECT display_name,
                legs_required_for_title,
                ab_eligibility_rule::text,
                has_jumps
         FROM canonical_classes
         WHERE code = 'akc_rally_choice'",
    )
    .fetch_optional(pool)
    .await
    .expect("select rally choice");

    let (display_name, legs, ab_rule, has_jumps) =
        row.expect("akc_rally_choice must be seeded");

    assert_eq!(display_name, "Rally Choice");
    assert_eq!(legs, Some(10));
    assert_eq!(ab_rule, "none");
    assert!(!has_jumps, "Rally Choice has no jumps per AKC Rally Regs Ch 3 §18");
}

#[tokio::test]
async fn ab_eligibility_rules_match_domain_glossary() {
    let pool = testing::pool().await;
    let pool = &pool;

    // Obedience: handler_based for A classes, none for B and single
    // classes. Spot-check Novice A / Novice B / Preferred Novice.
    let novice_a: String = sqlx::query_scalar(
        "SELECT ab_eligibility_rule::text FROM canonical_classes WHERE code = 'akc_obed_novice_a'",
    )
    .fetch_one(pool)
    .await
    .expect("obed novice_a");
    assert_eq!(novice_a, "handler_based");

    let novice_b: String = sqlx::query_scalar(
        "SELECT ab_eligibility_rule::text FROM canonical_classes WHERE code = 'akc_obed_novice_b'",
    )
    .fetch_one(pool)
    .await
    .expect("obed novice_b");
    assert_eq!(novice_b, "none");

    // Rally: dog_based for Novice A, dog_and_handler_based for
    // Advanced A / Excellent A, none for B classes and single classes.
    let rally_novice_a: String = sqlx::query_scalar(
        "SELECT ab_eligibility_rule::text FROM canonical_classes WHERE code = 'akc_rally_novice_a'",
    )
    .fetch_one(pool)
    .await
    .expect("rally novice_a");
    assert_eq!(rally_novice_a, "dog_based");

    let rally_advanced_a: String = sqlx::query_scalar(
        "SELECT ab_eligibility_rule::text FROM canonical_classes WHERE code = 'akc_rally_advanced_a'",
    )
    .fetch_one(pool)
    .await
    .expect("rally advanced_a");
    assert_eq!(rally_advanced_a, "dog_and_handler_based");

    let rally_excellent_a: String = sqlx::query_scalar(
        "SELECT ab_eligibility_rule::text FROM canonical_classes WHERE code = 'akc_rally_excellent_a'",
    )
    .fetch_one(pool)
    .await
    .expect("rally excellent_a");
    assert_eq!(rally_excellent_a, "dog_and_handler_based");
}

#[tokio::test]
async fn tenant_role_can_read_reference_tables() {
    // Permissive-read policy check. From a tenant context with no role
    // grants at all, we must still be able to read every reference
    // row. If this regresses, tenant requests cannot render their own
    // entries because canonical class lookups would 0-row silently.
    let unused_user = Uuid::new_v4();
    let unused_club = Uuid::new_v4();
    let mut tx = testing::begin_as_tenant(unused_user, unused_club)
        .await
        .expect("begin tenant");

    let registry_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM registries")
        .fetch_one(&mut *tx)
        .await
        .expect("tenant reads registries");
    assert_eq!(registry_count, 1);

    let class_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM canonical_classes")
        .fetch_one(&mut *tx)
        .await
        .expect("tenant reads canonical_classes");
    assert_eq!(class_count, 23);

    let fee_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM akc_fee_schedules")
        .fetch_one(&mut *tx)
        .await
        .expect("tenant reads akc_fee_schedules");
    assert_eq!(fee_count, 4);
}
