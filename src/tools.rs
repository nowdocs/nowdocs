//! MCP tool handlers (nowdocs_search / nowdocs_list).

use serde_json::Value;

pub fn handle_call(name: &str, args: Value) -> Value {
    let _ = (name, args);
    todo!("implement nowdocs_search and nowdocs_list handlers")
}
