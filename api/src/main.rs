//! OffLeash API skeleton.
//!
//! Phase 0 scaffolding only. The real axum + sqlx wiring lands in the
//! next session. This entry point exists so we can verify the build
//! graph, the tracing subscriber, and the workspace shape end to end.

fn main() {
    offleash_shared::tracing_init::init("offleash-api");
    tracing::info!("starting");
}
