# Privacy Policy

> Copyright (c) 2026 GWMM LLC.
> 最后更新：2026-07-02

## 核心承诺

nowdocs 本地运行。你的 **query、embedding、文档内容永不出网**。

## 三类网络行为

nowdocs 仅在以下用户主动触发的场景联网：

| 场景 | 是否联网 | 连接目标 |
|---|---|---|
| `nowdocs ingest`（导入本地文档） | ❌ 不联网 | 仅读本地目录 |
| `nowdocs install` / `update`（安装/更新 docset） | ✅ | registry 白名单：`github.com/nowdocs-registry/*`、`registry.nowdocs.rs/*` |
| 首次 `nowdocs serve`（加载 embedder） | ✅ 一次性 | HuggingFace（`hf-hub`，下载 jina-v2-small 模型，66MB），之后本地缓存 |

`nowdocs ingest` 连接 GitHub 等源站属**用户自行 clone 源 repo 的行为**，与 nowdocs 无关。

## 不收集

nowdocs **不收集任何数据**：

- 无遥测（telemetry）
- 无分析（analytics）
- 无追踪（tracking）
- 无用户账号
- 无云服务

代码中无 telemetry/analytics/tracking 调用。

## 本地存储

| 数据 | 位置 |
|---|---|
| docset 数据 | `~/.local/share/nowdocs/`（或平台等价路径） |
| embedder 模型缓存 | `~/.cache/huggingface/` |
| 配置 | `~/.config/nowdocs/` |

## GDPR / CCPA

nowdocs 不收集个人数据，GDPR（欧盟）与 CCPA（加州）不适用。本声明以示透明。

## registry 网站隐私

当前 registry 走 GitHub，适用 [GitHub 隐私政策](https://docs.github.com/en/site-policy/privacy-policies)。
若未来 `registry.nowdocs.rs` 提供网页，将另行发布隐私声明。

## 变更

本政策变更会在仓库 commit 历史中记录。重大变更通过 release notes 通告。

## 联络

邮箱：`legal@gwmmai.com`
