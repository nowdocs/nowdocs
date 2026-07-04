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

"nowdocs"™ 名称及 logo 为 GWMM LLC 的商标（common-law，未注册）。MIT/Apache-2.0
授予代码使用权，**不授予商标权**；分发未修改的官方版本可使用原名。完整政策见
[TRADEMARK.md](docs/TRADEMARK.md)（English）。

## 隐私与遥测

nowdocs 本地运行，**query、embedding、文档内容永不出网**，无遥测、无分析、无
追踪。联网仅限用户主动触发的 `install` / `update`（registry 白名单）与首次
embedder 模型下载（HuggingFace）。完整政策见 [PRIVACY.md](docs/PRIVACY.md)。

## 安全漏洞披露

**请勿为安全漏洞开启公开 GitHub issue**——这会在修复前公开暴露风险。

通过 GitHub 仓库 **Security** 标签页 → **Report a vulnerability**（私有渠道）
报告；或邮件 `legal@gwmmai.com`（标题加 `[nowdocs security]`）。详见
[SECURITY.md](.github/SECURITY.md)。响应窗口：3 个工作日内确认，高危 30 天内修复。

## 侵权下架（DMCA Takedown）

公共 registry 为**策展制**（curated），上架前审核许可证。侵权举报走 **GitHub
内置 DMCA 流程**（[github.com/contact/dmca](https://github.com/contact/dmca)），
备用邮箱 `legal@gwmmai.com`（标题 `[nowdocs DMCA]`）。通知要件与响应流程见
[DMCA.md](docs/DMCA.md)（English）。

## 法务与合规

| 文件 | 内容 |
|---|---|
| [DMCA.md](docs/DMCA.md) | DMCA takedown 流程 + registry 上架许可证审核（English） |
| [PRIVACY.md](docs/PRIVACY.md) | 隐私政策：本地运行，软件不收集数据 |
| [TRADEMARK.md](docs/TRADEMARK.md) | 商标政策（English） |
| [AUP.md](docs/AUP.md) | Acceptable Use Policy：registry 准入与软件使用边界 |
| [SECURITY.md](.github/SECURITY.md) | 安全漏洞披露流程 |
| [CONTRIBUTING.md](CONTRIBUTING.md) | 贡献流程：DCO + 质量门禁 + 策展审核 |
| [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) | 行为准则（Contributor Covenant 2.1） |
