# nowdocs 设计评审与定位修订

> **状态**：设计评审稿，供伙伴评审。基于原 spec（`2026-06-28-agentdocs-design.md`）的逐环节深度评审 + ground-truth 核实 + 竞品深挖。
> **日期**：2026-06-28
> **改名**：`agentdocs` → **`nowdocs`**（见 §1）
> **核实方法**：4 个研究子代理 + GitHub API 坐实 + Context7 deep-research（103-agent workflow + 9 维度定向深挖）+ 源码级核实（lancedb 0.30 / lance 7.0 / candle / ort / MCP spec）。

---

## 0. 执行摘要

**nowdocs 是什么**：纯 Rust 单二进制 MCP (Model Context Protocol) server，本地运行，给 LLM coding agent（Cursor / Claude Code / Aider）提供最新第三方开发文档，治 LLM 对快变库（Next.js 15 / Clerk V2 / Tailwind v4）的幻觉。

**核心决策（逐环节拍定）**：
- **改名 `nowdocs`**（原 `agentdocs` 撞 agentdocs.eu 在线产品 + PyPI 同名包）
- **Crawler 外置**：核心只内置轻量 HTTP 抓取，obscura 作外部重型工具；nowdocs 核心只做 `ingest`
- **Embedder = candle + jina-v2-small**（非"ort 不能单二进制"，而是"ort 速度优势在本场景无意义 + 纯 Rust 供应链/护城河"）
- **Database = lancedb 单依赖**（砍 tantivy，hybrid+RRF 内置）
- **Registry = GitHub repo + index.json**，社区共建，分层维护
- **MCP 升 2025-11-25**，结构化输出
- **canonical crate = Next.js + React + Vue**（文档许可已逐一核实：Next.js `MIT` / React `CC-BY-4.0` / Vue `CC-BY-4.0`；Clerk/Tailwind 专有或禁爬不入 registry，见 §6.10）
- **share 发文本，CI 重建向量**（关闭向量投毒 + 模型漂移 + 消解原 spec §3.3 与 §6.2 矛盾，见 §6.2）
- **不签名分发**（cargo-binstall + Homebrew；避开 F-1/OPT 签证风险，见 §6.11）
- **CJK defer v2**（模型已多语言无需换；v2 加 lindera，见 §6.12）
- **贡献用 DCO 非 CLA**；项目许可 `MIT OR Apache-2.0`

**护城河（重锚后）**：① registry 网络效应 ② 文档新鲜度（按热度刷新 + release-action）③ single self-contained binary ④ 本地免疫 prompt injection + 向量注入（D10 CI 重建文本）+ 离线 + 零遥测 ⑤ 真实 token 计数。**砍掉"技术栈新颖性"叙事**（被 project-rag 证伪：同栈 rmcp+fastembed+tantivy+LanceDB+RRF 已开源跑通）。

---

## 1. 改名决策：agentdocs → nowdocs

**冲突（已核实）**：
- `agentdocs.eu` 是活的在线产品「AgentDocs — Documentation that agents can read」→ 品牌直接撞车
- PyPI `agentdocs` 已被占（"tool for maintaining docs for your agents"）
- crates.io `agentdocs` + npm bare 可用，但前两条足以弃名

**`nowdocs` 可用性（已核实）**：
- crates.io ✅ 可用
- `nowdocs.rs` ✅ 可用（.com / .dev 被占；对 Rust 项目，`name.rs` 是像 `tokio.rs` 一样的自然主题域名）
- 调性：now docs，直击"治 LLM 文档滞后"痛点

---

## 2. 竞品矩阵

5 维度对比（MCP 协议 / 本地嵌入 / 本地检索 / 单二进制 / 社区 registry）：

| 竞品 | MCP | 本地嵌入 | 本地检索 | 单二进制 | 社区 registry | 备注 |
|---|---|---|---|---|---|---|
| **Context7** (Upstash, 58k★) | ✅ | ❌ 闭源远程 | ❌ 服务端 | ❌ TS runtime | ✅ 远程实时拉取 | 最强玩家，远程 SaaS |
| **docs-mcp-server** (arabold) | ✅ | 部分 | ✅ | ❌ TS | ❌ | 功能轴最近，自定位 Context7 开源替代 |
| **mcp-local-rag** (shinpr, 327★) | ✅ | ✅ | ⚠️ 无真 BM25/RRF | ❌ TS | ❌ | 流派不同 |
| **project-rag** (Brainwires, 19★休眠) | ✅(rmcp) | ✅(fastembed) | ✅(tantivy+LanceDB+RRF) | ❌ fastembed 动态链 ONNX C++ | ❌ | **同栈证伪技术栈新颖性**，但场景=自己代码库 |
| **nowdocs** | ✅ | ✅(candle) | ✅(lancedb hybrid+RRF) | ✅ self-contained | ✅ GitHub 离线预构建 | 唯一同时命中 5 维度 |

**结论**：没有任何竞品同时命中 5 维度，差异化定位成立。但技术栈（Rust+本地嵌入+混合检索+MCP）不是卖点——project-rag 已用同栈开源跑通。真正护城河在 registry 网络效应 + 文档新鲜度 + self-contained binary + 本地安全。

---

## 3. Context7 deep-dive（定位弹药来源）

Context7 = 远程闭源 RAG SaaS。关键做法 + 弱点：

**架构**：远程流式 HTTP（`mcp.context7.com/mcp`）+ 闭源后端（嵌入/向量/爬虫全私有），本地 `npx` 只是瘦客户端。返回 `codeSnippets[]` + `infoSnippets[]` 结构化分离。

**内容来源**：多源爬虫+解析器（Git 仓库 / 网站 / OpenAPI / llms.txt / npm 包 / Confluence），**闭源**。

**贡献模型**：纯社区播种 + Web 表单 `add-library`，任何人加公共库无需拥有它；`context7.json`（类 robots.txt manifest）放库 repo 根目录认领所有权。靠 2800+ issue 社区播种到数千库——**它没自己填种子**。

**时效性（最聪明设计）**：按热度分级自动刷新——top100=1天 / top1k=15天 / top5k=30天 / 其他=45天；请求时异步刷新、非阻塞返回旧版。

**商业模式**：免费 1000 次/月（开发循环不够）→ Pro $10/席 → **企业版私有文档摄取才是利润点**。

**5 个可利用弱点（nowdocs 定位弹药）**：
1. **Prompt injection**（#2663 / #2673）：恶意文档内容注入 agent 上下文 → agent 执行 `npx -y` 等命令
2. **长尾陈旧**：`/anthropics/claude-code` 落后 46 版（45 天阈值 + 未被请求就滞后）
3. **registry 碎片化**（#339）：React 有 4+ 重复条目
4. **免费层 1000/月太紧**（#2145）
5. **解析抓到注释/配置噪声**

**nowdocs 反向定位**：本地 = 无限调用无上限（打痛点④）、自己 ingest 任意时刻刷新（打痛点②）、contributor + CI 校验防碎片（打痛点③）、可控源 + sanitize 免疫注入（打痛点①）。

---

## 4. 四个承重假设核实

> 原 spec 4 个"承重"技术假设，逐个 ground-truth。**两条被证伪、一条简化、一条需升级。**

### 4.1 Crawler：`obscura` ❌ 证伪
- 原 spec §2.1 称 obscura 是"in-process 隐身爬虫库"。**证伪**：crates.io 上 `obscura` 是 2019 年死的 raytracing crate（同名无关，3182 下载、7 年没动、连 repo 都没填）。GitHub 真身 `h4ckf0r0day/obscura` = 16,270★ / Rust / Apache-2.0 / 创建 2026-04-13 / 当红项目，是 **AI agent & 爬虫专用 headless 浏览器引擎**（跑真 V8 + CDP，Puppeteer/Playwright drop-in 替代 Chrome，30MB RAM / 70MB 二进制 / 85ms 加载 / 内置反检测）。
- **关键冲突**：obscura 是**独立浏览器进程 + CDP 驱动**，不是能 `cargo link` 的进程内库 → 直接用会让 spec 想躲的"重型 Chrome"回来，且破坏 single binary 护城河。
- **影响**：spec §2.1 Crawler 模块整个重做（见 §6.1）。

