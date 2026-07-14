//! Read-only inspection of local nowdocs state.
//!
//! Ownership boundary for the inspector slice (parent design Section 5.2):
//! this module will aggregate pure, offline-safe doctor and cache
//! observations into one state snapshot for `nowdocs status`. Pure
//! observation must stay separated from the mutating initialization and
//! writability probes the current `doctor` default path performs, so
//! `status` can report `not_initialized` or `writability_not_probed`
//! without creating anything.
//!
//! C1 establishes the module boundary only; it intentionally exposes no API.
