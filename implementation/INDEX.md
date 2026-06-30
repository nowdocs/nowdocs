# nowdocs 实施编排中枢（INDEX）

> 这是 Main/编排者维护的唯一权威看板。每个剩余工作块拆成 `prompts/NN-*.md` 一份**自包含 agent prompt**。
> 分发时只需对 agent 说：**「读 `implementation/prompts/02-4bc-tools.md` 并严格执行」**——prompt 内已含全部上下文，agent 不必读本文件、不必读完整 plan。

---

## 0. 怎么用（分发协议）

1. **挑一个 prompt**：看下方「分发就绪表」，选状态为 ✅ 可派 且依赖已满足的块。
2. **隔离工作树（强制）**：任何要写代码的 agent 必须在**独立 git worktree** 里跑，严禁多个 agent 共用主工作树——否则它们的 `git switch` / `git restore` 会互相 stomp（已踩坑：被杀 agent 冲掉了主树的未提交编辑）。
   - 派发方式：用 Agent 工具 `isolation: "worktree"`，或手动 `git worktree add`。
3. **告诉 agent**：「读 `implementation/prompts/NN-xxx.md`，按里面的 step 严格 TDD 执行，做完按 prompt 末尾的『完成清单』汇报」。
4. **agent 汇报后**：编排者（你/Main）回填下方进度看板的 commit sha + 状态，并把该块分支合回集成分支 `feat/4-wave-assembly-stubs`。
5. **agent 不写本看板**（避免并发表冲突）；agent 只打勾它自己 plan 里的 step。

**集成分支**：`feat/4-wave-assembly-stubs` @ `ba372c4`（含全部 W0/W1/W2 + 3a + 已合入 **3b/4bc/4efg/4d/5a/5c/5e**）。所有剩余工作块从这里拉分支。组合态已验证：`cargo build --all-targets` ✅ + `cargo test --test-threads=1` ✅（全 suite 0 failed）。
> ⚠️ 本机无 rustfmt/clippy（源码装 Rust 无 rustup），fmt/clippy 只能在 CI 跑；pre-push hook 默认并行跑 registry 测试会 flake，须 `--test-threads=1`。

---

## 1. 已完成（不要重派）

| Task | 说明 | Commit |
|---|---|---|
| S0 | candle+jina-v2-small spike + E2 余弦门禁（命门） | `9e4d8d2` |
| 1a | Cargo 骨架 + 全 module stub + 锁签名 | `73e39e3` |
| 1b | manifest 解析 + 校验 | `3e93efd` |
| 1c | code-aware markdown chunker | `9720825` |
| 1d | tiktoken count_tokens | `84edc5d` |
| 1e | cache 目录 + LAYOUT_VERSION | `aaa1d1f` |
| 1f | prompt-injection sanitizer | `c4469a5` |
| 1g | 工具输入校验 | `b62d5b5` |
| 1h | MCP stdio 骨架 2025-11-25 | `8969209` |
| 2a | embedder 加固 load_for+sha | `14d21ae` |
| 2b | lancedb hybrid store | `4b098a3` |
| 2c | ingest + manifest 落盘 | `622bc22` |
| 3a | 检索管线 hybrid+邻窗组装 | `a6e6b0d` |
| (stub) | Wave-4 tools/registry 模块 stub | `f1dd637` |
| 3b | golden eval + recall@5/MRR 门禁（合入 `c35ecaa`） | `f65dc0e` |
| 4b+4c | nowdocs_search + nowdocs_list 工具（合入 `1a1ba6b`） | `72bc53a` |
| 4e+4f+4g | registry install/share/update/uninstall（合入 `b159784`） | `35a4e65` |
| 4d | CLI subcommand 真实接线（合入 `7afe9d1`） | `41aa874` |
| 5a | cargo-binstall 矩阵 + release workflow（合入 `a7314aa`） | `e71fede` |
| 5c | CI 安全闸门 + 威胁模型（合入 `8474f8c`） | `2f62578` |
| 5e | L1-L4 质量门禁（合入 `785d031`） | `88292f7` |

---

## 2. 分发就绪表（剩余 3 块）