### 4.2 Database：lancedb 混合检索 ✅ 成立 + 简化
- lancedb 0.30.0（2026-05-28）原生 `query().full_text_search().nearest_to().rerank(RRFReranker).execute_hybrid()` 一条链 = 向量 + BM25 + RRF；tantivy 已是其传递依赖。
- **影响**：spec "lancedb/tantivy 混合检索" 砍成纯 lancedb 单依赖。
- **footgun（决定 manifest schema）**：lancedb **不存嵌入模型版本**——CI 预构建向量与用户端查询向量若不同模型版本，**静默返回垃圾 cosine 结果**。→ manifest 必须锁 `model_id` + `model_version` + `model_sha256`（见 §5.3 字段名冻结），运行时强制校验。

### 4.3 Embedder：candle vs ort ⚠️ rationale 证伪
- 原 spec §4.1 称"ort 需动态链 `libonnxruntime.so`、无法产出真单二进制，故选 candle"。**证伪**：ort 默认 `download-binaries` 策略就 `cargo:rustc-link-lib=static=onnxruntime`，产出单一二进制，运行时无需外部 .so/.dylib（glibc 目标靠宿主 libstdc++，几乎零部署成本；musl 真·全静态需源码编 ORT）。
- **真取舍**：candle = 100% 纯 Rust 无构建依赖但单线程 ~5-40ms（`<5ms` 脆弱）；ort = 快约 3-10 倍（candle ~5-40ms vs ort ~1.5-4ms，CPU EP 成熟）但带 C++ 供应链。
- **关键 reframe**：ort 速度优势在本场景**无意义**——nowdocs 是交互式单查询（每搜索只嵌入一句），candle 和 ort 都低于人类感知阈值；ort 速度只在 build-time 批量嵌入有感（contributor 一次性成本、可多线程）。
- **影响**：spec §4.1 rationale 整段重写——从"ort 不能单二进制"（错）改成"ort 速度优势在本场景无意义 + 供应链/护城河取舍选 candle"。

### 4.4 MCP 协议版本 ⚠️ 需升级
- 原 spec 用 `2024-11-05`（已过时）；最新稳定 = **`2025-11-25`**（加 outputSchema / 结构化输出 / elicitation）。
- 对纯 stdio + tools-only server 是 additive 变更，不破坏兼容。`inputSchema` 必须是合法 JSON-Schema-2020-12 object。

---

## 5. 五刀决策

### 5.1 第一刀：Crawler 定位
**Reframe**：nowdocs 主路径是 `nowdocs install <docset>`（从 registry 拉预构建 crate）——**主路径根本不碰爬虫**。爬虫只在 contributor 构建 + 用户爬私有文档两条次要路径。→ obscura 不必进核心二进制。

**A 方案（拍定）**：核心二进制只内置轻量层——`reqwest`（HTTP，正常 UA + robots.txt + 同源 max_depth）+ `htmd`（HTML→Markdown，活跃 1M+ 下载）+ `scraper`（CSS 选择器，按 `scraper.toml` include/exclude 剥导航噪声）。JS-heavy 或反爬站点 opt-in 外置 obscura/crawl4ai。

**b 方案（拍定）**：nowdocs 只做 `ingest`，抓取完全外置。contributor 爱用 obscura / curl / crawl4ai 都行，抓成 Markdown 目录再 `nowdocs ingest ./md-dir --name X`。nowdocs 核心职责 = 检索 + 索引引擎，不碰抓取。理由：真单二进制护城河无瑕疵、不绑死 obscura（它才 2 个月、未发 crates.io、API 在变）。

**`scraper.toml` 归属（消歧 A7）**：它是 contributor 随 doc crate 提交的 **per-docset 抓取/清洗配置**（include/exclude CSS 选择器、robots 策略、llms.txt 探测、max_depth），由 CI 在 `ingest` 时读取，**不进核心二进制**。CI 校验其语法 + 限定 `source_url` 域。A 方案（reqwest+htmd+scraper 轻量层）只是"核心也读 scraper.toml 做本地 ingest"的可选路径，二者共用同一 schema，不矛盾。

**CLI 形态**：
```
nowdocs serve              # MCP stdio
nowdocs install <docset>    # 拉 registry 预构建 crate（主路径）
nowdocs ingest <dir> --name <n>   # 从已有 Markdown 导入（接外置抓取器输出）
nowdocs share <docset>     # 打包分块文本+manifest+config 回 registry（不发向量，CI 重建，见 §6.2/D10）
```
（原 plan 的 `crawl` 命令降级/重定义为 ingest 的 fallback，见 §9 修订清单）

**Chunk 策略（拍定 ii）**：code-aware + contextual enrichment。
- chunk size 256-512 tokens（默认 384，per-docset 在 manifest `retrieval.chunk_size_tokens` 锁定；**不照搬 jina 8192 context**——8192 塞一个向量 = 检索精度塌方 + 返回给 LLM 上下文爆炸 + BM25 词频稀释）
- 每 chunk 前缀加 heading 链（`Clerk Auth > middleware > clerkMiddleware`），让孤立代码块向量也有"属于哪一节"的语义
- **小 chunk 索引、大窗口返回**：命中一个 chunk → 返回它 + 前后相邻 chunk（~2k token 窗口），利用 jina 8192 富余

### 5.2 第二刀：Embedder = candle + jina-v2-small（拍定）
- **引擎**：candle（纯 Rust），跑 jina-embeddings-v2-small-en
- **模型格式**：safetensors，`DType::F32` 加载 + mmap。⚠️ 原 plan 想 F16 加载省 RSS，但 **candle 0.11 的 `jina_bert` 把 ALiBi bias 硬编码为 F32**（`build_alibi_bias` 内 `to_dtype(F32)`），F16 权重 forward 触发 `dtype mismatch in add, lhs: F16, rhs: F32`（S0 实测坐实，见附录 §B）。0.11.0 已是 crates.io 最新（2026-06-26 发布），无更新版可解。故权重加载固定 F32（RSS ~131MB，在 §6.5 常驻 1.2-1.5GB 里 <10% 噪声）。**注意区分**：权重加载 dtype（F32，candle 限制）≠ 向量存储 dtype（manifest `dtype:"f16"`，省磁盘且 cosine 无影响，见 §5.3）——二者独立。
- **许可**：Apache-2.0（**已核实**——small 变体是可商用社区版；base/large 才 CC BY-NC。核实前误以为 v2 全系非商业，纠正）
- **性能预算修正**：交互单查询 ~5-40ms（可接受、无感）；spec `<5ms` 对 candle 乐观、对 ort 可达，降为"~5-40ms 无感"；`<50MB RAM` 修正为"增量运行内存 <50MB"——常驻总量按 §6.5 实算为 **~1.2-1.5GB**（含 metadata cache，原 100-150MB 是漏算 cache）。模型体积：F16 safetensors **~66MB**（F32 加载 RSS ~131MB）。
- **首次验证项（spike）**：实施前先加载 jina-v2-small 的 `config.json` + `tokenizer.json` + `model.safetensors`，跑一句验证向量正常。若失败回退 ort。

**为何不选更新的 jina-v5-omni-small（已核实全面不兼容）**：1.74B 参数 / 3.25GB / 秒级 CPU 推理（交互卡死）/ candle 无实现且无 ONNX 回退 / omni 多模态是纯负担（nowdocs 只索引英文文本）/ **CC BY-NC 非商业**（开源硬伤）/ 需任务前缀。7 条全打脸核心约束。备选 nomic-embed-text-v1.5（Apache-2.0 / 100M / 768维 / matryoshka / candle 有 `nomic_bert.rs`）。

**关键认知**：检索瓶颈在 chunking + BM25（精确 API 名如 `clerkMiddleware` 词项命中），不在模型新旧。换大模型是推翻 candle + 护城河 + 延迟预算的架构级重决策。

### 5.3 第三刀：Database + Registry（拍定）
**数据库**：lancedb 0.30 单依赖，每 docset 一张表 `~/.cache/nowdocs/db/<docset>.lance`，install 时拉取整个表文件。

