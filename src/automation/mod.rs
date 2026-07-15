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

// C4 owns registry-aware ensure planning and C5 exposes its operation contract
// for integration tests. C7 owns setup orchestration.
pub mod docset;
pub mod operation;
pub mod setup;
