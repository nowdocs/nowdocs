# nowdocs 派发手册（Agent Dispatch）

> **用途**：本文件是 nowdocs 实现的 agent 派发清单。每个 task 一份可直接复制粘贴的 prompt。
> 所有 prompt 共享 §0 派发总则（铁律 + 完成动作 + 打勾机制），各 task prompt 只写专属契约。
> **plan**：`docs/superpowers/plans/2026-06-28-nowdocs-impl.md`（TDD step 全文）
> **spec**：`docs/superpowers/specs/2026-06-28-nowdocs-design-review.md`（设计依据，矛盾时以 spec 决策为准）
> **进度看板**：见 §7（Main 维护，记录每个 task 的 ✅ + commit sha）

---

## §0 派发总则（每份 prompt 的共享前缀）

把下面这段贴在每份 task prompt 前面，再接 task 专属段。

```
你是 nowdocs 项目的实现 agent。

【共享铁律】
- TDD 严格执行：先写失败测试→验证失败→最小实现→验证通过→commit。每步都跑测试验证，禁止跳过。
- 签名锁定：函数签名必须与 plan「Interfaces (LOCKED)」一致；1a 已建好 stub，你只填函数体，不改签名。
- 边界：只改本 task 声明的文件（src/<mod>.rs + tests/<mod>_tests.rs）。不动 Cargo.toml（除非本 task 明确要求）、不动其他 module、不改其他 task 的测试。
- 中文交流（专业词带英文标注），干练直接，不背诵免责套话。
- 子代理非交互：遇阻塞或未明决策，用合理默认值推进或记为 Open Question 上报，严禁调用交互提问工具。
- 命令输出管控：dev/build/test 产生大量日志时后台重定向，不直接 dump 进上下文。
- 提交规范：conventional commits 英文，消息带 task 编号（如 feat(manifest): ... (1b)）。每个 task 末尾必 commit。

【完成后必做（三件事，缺一不可）】
1. 打勾：用 Edit 把 plan 文件中【本 Task】的所有 `- [ ]` 改为 `- [x]`。
   - 精确匹配本 task 的 step 文本（每个 step 的 `- [ ] **Step N...` 带上下文唯一）。
   - 若报 "File has been modified since read"：重新 Read 再 Edit（并发安全重试）。
   - 这是计划内进度更新，直接执行，不需额外请示。
2. spec 修订（仅限「实现核实类」事实）：若实现中验证了 spec 里的技术假设有误
   （如真实 API 名/方法签名/版本号/许可证核实结果），更新 spec 对应章节 + 附录来源，
   并在最终报告列出 spec diff 供 Main 复核。**边界**：架构级变更（删功能/换依赖/换模型/推翻已拍板决策）
   不在你权限内——列为 Open Question 上报，不擅自动 spec。
3. 最终报告：返回 ① task 编号 ② commit sha ③ 测试结果（PASS/FAIL + 关键断言）④ spec diff（若有）⑤ Open Questions（若有）。然后停下，不做未分配的任务。

【读取顺序】
先读 plan 的 Global Constraints + File Structure + 你的 Task 全文，再读 spec 相关章节。
矛盾时：spec 的架构决策 > plan 的实现细节；若 plan 与 spec 冲突，停下上报（属 Open Question）。
```

---

## §1 Wave 0 — S0 命门（单 agent，先验）

### Task S0：candle + jina-v2-small spike + E2 余弦门禁
> **依赖**：无（可先于 1a）。**派给**：Rust 主力（CC/Codex）。**阻塞**：失败则 candle 路线回退 ort，全 Wave 2 重评估。

