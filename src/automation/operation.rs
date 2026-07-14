//! Operation journaling and rollback boundary.
//!
//! This module is reserved for a future slice. Per the parent design (Sections
//! 10 and 11), **C5** owns operation journaling and rollback: the per-operation
//! journal (action state and hashes, never secrets or full configuration
//! values), backup/restore of changed files, `applied_but_unverified` handling,
//! and `setup rollback`.
//!
//! C3 deliberately exposes no public API here. Do not add placeholder types or
//! functions; C5 will populate this module with the journal and rollback
//! contract.