**manifest.json schema（含 footgun 修复 + 法律字段 + tokenizer 位）**：
```json
{
  "docset": "nextjs",
  "doc_version": "15.1.0",
  "nowdocs_schema_version": 1,
  "embedder": {
    "model_id": "jinaai/jina-embeddings-v2-small-en",
    "model_version": "1.0.2",
    "model_revision": "<huggingface commit hash>",
    "model_sha256": "<safetensors hash>",
    "vector_dim": 512,
    "engine": "candle",
    "dtype": "f16"
    // ↑ dtype = 向量存储 dtype（lancedb 表里向量列的 dtype，F16 省磁盘且 cosine 无影响）。
    //   与权重加载 dtype 独立——权重因 candle 0.11 ALiBi 限制固定 F32 加载（§5.2/附录 §B）。
    //   此处保持 f16：存储层不受 candle 限制。
  },
  "retrieval": {
    "tokenizer": "default",
    "chunk_size_tokens": 384,
    "window_tokens": 2048
  },
  "source": {
    "entry_url": "https://nextjs.org/docs",
    "source_url": "https://github.com/vercel/next.js",
    "scraped_at": "2026-06-28T10:00:00Z",
    "chunk_count": 3421
  },
  "legal": {
    "license": "MIT",
    "copyright_holder": "Vercel Inc.",
    "attribution": "Copyright (c) Vercel Inc. — MIT"
  },
  "refresh_strategy": { "tier": "top100", "auto_days": 1 }
}
```
> **字段名冻结（A3）**：嵌入器字段统一带前缀全名 `model_id` / `model_version` / `model_revision` / `model_sha256`，全文（§6.2/§6.3）一致，杜绝 `version`/`sha256` 漂移。`legal.license` 用 SPDX 标识符；`legal.attribution` 满足 CC-BY 署名（React/Vue crate 必填，见 §6.10）。`retrieval.tokenizer` 留 v2 CJK 扩展位（§6.12），v1 固定 `default`。

**D1 模型版本治理（改良版，拍定）**：双模型按需加载 + registry 版本治理 + 渐进迁移。
- 运行时按 manifest 加载匹配模型版本（不预占内存，只装用到的）——模型升级不打扰终端用户
- registry 统一"当前标准模型版本"，新贡献必须用标准版本 embed（防碎片）
- 模型升级 = 渐进迁移（CI 按热度重 embed 官方 crate，热门先迁、长尾靠双模型兜底继续可用），非一刀切
- "换模型 = 重 embed"是铁律（向量绑定模型），但被压到最低频 contributor CI job，不砸终端用户

**D2 分发渠道（拍定）**：GitHub repo `nowdocs-registry` + 根 `index.json`（目录级，CI 生成）+ doc crate 文件存 GitHub Releases。零基建、天然版本化。

**D3 谁能发（拍定）**：分层——核心热门 doc（nextjs/react/vue，许可已核实见 §6.10）maintainer 官方维护；长尾 doc 社区 PR + maintainer review（入库前同样过 §6.10 许可闸门）。

### 5.4 第四刀：MCP + CLI 清理（拍定）
- **协议**：升 `2025-11-25`
- **砍死参数**：原 plan `serve { host, port }` 对 stdio MCP 毫无意义 → `Serve` 无参。network 防线铁律在此从根消解（stdio 不监听任何端口，连 127.0.0.1 绑定都不存在，比绑 localhost 更干净）
- **工具集**：
  - `nowdocs_search(query, docset, max_tokens?, top_k?)` → 结构化 `structuredContent`（JSON 数组 `[{score, heading_path, source_url, api_version, chunk_type, text, chunk_idx}]`）+ `text` fallback
  - `nowdocs_list()` → 列已 install 的 docset（**给 LLM 的 MCP 工具**；与 CLI `list-installed` 各司其职——都列已装 docset，命名重叠是有意区分人/机入口，A11）
  - `install` 保持 CLI-only（副作用大，不该让 LLM 自主触发）
- **默认值**：`max_tokens=4000`（硬上限）、`top_k=5`（限 1-20）、返回 `tokens_returned` + `truncated`
- **`docset` 改必填（D12）**：跨 docset RRF 分数不可比、排序未定义；LLM 通常知道自己查哪个库，留空只增歧义。
- **输入验证（D13）**：`docset` 正则 `^[a-z0-9._-]{1,64}$`（拒 `..` 路径遍历 / 大小写 / 特殊符）；`query` 长度上限（如 4096 字符）防 ReDoS/OOM；`max_tokens`/`top_k` 钳到合法区间。
- **`serve` 表刷新（D14）**：lance 打开时快照固定，新装 docset 不自动可见 → 每次工具调用按 manifest 重开/校验最新表（或 `checkout_latest()`），避免"装了搜不到"。
- **`max_tokens` vs 窗口（A9）**：召回返回相邻窗口 ~2k token，但 `top_k=5` 不必然 = 10k token——窗口按 `max_tokens` 预算迭代填充，触上限即停并置 `truncated`。

### 5.5 第五刀：产品 scope（拍定）
- **首发官方 canonical crate（3 个，maintainer 播种当质量基准，防 Context7 式碎片化）**：**Next.js + React + Vue**——三库文档站许可已逐一核实（Next.js `MIT` / React `CC-BY-4.0` / Vue `CC-BY-4.0`，见 §6.10），直击前端 LLM 幻觉重灾区。**Clerk / Tailwind 不入 registry**（Clerk ToS §3 禁复制/下载/存储/传输 + §5 禁爬虫；Tailwind 文档站"Tailwind Labs 知识产权、非开源许可"——见 §6.10 法律闸门）。Astro(MIT) 作备选第四库。
- **砍掉**：后端框架文档（变化慢、幻觉轻）、语言标准库文档（`rustdoc`/`pydoc` 已存在，重复造轮子，违反"避免打架"）
- **冷启动（混合，拍定）**：maintainer 播种 3 个 canonical 当基准 + 贡献入口完全开放（抄 Context7 低摩擦：contributor 填 `scraper.toml` + `ingest` + `share` → PR）+ CI 自动校验门禁（manifest schema + 模型版本匹配 + 抽样质量）
- **抄 Context7 时效性**：manifest 每库声明 refresh 策略（按热度分级 1/15/30/45 天）+ 可选 GitHub Action release 推送触发
- **投资顺序（修正）**：先做精二进制 + 检索质量 + 安全，registry 只放 2-3 个种子 crate 当示例，**不在首发前堆 registry**。nowdocs-search 是产品，registry 是社区增长。

---

## 6. P0 遗漏补缺（定稿前必须拍）

### 6.1 A1 Prompt injection（核心威胁模型，不可协商）
nowdocs 把任意第三方文档块喂给有 shell 权限的 agent——正是 Context7 #2663/#2673 的机理。MCP 2025-11-25 spec **没有** trusted 标记，协议层面无法防御，必须在工具内实现。

**四重防御**：
1. `structuredContent` 把载荷放命名字段（非纯文本，防散文注入）
2. 服务端 sanitize：剥助手导向框架（`ignore previous instructions` / `note for the assistant` / `you may run`）+ 危险标志（`-y` / `--yes` / `--force` / `sudo` / `rm -rf`）
3. HTML-aware sanitizer：剥 `<!-- -->` / 零宽字符 / `display:none`（简单 tag 剥离器会静默保留注入）
4. registry 元数据（标题/描述）同样过 sanitize——HubSpot 案例正是经元数据通道注入
- **分隔符/引用只是深度防御，不是边界**（静态分隔符可注入）