```
【任务】Task S0：jina-v2-small 在 candle 上跑通 + E2 余弦>0.99 断言（命门）
【plan】读 plan 的 Task S0 全部 step（含 gen_reference.py / embedder_tests.rs / embedder.rs 完整代码）
【spec】§5.2 Embedder、§6.3 A3 模型完整性、§6.9 E2

【专属契约】
- 模型：jinaai/jina-embeddings-v2-small-en，vector_dim=512，DType::F16。
- 产出的 Embedder 接口：pub fn load() -> anyhow::Result<Self> + pub fn embed(&self, text:&str) -> anyhow::Result<Vec<f32>>（512 维）。Wave 2 Task 2a 会扩展为 load_for(spec) + sha 校验，保持 load/embed 签名稳定。
- E2 三层断言：① dim==512 ② 语义自洽（近查询 cosine>0.7、无关<0.5）③ 跨实现 cosine>0.99（需 tests/fixtures/jina_ref.json，由 gen_reference.py 生成；无 Python 环境则标 #[ignore] 后补）。
- spike 阶段 model_revision/sha256 可暂不 pin（Wave 2 Task 2a 补），但若已能 pin 则顺手 pin。
- candle API（JinaBertModel::Config::base_v2 / load 签名）跨版本会变：以 cargo 解析的实际版本为准适配调用，保持 load/embed 签名不变。适配结果写入 spec 附录（属「实现核实类」修订）。

【命门判定】
- 三层断言全 PASS（③ 可 #[ignore]）→ 绿，汇报可开 Wave 1。
- load 失败 / dim 错 / 语义断言 FAIL → 不强凑。记录失败根因（model load error? pooling 错? dim 不对?），列为 Open Question 上报 ort 回退决策，停下。

【注意】若 1a 尚未跑、Cargo.toml 不存在，你需在本 task 的 Step 自行创建 Cargo.toml（按 plan 1a Step 1 的依赖清单），或先跑 1a。优先先跑 1a 再跑 S0（1a 已含 candle 依赖）。与 Main 确认 1a 状态后再开。
```

---

## §2 Wave 1 — 地基（1a 先行，1b-1h 并行）

### Task 1a：Cargo 骨架 + 全 stub + 锁签名（BLOCKER，单 agent）
> **依赖**：无。**派给**：Rust 主力。**阻塞**：1b-1h 全等 1a。

```
【任务】Task 1a：Cargo.toml + lib/bin 双 target + 7 个 module stub + 锁定函数签名
【plan】读 plan 的 Task 1a 全部 step（含 Cargo.toml / cli.rs / main.rs / lib.rs / 各 stub 完整代码）
【spec】§5.4 MCP+CLI、Global Constraints

【专属契约】
- crate name=nowdocs，license="MIT OR Apache-2.0"，edition 2021，lib+bin 双 target。
- Cargo.toml 一次性加全 Wave1+S0 依赖（clap/serde/serde_json/anyhow/thiserror/regex/sha2/dirs/tiktoken-rs/candle-*/tokenizers/hf-hub/lancedb）。candle-core default-features=false。版本以 cargo add 核实最新兼容版为准（plan 给的是起点）。仅 lancedb 行允许 Wave2 store task 后续改。
- CLI 7 子命令：Serve(无参！)/Install/Ingest/Share/Uninstall/ListInstalled/Update。serve 绝不带 --host/--port（stdio 不绑端口，network 防线铁律）。
- 7 个 stub module 各自含 plan「Interfaces (LOCKED)」的完整 struct + 函数签名，函数体 todo!("1x")。签名必须与 plan 完全一致——这是 1b-1h 并行无冲突的前提。
- 若 S0 未跑，创建占位 src/embedder.rs（// placeholder，见 S0）使 lib.rs 编译通过；S0 跑完后替换。
- 验收：cargo test --test cli_tests PASS + cargo check 编译通过（stub）。
```

**1a 合并后，下面 1b-1h 可并行。建议两批（1c↔1d 软依赖，1d 先）：**
- 批次 1 并行 3 路：1d →（1c 等 1d）/ 1b / 1e
- 批次 2 并行 3 路：1f / 1g / 1h（1h 可先 stub 接 1e+1g）

### Task 1b：Manifest 解析 + 校验
> **依赖**：1a。**派给**：Rust 副手 / Legal（法律白名单段）。

```
【任务】Task 1b：parse_manifest + validate（schema 版本 + 模型版本锁 + 法律白名单）
【plan】读 Task 1b 全部 step（含 manifest_tests.rs 完整测试 + validate 规则）
【spec】§5.3 manifest schema、§6.10 A4 法律闸门、§4.2 footgun

【专属契约】
- 填 src/manifest.rs 的 parse_manifest（serde_json::from_str）+ validate，不改 struct 定义（1a 已建）。
- validate 规则：nowdocs_schema_version==1；embedder.model_id=="jinaai/jina-embeddings-v2-small-en" && vector_dim==512 && engine=="candle" && dtype=="f16"；legal.license∈{MIT,Apache-2.0,CC-BY-4.0}，CC-BY-4.0 时 attribution 非空；retrieval.tokenizer=="default"（v1）。
- 字段名冻结：model_id/model_version/model_revision/model_sha256（杜绝 version/sha256 漂移）。
- 只改 src/manifest.rs + tests/manifest_tests.rs。
```

