# prompts/06 — Task 5b：Homebrew tap（unsigned formula）

> **自包含 agent prompt**。模块归属 = homebrew tap 仓库 + formula。**软依赖 05**（formula 从 5a 的 Release 产物拉取）。

## 角色与分支
你是 nowdocs Task 5b agent。**独立 git worktree 工作**。
- 基点 `feat/4-wave-assembly-stubs`：`git switch -c feat/5b-homebrew feat/4-wave-assembly-stubs`。
- 前置认知：5a 已定义 GitHub Release 资产命名（tgz，target 后缀）。若 05 未落地，可先按假定命名写 formula 模板并标 TODO，汇报里说明依赖。

## 任务
交付 unsigned Homebrew formula + tap 设置说明（D9 不签名）。

## 要做
1. 在仓库内放 formula 草稿（如 `dist/homebrew/nowdocs.rb`）+ tap 仓库（`nowdocs-homebrew`）设置说明文档。
2. formula 内容：
   - `desc` / `homepage`（nowdocs.dev）/ `license "MIT OR Apache-2.0"`。
   - 多平台 `url` + `sha256`：从 GitHub Releases 拉 5a 产出的 darwin (arm64/x86_64) tgz；用 `on_macos` + `on_arm`/`on_intel` 分支选资产。
   - `def install; bin.install "nowdocs"; end`。
   - `test do; system "#{bin}/nowdocs", "--version"; end`。
   - 不做签名（无 `disable!`/notarization）。
3. tap 说明：`brew tap nowdocs-registry/nowdocs && brew install nowdocs` 的发布流程文档（formula 放哪个 repo、发版时怎么更新 sha256，建议用 CI 自动 bump）。

## 验证
- `brew style dist/homebrew/nowdocs.rb`（若本机有 brew）或人工核对 Ruby 语法。
- sha256 占位用 `TODO_FILL_AFTER_RELEASE`，并在说明里写「发版后由 CI/手动填」。

## 约束
- 只交付 formula + tap 说明文档（如 `dist/homebrew/`）。**不碰 src/、Cargo.toml、其它 workflow**。
- 不 push；非交互列 Open Question。

## 完成清单
1. 打勾：Edit plan 的 Task 5b contract。
2. 汇报：① task=5b ② commit sha ③ formula 语法核对结果 ④ 对 5a 资产命名的依赖假设 ⑤ Open Questions。
commit message：`build(dist): unsigned Homebrew formula + tap setup (5b)`。
