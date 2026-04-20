//! Tenant context enforcement.
//!
//! Every request that should be subject to row-level security opens a
//! transaction and, as its first work, calls [`begin_as_tenant`] or
//! [`with_tenant_context`]. These helpers:
//!
//! 1. Start a transaction on the pool.
//! 2. Set `app.current_user_id` and `app.current_club_id` session
//!    variables via `set_config(name, value, true)` (the `true`
//!    argument scopes the setting to the transaction, equivalent to
//!    `SET LOCAL`).
//! 3. Issue `SET LOCAL ROLE offleash_tenant`, which takes the
//!    connection out of the table-owner identity and subjects it to
//!    the RLS policies created in the Phase 0 tenancy migration.
//!
//! Because every part of that setup is transaction-scoped, there is no
//! way for a request's tenant context to leak into a later request
//! that reuses the same pooled connection: closing or rolling back
//! the transaction implicitly resets the role and settings.
//!
//! Platform admin paths (not built in this session) will deliberately
//! skip this helper. Those paths run as the `offleash` role which
//! owns the tables and bypasses RLS, and they MUST log every access.

use sqlx::{PgPool, Postgres, Transaction};
use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;

/// Boxed future alias used by [`with_tenant_context`]'s closure bound.
/// Keeping the alias at the module level lets callers write the bound
/// without repeating the full `Pin<Box<dyn Future<...>>>` incantation.
pub type TenantFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, sqlx::Error>> + Send + 'a>>;

/// Open a transaction with tenant context applied and return it to the
/// caller. The caller is responsible for committing on success;
/// dropping without a commit rolls back, which is the correct behavior
/// for tests and for any code path that hits an early return.
///
/// Prefer [`with_tenant_context`] in production request handlers where
/// the commit-on-Ok / rollback-on-Err discipline should be enforced by
/// the helper rather than by the caller.
pub async fn begin_as_tenant(
    pool: &PgPool,
    user_id: Uuid,
    club_id: Uuid,
) -> Result<Transaction<'static, Postgres>, sqlx::Error> {
    let mut tx = pool.begin().await?;

    // set_config(name, value, is_local=true) is the parameterizable
    // equivalent of `SET LOCAL name = value;`. We cannot bind
    // parameters into a bare SET statement, so this form is required.
    sqlx::query("SELECT set_config('app.current_user_id', $1, true)")
        .bind(user_id.to_string())
        .execute(&mut *tx)
        .await?;

    sqlx::query("SELECT set_config('app.current_club_id', $1, true)")
        .bind(club_id.to_string())
        .execute(&mut *tx)
        .await?;

    // SET LOCAL ROLE cannot be parameterized either, but the target
    // role name is a compile-time constant under our control, so the
    // string is safe.
    sqlx::query("SET LOCAL ROLE offleash_tenant")
        .execute(&mut *tx)
        .await?;

    Ok(tx)
}

/// Run a closure against a tenant-scoped transaction. Commits on Ok,
/// rolls back on Err. The closure receives `&mut Transaction` so it
/// can execute queries and hand the transaction to `sqlx::query!`
/// calls. The closure must return a `TenantFuture`, which in practice
/// means wrapping the inner async block with `Box::pin(async move {
/// ... })`.
///
/// Typical use at a request handler:
///
/// ```no_run
/// # use offleash_shared::tenancy::{with_tenant_context, TenantFuture};
/// # use sqlx::PgPool;
/// # use uuid::Uuid;
/// # async fn example(pool: &PgPool, user_id: Uuid, club_id: Uuid) -> Result<(), sqlx::Error> {
/// let club_count: i64 = with_tenant_context(pool, user_id, club_id, |tx| {
///     Box::pin(async move {
///         let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM clubs")
///             .fetch_one(&mut **tx)
///             .await?;
///         Ok(count)
///     })
/// })
/// .await?;
/// # Ok(()) }
/// ```
pub async fn with_tenant_context<T, F>(
    pool: &PgPool,
    user_id: Uuid,
    club_id: Uuid,
    f: F,
) -> Result<T, sqlx::Error>
where
    T: Send,
    F: for<'c> FnOnce(&'c mut Transaction<'static, Postgres>) -> TenantFuture<'c, T>,
{
    let mut tx = begin_as_tenant(pool, user_id, club_id).await?;
    let result = f(&mut tx).await;
    match result {
        Ok(value) => {
            tx.commit().await?;
            Ok(value)
        }
        Err(e) => Err(e),
    }
}
