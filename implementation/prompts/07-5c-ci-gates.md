# prompts/07 — Task 5c：registry 安全闸门 CI（`.github/workflows/ci.yml`）

> **自包含 agent prompt**。模块归属 = `.github/workflows/ci.yml` + 威胁模型文档。**安全核心**。软依赖 01（golden eval 门禁）。

## 角色与分支
你是 nowdocs Task 5c agent。**独立 git worktree 工作**。
- 基点 `feat/4-wave-assembly-stubs`：`git switch -c feat/5c-ci feat/4-wave-assembly-stubs`。
- 若 01(3b eval) 已合入集成分支，从含 eval 的 sha 拉以便接 golden 门禁；否则 golden gate 步骤先标 TODO 并说明。

## 任务
建 registry contributor PR 的 CI 安全闸门，并发布为 contributor 事先可见的威胁模型文档。

## CI 强制规则（缺一不可，全部在 PR 上跑）
1. **manifest schema 校验**：每个提交的 docset manifest 过 `manifest::validate`（schema 版本=1）。
2. **模型版本匹配**：`model_id` + `model_version` + `model_sha256` 必须等于锁定值（jinaai/jina-embeddings-v2-small-en / 512 / candle / f16）。
3. **法律白名单**：`legal.license ∈ {MIT, Apache-2.0, CC-BY-4.0}`；CC-BY-4.0 必须 `attribution` 非空。
4. **下载 URL 域校验**：manifest/source 内所有 URL 必须指向 `github.com/nowdocs-registry/*` 或 `registry.nowdocs.rs/*`。
5. **CI 从文本重建（D10）**：拒绝 contributor 提交的向量文件；CI 从 `chunks.jsonl` 文本本地重新 embed+build 表，确保向量可信。
6. **golden eval 门禁**：跑 3b 的 `evaluate`，recall@5/MRR 必须达阈值（embedder/chunking 改动必过）。
7. **DCO 检查**：每个 commit 必须有 `Signed-off-by:`（D8，DCO 非 CLA）。

## 要做
- `.github/workflows/ci.yml`：上述规则编排为 job/step。能用现有 `cargo test` / `cargo run` 子命令验证的就调它们（如用一个 `nowdocs` 校验子命令或测试），别在 YAML 里重写 Rust 逻辑。
- 威胁模型文档（如 `docs/THREAT_MODEL.md` 或 spec 章节）：把这 7 条作为 contributor 守则发布，写清「为什么」（A2 域校验防 SSRF/恶意源、D10 防向量投毒、A4 法律合规）。

## 验证
- `actionlint .github/workflows/ci.yml`（若可用）或人工核对。
- 至少写一个能本地跑的校验脚本/测试，CI step 调它（便于离线验证逻辑而非只靠 GitHub 跑）。

## 约束
- 改 `.github/workflows/ci.yml` + 威胁模型文档（+ 可选一个校验脚本/子命令）。**勿碰 release.yml（05）、pre-commit 配置（09）**。
- 不 push；非交互列 Open Question。

## 完成清单
1. 打勾：Edit plan 的 Task 5c contract。
2. spec：§6.2/§6.10/§6.11 引用威胁模型文档。
3. 汇报：① task=5c ② commit sha ③ actionlint/本地校验结果 ④ 7 条规则落点清单 ⑤ Open Questions（尤其 D10 重建在 CI 的算力/时长）。
commit message：`ci(registry): security gates + threat model (5c)`。
