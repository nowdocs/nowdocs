# prompts/01 — Task 3b：golden eval（检索质量门禁）

> **自包含 agent prompt**。读完本文件即可执行，无需读完整 plan。

## 角色与分支
你是 nowdocs Task 3b agent。**在独立 git worktree 里工作**，不要切主树分支。
- 集成基点：分支 `feat/4-wave-assembly-stubs`（含全部 W0/W1/W2 + 3a）。
- 你的分支：`git switch -c feat/3b-eval feat/4-wave-assembly-stubs`（若在隔离 worktree 由 harness 建分支，则确认 `git log --oneline -3` 含 3a 提交 `a6e6b0d`，缺则 merge 进来）。

## 任务
新建 `src/eval.rs` + `tests/eval_tests.rs`：对每个 docset 的 golden 查询集算 **recall@5 + MRR**，作为 embedder/chunking 改动的 CI 质量门禁。

## 先读这些（当前接口）
- `src/retrieve.rs` — `pub fn search(docset, query, max_tokens: Option<u32>, top_k: Option<u32>) -> Result<SearchResult>`；`SearchResult { chunks: Vec<ResultChunk>, .. }`；`ResultChunk { chunk_idx, heading_path, source_url, api_version, chunk_type, text }`。
- `src/ingest.rs` — `ingest_dir(dir, docset_name) -> Result<IngestStats>`（先 ingest 再 eval）。
- `src/lib.rs` — 已有 `pub mod eval;`？若无，加上（这是你唯一允许动 lib.rs 的点）。
- `src/cache.rs` — `db_path` / `manifest_path`，测试用 `XDG_CACHE_HOME` 指 tempdir。

## 要实现（签名锁定）
```rust
/// 一条 golden 查询：query + 期望召回的 chunk 标识（用 source_url 或 chunk_idx）。
pub struct GoldenQuery { pub query: String, pub expected_source_url: String }

pub struct EvalReport { pub recall_at_5: f32, pub mrr: f32, pub n: usize }

/// 对已 ingest 的 docset 跑 golden set，返回 recall@5 + MRR。
pub fn evaluate(docset: &str, golden: &[GoldenQuery]) -> anyhow::Result<EvalReport>;
```
- recall@5：top-5 结果里命中 expected 的查询比例。
- MRR：命中 rank 倒数的均值（未命中记 0）。
- 用 `search(docset, &q.query, Some(4000), Some(5))`，比对返回 chunk 的 `source_url`。

## golden fixture
- 在 `tests/fixtures/golden/` 放一个 canonical docset 的小 md 语料（自造 3-5 篇即可，不必真 Next.js）+ 一个 `golden.json`（10-15 条 query + expected_source_url）。
- 设计查询使期望 recall@5 ≥ 0.8、MRR ≥ 0.6（门禁阈值，写进测试断言）。

## 测试 `tests/eval_tests.rs`
- `test_evaluate_meets_threshold`（**#[ignore]**，真 embedder ~30s）：ingest fixture 语料 → `evaluate` → 断言 `recall_at_5 >= 0.8 && mrr >= 0.6`。
- `test_eval_report_math`（快，不 load embedder）：直接构造已知 rank 列表，验证 recall/MRR 计算公式正确（可把 recall/MRR 纯算逻辑抽成可单测的纯函数）。

## 约束
- 只改 `src/eval.rs` + `tests/eval_tests.rs` + `src/lib.rs`（仅加 mod 行）+ golden fixture。**不改 Cargo.toml**。
- 命令输出管控：`cargo test --test eval_tests > 3b-test.log 2>&1` 后看 tail；#[ignore] 用 `-- --ignored --test-threads=1` 单跑。
- TDD；不 push；非交互遇阻列 Open Question。

## 完成清单（缺一不可）
1. 打勾：Edit `docs/superpowers/plans/2026-06-28-nowdocs-impl.md` 的 Task 3b contract（若有 step）。
2. spec：仅「实现核实类」——recall@5/MRR 定义 + 阈值写进 spec §6.8 附近。
3. 汇报：① task=3b ② commit sha ③ 测试结果（快测全绿 + #[ignore] 手跑达阈值）④ eval.rs 摘要 ⑤ Open Questions。
