# nowdocs

> 纯 Rust 单二进制 MCP server，本地运行，给 LLM coding agent（Cursor / Claude Code / Aider）提供最新第三方开发文档，治 LLM 对快变库的幻觉。

**状态**：v0.1.0 pre-release — 源码可用（`cargo install --git`）；预编译二进制与 Homebrew 待正式 release。

---

## 为什么需要

LLM 训练有截止日期，对快速变化的库（Next.js 15 / React 19 / Vue 3.5）会产生幻觉。nowdocs 在本地跑一个 MCP server，把官方文档做成本地可混合检索（hybrid search：向量语义 + BM25 关键词 + RRF）的 docset，LLM agent 通过 MCP 工具 `nowdocs_search` 查到**最新且精确**的 API——零 API 费用、完全离线、query 永不离开设备。

核心定位：MCP 协议 + 本地嵌入（candle + jina-v2-small）+ 本地混合检索（lancedb）+ 单一自包含二进制 + 社区 registry。

## 安装

### 当前可用（从源码构建）

```bash
cargo install --git https://github.com/nowdocs/nowdocs
```

需 Rust 工具链（stable）、`protoc`（prost-build 依赖；macOS `brew install protobuf`，Linux `sudo apt-get install protobuf-compiler`）。另外 nowdocs 需要 `curl` 已安装且在 PATH 上可用——registry 联网下载预编译 docset 必经 curl，`nowdocs doctor` 会检查并在缺失时告警。首次 `serve` 会从 HuggingFace 下载 embedder 模型（jina-v2-small-en，约 66 MB），之后本地缓存；可提前跑 `nowdocs doctor --model` 预下载，避免首次检索阻塞。

### v0.1.1 正式发布后（预编译二进制）

release 产物就绪后，免编译安装：

```bash
# cargo-binstall（推荐）
cargo binstall nowdocs

# Homebrew（macOS / Linux）
brew tap nowdocs-registry/nowdocs
brew install nowdocs
```

release 二进制覆盖 linux musl（x86_64 / aarch64）、macOS（arm64 / x86_64）、Windows（msvc）。不做代码签名，完整性靠 SHA-256 checksum + `cargo-binstall` 校验。

## 快速开始

完整教程见 [Getting Started](docs/GETTING_STARTED.md)，常见问题见 [Troubleshooting](docs/TROUBLESHOOTING.md)，MCP 客户端配置见 [MCP Clients](docs/MCP_CLIENTS.md)，发版门禁见 [Release Readiness](docs/RELEASE_READINESS.md)。

1. **先跑诊断**：
   ```bash
   nowdocs doctor
   nowdocs doctor --json
   ```

2. **导入本地文档**（Markdown 目录）：
   ```bash
   nowdocs ingest ./my-docs my-docset --license MIT --source-url https://github.com/org/repo
   ```
   或从 registry 安装（registry 早期，可用 docset 有限）：
   ```bash
   nowdocs install <docset>
   ```

3. **跑真实检索冒烟测试**：
   ```bash
   nowdocs smoke my-docset "installation configuration example"
   ```

4. **启动 MCP server**：
   ```bash
   nowdocs serve
   ```
   `serve` 通过 stdio 通信，不绑定端口/Host。

5. **配置 MCP client**：将 `nowdocs serve` 注册为 stdio MCP server。示例（多数 client 兼容的 `mcp.json` 格式）：
   ```json
   {
     "mcpServers": {
       "nowdocs": { "command": "nowdocs", "args": ["serve"] }
     }
   }
   ```
   配好后，LLM agent 可调用 `nowdocs_search`（语义检索）与 `nowdocs_list`（列出已装 docset）两个工具。

## CLI 命令

| 命令 | 说明 |
|---|---|
| `nowdocs serve` | 启动 MCP stdio server |
| `nowdocs ingest <dir> <name>` | 导入本地 Markdown 目录为 docset |
| `nowdocs install <docset>` | 从 registry 安装预构建 docset |
| `nowdocs update <docset>` | 更新 docset 至最新 registry 版本 |
| `nowdocs uninstall <docset>` | 卸载 docset |
| `nowdocs list-installed` | 列出已安装 docset |
| `nowdocs smoke <docset> [query]` | 对已安装 docset 跑真实检索冒烟测试 |
| `nowdocs doctor [--json] [--docset <name>] [--mcp] [--model] [--repair]` | 诊断环境、缓存、docset、MCP 与模型状态 |
| `nowdocs cache status [--json]` | 查看 cache 路径、大小、已安装 docset 与 staging 状态 |
| `nowdocs cache clean-staging [--older-than <duration>]` | 安全清理 nowdocs-owned staging 目录 |
| `nowdocs share <docset>` | 打包 docset 供 registry 贡献（文本 + manifest，不含向量） |