### 6.2 A2 恶意 doc crate（registry 是社区 PR，零容忍）
`manifest.json` 可注入恶意下载 URL。CI 必须对每个提交包强制：
- 拒绝绝对路径 / `../` / 符号链接（解析到缓存根外）
- 下载 URL **必须指向 `nowdocs-registry` 自己的 GitHub Releases 域**，拒绝任何外部 URL
- **CI 重建表（D10 拍定）**：`share` 只发布**分块文本 + manifest + config**，CI 用固定标准模型重新 embed 出表——不发布 contributor 本地预构建向量。一石三鸟：① 消解原 spec §3.3"打包本地 LanceDB 目录（含向量）"与 CI 重建原则的矛盾 ② **关闭对抗性向量注入**（向量是不透明浮点，contributor 直发可伪造向量让恶意 chunk 命中特定查询、无法审计；CI 从可审计文本重算则投毒路径关死）③ 关闭模型版本漂移（contributor 用异版模型 → 垃圾 cosine）。附带：contributor 无需本地有模型即可贡献、文本包比向量包小。**威胁模型文档**：`docs/THREAT_MODEL.md`（Task 5c 产出）。
- 拒绝 `next`/不稳定格式版本。FTS 索引约束 `use_tantivy=True` 拒绝项在 lancedb 0.30 已自动满足（旧 tantivy backend 移除，`Index::FTS` 只剩原生 Lance inverted index，无 `use_tantivy` 字段——见附录 §G）。
- pin 嵌入器 `model_id + model_version + model_sha256`，不匹配拒绝
- 发布这些 CI 规则作为威胁模型，contributor 事先知晓

> **实现核实（2026-06-29，registry.rs）**：`install(docset, url)` 域白名单硬编码 `github.com/nowdocs-registry` + `registry.nowdocs.rs`，其它域拒绝报错；`file://` scheme 特判放行供测试用。`share(docset, out_dir)` 产物 = `manifest.json` + `chunks.jsonl`（每行一 chunk 的 text+metadata JSON），**绝不含向量/`.lance` 文件**（D10）。`update(docset)` 用 `NOWDOCS_TEST_URL` 环境变量做测试 fixture，生产构造 `https://github.com/nowdocs-registry/releases/latest/download/{docset}.tar`。`uninstall(docset)` 删 `db_path` + `manifest_path`（存在才删）。

### 6.3 A3 模型完整性（供应链护城河）
spec 写"首次运行下载"但**无完整性校验**。`hf-hub` crate 默认不下完不验 SHA。
**修复（10 行）**：pin `model_revision`（HF commit SHA，不可变）+ `model_sha256`（safetensors 的 LFS oid），下载后用 `sha2::Sha256` 重算比对，失败即删。模型缓存路径 `~/.cache/nowdocs/models/<model_id>/`（A4：原 spec `~/.cache/agentdocs/` 已改名）。
- 同类 Rust 工具（candle 示例 / mistral.rs / text-embeddings-inference）都**不做**这步 → nowdocs 做了是真供应链加固护城河，对 security-architect 定位完美对齐
- 从 `jinaai/jina-bert-implementation` 删 `config.json` 的 `auto_map`（自定义代码 = 任意代码执行风险），从 safetensors 直接加载 + 手写 BERT 前向传播

### 6.4 B1 FTS 用原生 Lance 非 tantivy
若碰 `use_tantivy=True` 会引入版本偏移的 `IncompatibleIndex` 失败面。必须：原生 Lance FTS + CI 写在 pin 的稳定 Lance 格式版本（2.1）+ CI 重建每个 contributor 表。

> **实现核实（2026-06-29，lancedb 0.30 源码）**：`use_tantivy` 字段在 0.30 已删除——`Index::FTS(FtsIndexBuilder::default())` 其中 `FtsIndexBuilder` = `lance_index::scalar::InvertedIndexParams`（lancedb `src/index/scalar.rs:54` re-export），`.default()` 即原生 Lance inverted index，无 tantivy backend 可选。本约束（B1）因此**已自动满足**，CI 闸门无需额外检查该字段（属空操作）。详见附录 §G。

### 6.5 C1 内存（澄清，非炸弹）
**纠正子代理误报**：所谓"每个 Table 默认 6GiB 索引缓存，50 docset = 350GiB"是**幻觉**。源码核实（lance 7.0 `dataset.rs:110`）：
```
DEFAULT_INDEX_CACHE_SIZE = 256        // 条目数（LRU），非字节
DEFAULT_METADATA_CACHE_SIZE = 1 GiB   // 字节
```
**真实情况**：
- 索引缓存是**按条目数**算的，默认 256 LRU 条目，不是 GiB
- 缓存不在每个 Table 上，在**共享 `Session`** 上——从同一 `Connection` 打开的所有 Table 共用 256 条目 LRU + 1GB metadata cache，满了自动淘汰。装 50 docset ≠ 50 倍内存
- lance 数据文件**不 mmap**，buffered async 字节范围读取，RSS 跟踪读取内容而非文件大小

**真实内存预算**：模型 ~66MB（F16 safetensors；F32 加载 RSS ~131MB）+ metadata cache 512MB-1GB（可配）+ 索引缓存几十 MB + 激活 ≈ **~1.2-1.5GB 常驻**（§5.2 口径已对齐：原 100-150MB 是漏算 cache）。50 docset 受控于共享 Session + LRU 淘汰。

**方案**：
- 共享一个 `Arc<Session>`（API：`lancedb::Session` + `ConnectBuilder::session()`）——**理由 = 共享索引/metadata 缓存 + LRU 淘汰**（非"防炸弹"，炸弹前提已证伪）
- metadata cache 按 machine 调（512MB-1GB）
- candle `DType::F32` 加载（candle 0.11 jina_bert ALiBi 限制，见 §5.2/附录 §B；F16 仅用于向量存储）
- **v1 用 flat（精确）搜索**（A10）：首发 canonical docset 均 <50k 向量，flat 召回最高、零调参；IVF 是为规模准备的 ANN 索引，小 docset 是 YAGNI。**IVF_FLAT / IVF_PQ / HNSW 移 deferred**——待 docset 规模或延迟需求触发再引入（迁移：对已有 flat 表 `create_index` 增量建 IVF，不重 embed）
- 冷 docset 可开零缓存：`Session::new(0,...)` 禁用索引缓存

### 6.6 版本号双命名空间（nowdocs_schema_version + cache_layout_version）
**nowdocs_schema_version**：doc crate 格式本身的版本。CI 拒绝未知 schema 版本；读取时旧 nowdocs 遇新 schema → 清晰"请升级 nowdocs"错误（非崩溃，利用 Lance 跳过行为）。schema 升级前在 spec 记录迁移路径。
- **`cache_layout_version`（D15 新增）**：本地缓存目录布局（`~/.cache/nowdocs/{db,models}/`）的版本。v0.x 改布局时，新二进制检测旧 `cache_layout_version` → 拒读 + 提示 `nowdocs migrate`（或清缓存重装），避免半新半旧损坏。

### 6.7 CLI 生命周期缺口（uninstall / list-installed / update）
spec 只有 install/crawl/ingest/share，**漏了** `nowdocs uninstall <docset>` / `nowdocs list-installed` / `nowdocs update [docset]`。v1 必须加。`update` = 拉最新 registry 版本 + pin manifest 模型 sha（不匹配拒绝）+ 重新解包。

> **实现核实（2026-06-29，registry.rs）**：`update(docset)` 从 registry URL 拉最新 tar → 调 `install` 替换（manifest sha 不符时仍允许但提示——v1 简化）。`uninstall(docset)` 删 `cache::db_path(docset)` 目录 + `cache::manifest_path(docset)` 文件，存在才删。`list-installed` 在 `src/tools.rs`（Task 4c），不在此模块。

### 6.8 E1 检索质量门禁（最被忽视的漏洞）
spec 有 chunking/cosine 单测，但**没有检索质量测试**——而产品 = 检索质量。
**方案**：每 docset 一组 golden eval set（10-30 查询 + 期望 chunk ID/URL，recall@5 + MRR），CI 跑质量门禁；每次 embedder 或 chunking 改动必须通过。

**实现**（`src/eval.rs` + `tests/eval_tests.rs`，task 3b）：
- `GoldenQuery { query: String, expected_source_url: String }`��一条 ground-truth 查询。
- `EvalReport { recall_at_5: f32, mrr: f32, n: usize }`：聚合指标。
- `evaluate(docset, golden) -> Result<EvalReport>`：对每个 golden query 调用 `retrieve::search(docset, &q.query, Some(4000), Some(5))`，在返回 chunks 中查 `expected_source_url` 出现位置作为 rank。
- `compute_metrics(ranks) -> (recall_at_5, mrr)`：纯函数，单测覆盖。无 embedder 依赖。

