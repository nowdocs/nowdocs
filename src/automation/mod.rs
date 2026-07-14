//! Agent-automation internals: planning, execution, and rollback.
//!
//! Ownership boundary for the planner and executor slices (parent design
//! Sections 5.3 and 5.4): this module will own deterministic action plans,
//! plan storage and integrity, the global operation lock, journaling, and
//! rollback records beneath nowdocs' private `automation/` cache subtree.
//!
//! C1 establishes the module boundary only; it intentionally exposes no API.