### Task 1c：code-aware markdown chunker
> **依赖**：1a；软依赖 1d（count_tokens，未就绪用 char fallback 并标 TODO）。**派给**：Rust 副手 / Designer。

```
【任务】Task 1c：chunk_markdown（heading 路径前缀 + 不劈开 code fence + token 预算）
【plan】读 Task 1c 全部 step（含 Chunk/ChunkType/ChunkConfig 签名 + 测试）
【spec】§5.1 Chunk 策略

【专属契约】
- 填 src/chunker.rs：ChunkType{Code,Info}、Chunk{idx,heading_path,source_url,api_version,chunk_type,text}、ChunkConfig{min:256,max:512,target:384,window:2048}、default_config()、chunk_markdown(md,cfg)->Vec<Chunk>。
- 规则：按 heading 切（维护路径栈，heading_path="Title > Sub"）；段落按 count_tokens 切到 target；绝不切进 fenced code block（```），超 max 的 code block 独立成 chunk（允许超限）；每 chunk 文本前缀加 heading 路径行（contextual enrichment）；idx 顺序；chunk_type=Code 若主体是 fenced code 否则 Info。
- count_tokens 优先用 token::count_tokens（1d）；1d 未就绪则 char-based 估算并留 TODO 注释指向 1d。
- 只改 src/chunker.rs + tests/chunker_tests.rs。
```

### Task 1d：真实 token 计数
> **依赖**：1a。**派给**：Rust 副手。

```
【任务】Task 1d：count_tokens（tiktoken cl100k_base，OnceLock 缓存）
【plan】读 Task 1d 全部 step
【spec】§7.1 真 token 计数

【专属契约】
- 填 src/token.rs：pub fn count_tokens(text:&str)->usize。用 tiktoken_rs::cl100k_base()，OnceLock 缓存 tokenizer（BPE 加载贵）。encode_ordinary(text).len()。
- 测试：空串=0；"hello world" 在 2..6；确定性（同输入同输出）。
- 只改 src/token.rs + tests/token_tests.rs。
```

### Task 1e：缓存目录 + CACHE_LAYOUT_VERSION
> **依赖**：1a。**派给**：Rust 副手。

```
【任务】Task 1e：cache_root/db_path/model_path/ensure_layout + CACHE_LAYOUT_VERSION=1
【plan】读 Task 1e 全部 step
【spec】§6.6 cache_layout_version、Global Constraints（缓存路径 nowdocs 非 agentdocs）

【专属契约】
- 填 src/cache.rs：const CACHE_LAYOUT_VERSION:u32=1；cache_root()->PathBuf（dirs::cache_dir().join("nowdocs")）；db_path(docset)->join("db/{docset}.lance")；model_path(model_id)->join("models/{model_id}/")；ensure_layout()->Result（建 db/+models/，读写 .layout_version 文件，不匹配→Err 提示 "run nowdocs migrate"）。
- 路径必须用 nowdocs，禁止 agentdocs（A4 已改名）。
- 测试用 tempfile 或设 HOME。
- 只改 src/cache.rs + tests/cache_tests.rs。
```

### Task 1f：prompt-injection sanitizer
> **依赖**：1a。**派给**：Rust 副手（安全核心，建议 Test agent 复核）。

```
【任务】Task 1f：sanitize_chunk + sanitize_metadata（注入防御）
【plan】读 Task 1f 全部 step（含完整测试用例 + 5 步清洗规则）
【spec】§6.1 A1 四重防御

