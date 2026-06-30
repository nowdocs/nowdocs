# prompts/02 — Task 4b+4c：MCP tools（`src/tools.rs`）

> **自包含 agent prompt**。读完即可执行。模块归属 = `src/tools.rs`（4b search + 4c list 合并，因共用此文件）。

## 角色与分支
你是 nowdocs Task 4b+4c agent。**独立 git worktree 工作**，不切主树。
- 基点分支 `feat/4-wave-assembly-stubs`：`git switch -c feat/4bc-tools feat/4-wave-assembly-stubs`。
- 确认 `git log --oneline -5` 含 stub 提交 `f1dd637`（tools/registry 模块 stub）；缺则 merge。

## 任务
实现 `src/tools.rs` 的工具处理器并接进 `src/mcp.rs`；新增 `tests/tools_tests.rs`。

## 先读这些（当前接口，勿改它们）
- `src/mcp.rs` — `handle_tools_call(params)` 现把 `nowdocs_search`/`nowdocs_list` stub 成 JSON-RPC error `-32601`。你接线到 `tools::handle_call`。
- `src/retrieve.rs` — `search(docset, query, max_tokens: Option<u32>, top_k: Option<u32>) -> Result<SearchResult>`；`SearchResult { chunks: Vec<ResultChunk>, tokens_returned: u32, truncated: bool }`；`ResultChunk { chunk_idx: u32, heading_path, source_url, api_version: Option<String>, chunk_type: ChunkType, text }`。
- `src/sanitize.rs` — `sanitize_chunk(text) -> String`、`sanitize_metadata(text) -> String`。
- `src/input.rs` — `validate_docset`/`validate_query`/`resolve_max_tokens(Option<u32>)`/`resolve_top_k(Option<u32>)`。
- `src/cache.rs` — `cache_root()`、`db_path(docset)`。
- `src/lib.rs` — 已有 `pub mod tools;`。

## 要实现
```rust
pub fn handle_call(name: &str, args: serde_json::Value) -> serde_json::Value
```
**`nowdocs_search`**：
1. 取 `query`(必填 str)、`docset`(必填 str)、`max_tokens`(可选 u32)、`top_k`(可选 u32)。
2. `input::validate_query` + `validate_docset` 校验；非法 → JSON-RPC error `-32602`。
3. 调 `retrieve::search`。每个返回 chunk 的 `text`、`heading_path` 过 `sanitize_chunk`；`source_url`、`api_version` 过 `sanitize_metadata`。
4. 返回 result：
```json
{"content":[
  {"type":"text","text":"<fallback: N chunks, T tokens, truncated=B>"},
  {"type":"structuredContent","content":{"chunks":[{"chunk_idx":..,"heading_path":..,"source_url":..,"api_version":..,"chunk_type":"Code"|"Info","text":..}],"tokens_returned":N,"truncated":false}}
]}
```
**`nowdocs_list`**：忽略 args。读 `cache_root().join("db")`，枚举名字以 `.lance` 结尾的目录，取 stem（去 `.lance`）。返回 `{"content":[{"type":"text","text":"<逗号分隔，或 \"no docsets installed\">"}]}`。
**未知工具**：error `-32601` "unknown tool: X"。**内部错误**：error `-32602` + message。

`src/mcp.rs` 改动：`handle_tools_call` 改为取出 name+arguments 后委托 `tools::handle_call(name, args)` 返回其 Value。可保留原有边界 input 校验或移进 handle_call（任选其一，别重复两遍）。

## 测试 `tests/tools_tests.rs`
- `test_list_two_docsets`：`XDG_CACHE_HOME` 指 tempdir，造 `db/a.lance`、`db/b.lance` 空目录，`handle_call("nowdocs_list", json!({}))` 断言两名字都在。
- `test_search_invalid_docset`：`handle_call("nowdocs_search", json!({"query":"x","docset":"../bad"}))` → error。
- `test_search_rejects_empty_query`：空 query → error。
- `test_search_end_to_end`（**#[ignore]**，真 embedder）：`ingest::ingest_dir` 造临时 docset → `handle_call` 搜索 → 断言 chunks 非空。

## 约束
- 只改 `src/tools.rs` + `src/mcp.rs`（接线点）+ `tests/tools_tests.rs`。**不改 Cargo.toml**。
- 网络防线：无 host/port 绑定。命令输出 `> 4bc-test.log 2>&1` 看 tail。TDD；不 push；非交互列 Open Question。

## 完成清单
1. 打勾：Edit plan 的 Task 4b、4c contract。
2. spec：仅「实现核实类」——structuredContent 形态、sanitize 接入点写进 spec §5.4/§6.1 附近。
3. 汇报：① task=4b+4c ② commit sha ③ 测试结果（快测全绿 + #[ignore] 端到端手跑过）④ mcp.rs diff 摘要 ⑤ Open Questions。
commit message：`feat(tools): nowdocs_search + nowdocs_list handlers (4b/4c)`。
