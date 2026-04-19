//! OffLeash workers skeleton.
//!
//! Phase 0 scaffolding only. The NATS consumer loop, PDF generation,
//! and AKC submission pipelines land in later sessions. This entry
//! point exists so we can verify the build graph and tracing setup.

fn main() {
    offleash_shared::tracing_init::init("offleash-workers");
    tracing::info!("starting");
}
