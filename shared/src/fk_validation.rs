//! Cross-tenant FK target validation.
//!
//! Postgres FK integrity checks bypass RLS on the referenced table,
//! which leaves a cross-tenant write hole: if a UUID belonging to
//! tenant B is supplied to an INSERT running in tenant A's context,
//! the FK check succeeds because the target exists somewhere in the
//! database, and RLS on tenant A's new row checks only the row
//! itself, not the rows it points at. The SELECT-side defense (RLS
//! hides cross-tenant rows) works only as long as the UUID never
//! leaks via other channels.
//!
//! This module is defense-in-depth: API handlers call
//! [`verify_fk_targets_in_tenant`] before any tenant-scoped insert
//! that carries FK columns. The helper runs an RLS-scoped EXISTS
//! query against every target's parent table; any target that is
//! not visible under the current tenant context is rejected.
//! Because the query itself runs under RLS, cross-tenant rows
//! naturally return false, and we need not distinguish "does not
//! exist" from "exists in another tenant" at the application
//! layer. That distinction would also be an information leak.
//!
//! **Caller contract:** the passed executor must already be in a
//! tenant context, specifically with `SET LOCAL ROLE qtrial_tenant`
//! and `app.current_club_id` set for the current tenant. The helpers
//! in [`crate::tenancy`] (`begin_as_tenant`, `with_tenant_context`)
//! establish exactly this shape. If the helper is called from an
//! owner-role session (migrations, platform admin paths), RLS is
//! bypassed and the validation becomes a no-op, silently approving
//! cross-tenant targets. Do not call from owner-role sessions.

use thiserror::Error;
use uuid::Uuid;

/// Tables that can be FK targets of tenant-scoped INSERTs. The
/// variants are fixed at compile time so the SQL matches on a
/// closed set of known-safe identifiers; user input never reaches
/// the SQL's table-name position.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TenantTable {
    Event,
    EventDay,
    Trial,
    TrialClassOffering,
    Judge,
    Owner,
    Dog,
    Team,
    Entry,
    EntryLine,
    /// Special case: users are not tenant-scoped. A user is a valid
    /// FK target for this tenant if they hold an active role grant
    /// at the current club, not if their row has a matching club_id.
    User,
}

impl TenantTable {
    /// The SQL identifier for this table. Used in the validation
    /// query's CASE dispatch; all values here are hard-coded
    /// literals under our control.
    fn table_name(self) -> &'static str {
        match self {
            Self::Event => "events",
            Self::EventDay => "event_days",
            Self::Trial => "trials",
            Self::TrialClassOffering => "trial_class_offerings",
            Self::Judge => "judges",
            Self::Owner => "owners",
            Self::Dog => "dogs",
            Self::Team => "teams",
            Self::Entry => "entries",
            Self::EntryLine => "entry_lines",
            Self::User => "users",
        }
    }

    fn from_table_name(name: &str) -> Option<Self> {
        match name {
            "events" => Some(Self::Event),
            "event_days" => Some(Self::EventDay),
            "trials" => Some(Self::Trial),
            "trial_class_offerings" => Some(Self::TrialClassOffering),
            "judges" => Some(Self::Judge),
            "owners" => Some(Self::Owner),
            "dogs" => Some(Self::Dog),
            "teams" => Some(Self::Team),
            "entries" => Some(Self::Entry),
            "entry_lines" => Some(Self::EntryLine),
            "users" => Some(Self::User),
            _ => None,
        }
    }
}

/// One FK target to check. The `table` field tells the helper which
/// parent table to look in; `id` is the target row's UUID.
#[derive(Clone, Copy, Debug)]
pub struct FkTarget {
    pub table: TenantTable,
    pub id: Uuid,
}

