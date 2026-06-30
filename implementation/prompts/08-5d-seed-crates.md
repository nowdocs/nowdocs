# prompts/08 — Task 5d：种子 docset crate（Next.js / React / Vue）

> **自包含 agent prompt**。模块归属 = 种子 docset 数据 + 各自 LICENSE/NOTICES。**软依赖 07**（过 5c CI 规则才发布）+ 02/03（ingest/share 工具链）。

## 角色与分支
你是 nowdocs Task 5d agent。**独立 git worktree 工作**。
- 基点：建议从含 ingest(2c ✅) 的集成分支拉；若要走 share 流程则需 03 已落地。`git switch -c feat/5d-seed <集成sha>`。

## 任务
做 3 个种子 docset crate，逐个核实文档站许可，附 LICENSE/NOTICES，过 5c CI 才发布。**只再分发文本/Markdown，绝不抓图片。**

## 许可（已核实，按此为准）
- **Next.js** — MIT。
- **React** — 文档 CC-BY-4.0 → `attribution` 必填。
- **Vue** — vuejs docs CC-BY-4.0（**图片除外条款**，故绝不抓图）→ `attribution` 必填。
- Astro — MIT（备选，本任务不强制）。

## 要做（每个 crate）
1. 抓取/准备文档 Markdown 文本（**纯文本，去图片**）。带 JS 渲染的站点强制用 playwright/gstack-browser（项目铁律 4）。
2. `ingest_dir` 生成本地 docset；填 manifest 的 `legal` 段：`license` + `copyright_holder` + `attribution`（CC-BY 必填非空）。
3. `registry::share` 打包成 share 产物（manifest + chunks.jsonl，无向量，D10）。
4. 附 `LICENSE` + `NOTICES`（CC-BY 的署名信息）。
5. 产物必须能过 prompt 07 的 CI 7 条规则（schema/模型版本/许可白名单/URL 域/无向量/golden/DCO）。

## 验证
- 每个 docset 的 manifest 过 `manifest::validate`。
- share 产物无任何 `.lance`/向量/图片文件。
- 抽样 `nowdocs_search` 召回合理（可借 3b eval）。

## 约束
- 交付 3 个 docset 的 share 产物 + LICENSE/NOTICES + manifest。**不碰 src/ 核心代码**（如发现 ingest/share bug，列 Open Question 给对应模块 agent，别自己改）。
- 法律零容忍：许可不明 → 不入库，列 Open Question。
- 不 push；非交互列 Open Question。命令输出重定向。

## 完成清单
1. 打勾：Edit plan 的 Task 5d contract。
2. 汇报：① task=5d ② commit sha ③ 3 个 docset 的许可核实结论 + share 产物路径 ④ 无图片/无向量核验 ⑤ Open Questions。
commit message：`feat(seed): Next.js/React/Vue seed docsets with license compliance (5d)`。