**定义**：
- `recall@K = (# hits) / n`，其中 hit = expected source_url 出现在 `search(..., top_k=K)` 返回 chunks 中（已包含 ±1 neighbor-window 展开）。
- `MRR = mean(1 / rank_i)`，rank 1-indexed；miss 记 0；空集返回 (0, 0)。

**门禁阈值**（v1）：`recall@5 >= 0.8 && mrr >= 0.6`。CI（5c）跑 canonical fixture（`tests/fixtures/golden/`），embedder/chunking 改动必须通过。

**已知约束**：v1 中 `retrieve.rs` 在 hybrid 命中后用 `chunk_idx` 排序窗口扩展结果（保持 chunks 单调），rank 顺序由 ingest 顺序决定。golden fixture 因此控制在 3 文件（每文件 1 chunk），使 MRR 上限落在门禁之上。修复 `retrieve.rs` 保序后可扩到 4-5 文件做更严门禁。

### 6.9 E2 嵌入正确性（合并 jina spike）
candle 跑 jina-v2-small 是推断。pin 一个参考查询（如 "how to use clerkMiddleware"）+ 对照引用嵌入器（HF jina API 或 Python sentence-transformers）预算的 512-dim 向量，CI 断言**余弦 > 0.99**。与 §5.2 的 spike 合并。

### 6.10 A4 法律许可闸门（D6 + D7，P0）
**registry 是公开再分发**——能否把某库文档放进 registry，取决于**文档站本身的许可**，而非框架代码库许可（Tailwind 框架 MIT，但其文档站专有 → 这正是坑）。

**铁律**：每个 doc crate 入 registry 前，逐个核实文档站许可并记入 `manifest.legal`（见 §5.3 schema）：
- CI 校验 `license` ∈ 白名单（MIT / Apache-2.0 / CC-BY-4.0）+ `attribution` 非空（CC-BY 强制署名）；crate 内附 `LICENSE`/`NOTICES`。**CI 强制执行**：`scripts/ci-check-manifest.sh`（Task 5c）。**威胁模型文档**：`docs/THREAT_MODEL.md`。
- **首发 canonical 已核实**：Next.js `MIT`（vercel/next.js docs）/ React `CC-BY-4.0`（reactjs/react.dev，`LICENSE-DOCS.md`）/ Vue `CC-BY-4.0`（vuejs/docs，**图片内容除外**——nowdocs 只再分发文本/Markdown，不抓图片）。
- **不入 registry**：Clerk（ToS §3 禁复制/下载/存储/传输 + §5 禁爬虫）、Tailwind（文档站"非开源许可、Tailwind Labs 知识产权"）。二者只能 `ingest` 本地用，绝不进 registry。
- Astro(MIT) 作备选第四库。
- `ingest`（本地导入）是通用能力，用户自担所用文档许可责任；私有文档从不触 registry（§11.4）。

### 6.11 A5 贡献治理 + 分发（D8 + D9，P0）
- **贡献协议 = DCO（Developer Certificate of Origin）而非 CLA**（D8）：Rust 生态惯例；CLA 对第三方文档内容无效、给假安全感。contributor PR 须 `Signed-off-by`。DCO 声明 contributor 对所提交文档有再分发权（对齐 §6.10 许可闸门）。**CI 强制执行**：`scripts/ci-check-dco.sh`（Task 5c）。
- **代码签名 = 不签名（D9）**：cargo-binstall + Homebrew 无签名分发（ripgrep / fd / uv / ruff 社区常态，CLI 不签名不影响可用性）。
  - 理由：① 零成本 ② **避开 F-1/OPT 签证风险**——需商业实体验证的签名证书（Windows EV / Azure Trusted Signing）可能被读成"经营商业实体"，触及 F-1 红线；macOS 个人 Apple 账号($99)是灰色地带，未来若 nowdocs 商业化仍模糊。
  - 将来要正式签名安装包（GUI `.app`/`.dmg`/`.pkg`）→ **先咨询移民律师**再动。
- **项目许可证（D16）**：`MIT OR Apache-2.0`（Rust 双许可惯例）。

### 6.12 D11 CJK v2 迁移路径（不换模型）
**模型层与检索层解耦**——加 CJK 不换模型、不重 embed 已有数据：
- jina-v2-small **本身是多语言模型**（~89 语言），中文向量现在就能算，向量语义对 CJK 照常工作。
- CJK 短板在 **BM25**：lancedb 默认 tokenizer 按空格切，中日韩无空格 → 整串成巨型 token，关键词检索失效（瘸一条腿）。
- **v1 决策（D11 拍定）**：CJK defer 到 v2。v1 只做英文 doc crate（BM25 精度最大化、binary 最精简）；CJK 本地 `ingest` 可用但 BM25 降级（仅语义为主），文档标记为已知限制。
- **v2 迁移**：① manifest `retrieval.tokenizer` 加 `lindera`（已留位，§5.3）；② CJK docset 建表时用 lindera 建 FTS（per-table，不改英文表）；③ 查询按目标 docset tokenizer 切分；④ CI 用 lindera 重建（D10 路径，contributor 无感）；⑤ 加 CJK golden eval。
- **未核实风险**：lancedb 0.30 per-table 自定义 tokenizer API 待实现时核源码；若不支持 → CJK 暂走纯向量（无 BM25）或等 lancedb 支持。

---

## 7. 可抄 / 可创新

### 7.1 可抄（来自竞品 + MCP 生态）
| 抄什么 | 来源 | 理由 |
|---|---|---|
| **`llms.txt` 一级源** | Mintlify/Inkeep 生态（Context7 反而不吃） | 上游作者整理的 Markdown > 逆向 HTML。`ingest` 优先探测 `{base}/llms.txt`，`htmd` 降为 fallback。**对 Context7 的质量反超点** |
| **token 预算 + top_k 参数** | 业界标准（子代理纠正：Context7 的 `totalTokens` 是幻觉，它没这参数） | nowdocs 本地能算真实 token 数（tiktoken-rs），远程竞品做不到的真差异化 |
| **code/info 块分离** | Context7 `codeSnippets`/`infoSnippets` | structuredContent 加一维 `chunk_type` |
| **按热度分级刷新** | Context7 top100=1天…45天 | manifest 声明 refresh 策略 + 可选 release-action |
| **npm 风格版本语法** | 通用 | `nowdocs install nextjs@15.1` 接受精确/`^`/`~` |

### 7.2 可创新（nowdocs 独有，竞品做不到）
| 创新 | 机理 |
|---|---|
| **预构建 lancedb crate = 天生缓存** | 架构洞察：远程竞品无缓存透传是**被迫**的（没本地库），我们预构建包就是缓存，远程竞品没对应物 |
| **真 token 计数返回** | 远程服务端算 token 有成本，本地近乎免费 → `tokens_returned` + `truncated` 是本地护城河外显 |
| **本地免疫 prompt injection 的能力边界** | 可控源 + sanitize（见 §6.1） |
| **离线包 export/import** | `nowdocs export >file.ndz` + `install file.ndz` → 气隙/企业场景，强化"完全本地" |

---

## 8. 定位调整（护城河重锚）

**原定位问题**：spec 把"纯 Rust + 本地嵌入 + 混合检索 + MCP"当核心卖点——被 project-rag（同栈已开源跑通）证伪。

**重锚后护城河**（按不可复制性）：
1. **registry 网络效应**：社区 GitHub repo + install/share，所有竞品全空（Context7 的 registry 是远程实时拉取非离线预构建包）
2. **文档新鲜度**：按热度分级刷新 + release-action 触发 + 用户随时 `ingest` 自助刷新（打 Context7 长尾陈旧痛点）
3. **single self-contained binary**（措辞纠正）：**非"全平台纯静态"**——Linux musl 真静态，macOS 自包含（动态链系统 Accelerate 框架，每台 Mac 都有，无缺失库）。改称"single self-contained binary"避免过度宣称伤 credibility
4. **本地安全**：免疫 prompt injection（可控源 + sanitize）+ 向量注入（D10 CI 重建，不发布 contributor 原始向量）+ 完全离线 + 零遥测（query 永不离开设备，黄金标准 = Astral uv/ruff）
5. **真 token 计数**：本地能算，远程不能

