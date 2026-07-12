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

### 本地门禁依赖

首次参与代码贡献时，先安装本地门禁依赖并注册 hook：

```bash
cargo install cargo-deny --locked
cargo install cargo-audit --locked
pre-commit install
```

`cargo-audit` 使用的两条 RUSTSEC 例外与 `deny.toml` 及 CI 保持一致，均有
明确的 `Revisit-date`（2026-10-10）。它们是限期、逐条记录的例外；新的 advisory
不会被自动忽略，仍必须使门禁失败并经过单独评估。

### 质量门禁（L1-L4）

| 级别 | 触发 | 内容 |
|---|---|---|
| L1 Commit | pre-commit（本地，秒级） | `cargo fmt --check` + `cargo clippy -- -D warnings` + `cargo deny check` + `cargo audit`（仅忽略 `RUSTSEC-2026-0194`、`RUSTSEC-2026-0195`）+ gitleaks 密钥扫描（配置见 `.pre-commit-config.yaml`，安装：`pre-commit install`） |
| L2 Push | `scripts/pre-push.sh`（本地，分钟级） | 调 `scripts/check.sh`：fmt + clippy + `cargo test -- --test-threads=1`（与 CI 同款）；另有特性分支代码改动 ≥15 行下限（纯 `.md` push 豁免） |
| L3 PR & CI | GitHub Actions `gates.yml`（云端） | DCO 校验 + fmt/clippy + test + release build + cargo-deny + 合规检查（manifest schema / no-vectors / 许可证审计单测） |
| L4 周期 | `weekly-audit.yml`（每周五） | `cargo audit` 漏洞扫描 + `cargo-udeps` 死代码 |

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
- **许可证禁入**：专有文档（Clerk/Tailwind 类）、ToS 禁爬、未授权爬取、
  ShareAlike 类（CC-BY-SA / GFDL，暂不收录）
- **manifest `legal` 块必填**：`license`（SPDX）/ `copyright_holder` / `attribution`
- 详见 [DMCA.md](docs/DMCA.md) 与 [AUP.md](docs/AUP.md)

## 不签名分发

nowdocs release 二进制**不做代码签名**，这是有意的发布策略。完整性验证通过
GitHub Releases 附带的 SHA-256 checksum 与 `cargo-binstall` 校验完成。分发走
`cargo-binstall` + Homebrew，贡献者不涉及签名流程。

## 行为准则

参与本社区即同意遵守 [Code of Conduct](CODE_OF_CONDUCT.md)。

## 联络

- 安全漏洞：见 [.github/SECURITY.md](.github/SECURITY.md)（**勿开公开 issue**）
- 一般讨论：GitHub Discussions / Issues
- 法务：`legal@gwmmai.com`
