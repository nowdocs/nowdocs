# Contributing to nowdocs

感谢你有兴趣为 nowdocs 贡献！本文档说明代码与文档/docset 的贡献流程。

## 贡献者协议：DCO（非 CLA）

nowdocs 使用 **Developer Certificate of Origin (DCO)**，不使用 CLA。

- 每个 commit 须带 `Signed-off-by:` 行（`git commit -s` 自动添加）
- 这表示你 certify 该贡献你有权提交
- CI 由 `scripts/ci-check-dco.sh` 强制校验，缺签 commit 会被拒
- DCO 全文：https://developercertificate.org/

## 代码贡献流程

1. **fork** 仓库
2. **创建特性分支**（严禁直接在 `main` 上工作）：
   - 分支命名：`feat/`、`fix/`、`docs/`、`chore/` 前缀 + 简短描述
3. **提交**：`git commit -s`（确保 DCO signed-off）
4. **推送**到你的 fork
5. **开 PR** 到 `main`

commit message 约定：项目惯例仅含 `Signed-off-by:`，不加 `Co-Authored-By`。

### 质量门禁（L1-L4）

| 级别 | 触发 | 内容 |
|---|---|---|
| L1 Commit | pre-commit（本地毫秒级） | ruff/biome 规范 + gitleaks 密钥 + semgrep 安全 |
| L2 Push | pre-push（本地十秒级） | tsc/pyright 类型 + 核心单测 |
| L3 PR & CI | GitHub Actions（云端分钟级） | playwright E2E + 生产构建 |
| L4 周期 | OpenClaw（周级异步） | knip 死代码 + pip-audit/pnpm audit 漏洞 |

L1/L2 本地校验通过后再推。紧急场景需 `--no-verify` 绕过须维护者明确同意。

## 文档/docset 贡献流程

向 registry 贡献新 docset：

1. 用 `nowdocs ingest <dir> <name>` 本地导入
2. 用 `nowdocs share <docset>` 打文本包（**text + manifest，不含向量**）
3. 提交至 `nowdocs-registry/<docset>` repo
4. GWMM LLC 策展人审核（见下 checklist）
5. CI 重建向量（防向量投毒 + 模型漂移，设计评审 D10）

**为何不含向量**：share 只发文本，向量由 registry CI 用锁定模型重建。这关闭了
向量投毒与模型漂移两个攻击面。

### registry 策展审核 checklist

每个 docset 上架前由 GWMM LLC 策展人审核：

- **许可证可入**：MIT / Apache-2.0 / CC-BY-4.0 / CC0 / BSD / ISC
- **许可证禁入**：专有文档（Clerk/Tailwind 类）、ToS 禁爬、未授权爬取
- **manifest `legal` 块必填**：`license`（SPDX）/ `copyright_holder` / `attribution`
- 详见 [DMCA.md](docs/DMCA.md) 与 [AUP.md](docs/AUP.md)

## 不签名分发

nowdocs release 二进制**不代码签名**（设计评审 §6.11）。这是有意决策——避开
F-1/OPT 签证下的商业签名义务。分发走 `cargo-binstall` + Homebrew，贡献者不
涉及签名流程。

## 行为准则

参与本社区即同意遵守 [Code of Conduct](CODE_OF_CONDUCT.md)。

## 联络

- 安全漏洞：见 [.github/SECURITY.md](.github/SECURITY.md)（**勿开公开 issue**）
- 一般讨论：GitHub Discussions / Issues
- 法务：`legal@gwmmai.com`