【专属契约】
- 填 src/sanitize.rs：sanitize_chunk(text)->String、sanitize_metadata(text)->String。
- sanitize_chunk 五步：① 剥 HTML 注释 <!--..--> ② 剥零宽字符（U+200B/C/D, U+FEFF, U+2060）③ 剥 display:none 元素 ④ 剥助手导向短语（ignore previous/prior instructions、note for the assistant、you may run、as an ai、system prompt）⑤ 剥危险标志独立 token（-y/--yes/--force/sudo/rm -rf）。
- sanitize_metadata：仅剥零宽 + 长度上限 500 字符（metadata 短，不必全 HTML 剥）。
- 测试覆盖：注入短语、HTML 注释、零宽、display:none、危险标志、metadata。
- 只改 src/sanitize.rs + tests/sanitize_tests.rs。
```

### Task 1g：工具输入校验
> **依赖**：1a。**派给**：Rust 副手。

```
【任务】Task 1g：validate_docset/validate_query/resolve_max_tokens/resolve_top_k
【plan】读 Task 1g 全部 step
【spec】§5.4 默认值、D12/D13

【专属契约】
- 填 src/input.rs：validate_docset（正则 ^[a-z0-9._-]{1,64}$ + 拒 ..）、validate_query（max 4096 字符）、resolve_max_tokens（None→4000，Some(v)→min(v,4000)，0→Err）、resolve_top_k（None→5，钳 [1,20]）。
- 测试：大小写/路径遍历/超长 docset 拒；query 边界；max_tokens/top_k 钳位与默认。
- 只改 src/input.rs + tests/input_tests.rs。
```

### Task 1h：MCP stdio 骨架（2025-11-25）
> **依赖**：1a；软依赖 1e+1g（未就绪先 stub 校验，后补）。**派给**：Rust 主力。

```
【任务】Task 1h：run_loop（initialize + tools/list + tools/call stub）
【plan】读 Task 1h 全部 step
【spec】§4.4 MCP 升级、§5.4 工具集、§6.1 structuredContent

【专属契约】
- 填 src/mcp.rs：run_loop()->io::Result<()>。NDJSON 行读写（单 \n 分隔，非 Content-Length）。
- initialize→protocolVersion:"2025-11-25"（非 2024-11-05！），capabilities:{"tools":{"listChanged":false}}，serverInfo.name="nowdocs"。
- tools/list→两个工具：nowdocs_search（inputSchema required:["query","docset"]，含 max_tokens/top_k 可选，annotations readOnlyHint:true/openWorldHint:false）、nowdocs_list（inputSchema 空 object，同 annotations）。
- tools/call→暂返 JSON-RPC error code:-32601 "tool not yet implemented"（Wave 4 Task 4b 接真 search）。先调 input::validate_* 校验输入，非法→error with message。
- 1e/1g 未就绪时：校验段先 stub（todo 或宽松放行），后补；ensure_layout 在 run_loop 开头调（若 1e 未就绪先 skip 留 TODO）。
- 测试：spawn `cargo run -- serve`，pipe NDJSON，验 initialize/tools/list/tools/call 三响应。
- 只改 src/mcp.rs + tests/mcp_tests.rs。
```

---

## §3 Wave 2 — 引擎（2a→2b 串行；2c off 2b）

### Task 2a：embedder 加固
> **依赖**：S0 绿、1b、1e。**派给**：Rust 主力。

```
【任务】Task 2a：Embedder::load_for(spec) + sha 校验 + auto_map 剥除 + F16/mmap
【plan】读 Wave 2「2a embedder hardening」contract；TDD step 自行展开（plan 给了 contract，需你补测试细节）
【spec】§5.2、§6.3 A3 模型完整性

【专属契约】
- 扩展 src/embedder.rs（S0 已有 load/embed）：加 load_for(spec:&EmbedderSpec)->Result<Self>，按 manifest 的 model_revision pin HF commit SHA + model_sha256，下载后 sha2::Sha256 重算比对，不符即删；从 config.json 删 auto_map（防任意代码执行），手写 BERT 前向；F16 加载 + mmap。
- 保持 load()/embed() 签名不变（S0 锁定，2a 不破坏）。
- 新测试：拒篡改 sha 的模型；E2 仍绿。
- 依赖：需 1b（EmbedderSpec）、1e（model_path）。只改 src/embedder.rs + tests/embedder_tests.rs。
```

### Task 2b：lancedb store
> **依赖**：2a、1e。**派给**：Rust 主力。

```
【任务】Task 2b：共享 Arc<Session> + 表 schema + 原生 Lance FTS + hybrid 查询
【plan】读 Wave 2「2b lancedb store」contract
【spec】§4.2、§6.4 B1、§6.5 C1 内存

