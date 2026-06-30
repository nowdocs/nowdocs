# prompts/04 — Task 4d：CLI 集成（`src/main.rs`）

> **自包含 agent prompt**。模块归属 = `src/main.rs`。**集成点，依赖 02(tools) + 03(registry) 已落地**。

## 角色与分支
你是 nowdocs Task 4d agent。**独立 git worktree 工作**。
- 基点：必须从**已合入 02 + 03 的集成分支**拉（编排者会告诉你确切 sha）。`git switch -c feat/4d-cli <集成sha>`。
- 确认 `src/tools.rs` 的 `handle_call` 与 `src/registry.rs` 的 install/share/update/uninstall 都已是真实现（非 todo!()）；若仍是 stub → 报 BLOCKED，别接线到 stub。

## 任务
把 `src/main.rs` 里 1a 留下的 `println!` 占位换成真实模块调用。

## 当前 `src/main.rs`（占位）
```rust
Commands::Serve => nowdocs::mcp::run_loop().expect("mcp loop error"),
Commands::Install { docset } => println!("install {}", docset),
Commands::Ingest { dir, name } => println!("ingest {} -> {}", dir, name),
Commands::Share { docset } => println!("share {}", docset),
Commands::Uninstall { docset } => println!("uninstall {}", docset),
Commands::ListInstalled => println!("list-installed"),
Commands::Update { docset } => println!("update {}", docset),
```

## 要接线
- `Serve` → 保持 `mcp::run_loop()`（已对）。
- `Install { docset }` → `registry::install(&docset, &<解析出的 registry URL>)`。URL 解析规则：默认 `https://github.com/nowdocs-registry/releases/.../<docset>.tar`（用域内 pattern；可加 `--url` 可选覆盖仅供测试）。
- `Ingest { dir, name }` → `ingest::ingest_dir(Path::new(&dir), &name)`，打印 stats。
- `Share { docset }` → `registry::share(&docset, &<默认 out_dir，如当前目录 ./<docset>-share>)`，打印产物路径。
- `Uninstall { docset }` → `registry::uninstall(&docset)`。
- `ListInstalled` → 复用 `tools::handle_call("nowdocs_list", json!({}))` 或直接枚举 `cache::db_path` 父目录，打印。
- `Update { docset }` → `registry::update(&docset)`。
- 各分支错误统一 `eprintln!` + 非零退出码（`std::process::exit(1)`），别 `expect` 直接 panic（除 Serve 保持现状或也改优雅退出）。

## 测试 `tests/cli_tests.rs`（在现有文件追加，别覆盖 1a 的测试）
- 各子命令端到端冒烟：用 `assert_cmd` 风格或直接 `Command::cargo_bin`，`XDG_CACHE_HOME` 指 tempdir。
- `test_cli_ingest_then_list`：ingest 一个临时 md 目录 → list-installed 输出含该 docset 名。
- `test_cli_uninstall`：装后 uninstall → list 不再含。
- 注意：涉及真 embedder 的 ingest 走 `#[ignore]`。

## 约束
- 只改 `src/main.rs` + `tests/cli_tests.rs`。**不改 Cargo.toml**（若需 `assert_cmd` dev-dep → 列 Open Question）。
- 命令输出 `> 4d-test.log 2>&1` 看 tail。TDD；不 push；非交互列 Open Question。

## 完成清单
1. 打勾：Edit plan 的 Task 4d contract。
2. 汇报：① task=4d ② commit sha ③ 测试结果 ④ main.rs diff 摘要 ⑤ Open Questions。
commit message：`feat(cli): wire subcommands to real modules (4d)`。