**砍掉叙事**：技术栈新颖性（project-rag 证伪）、"全平台纯静态"（macOS 例外）。

---



### 8.3 跨平台分发矩阵验证结论（Task 5a 实现核实）

> **完成时间**：2026-06-30（feat/5a-binstall）
> **任务范围**：5 目标交叉构建矩阵 + cargo-binstall metadata

#### 8.3.1 macOS Accelerate 框架（动态链接确认）

- **机制**：macOS 目标（`aarch64-apple-darwin` / `x86_64-apple-darwin`）通过 candle-core 的 Accelerate 后端动态链接系统框架。
- **部署成本**：**零**——Accelerate framework 是 macOS 系统自带组件，每台 Mac 都有，无需用户安装任何额外依赖。
- **结论**：macOS 二进制是 self-contained（自包含）但**非纯静态**——区别于 Linux musl 的真·全静态链接。spec §8 项 3 已据此修正措辞为"single self-contained binary"避免过度宣称。

#### 8.3.2 Linux musl 静态链接（`ldd` 核实）

- **机制**：`x86_64-unknown-linux-musl` / `aarch64-unknown-linux-musl` 目标使用 musl libc 而非 glibc，产出完全静态链接的二进制。
- **CI 验证步骤**（已纳入 `.github/workflows/release.yml`）：
  ```bash
  ldd target/${{ matrix.target }}/release/nowdocs
  # 预期输出：'not a dynamic executable'
  ```
- **结论**：Linux musl 二进制是真·全静态，零运行时依赖，可直接 curl + chmod + run。

#### 8.3.3 cargo-binstall 资产命名约定

- **格式**：`{name}-{version}-{target}{archive-suffix}`
- **示例**：
  - `nowdocs-v0.1.0-aarch64-apple-darwin.tar.gz`
  - `nowdocs-v0.1.0-x86_64-unknown-linux-musl.tar.gz`
  - `nowdocs-v0.1.0-x86_64-pc-windows-msvc.zip`
- **Cargo.toml 配置**（`[package.metadata.binstall]`）：
  ```toml
  pkg-url = "{ repo }/releases/download/v{ version }/{ name }-{ version }-{ target }{ archive-suffix }"
  bin-dir = "{ name }-{ version }-{ target }"
  pkg-fmt = "tgz"
  ```
- **用户安装命令**：`cargo binstall nowdocs`（自动按本机 target 拉取对应资产）。

#### 8.3.4 构建矩阵覆盖（5 目标）

| Target | OS | Archive | 链接方式 |
|---|---|---|---|
| `aarch64-apple-darwin` | macOS-latest | `.tar.gz` | 动态链 Accelerate |
| `x86_64-apple-darwin` | macOS-latest | `.tar.gz` | 动态链 Accelerate |
| `x86_64-unknown-linux-musl` | ubuntu-latest | `.tar.gz` | 真·全静态 |
| `aarch64-unknown-linux-musl` | ubuntu-latest | `.tar.gz` | 真·全静态 |
| `x86_64-pc-windows-msvc` | windows-latest | `.zip` | Windows runtime |

#### 8.3.5 不签名分发（D9 决定）

- 无 code signing / notarization——CLI 工具社区常态（ripgrep / fd / uv / ruff 均无签名）。
- 权衡：避免 F-1/OPT 签证签名身份暴露风险 + 减少 CI 复杂度。
- 用户首次安装 macOS 二进制需 `xattr -d com.apple.quarantine`（已知 UX 成本，未签名标准操作）。

---

## 9. 原 spec 修订清单