【专属契约】
- 新建 src/store.rs：open 共享 Arc<Session>（cache::db_path），表 schema(id,vector,heading_path,source_url,api_version,text,chunk_type)；建原生 Lance FTS（**禁 use_tantivy=True**，§6.4）；hybrid query().full_text_search().nearest_to().rerank(RRFReranker).execute_hybrid()。
- v1 用 flat 精确搜索（IVF/HNSW 移 deferred，§6.5 A10）。
- lancedb 版本以 cargo 解析为准（plan 起点 0.18，spec 内部参考 0.30）；实际 API 名核实后写入 spec 附录（属实现核实类修订）。
- 测试：insert→hybrid-search 召回 round-trip。只改 src/store.rs + tests/store_tests.rs + Cargo.toml 的 lancedb 行（仅此 task 可改该行）。
```

### Task 2c：ingest
> **依赖**：1c、2a、2b。**派给**：Rust 副手。

```
【任务】Task 2c：md 目录→chunker→embedder→store.insert
【plan】读 Wave 2「2c ingest」contract
【spec】§5.1 ingest 路径

【专属契约】
- 新建 src/ingest.rs：ingest_dir(dir,name)->Result，读 md→chunker::chunk_markdown→embedder.embed 每 chunk→store.insert；写 manifest。
- 测试：ingest fixture 目录→search 返回期望 chunk。只改 src/ingest.rs + tests/ingest_tests.rs + fixture。
```

---

## §4 Wave 3 — 检索 + 评估（3a→3b）

### Task 3a：retrieval pipeline
> **依赖**：2b、1c、1d。**派给**：Rust 主力 / Designer（窗口 UX）。

```
【任务】Task 3a：search(docset,query,max_tokens,top_k)->结果（hybrid+邻窗口+token 预算）
【plan】读 Wave 3「3a retrieval pipeline」contract
【spec】§5.4 max_tokens vs 窗口、D12 docset 必填

【专属契约】
- 新建 src/retrieve.rs：search(docset,query,max_tokens,top_k)->SearchResult。hybrid 查询→取 top_k chunk→拼邻窗口（~2048 token）→按 max_tokens 预算迭代填充，触上限停并置 truncated；返回 tokens_returned（用 count_tokens）。docset 必填（D12）。
- 测试：返回 token 数 ≤ max_tokens；truncated 正确置位。只改 src/retrieve.rs + tests/retrieve_tests.rs。
```

### Task 3b：golden eval
> **依赖**：3a。**派给**：Test agent。

```
【任务】Task 3b：per-docset golden set（recall@5 + MRR）+ CI 门禁
【plan】读 Wave 3「3b golden eval」contract
【spec】§6.8 E1 检索质量门禁

【专属契约】
- 新建 src/eval.rs + tests/eval_tests.rs：每 docset 10-30 查询 + 期望 chunk url，算 recall@5 + MRR；CI 质量门禁（embedder/chunking 改动必过）。
- 用 canonical fixture（Next.js 等）。只改 src/eval.rs + tests/eval_tests.rs + golden fixture。
```

---

## §5 Wave 4 — 装配

### Task 4b：search 工具接线
> **依赖**：3a、1f、1g、1h。**派给**：Rust 主力。

```
【任务】Task 4b：tools/call nowdocs_search→retrieve::search + sanitize + structuredContent
【plan】读 Wave 4「4b search tool wiring」contract
【spec】§5.4 工具集、§6.1 sanitize

【专属契约】
- 新建 src/tools.rs：替换 1h 的 tools/call stub。nowdocs_search 调 retrieve::search，返回 text 经 sanitize_chunk、metadata 经 sanitize_metadata；structuredContent JSON 数组[{score,heading_path,source_url,api_version,chunk_type,text,chunk_idx}] + text fallback。
- 测试：端到端 stdio 搜索。只改 src/tools.rs + src/mcp.rs（接线点）+ tests/tools_tests.rs。
```

### Task 4c：list 工具
> **依赖**：1e、1h。**派给**：Rust 副手。

```
【任务】Task 4c：nowdocs_list→枚举 cache::db_path 已装 docset
【plan】读 Wave 4「4c list tool」contract
【专属契约】
- src/tools.rs 加 list 处理：枚举 cache::db_path 下 *.lance 目录。测试：装 2 docset→list 返回 2。只改 src/tools.rs + tests。
```

### Task 4e：install / 4f：share（registry CLI）
> **依赖**：1b、1e（4f 另需 1c）。**派给**：Rust 副手 / Legal（share 的法律字段校验）。

```
【任务】Task 4e+4f：install（拉 registry Releases）+ share（打包文本+manifest+config，禁发向量）
【plan】读 Wave 4「4e install / 4f share」contract
【spec】§6.2 A2（URL 必须指向 nowdocs-registry 域）、§6.10 法律闸门、D10（share 发文本 CI 重建）

