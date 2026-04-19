//! Shared types and utilities for the OffLeash API and workers.
//!
//! Phase 0 contains only the tracing initializer so both binaries emit
//! logs in the same shape. Domain types and tenancy helpers will land
//! here as the API and workers grow past the skeleton.

pub mod tracing_init;