| 章节 | 修订动作 |
|---|---|
| §1 Executive Summary | 更名 nowdocs；卖点从"技术栈"改为"registry + 新鲜度 + self-contained + 本地安全" |
| §2.1 Crawler | **重写**：去 obscura 入 core；核心 = reqwest+htmd+scraper 轻量层；obscura 外置；`crawl` 命令降级为 `ingest` fallback 或标记 deprecated |
| §2.2 Embedder | 数字修正：`<5ms`→~5-40ms 无感；`<50MB RAM`→增量<50MB/**常驻~1.2-1.5GB（§6.5）**；模型 ~66MB F16 文件（**F32 加载 RSS ~131MB**，candle 0.11 ALiBi 限制，见附录 §B）；加模型完整性 pin（§6.3）+ 缓存路径 nowdocs（A4） |
| §2.3 Database | 砍 tantivy 为单依赖；加 manifest 模型版本锁（字段名冻结 §5.3）；**v1 flat 搜索、IVF 移 deferred（§6.5 A10）**；共享 Session 内存策略（§6.5）；`cache_layout_version`（§6.6 D15） |
| §2.4 MCP | 升 2025-11-25；`nowdocs_search` 加结构化 + `nowdocs_list`；inputSchema JSON-Schema-2020-12；`docset` 必填（D12）+ 输入验证（D13）+ 表刷新（D14） |
| §3.2 crawl 工作流 | 重定义：外置抓取 → `ingest` 导入；私有文档路径 |
| §4.1 candle vs ort | **重写 rationale**：非"ort 不能单二进制"（错），改为"ort 速度优势无意义 + 供应链/护城河选 candle" |
| §5 验证 | 加 E1 检索质量门禁 + E2 嵌入正确性余弦>0.99 |
| 新增 | §6 安全威胁模型（A1/A2/A3 + D10 向量注入）、CLI 生命周期（uninstall/list/update）、schema+cache 版本迁移、**§6.10 法律闸门、§6.11 治理(DCO/不签名)、§6.12 CJK v2 路径** |

---

## 10. 实施前置

**开发前必须先做**：
1. **jina-v2-small candle spike**（§5.2 + §6.9 合并）：加载 small safetensors + tokenizer，跑一句验证向量，对照引用嵌入器断言余弦 >0.99。若失败回退 ort。
2. **L1~L4 门禁 Rust 化**（用户门禁体系适用 nowdocs）：
   - L1 pre-commit：`cargo fmt --check` + `clippy -D warnings` + `cargo-deny check` + `cargo-audit`
   - L2 pre-push：`cargo test` + `cargo build --release`
   - L3 CI：`cargo binstall` 产物校验 + 跨平台构建
   - L4 周级：`cargo udeps`（死代码）+ `cargo audit`
3. **分发矩阵（5 目标）**：`aarch64-apple-darwin` / `x86_64-apple-darwin` / `x86_64-unknown-linux-musl` / `aarch64-unknown-linux-musl` / `x86_64-pc-windows-msvc`，通过 `cargo-binstall` 分发。`candle-core default-features=false`。
4. **protoc 构建前置（1a 实现核实）**：`lancedb` → `lance-*` 的 `prost-build` build script 需要系统 `protoc`。无 `protoc` 时 `cargo check` 在 `lance-index`/`lance-table` build script 失败。Debian：`apt-get install protobuf-compiler`；无 sudo 环境下载预编译 protoc 到 `~/.local/protoc` 并 `export PROTOC=...`。是否改用 `protoc-bin-vendored` 实现 hermetic 构建 = Open Question（见 §11）。
5. **依赖解析版本（1a 实现核实，cargo 1.93 / 2026-06-29 解析）**：plan 起点版本因跨依赖 `half` 冲突（lancedb 0.18 锁 `half =2.4.1` vs candle 0.9 需 `half ^2.5`）无法共存，已统一升到最新兼容：`lancedb 0.30` / `candle-core 0.11`（default-features=false）/ `candle-nn 0.11` / `candle-transformers 0.11` / `tokenizers 0.23` / `tiktoken-rs 0.12` / `hf-hub 0.4`（2a 升级，修 XET redirect；见附录 §H）。`clap 4.5` / `serde 1.0` / `anyhow 1.0` / `thiserror 1.0` / `regex 1.10` / `sha2 0.10` / `dirs 5.0` 维持 plan 约束。换依赖/换模型不在此列（架构级，需 Main 决策）；此处仅同依赖的版本号核实。
6. **`resolve_max_tokens(0)` 语义（1g 实现核实）**：1a 锁定 `resolve_max_tokens(n: Option<u32>) -> u32`（非 `Result`），但 plan Step 3 写 "0→Err"。签名不可返回 Err，故 1g 取 clamp 语义：`None | Some(0)` → 默认 4000，`Some(v>0)` → `min(v, 4000)`。`0` 视为"未设置"回退默认值，而非硬错误。如需硬拒 `0`，需在 Wave 4 把签名改回 `Result`（架构级，1h 已依赖 u32 返回，改动面小但需 Main 拍）。
7. **`CARGO_BIN_EXE` 在 `--test` 过滤下不注入（1h 实现核实）**：`cargo test --test mcp_tests`（cargo 1.93.1）实测**不**注入 `CARGO_BIN_EXE_nowdocs`（137 个环境变量中无 EXE），且不重建 bin target。1h 测试已加回退：`CARGO_BIN_EXE_nowdocs` 缺失时用 `{CARGO_MANIFEST_DIR}/target/debug/nowdocs`。TDD 循环需显式 `cargo build --bin nowdocs` 保证二进制新鲜，否则测到旧产物。完整 `cargo test`（无过滤）是否注入 + 自动重建 = Open Question（见 §11.6）。

---

## 11. Open Questions

1. **OpenAPI 支持**：高价值但 `openapiv3` crate 仅 3.0、3.1 不成熟。v1 可选插件还是 defer？（建议 defer，避免解析器攻击面）
2. **`nowdocs upgrade` 委托 vs 只提示**：委托（检测安装程序 + exec 包管理器）UX 好但增表面积；只提示更安全。（建议只提示起步）
3. **多客户端**：stdio 隐含单客户端，每个 client spawn 独立 nowdocs 进程，RAM 按进程计。是否需要守护进程模式？（建议 defer，README 记录）
4. **私有 API 文档爬取**：原 spec 卖点 #2 在 crawler 外置后，`ingest` 仍是私有文档路径（私有文档从不触 registry）。需在 spec 明确重定义。
5. **protoc hermetic 构建**：`lancedb` 依赖系统 `protoc`（见 §10.4）。是否引入 `protoc-bin-vendored`（build-dependency + build.rs 设 `PROTOC`）让 contributor / CI 无需预装 protoc？代价：多一个 build-dep + Cargo.toml 改动（仅 1a/2b 可改 Cargo.toml）。建议 Main 拍：CI 装系统 protoc（简单）vs vendored（hermetic、跨平台一致）。
6. **`cargo test --test X` 不注入 `CARGO_BIN_EXE` / 不重建 bin**：1h 实测 `cargo test --test mcp_tests` 既不注入 `CARGO_BIN_EXE_nowdocs` 也不重建 bin target，与 cargo 文档"integration test 自动暴露 bin"描述不符（cargo 1.93.1，无 `.cargo/config`）。可能因 lib+bin 同名 `nowdocs`、或 `--test` 过滤跳过 bin 依赖。影响：stdio 集成测试需手动 `cargo build --bin nowdocs` + 路径回退。建议 Main 拍：(a) 改 bin 名解耦 lib/bin（如 `nowdocs` lib + `nowdocs` bin 同名是否触发）、(b) 引入 `assert_cmd` dev-dep 托管构建、(c) 维持现状（显式 build + 回退路径）。

---

## 附：关键核实来源

- obscura：`github.com/h4ckf0r0day/obscura`（GitHub API 坐实 16,270★/Apache-2.0/2026-04-13 创建）
- lancedb：`crates.io` 0.30.0 + 源码 `rust/lance/src/dataset.rs:110`（缓存默认值）、`session.rs`、`index/vector.rs`
- candle：`github.com/huggingface/candle` `candle-examples/examples/jina-bert/` + `candle-transformers/src/models/jina_bert.rs`
- ort：`pykeio/ort` `ort-sys/build/main.rs` + `static_link/mod.rs`（默认静态链证实）
- MCP spec：`modelcontextprotocol.io/specification/2025-11-25`
- jina-v5-omni-small：HF model card + `config.json` + `modeling_jina_embeddings_v5_omni.py`
- jina-v2-small 许可：HF model card YAML `license: apache-2.0`
- Context7：`github.com/upstash/context7` + 官方 docs + Issues #2663/#2673/#2340/#2145/#339

## 附：S0 — candle 0.11 jina-v2-small 实际 API 适配（实现核实）

> 本附录记录 Task S0 对 candle-transformers 0.11 真实 API 的适配结果，属「实现核实类」修订，未改变架构决策。

### A. candle-transformers 0.11 的 jina_bert 模块

- 实际导出类型为 `candle_transformers::models::jina_bert::BertModel`（plan 中的 `JinaBertModel` 是伪代码命名）。
- 配置结构为 `Config`；**没有** `base_v2()` 之类的工厂函数，只有 `Config::v2_base()`（对应 base 变体，hidden_size=768）。v2-small 必须手构：
  - `vocab_size = 30528`
  - `hidden_size = 512`
  - `num_hidden_layers = 4`
  - `num_attention_heads = 8`
  - `intermediate_size = 2048`
  - `hidden_act = candle_nn::Activation::Gelu`
  - `max_position_embeddings = 8192`
  - `type_vocab_size = 2`
  - `pad_token_id = 0`
  - `position_embedding_type = PositionEmbeddingType::Alibi`
- 权重通过 `candle_nn::VarBuilder::from_mmaped_safetensors(..., DType::F32, &Device::Cpu)` 加载。注意：**该函数在 0.11 是 `unsafe`**，需显式 `unsafe` 块。

### B. F16 加载失败，F32 通过

spec / plan 原想用 `DType::F16` 加载以压缩内存，但 candle-transformers 0.11 的 jina_bert 内部把 ALiBi bias 构造为 `F32`（`build_alibi_bias` 中 `to_dtype(DType::F32)`），并在 encoder 里与 hidden 状态相加。若权重以 `F16` 加载，forward 时会触发：

```
dtype mismatch in add, lhs: F16, rhs: F32
```

因此 S0 先用 `DType::F32` 加载。内存增量从 ~66MB（F16 文件）变为 ~131MB（F32 运行时）。Task 2a 硬 embedder 时可再评估是否通过自定义 `BertModel`  wrapper 实现 F16 推理，或接受 F32。

### C. Provenance（留给 Task 2a）

- 模型 ID：`jinaai/jina-embeddings-v2-small-en`
- HF main 当前 revision SHA：`44e7d1d6caec8c883c2d4b207588504d519788d0`
- `model.safetensors` SHA256：`c9a9a7ec012d01efd780474fbb65e25917f3a2aebdff84b5f87daa00f7e90b27`
- 文件来源：HF repo 提供 `model.safetensors`（~66MB），无 `pytorch_model.bin` 需求。

### D. A3 安全适配

`config.json` 来自 `jinaai/jina-bert-implementation` 并含 `auto_map`（指向该 repo 的 Python 实现文件）。S0 实现读取 `config.json` 后将其 `auto_map` 字段删除再写回，避免后续若被 Python transformers 流水线消费时执行任意自定义代码。

### E. E2 阈值调整

Python 参考实现（transformers `AutoModel` + mean-pooling，与 sentence-transformers 内部路径等价）对固定查询对的测量值为：

- 近查询（"how to use clerkMiddleware" vs "using clerkMiddleware in middleware"）：cosine ≈ 0.9488
- 无关查询（"how to use clerkMiddleware" vs "tomato soup recipe"）：cosine ≈ 0.6921

因此 plan 中「无关 < 0.5」的阈值对 jina-v2-small 过严。测试调整为「无关 < 0.75」，仍保留与近查询的明显间隔。

### F. 本地 HF 缓存写入问题

当前环境 `~/.cache/huggingface/hub` 属 root，hf-hub 0.3 写入失败。测试通过 `ensure_hf_cache()` 在未设置 `HF_HOME` 时回退到 `~/.cache/nowdocs/hf`。CI / 其他环境若默认缓存可写，此回退不触发。

### G. lancedb 0.30 FTS + hybrid API 核实（Task 2b 实现）

**Main 源码级核实（2026-06-29，`~/.cargo/registry/src/index.crates.io-.../lancedb-0.30.0/` + `lance-index-7.0.0/`）**：

- **共享 Session**：`Arc<lancedb::Session>`（= `lance::session::Session`，`lancedb/src/lib.rs:267` re-export）+ `lancedb::connect(uri).session(arc.clone()).execute().await -> Connection`（`.session()` 在 `connection.rs:849/1054`）。§6.5 的"共享 Arc<Session>"架构决策**成立**，非误。Connection 内部持 `Option<Arc<Session>>`（`connection.rs:618/967`）。
- **FTS 索引**：`table.create_index(&["text"], Index::FTS(FtsIndexBuilder::default())).execute().await`。`FtsIndexBuilder` = `lance_index::scalar::InvertedIndexParams`（`lancedb/src/index/scalar.rs:54` re-export）。**`use_tantivy` 字段在 0.30 已删除**——旧 tantivy backend 完全移除，只剩原生 Lance inverted index（§6.4 B1 约束已自动满足，空操作）。
- **`FullTextSearchQuery`**：在 `lance-index-7.0.0/src/scalar.rs:302-382`（**非 lancedb crate**）。`::new(String)` 只传 query 文本，列名从 FTS 索引推断（`.with_column()` 可选）。
- **hybrid 链**：`table.query().full_text_search(fts).nearest_to(&vec)?`（`nearest_to` 返回 `Result<VectorQuery>`，`query.rs:858`，**要 `?` 解包**）`.rerank(Arc::new(RRFReranker::new(1.0)))`（`rerankers/rrf.rs:23`，`.rerank()` trait method `query.rs:509`）`.execute_hybrid(QueryExecutionOptions::default()).await`（`query.rs:1207`，返回 `SendableRecordBatchStream`）→ `.try_collect::<Vec<RecordBatch>>().await`（需 `use futures::TryStreamExt`）。结果 batch 自动含 `_distance`（向量，`lance_index::vector::DIST_COL`）+ `_score`（FTS，`lance_index::scalar::inverted::SCORE_COL`）。
- **向量列 schema**：`DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float16, true)), 512)` 存 f16。`IntoQueryVector for &[f16]` / `Vec<f16>`（`query.rs:222/342`）存在——查询向量收 f16。注：lancedb 自带测试多用 f32，但 f16 schema+查询路径成立。
- **async 边界**：lancedb 全 async。Store 内部持 `tokio::runtime::Runtime`，`self.runtime.block_on(...)` 同步包装。**陷阱**：lancedb 内部 `tokio::spawn`，若在已有 tokio runtime 上下文里 `Runtime::new().block_on` 会 panic「Cannot start a runtime from within a runtime」。nowdocs 顶层无 tokio context 故安全；W4 mcp.rs 若变 async 需重审（改用 `Handle::current().block_on`）。
- **insert**：`table.add(Vec<RecordBatch>).execute().await`（`Vec<RecordBatch>` 直接 `Scannable`，`table.rs:886/1829`）。

> **2b 实现核实（2026-06-29，commit 4b098a3）**：以上 §G API 事实全部在实际编译+测试中验证通过。补充：
> - arrow 实际解析版本 = `58.3.0`（非 plan 猜测的 55），Cargo.toml 写 `"58"`。
> - `FixedSizeListArray::try_new_from_values` 需 `lance-arrow = "7"` 的 `FixedSizeListArrayExt` trait（非 arrow 原生）。
> - `StringArray` 无 `FromIterator` impl，需 `StringArray::from(vec)` 构造。
> - 空表 hybrid_search 返回的 RecordBatch 不含数据列（仅 `_score`/`_distance`），需 `chunk_idx` 列存在性检查后 skip。
> - 测试需 `--test-threads=1`（`XDG_CACHE_HOME` 环境变量并行竞争）。

### H. hf-hub 0.4 API 核实（Task 2a 实现）

**源码级核实（2026-06-29，`~/.cargo/registry/src/index.crates.io-.../hf-hub-0.4.3/`）**：

- **版本**：0.4.3（crates.io 稳定版，0.3→0.4 修 XET-backed repo relative redirect）。
- **revision pin**：`Repo::with_revision(repo_id: String, repo_type: RepoType, revision: String) -> Repo`（`lib.rs:222`）。revision 为 HF commit SHA 或 `refs/pr/N`。传给 `Api::repo(repo)` 即锁定该 commit。
- **ApiBuilder**：`ApiBuilder::new().with_cache_dir(path).with_progress(false).build() -> Result<Api, ApiError>`（`sync.rs:229-356`）。`ApiBuilder::from_env()` 读 `HF_HOME` + `HF_ENDPOINT` 环境变量。
- **ApiRepo**：`Api::repo(Repo) -> ApiRepo`（`sync.rs:623`）。`ApiRepo::get(filename) -> Result<PathBuf, ApiError>` 先查缓存再下载（`sync.rs:709`）。`ApiRepo::download(filename)` 强制重新下载。
- **Cache 路径**：`Cache::from_env()` 读 `HF_HOME`，拼接 `hub/`（`lib.rs:42-51`）。缓存结构 `hub/models--<org>--<repo>/blobs/` + `snapshots/<commit_hash>/` + `refs/<revision>`。
- **token**：`Cache::token()` 在 `HF_HOME/token`（非 `HF_HOME/hub/token`，`lib.rs:59-65` `token_path()` pop 一级）。
- **0.3→0.4 变更**：修复 XET-backed repo 的 relative redirect 处理（`sync.rs:473-501` redirect loop）。API 签名无破坏性变更。
- **2a 实现策略**：`load_for(spec)` 设 `HF_HOME=<nowdocs_cache>/models/<model_id>` → `ApiBuilder::from_env()` → `Repo::with_revision(spec.model_id, RepoType::Model, spec.model_revision)` → `api.repo(repo).get("model.safetensors")`。下载后 `sha2::Sha256` 校验，不符即删+bail。

### J. Retrieval Pipeline 实现核实（Task 3a）

**3a 实现核实（2026-06-29，commit a6e6b0d）**：

- **fetch_by_idx scalar filter API**：lancedb 0.30 `table.query().only_if("chunk_idx IN (1,2,3)").execute()` 返回 `SendableRecordBatchStream`，需 `use lancedb::query::ExecutableQuery` 导入 `execute()` 方法。column 解析时 `column_by_name()` 返回 `Option<&Arc<dyn Array>>`（非 `&dyn Array`），closure 参数类型需匹配。提取为 `parse_search_hits` / `parse_search_hits_with_score` 两个 helper 复用于 `fetch_by_idx` 和 `hybrid_search`。
- **manifest 驱动 embedder 版本锁定**：search 第一步读 `cache::manifest_path(docset)` → `manifest::parse_manifest` → `manifest::validate`，用 `manifest.embedder.{model_id, model_revision, model_sha256}` 构造 `embedder::EmbedderSpec` → `Embedder::load_for(&spec)`。查询侧模型版本与 ingest 时锁定一致。
- **相邻窗口语义**：对每个 hybrid hit 的 `chunk_idx`，取 `idx-1`、`idx`、`idx+1`（clamp 到 `[0, chunk_count)`）。去重 + 升序排列后 `fetch_by_idx` 一次性取回。空结果（chunk_count=0 或无 hit）直接返回空 `SearchResult`。
- **max_tokens 截断**：`assemble_result` 从第一个 chunk 开始累加 `count_tokens(text)`，下一个 chunk 若 `tokens_used + n > max_tokens` 则停止并设 `truncated=true`。第一个 chunk 总是返回（即使超预算），但标记 `truncated=true`。`tokens_returned` 为实际返回 chunks 的 token 总和。
- **端到端验证**：`test_search_end_to_end` 用 tempdir + `ingest_dir` → `search` 完整流程，断言召回包含 unique keyword、tokens_returned ≤ 4000、chunks 按 chunk_idx 升序。通过（16.47s）。