【专属契约】
- 新建 src/registry.rs：
  - install(docset)：从 nowdocs-registry GitHub Releases 拉到 cache::db_path；下载 URL 必须 registry 自有域（拒外部，§6.2）；校验 manifest 模型 sha。
  - share(docset)：打包分块文本+manifest+config（**绝不打包向量**，D10）；manifest 必须过 1b validate（含 legal 字段，§6.10）。
- 测试：URL 域校验拒外部；share 产物无向量文件。只改 src/registry.rs + tests/registry_tests.rs。
```

### Task 4d：CLI 集成
> **依赖**：4b/4c/4e/4f/4g。**派给**：Rust 主力（集成点）。

```
【任务】Task 4d：main.rs 把 install/ingest/share/uninstall/list-installed/update 接到真实模块
【plan】读 Wave 4「4d CLI integration」contract
【专属契约】
- 改 src/main.rs：把 1a 的 println! 占位换成真实调用（registry/ingest/mcp 等）。
- 集成点，最后做。测试：各子命令端到端冒烟。只改 src/main.rs + tests/cli_tests.rs。
```

### Task 4g：update / uninstall
> **依赖**：1e、4e。**派给**：Rust 副手。

```
【任务】Task 4g：update（拉最新+校验 sha+重解包）/ uninstall（删 db_path）
【plan】读 Wave 4「4g update/uninstall」contract
【spec】§6.7 CLI 生命周期
【专属契约】
- src/registry.rs 加 update（拉最新 registry 版本 + pin manifest 模型 sha 不符拒 + 重解包）+ uninstall（删 cache::db_path(docset)）。只改 src/registry.rs + tests。
```

---

## §6 Wave 5 — 分发 + 治理（大体独立，5 路并行）

### Task 5a：cargo-binstall 矩阵
> **派给**：Rust 主力 / Analyst。

```
【任务】Task 5a：5 目标构建矩阵 + cargo-binstall metadata
【plan】读 Wave 5「5a」+ §10 实施前置
【spec】§8.3 single self-contained binary、§6.11 D9 不签名

【专属契约】
- 5 目标：aarch64/x86_64-apple-darwin、x86_64/aarch64-unknown-linux-musl、x86_64-pc-windows-msvc。
- Cargo.toml 加 [package.metadata.binstall]；candle-core default-features=false。GitHub Actions 跨平台构建。
- 不签名（D9）。核实 macOS 动态链 Accelerate（每台 Mac 有）、Linux musl 真静态——写入 spec §8.3 附录（实现核实类）。
```

### Task 5b：Homebrew tap
> **派给**：Rust 副手。

```
【任务】Task 5b：unsigned Homebrew formula
【spec】§6.11 D9
【专属契约】
- nowdocs-homebrew tap 仓库 + formula（无签名，从 GitHub Releases 拉 binstall 产物）。只交付 formula + tap 设置说明。
```

### Task 5c：CI 规则（registry 安全闸门）
> **派给**：Test + Legal 协作。**安全核心**。

```
【任务】Task 5c：.github/workflows registry CI 规则
【plan】读 Wave 5「5c」
【spec】§6.2 A2、§6.10 A4、§6.11 D8 DCO、D10 CI 重建

【专属契约】
- CI 强制：manifest schema 校验 + 模型版本匹配（model_id+version+sha256）+ legal.license 白名单（MIT/Apache-2.0/CC-BY-4.0）+ CC-BY attribution 非空 + 下载 URL 必须指向 nowdocs-registry 域 + CI 从文本重建表（D10，拒 contributor 向量）+ golden eval 门禁（3b）+ DCO Signed-off-by 检查。
- 把这些规则作为威胁模型文档发布，contributor 事先知晓。
```

### Task 5d：种子 crate
> **派给**：Legal + Rust 副手。

```
【任务】Task 5d：Next.js(MIT)/React(CC-BY-4.0)/Vue(CC-BY-4.0) 种子 crate
【spec】§5.5、§6.10

