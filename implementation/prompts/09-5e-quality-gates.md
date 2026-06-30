# prompts/09 — Task 5e：L1-L4 门禁体系（`.pre-commit-config.yaml` + `scripts/`）

> **自包含 agent prompt**。模块归属 = `.pre-commit-config.yaml` + `scripts/pre-push.sh` + 周级 workflow。与 05/07 同改 `.github/workflows/` 时**各用不同文件名**避免冲突。

## 角色与分支
你是 nowdocs Task 5e agent。**独立 git worktree 工作**。
- 基点 `feat/4-wave-assembly-stubs`：`git switch -c feat/5e-gates feat/4-wave-assembly-stubs`。

## 任务
把项目 L1-L4 门禁体系 Rust 化落地。

## 四层门禁
- **L1 pre-commit（毫秒级，本地）**：`cargo fmt --check` + `cargo clippy -- -D warnings` + `cargo-deny check`（许可/重复依赖/advisory）+ `cargo-audit`（或合进 deny）。另加 `gitleaks`（密钥防线）。
- **L2 pre-push（十秒级，本地）**：`cargo test` + `cargo build --release`。脚本 `scripts/pre-push.sh`。
- **L3 CI（云端）**：交叉构建产物校验（与 05 的 release.yml 解耦，本块只管 PR CI 的 build/test 部分，**别和 07 的 ci.yml 重名**——用 `.github/workflows/gates.yml`）。
- **L4 周级（异步）**：`cargo udeps`（死代码）+ `cargo audit`（漏洞）。`.github/workflows/weekly-audit.yml`，cron 触发。

## 要做
- `.pre-commit-config.yaml`：L1 钩子（用 local hooks 调 cargo，或 pre-commit-rust hooks）。
- `scripts/pre-push.sh`：L2，可执行，失败非零退出；文档说明如何 `git config core.hooksPath` 或软链接安装。
- `.github/workflows/gates.yml`：L3 build+test（注意与 07 ci.yml、05 release.yml 文件名不同）。
- `.github/workflows/weekly-audit.yml`：L4 cron。
- `deny.toml`：cargo-deny 配置（许可白名单 MIT/Apache-2.0/CC-BY-4.0 等 + advisory）。
- 紧急避险：文档注明 `--no-verify` 仅经用户批准可临时绕过 L1/L2。

## 验证（本机可做的）
- `cargo fmt --check`、`cargo clippy -- -D warnings` 当前代码应通过（若不通过，列 Open Question 给对应模块 agent 修，别自己改 src/）。
- `cargo deny check`（若装）、`pre-commit run --all-files`（若装）跑一遍看是否绿。
- YAML 用 `actionlint` 核对。

## 约束
- 交付门禁配置 + 脚本，**不改 src/ 业务代码**（发现 lint 失败 → Open Question）。
- workflow 文件名避开 `ci.yml`（07）、`release.yml`（05）。
- 不 push；非交互列 Open Question。命令输出重定向。

## 完成清单
1. 打勾：Edit plan 的 Task 5e contract。
2. 汇报：① task=5e ② commit sha ③ 本地 fmt/clippy/deny 跑通结果 ④ 各门禁文件清单 ⑤ Open Questions（尤其当前代码是否过 clippy -D warnings）。
commit message：`chore(gates): L1-L4 quality gates (pre-commit/push/CI/weekly) (5e)`。