/// Result of a FK validation check.
///
/// `InvalidOrCrossTenant` is deliberately combined: from the
/// caller's perspective the two cases are equivalent (the insert
/// must fail) and distinguishing them would leak whether a given
/// UUID exists in some other tenant, which is a sensitivity we do
/// not want to expose.
#[derive(Debug, Error)]
pub enum FkValidationError {
    #[error("FK target invalid or belongs to a different tenant: {table:?} id={id}")]
    InvalidOrCrossTenant { table: TenantTable, id: Uuid },

    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

/// Verify that every target in `targets` refers to a row visible in
/// the current tenant context. Returns `Ok(())` if all targets are
/// valid; otherwise returns `InvalidOrCrossTenant` identifying the
/// first offending target.
///
/// The whole batch is evaluated in a single SQL round trip via an
/// `unnest($text_array, $uuid_array)` driver that dispatches on
/// table name through a CASE expression. The CASE arms each run an
/// `EXISTS` subquery against the target's parent table; because the
/// caller's session has tenant RLS applied, any row not belonging
/// to the current tenant is invisible and `EXISTS` returns false.
/// Soft-deleted targets also return false, matching the intent that
/// a new FK should never point at a deleted parent.
///
/// For `TenantTable::User` the CASE arm queries `user_club_roles`
/// instead: a user is a valid FK target if they hold an active role
/// grant at the current club. RLS on `user_club_roles` permits the
/// tenant to see grants at their current club, so this check works
/// under the same tenant role.
pub async fn verify_fk_targets_in_tenant<'c>(
    executor: impl sqlx::PgExecutor<'c>,
    targets: &[FkTarget],
) -> Result<(), FkValidationError> {
    if targets.is_empty() {
        return Ok(());
    }

    // Parallel arrays: table names and UUIDs, positional pairs.
    // unnest zips them into rows the CASE expression walks.
    let table_names: Vec<&'static str> = targets.iter().map(|t| t.table.table_name()).collect();
    let ids: Vec<Uuid> = targets.iter().map(|t| t.id).collect();

    // The CASE dispatches on the fixed set of known table names to
    // an EXISTS subquery against the appropriate parent. RLS on the
    // parent filters cross-tenant rows; soft-deleted parents are
    // excluded explicitly because deleted_at filtering does not
    // come for free from RLS.
    //
    // The `users` arm is different: users have no club_id, so we
    // validate via an active role grant at the current club. RLS on
    // user_club_roles lets the tenant see grants at current_club_id,
    // so the subquery returns true when the user is properly
    // associated with the current tenant.
    let rows: Vec<(String, Uuid, bool)> = sqlx::query_as(
        r#"
        SELECT
            t.expected_table::text AS table_name,
            t.expected_id::uuid    AS target_id,
            CASE t.expected_table
                WHEN 'events' THEN
                    EXISTS (SELECT 1 FROM events
                            WHERE id = t.expected_id AND deleted_at IS NULL)
                WHEN 'event_days' THEN
                    EXISTS (SELECT 1 FROM event_days
                            WHERE id = t.expected_id AND deleted_at IS NULL)
                WHEN 'trials' THEN
                    EXISTS (SELECT 1 FROM trials
                            WHERE id = t.expected_id AND deleted_at IS NULL)
                WHEN 'trial_class_offerings' THEN
                    EXISTS (SELECT 1 FROM trial_class_offerings
                            WHERE id = t.expected_id AND deleted_at IS NULL)
                WHEN 'judges' THEN
                    EXISTS (SELECT 1 FROM judges
                            WHERE id = t.expected_id AND deleted_at IS NULL)
                WHEN 'owners' THEN
                    EXISTS (SELECT 1 FROM owners
                            WHERE id = t.expected_id AND deleted_at IS NULL)
                WHEN 'dogs' THEN
                    EXISTS (SELECT 1 FROM dogs
                            WHERE id = t.expected_id AND deleted_at IS NULL)
                WHEN 'teams' THEN
                    EXISTS (SELECT 1 FROM teams
                            WHERE id = t.expected_id AND deleted_at IS NULL)
                WHEN 'entries' THEN
                    EXISTS (SELECT 1 FROM entries
                            WHERE id = t.expected_id AND deleted_at IS NULL)
                WHEN 'entry_lines' THEN
                    EXISTS (SELECT 1 FROM entry_lines
                            WHERE id = t.expected_id AND deleted_at IS NULL)
                WHEN 'users' THEN
                    EXISTS (
                        SELECT 1 FROM user_club_roles ucr
                        WHERE ucr.user_id = t.expected_id
                          AND ucr.club_id = NULLIF(
                              current_setting('app.current_club_id', TRUE), ''
                          )::uuid
                          AND ucr.revoked_at IS NULL
                          AND ucr.deleted_at IS NULL
                    )
                ELSE FALSE
            END AS valid
        FROM unnest($1::text[], $2::uuid[]) AS t(expected_table, expected_id)
        "#,
    )
    .bind(&table_names[..])
    .bind(&ids[..])
    .fetch_all(executor)
    .await?;

    for (table_name, id, valid) in rows {
        if !valid {
            // The table_name came out of our own enum; unwrap is
            // safe. If it somehow isn't recognized, report the raw
            // value rather than panicking: returning an error is
            // more graceful than losing the validation result.
            let table = TenantTable::from_table_name(&table_name).unwrap_or(
                // Defensive: should not be reachable because the
                // round-trip preserves what we sent in, but if it
                // somehow did we still reject.
                TenantTable::User,
            );
            return Err(FkValidationError::InvalidOrCrossTenant { table, id });
        }
    }
    Ok(())
}
