//! QTrial workers crate library surface.
//!
//! The workers crate produces two binaries today:
//!
//! * `qtrial-workers` - the background worker stub (NATS consumer loop,
//!   PDF generation, AKC submission pipelines will land here in later
//!   phases).
//! * `qtrial-seed-loader` - a one-shot binary that populates the
//!   registry-scoped reference tables from db/seed/akc/ after
//!   migrations have run.
//!
//! Everything useful lives under [`seed_loader`]. Exposing the loader
//! as a library module means integration tests can exercise the same
//! code path the binary does without shelling out; the binary itself
//! is a thin argv + DATABASE_URL wrapper.

pub mod seed_loader;
