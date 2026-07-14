//! Setup CLI orchestration boundary.
//!
//! This module is reserved for a future slice. Per the parent design (Sections
//! 6 and 16, slice 7), **C7** owns setup CLI orchestration: the `setup
//! plan`/`setup apply`/`setup rollback`/`ensure`/`verify` command wiring,
//! approval flow, and end-to-end validation that composes the planner,
//! executor, client adapters, and verifier.
//!
//! C3 deliberately exposes no public API here. Do not add placeholder types or
//! functions; C7 will populate this module with the setup orchestration
//! contract.