`ingest` 参数：`--license`（MIT / Apache-2.0 / CC-BY-4.0，默认 MIT）、`--copyright-holder`、`--attribution`（CC-BY-4.0 必填）、`--source-url`、`--entry-url`。

## 使用路径与架构边界

nowdocs 针对不同使用场景提供三条清晰路径：
1. **终端用户 (End User)**: 通过 `nowdocs install <docset>` 安装文档包，再通过 `nowdocs serve` 提供本地 MCP 服务。
2. **贡献者 (Contributor)**: 使用 `nowdocs ingest <dir>` 导入 Markdown 文档，并通过 `nowdocs share <docset>` 打包分享到 registry 社区。
3. **诊断与排错 (Troubleshooting)**: 使用 `nowdocs doctor` 诊断本地环境，用 `nowdocs smoke <docset> [query]` 冒烟测试检索质量。

> **架构安全声明 (Security Boundaries)**：
> - **MCP tools are read-only / MCP 工具是只读的**：MCP 接口暴露的工具（`nowdocs_search`、`nowdocs_list`）是纯只读查询 (read-only queries)，LLM agent 绝无可能通过 MCP 触发任何写操作（如安装、卸载、导入等）。
> - **Side-effect commands are CLI-only / 有副作用的操作均为 CLI 独占**：所有修改状态、写入文件和下载数据的敏感指令 (side-effect commands, such as `install`, `uninstall`, `ingest` etc.) 必须在终端中由用户手动运行 CLI 指令执行。

## 工作原理

```
ingest → chunk（按 token 切分，保留 source_url / heading 等 metadata）
       → embed（candle + jina-v2-small-en，512 维 f16）
       → store（lancedb：FTS + 向量列）

serve  ← MCP stdio
       ← nowdocs_search(query → embed → hybrid search[FTS BM25 + 向量 + RRF] → top-k)
       ← nowdocs_list(列出已装 docset)
```

- **chunk**：按 token 边界切分，保留 metadata（source_url / line / heading）
- **embed**：candle 纯 Rust 推理，jina-v2-small-en（512 维，Apache-2.0），结果缓存在 `~/.cache/huggingface/`
- **retrieve**：lancedb 0.30 hybrid search（FTS Tantivy BM25 + 向量近邻 + RRFReranker 融合）
- **share**：只发文本 + manifest，向量由 registry CI 重建（关闭向量投毒与模型漂移两个攻击面）

## 局限性（v0.1.0）

- **pre-release**：预编译二进制与 Homebrew 未发布，当前需从源码编译
- **registry 早期**：策展制，初始可用 docset 有限
- **embedding 模型固定**：jina-v2-small-en（512 维），暂不可配置
- **embedding backend 固定**：candle（纯 Rust），无 ONNX / 远程 API 选项
- **eval 覆盖有限**：仅 nextjs-corpus 验证（recall@5 = 0.8，MRR = 0.587）
- **平台**：CI 构建 linux musl（x86_64 / aarch64）+ macOS（arm64 / x86_64）+ Windows，未在所有平台广泛实测

## 技术栈

- **语言**：Rust（Edition 2021），lib + bin 双 target
- **嵌入**：candle（纯 Rust）+ jina-embeddings-v2-small-en（512 维，Apache-2.0）
- **检索**：lancedb 0.30（内置 hybrid + RRF）
- **协议**：MCP over stdio（NDJSON）
- **分发**：cargo-binstall + Homebrew，不签名（完整性靠 SHA-256 + cargo-binstall 校验）

## 贡献

贡献流程见 [CONTRIBUTING.md](CONTRIBUTING.md)：DCO（非 CLA）+ L1-L4 质量门禁 + registry 策展审核。行为准则见 [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)。

### 开发文档

- [设计 spec](docs/superpowers/specs/2026-06-28-nowdocs-design-review.md) — 逐环节决策 + ground-truth 核实
- [实施 plan](docs/superpowers/plans/2026-06-28-nowdocs-impl.md) — 6 wave TDD task
- [派发手册](docs/superpowers/plans/2026-06-28-nowdocs-dispatch.md) — 每 task 一份 agent prompt

## 仓库结构

```
nowdocs/
├── src/                 # Rust 源码（lib + bin）
│   ├── cli.rs           # 7 子命令
│   ├── store.rs         # lancedb hybrid search
│   ├── embedder.rs      # candle + jina 推理
│   ├── registry.rs      # install / share / update / uninstall
│   └── mcp.rs           # MCP stdio server
├── tests/               # 集成测试
├── docs/                # 法务政策 + 设计文档
├── dist/homebrew/       # Homebrew formula + tap 设置
├── scripts/             # CI / 门禁脚本
└── .github/workflows/   # gates / release / weekly-audit
```

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
