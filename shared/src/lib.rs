//! Shared types and utilities for the QTrial API and workers.
//!
//! Phase 0 modules:
//! * `tracing_init` — install the global tracing subscriber with the
//!   same JSON-vs-text env switch across every binary.
//! * `tenancy` — open Postgres transactions under the `qtrial_tenant`
//!   role with the session variables that RLS policies read.
//! * `testing` (feature-gated) — a reusable testcontainers fixture
//!   and helpers for writing integration tests against a real
//!   migrated Postgres instance.

pub mod fk_validation;
pub mod tenancy;
pub mod tracing_init;

#[cfg(feature = "testing")]
pub mod testing;
