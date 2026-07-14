//! Agent-automation internals: planning, execution, and rollback.
//!
//! Ownership boundary for the planner and executor slices (parent design
//! Sections 5.3 and 5.4): this module owns deterministic action plans, plan
//! storage and integrity, the global operation lock, journaling, and rollback
//! records beneath nowdocs' private `automation/` cache subtree.
//!
//! C3 establishes the plan storage and lock foundation. The future-boundary
//! modules ([`docset`], [`operation`], [`setup`]) carry only module-level
//! documentation: C4 owns registry-aware ensure planning, C5 owns operation
//! journaling/rollback, and C7 owns setup CLI orchestration.

pub mod lock;
pub mod plan;

// Future-slice boundaries (C4/C5/C7). Declared private with no public API.
mod docset;
mod operation;
mod setup;