【专属契约】
- 入库前逐个核实文档站许可（已核实：Next.js MIT/React CC-BY-4.0/vuejs docs CC-BY-4.0 图片除外/Astro MIT 备选）。
- 每个 crate 附 LICENSE/NOTICES；CC-BY 必填 attribution。过 5c CI 规则才发布。
- 只再分发文本/Markdown，不抓图片（Vue 图片除外条款）。
```

### Task 5e：L1-L4 门禁
> **派给**：Rust 副手。

```
【任务】Task 5e：pre-commit/pre-push/CI/weekly 门禁 Rust 化
【plan】读 §10 实施前置
【spec】用户 CLAUDE.md L1-L4 体系

【专属契约】
- L1 pre-commit：cargo fmt --check + clippy -D warnings + cargo-deny check + cargo-audit。
- L2 pre-push：cargo test + cargo build --release。
- L3 CI：cargo binstall 产物校验 + 跨平台构建。
- L4 周级：cargo udeps（死代码）+ cargo audit。
- 交付 .pre-commit-config.yaml + .github/workflows + scripts/pre-push.sh。
```

---

## §7 进度看板（Main 维护）

> Main 在每个 agent 汇报后更新本表。agent 不写本表（避免并发冲突）。

| Task | Wave | 状态 | Commit SHA | Agent | 备注 |
|---|---|---|---|---|---|
| S0 | 0 | ⬜ 待派 | — | — | 命门，先验 |
| 1a | 1 | ⬜ 待派 | — | — | BLOCKER |
| 1b | 1 | ⬜ 待派 | — | — | |
| 1c | 1 | ⬜ 待派 | — | — | 软依赖 1d |
| 1d | 1 | ⬜ 待派 | — | — | |
| 1e | 1 | ⬜ 待派 | — | — | |
| 1f | 1 | ⬜ 待派 | — | — | |
| 1g | 1 | ⬜ 待派 | — | — | |
| 1h | 1 | ⬜ 待派 | — | — | |
| 2a | 2 | ⬜ 待派 | — | — | |
| 2b | 2 | ⬜ 待派 | — | — | |
| 2c | 2 | ⬜ 待派 | — | — | |
| 3a | 3 | ⬜ 待派 | — | — | |
| 3b | 3 | ⬜ 待派 | — | — | |
| 4b | 4 | ⬜ 待派 | — | — | |
| 4c | 4 | ⬜ 待派 | — | — | |
| 4d | 4 | ⬜ 待派 | — | — | 集成点 |
| 4e | 4 | ⬜ 待派 | — | — | |
| 4f | 4 | ⬜ 待派 | — | — | |
| 4g | 4 | ⬜ 待派 | — | — | |
| 5a | 5 | ⬜ 待派 | — | — | |
| 5b | 5 | ⬜ 待派 | — | — | |
| 5c | 5 | ⬜ 待派 | — | — | 安全闸门 |
| 5d | 5 | ⬜ 待派 | — | — | |
| 5e | 5 | ⬜ 待派 | — | — | |

状态图例：⬜ 待派 / 🔄 进行中 / ✅ 完成 / ⚠️ 阻塞（看 Open Questions）/ ❌ 失败（S0 命门触发 ort 回退）

---

## §8 并发安全说明（Main 必读）

- **plan 文件打勾并发**：多个 agent 同时 Edit plan 打勾会触发 "File has been modified since read"。
  - 策略 A（推荐）：串行派发同 wave 内有 plan 写冲突的 task，或用 git worktree 隔离每个并行 agent（各自工作树改，最后合并）。
  - 策略 B：agent 遇该错误重新 Read 再 Edit（重试，多数能成功）。
- **进度看板**：只 Main 写，agent 不写（§7 已说明），避免表冲突。
- **Cargo.toml**：除 1a（建）和 2b（lancedb 行）外，任何 agent 改 Cargo.toml 视为越界。
- **spec 修订**：agent 只改「实现核实类」事实（API 名/版本/许可核实），架构决策变更上报 Main。多个 agent 改 spec 不同章节是安全的（Edit 精确匹配），但同一章节并发需协调。
