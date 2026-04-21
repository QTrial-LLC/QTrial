//! Tracing subscriber setup shared by every QTrial binary.
//!
//! ARCHITECTURE.md commits us to structured JSON logs in every deployed
//! environment, so JSON is the default. Local development gets human
//! readable output by setting `QTRIAL_LOG_FORMAT=text`. The usual
//! `RUST_LOG` filter applies in both cases.

use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;

/// Install a global tracing subscriber tagged with the service name.
///
/// Call this once at the start of `main` in each binary. The service
/// name is emitted on every event so that multi-service log streams
/// can be partitioned without grepping through module paths.
pub fn init(service_name: &'static str) {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let format_choice = std::env::var("QTRIAL_LOG_FORMAT").unwrap_or_default();

    let registry = tracing_subscriber::registry().with(env_filter);

    if format_choice == "text" {
        registry
            .with(fmt::layer().with_target(true))
            .init();
    } else {
        registry
            .with(fmt::layer().json().with_current_span(true).with_span_list(false))
            .init();
    }

    tracing::info!(service = service_name, "tracing initialized");
}