| # | 块 | Task | 依赖（均已满足？） | 可并行？ | 状态 |
|---|---|---|---|---|---|
| 01 | golden eval | 3b | 3a ✅ | — | ✅ 已合入 |
| 02 | MCP tools | 4b+4c | 3a ✅ 1f ✅ 1g ✅ 1h ✅ 1e ✅ | — | ✅ 已合入 |
| 03 | registry 生命周期 | 4e+4f+4g | 1b ✅ 1e ✅ 1c ✅ 2b ✅ | — | ✅ 已合入 |
| 04 | CLI 集成 | 4d | 02 ✅ 03 ✅ | — | ✅ 已合入 |
| 05 | binstall 矩阵 | 5a | 1a ✅ | — | ✅ 已合入 |
| 06 | Homebrew tap | 5b | 5a ✅（产物 URL） | 独立 | ✅ 已合入 |
| 07 | CI 安全闸门 | 5c | 1b ✅ 3b ✅ | — | ✅ 已合入 |
| 08 | 种子 crate | 5d | 2c ✅ 5c ✅（CI 校验就位） | 独立 | ✅ 已合入 |
| 09 | L1-L4 门禁 | 5e | 1a ✅ | — | ✅ 已合入 |

🎉 **波次 4 全部合入** —— 9 块（01-09）均完成，集成分支 `feat/4-wave-assembly-stubs` @ `b62e1af`（未 push）。下一步：codex review → 用户决策 push / 发版。

---

## 3. 依赖 DAG（剩余部分）

```
3a ✅ ──┬─ 3b (01)
        └─ 4b/4c (02) ─┐
1b/1e/1c/2b ✅ ─ 4e/4f/4g (03) ─┤
                                ├─ 4d (04, 集成)
5a (05) ─ 5b (06)               │
3b (01) ─ 5c (07) ─ 5d (08)
5e (09) 独立
```

---

## 4. 进度看板（编排者回填）

| 块 | Task | 状态 | 分支 | Commit | 备注 |
|---|---|---|---|---|---|
| 01 | 3b | ✅ 完成 | `feat/3b-eval` | `f65dc0e`→`c35ecaa` | recall@5=1.0 / MRR=0.65（门禁 0.8/0.6）；bug#1 retrieve 排序待修 |
| 02 | 4b+4c | ✅ 完成 | `feat/4bc-tools` | `72bc53a`→`1a1ba6b` | mcp 4/4 + tools 5/5 |
| 03 | 4e+4f+4g | ✅ 完成 | `feat/4efg-registry` | `35a4e65`→`b159784` | registry 7/7（须串行）|
| 04 | 4d | ✅ 完成 | `feat/4d-cli` | `→7afe9d1` | CLI 子命令接线 |
| 05 | 5a | ✅ 完成 | `feat/5a-binstall` | `→a7314aa` | 5 目标构建矩阵 + release.yml |
| 06 | 5b | ✅ 完成 | `feat/5b-homebrew` | `→35c4609` | unsigned formula（5 OQ 发布期决策，见下）|
| 07 | 5c | ✅ 完成 | `feat/5c-ci` | `→8474f8c` | manifest/no-vectors/DCO 闸门 + 威胁模型 |
| 08 | 5d | ✅ 完成 | `feat/08-seed-crates` | `→b62e1af` | nextjs/react/vue share 产物 3×4；chunk_count MATCH ×3；src/ 已 untrack |
| 09 | 5e | ✅ 完成 | `feat/5e-gates` | `→785d031` | pre-commit/push/CI/weekly 闸门 |

图例：⬜ 待派 / 🔄 进行中 / ✅ 完成 / ⏸ 阻塞 / ⚠ 软依赖建议等

---

## 5. 全块通用铁律（每个 prompt 已内嵌，此处汇总备查）

- **TDD**：先写失败测试 → 验证失败 → 最小实现 → 验证通过 → commit。
- **隔离工作树**：写代码 agent 必须独立 worktree，禁止共用主树。
- **不擅自 push**：所有 `git push` 需用户显式批准。
- **网络防线**：本地服务绑 `127.0.0.1`；唯二允许的外部域 = `github.com/nowdocs-registry/*` 与 `registry.nowdocs.rs/*`；禁硬编码其它公网 IP。
- **密钥防线**：凭证只入 `.env`/`*.local`，写前确认已 gitignore。
- **命令输出管控**：build/test 一律 `> xxx.log 2>&1` 后看 tail，禁 dump 全量日志进上下文。
- **Cargo.toml 红线**：除非 prompt 明确授权，任何 agent 改 Cargo.toml 视为越界——遇缺依赖列 Open Question 上报。
- **子代理非交互**：遇未明决策按默认推进或列 Open Question，禁调交互提问工具。
- **完成三件事**：① 打勾自己 plan 的 step ② 仅「实现核实类」改 spec 附录 ③ 按 prompt 末尾格式汇报（task / commit sha / 测试结果 / diff 摘要 / Open Questions）。

参考文件（agent 按需读）：`AGENTS.md`（项目铁律）、`docs/superpowers/plans/2026-06-28-nowdocs-impl.md`（完整 plan）、`docs/superpowers/specs/2026-06-28-nowdocs-design-review.md`（spec）。
