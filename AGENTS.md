# AGENTS.md

> **任何 agent 在 nowdocs 仓库工作前必读本文。** 本文是项目级 context + 不可违反的铁律。
> 详细派发操作见 `docs/superpowers/plans/2026-06-28-nowdocs-dispatch.md`。

---

## 1. 项目是什么

nowdocs = 纯 Rust 单二进制 MCP server，本地运行，给 LLM coding agent 提供最新第三方开发文档（治 LLM 对快变库的幻觉）。本地 candle+jina 嵌入 + lancedb 混合检索 + 社区 registry（GitHub 离线预构建 doc crate）。

## 2. 当前阶段

🟡 **设计定稿，待实现**。spec 已评审+自审，plan 已拆 6 wave。尚无代码。实现从 Task 1a（Cargo 骨架）开始。

## 3. 必读文档（按顺序）

| 顺序 | 文档 | 解决什么 |
|---|---|---|
| 1 | `docs/superpowers/specs/2026-06-28-nowdocs-design-review.md` | 设计决策（为什么这么选）+ Global Constraints |
| 2 | `docs/superpowers/plans/2026-06-28-nowdocs-impl.md` | 你 task 的 TDD step 全文 |
| 3 | `docs/superpowers/plans/2026-06-28-nowdocs-dispatch.md` | 你 task 的专属 prompt + 派发总则 |

矛盾时：**spec 的架构决策 > plan 的实现细节**。若 plan 与 spec 冲突，停下上报（Open Question），不要擅自决定。

## 4. 不可违反的铁律

### 4.1 协议与接口
- **MCP 版本 = `2025-11-25`**（非 2024-11-05）。stdio 传输 = NDJSON（单 `\n` 行分隔，**非** Content-Length）。
- **`serve` 命令无 `--host`/`--port`**（stdio 不绑端口，network 防线铁律）。
- **嵌入器字段名冻结**：`model_id` / `model_version` / `model_revision` / `model_sha256`（杜绝 `version`/`sha256` 漂移）。
- **Embedder 接口**：`load() -> Result<Self>` + `embed(&self, &str) -> Result<Vec<f32>>`（512 维）。
- `nowdocs_search(query, docset, max_tokens?, top_k?)` —— `docset` 必填（D12）。

### 4.2 数据与路径
- **缓存路径用 `nowdocs`**（非 `agentdocs`）：`~/.cache/nowdocs/db/<docset>.lance`、`~/.cache/nowdocs/models/<model_id>/`。
- `nowdocs_schema_version = 1`，`CACHE_LAYOUT_VERSION = 1`。
- 模型：`jinaai/jina-embeddings-v2-small-en`，`vector_dim=512`，`engine="candle"`，`dtype="f16"`，许可 Apache-2.0。

### 4.3 安全（零容忍）
- **prompt-injection 防御**：返回给 LLM 的 text + metadata 必须经 `sanitize`（剥注入短语/危险标志/HTML 注释/零宽字符/display:none）。
- **share 只发文本，不发向量**（D10）：CI 用固定标准模型重建向量——关闭对抗性向量注入 + 模型漂移。
- **模型完整性**：pin `model_revision`（HF commit SHA）+ `model_sha256`，下载后 `sha2` 重算比对，不符即删。删 `config.json` 的 `auto_map`（防任意代码执行）。
- **registry URL**：install 的下载 URL 必须指向 `nowdocs-registry` 自己的 GitHub Releases 域，拒外部。
- **法律闸门**：doc crate 入 registry 前核实文档站许可（MIT/Apache-2.0/CC-BY-4.0）。Clerk/Tailwind 文档专有/禁爬，**不入 registry**，仅本地 `ingest`。首发 canonical = Next.js(MIT) + React(CC-BY-4.0) + Vue(CC-BY-4.0)。

### 4.4 工程边界
- **v1 用 flat 精确搜索**（IVF/HNSW 移 deferred）。**v1 英文 only**（CJK defer v2，模型已多语言无需换）。
- **不签名分发**（cargo-binstall + Homebrew，避 F-1/OPT 签证风险）。
- 项目许可 `MIT OR Apache-2.0`，贡献用 DCO 非 CLA。

### 4.5 文件改动边界（并发安全）
- **Cargo.toml**：只有 Task 1a（建）和 Task 2b（lancedb 行）可改。其他 task 改 Cargo.toml = 越界。
- **签名锁定**：1a 已建好全部 module stub + 函数签名。1b-1h 只填函数体，**不改签名**。
- **只改本 task 声明的文件**（`src/<mod>.rs` + `tests/<mod>_tests.rs`），不动其他 module、不改其他 task 的测试。
- **进度看板**（dispatch §7）只 Main 维护，agent 不写。

## 5. 工作流（TDD + 打勾 + 报告）

每个 task 严格执行：

1. **TDD**：写失败测试 → 验证失败 → 最小实现 → 验证通过 → commit。禁止跳过验证。
2. **commit**：conventional commits 英文，消息带 task 编号（如 `feat(manifest): parse + validation (1b)`）。每 task 末尾必 commit。
3. **完成后三件事（缺一不可）**：
   - **打勾**：Edit plan 文件，把本 task 的所有 `- [ ]` 改 `- [x]`（精确匹配 step 文本；遇 "File modified since read" 重新 Read 再 Edit）。
   - **spec 修订**：仅限「实现核实类」事实（真实 API 名/版本号/许可核实）。架构级变更（删功能/换依赖/换模型）不在权限内，列 Open Question 上报。
   - **报告**：返回 ① task 编号 ② commit sha ③ 测试结果 ④ spec diff（若有）⑤ Open Questions。然后停下，不做未分配任务。

## 6. 分支与提交

- 仓库默认分支 `main`。feature 工作在用途明确的 feature branch 上（`feat/<task>`），不直接在 main 干代码。
- 文档/setup 类改动（README/AGENTS/.gitignore）可在 main 上做。
- **严禁擅自 `git push`**（需用户明确批准）。

## 7. 命令输出管控

dev/build/test 产生大量日志时，后台 + 重定向（`> log 2>&1`），不直接 dump 进上下文。

## 8. 命门

**Task S0**（jina-v2-small 在 candle 跑通 + E2 余弦>0.99）是技术栈命门。失败则 candle 路线回退 ort，Wave 2 重评估。S0 绿了才放心开 Wave 2 的 embedder 相关任务。

---

**技术栈速查**：Rust 2021 / clap / serde / anyhow / thiserror / regex / sha2 / dirs / tiktoken-rs / candle-core(default-features=false)+candle-nn+candle-transformers / tokenizers / hf-hub / lancedb。
