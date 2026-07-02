# nowdocs

> 纯 Rust 单二进制 MCP server，本地运行，给 LLM coding agent（Cursor / Claude Code / Aider）提供最新第三方开发文档，治 LLM 对快变库的幻觉。

**状态**：🟢 Wave 1 完成（8 task / 48 tests 绿，在 `feat/1a-cargo-skeleton` 分支）→ S0 命门 spike 进行中。

---

## 这是什么

LLM 训练有截止日期，对快速变化的库（Next.js 15 / React 19 / Vue 3.5）会产生幻觉。nowdocs 在本地跑一个 MCP server，把官方文档做成本地可混合检索（hybrid search：向量语义 + BM25 关键词 + RRF）的 docset，LLM agent 通过 MCP 工具 `nowdocs_search` 查到**最新且精确**的 API——零 API 费用、完全离线、query 永不离开设备。

核心定位（5 维度唯一命中）：MCP 协议 + 本地嵌入（candle + jina-v2-small）+ 本地混合检索（lancedb）+ 单一自包含二进制 + 社区 registry。

## 当前阶段

| 产物 | 路径 | 说明 |
|---|---|---|
| 设计 spec | [`docs/superpowers/specs/2026-06-28-nowdocs-design-review.md`](docs/superpowers/specs/2026-06-28-nowdocs-design-review.md) | 定稿，逐环节决策 + ground-truth 核实 |
| 实施 plan | [`docs/superpowers/plans/2026-06-28-nowdocs-impl.md`](docs/superpowers/plans/2026-06-28-nowdocs-impl.md) | 6 wave TDD task |
| 派发手册 | [`docs/superpowers/plans/2026-06-28-nowdocs-dispatch.md`](docs/superpowers/plans/2026-06-28-nowdocs-dispatch.md) | 每 task 一份 agent prompt |

## 快速开始

**给实现 agent**：先读 [`AGENTS.md`](AGENTS.md)，再按 `docs/superpowers/plans/2026-06-28-nowdocs-dispatch.md` 找到分配给你的 task。

**给人**：读 spec 评审稿了解全貌；看 plan §7 进度看板跟踪状态。

## 仓库结构

```
nowdocs/
├── AGENTS.md                     # agent 必读：约束 + 工作流
├── README.md                     # 本文件
└── docs/superpowers/
    ├── specs/                    # 设计文档
    └── plans/                    # 实施计划 + 派发手册
```

Wave 1 基础层已落地（manifest / chunker / token / cache / sanitize / input / mcp 共 8 module，48 tests 全绿）。S0 命门（candle + jina 验证）进行中，green 后开 Wave 2 引擎层。

## 技术栈

- **语言**：Rust（Edition 2021），lib + bin 双 target
- **嵌入**：candle（纯 Rust）+ jina-embeddings-v2-small-en（512 维，Apache-2.0）
- **检索**：lancedb（内置 hybrid + RRF，砍 tantivy）
- **协议**：MCP 2025-11-25 over stdio（NDJSON）
- **分发**：cargo-binstall + Homebrew，不签名

## 许可证

`MIT OR Apache-2.0`（Rust 双许可惯例）。完整文本见 [LICENSE-MIT](LICENSE-MIT) 与
[LICENSE-APACHE](LICENSE-APACHE)，二者任选其一遵守，义务不叠加。版权归 GWMM LLC。

依赖许可证审计：全树 616 个 crate，零强 copyleft（无 GPL/AGPL 传染），见
[NOTICE](NOTICE) 与 `deny.toml`。其中 `option-ext` 为 MPL-2.0（文件级 copyleft，
不感染整个二进制），其 notice 在 NOTICE 中保留。

贡献遵循 DCO（Developer Certificate of Origin），不使用 CLA。每个 commit 须带
`Signed-off-by:`（`git commit -s`），CI 由 `scripts/ci-check-dco.sh` 强制校验。

## 商标

"nowdocs" 及 nowdocs logo 为 GWMM LLC 的商标。MIT/Apache-2.0 授予的是代码使用权，
**不授予商标权**。你不可用 "nowdocs" 名称或 logo 来命名、推广或标识衍生产品。
描述来源时合理使用（"基于 nowdocs"）不受限制。

## 隐私与遥测

**nowdocs 不收集任何遥测数据，不向任何分析服务 phone-home。** 代码中无
telemetry/analytics/tracking。

网络访问仅限以下用户主动触发的场景：

- `nowdocs ingest`：**不联网**。仅读取本地目录（用户自行 clone 官方 repo，连接
  的是 GitHub 等源站，属用户行为，与 nowdocs 无关）。
- `nowdocs install` / `update`：从 registry 下载 docset，仅限白名单域
  `github.com/nowdocs-registry/*` 与 `registry.nowdocs.rs/*`。
- 首次 embed 时从 HuggingFace 下载 embedder 模型（`hf-hub`），之后本地缓存。

无任何使用数据、分析或追踪离开你的机器。

## 安全漏洞披露

**请勿为安全漏洞开启公开 GitHub issue**——这会在修复前公开暴露风险。

通过 GitHub 仓库 **Security** 标签页 → **Report a vulnerability**（私有渠道）
报告；或邮件 `legal@gwmmai.com`（标题加 `[nowdocs security]`）。详见
[SECURITY.md](.github/SECURITY.md)。响应窗口：3 个工作日内确认，高危 30 天内修复。

## 侵权下架（Takedown）

公共 registry 为**策展制**（curated），非开放提交，但我们仍提供侵权下架流程。
若认为 registry 上某 docset 侵犯你的版权，请邮件 `legal@gwmmai.com`，附：

1. 被侵权作品的标识与权属证明；
2. 被指控的 docset 名称及位置；
3. 善意声明（你有权主张、且确信对方未授权）。

我们将在合理期限内（高危版权争议数日内）先行下架，待反通知后再行处理。
