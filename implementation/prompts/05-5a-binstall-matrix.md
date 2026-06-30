# prompts/05 — Task 5a：cargo-binstall 构建矩阵（`Cargo.toml` meta + release workflow）

> **自包含 agent prompt**。模块归属 = `Cargo.toml [package.metadata.binstall]` + `.github/workflows/release.yml`。

## 角色与分支
你是 nowdocs Task 5a agent。**独立 git worktree 工作**。
- 基点 `feat/4-wave-assembly-stubs`：`git switch -c feat/5a-binstall feat/4-wave-assembly-stubs`。

## 任务
5 目标交叉构建矩阵 + cargo-binstall 元数据，产出单一自包含二进制（不签名，D9）。

## 5 个目标
- `aarch64-apple-darwin`、`x86_64-apple-darwin`
- `x86_64-unknown-linux-musl`、`aarch64-unknown-linux-musl`
- `x86_64-pc-windows-msvc`

## 要做
1. `Cargo.toml` 加 `[package.metadata.binstall]`：
   - `pkg-url` 模板指向 GitHub Releases 资产名（如 `{ name }-{ version }-{ target }{ archive-suffix }`）。
   - `pkg-fmt`（unix=`tgz`，windows=`zip`）。
   - **本任务是唯一授权改 Cargo.toml 的块**，但仅限新增 `[package.metadata.binstall]` 段，勿动 `[dependencies]`。
   - 确认 `candle-core default-features = false` 已在（slim build），若被改回需修正并说明。
2. `.github/workflows/release.yml`：tag 触发，matrix 5 目标交叉构建，`cargo build --release --target <t>`，打包二进制为 tgz/zip，上传到 GitHub Release。
   - macOS：核实是否动态链 Accelerate framework（每台 Mac 自带）——写进 spec §8.3 附录。
   - Linux musl：核实是否真静态（`ldd` 应报 not a dynamic executable）——写进附录。
   - 不做代码签名 / notarization（D9）。

## 验证（本地可做的部分）
- `cargo build --release`（本机目标）通过，二进制能 `nowdocs --version`。
- `cargo binstall --help` 能解析 metadata（若装了 cargo-binstall）。
- workflow YAML 用 `actionlint`（若可用）或人工核对 matrix 语法。

## 约束
- 改 `Cargo.toml`（仅 binstall meta 段）+ `.github/workflows/release.yml`。**勿碰 ci.yml**（那是 prompt 07）。
- 不 push；非交互列 Open Question。命令输出重定向。

## 完成清单
1. 打勾：Edit plan 的 Task 5a contract。
2. spec：§8.3 附录写 macOS Accelerate / Linux musl 静态核实结论 + binstall 资产命名约定。
3. 汇报：① task=5a ② commit sha ③ 本地 release build 结果 ④ Cargo.toml diff（仅 binstall 段）⑤ Open Questions。
commit message：`build(dist): cargo-binstall matrix + release workflow (5a)`。


---

## 完成状态（2026-06-30）

- [x] **Cargo.toml** 加 `[package.metadata.binstall]` 段（pkg-url / bin-dir / pkg-fmt=tgz）
- [x] **确认 `candle-core default-features=false`** 已在原 [dependencies] 中（未改动）
- [x] **`.github/workflows/release.yml`** 创建：tag push 触发，5 目标 matrix，musl 静态 `ldd` 校验
- [x] **本地 release build**：`cargo build --release` ✅（5m 41s）+ `./target/release/nowdocs --version` ✅
- [x] **spec §8.3 附录** 已写入 docs/superpowers/specs/2026-06-28-nowdocs-design-review.md
  - 8.3.1 macOS Accelerate 动态链接（系统自带，零部署成本）
  - 8.3.2 Linux musl 真·全静态（`ldd` 验证 not a dynamic executable）
  - 8.3.3 cargo-binstall 资产命名约��
  - 8.3.4 构建矩阵覆盖（5 目标）
  - 8.3.5 不签名分发（D9）
- [x] **commit message**：`build(dist): cargo-binstall matrix + release workflow (5a)`

**未触碰文件**：
- ✅ `ci.yml` 未修改（prompt 07 territory）
- ✅ [dependencies] 段未改动
