# prompts/03 — Task 4e+4f+4g：registry 生命周期（`src/registry.rs`）

> **自包含 agent prompt**。读完即可执行。模块归属 = `src/registry.rs`（install/share/update/uninstall 合并，因共用此文件）。

## 角色与分支
你是 nowdocs Task 4e+4f+4g agent。**独立 git worktree 工作**，不切主树。
- 基点 `feat/4-wave-assembly-stubs`：`git switch -c feat/4efg-registry feat/4-wave-assembly-stubs`。
- 确认 `git log --oneline -5` 含 stub 提交 `f1dd637`；缺则 merge。

## 任务
实现 `src/registry.rs` 的 install/share/update/uninstall + `tests/registry_tests.rs`。

## 先读这些（当前接口）
- `src/cache.rs` — `cache_root()`、`db_path(docset)`、`manifest_path(docset)`、`ensure_layout()`。
- `src/manifest.rs` — `Manifest` struct + `validate(m) -> Result<()>`（schema/模型版本锁/法律白名单）。
- `src/store.rs` — `Store::open(docset)`、`insert`、`hybrid_search`、`fetch_by_idx`。share 若需读全部文本块，可在 store.rs 加一个最小 helper（如 `dump_chunks`）并在汇报里说明。
- `src/lib.rs` — 已有 `pub mod registry;`。

## 要实现（签名锁定）
```rust
pub fn install(docset: &str, url: &str) -> anyhow::Result<()>;
pub fn share(docset: &str, out_dir: &std::path::Path) -> anyhow::Result<std::path::PathBuf>;
pub fn update(docset: &str) -> anyhow::Result<()>;
pub fn uninstall(docset: &str) -> anyhow::Result<()>;
```
**install(docset, url)**：从 `url` 下载归档到临时文件 → 验证内含 `manifest.json` 且过 `manifest::validate` → 解包文本+manifest 到 `db_path(docset)` 与 `manifest_path(docset)`。
- **安全闸门（强制）**：生产 URL 必须在 `https://github.com/nowdocs-registry/...` 或 `https://registry.nowdocs.rs/...` 域下，其它域明确拒绝报错。测试用 `file://` scheme 特判放行。
**share(docset, out_dir)**：读已装 docset 的 manifest + 全部文本块 → 写 share 包到 out_dir，含 `manifest.json` + `chunks.jsonl`（每行一 chunk 的文本+元数据，**绝不含向量**，D10）→ 返回包目录路径。manifest 必须过 validate。
**update(docset)**：解析最新版 registry URL（测试用本地 fixture，生产用域内 URL pattern）→ 调 install 替换。下载 manifest 的 `model_sha256` 与现装不符时仍允许但明确提示。
**uninstall(docset)**：删 `db_path(docset)` + `manifest_path(docset)`（存在才删）。

## 测试 `tests/registry_tests.rs`（只用本地 file fixture，禁真网络）
- `test_uninstall_removes_db_and_manifest`
- `test_install_rejects_external_url`（如 `https://evil.com/x.tar` → Err）
- `test_install_from_file_url`
- `test_share_produces_no_vectors`（断言产物无 `.lance`/向量文件）
- `test_update_refreshes_manifest`

## 约束
- 尽量不改 Cargo.toml；若测试 helper 确需新依赖（如 tar/zip/ureq），**列 Open Question 上报**别擅自加。
- 网络防线：install 域校验强制；除两个 registry 域外禁硬编码公网 IP。
- 命令输出 `> 4efg-test.log 2>&1` 看 tail。TDD（先写失败测试验证失败）；不 push；非交互列 Open Question。

## 完成清单
1. 打勾：Edit plan 的 Task 4e、4f、4g contract。
2. spec：仅「实现核实类」——share 包格式（manifest+chunks.jsonl 无向量）、install 域白名单写进 spec §6.2/§6.7 附近。
3. 汇报：① task=4e+4f+4g ② commit sha ③ 测试结果 ④ 是否加了 store.rs helper ⑤ Open Questions（尤其依赖增项）。
commit message：`feat(registry): install/share/update/uninstall lifecycle (4e/4f/4g)`。
