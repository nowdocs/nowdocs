pub mod agent_contract;
pub mod automation;
pub mod cache;
pub mod chunker;
pub mod cli;
pub mod confidence;
pub mod doctor;
pub mod errors;
pub mod input;
pub mod inspect;
pub mod manifest;
pub mod mcp;
pub mod sanitize;
pub mod token;
// Wave 0/2+ modules registered when implemented:
pub mod embedder; // from S0; if S0 not yet run, create an empty src/embedder.rs with `// placeholder, see S0`
pub mod eval;
pub mod ingest;
pub mod registry;
pub mod registry_build;
pub mod retrieve;
pub mod smoke;
pub mod store;
pub mod tools;

// ---- Module stubs (1b-1h fill these) ----
