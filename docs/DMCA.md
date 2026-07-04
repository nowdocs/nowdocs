# DMCA Takedown Policy

> Copyright (c) 2026 GWMM LLC.
> 适用管辖区：Wyoming, USA（US 联邦法 DMCA 17 U.S.C. §512）

## 政策适用范围

本政策适用于 nowdocs 公共 registry 托管的 docset，当前位于：

- `github.com/nowdocs-registry/*`（GitHub 托管）
- 未来：`registry.nowdocs.rs/*`（自建镜像，届时另行通告）

nowdocs 是聚合第三方开发文档的本地 MCP server。我们尊重第三方版权，对
registry 实行**策展制**（curated）——上架前审核许可证，收到通知后响应下架。

## 举报渠道（GitHub 内置）

registry repo 托管于 GitHub，侵权举报走 **GitHub 内置 DMCA takedown 流程**：

- **主渠道**：[GitHub DMCA Takedown](https://github.com/contact/dmca)
  - GitHub 收到合规通知后直接处理其托管内容，nowdocs 维护者配合执行下架。
- **备用**：`legal@gwmmai.com`（标题 `[nowdocs DMCA]`）
  - 收到后我们转交 GitHub 流程；不能替代 GitHub 官方渠道。

## 有效的 DMCA 通知须包含（17 U.S.C. §512(c)(3)）

1. 被侵权作品的权属证明（如原作品 URL、版权登记号）。
2. 被指控侵权的 docset 名称及其在 registry 中的位置（repo URL）。
3. 善意声明："我善意确信该使用方式未获版权所有人或其代理人授权。"
4. 伪证处罚声明："我确信通知中信息准确，且我是版权所有人或授权代表，伪证愿受处罚。"
5. 物理或电子签名。

## 响应流程

1. **下架**：GitHub 下达 takedown 判定 → 涉事 docset 立即从 registry 下架
   （不等待 GWMM LLC 复核，遵循 GitHub 判定）。
2. **反通知**（counter-notification，§512(g)(3)）：贡献者可经 GitHub 提交反通知，
   须包含反通知法定要件。
3. **恢复**：反通知后 10-14 个工作日内决定是否恢复，除非原举报方声明已提起诉讼。

## 事前防线：registry 上架许可证审核 checklist

策展制下，每个 docset 上架前由 GWMM LLC 策展人审核：

**✅ 可入许可证**：
MIT / Apache-2.0 / CC-BY-4.0 / CC0 / BSD-2-Clause / BSD-3-Clause / ISC

**❌ 禁入**：
- 专有文档（如 Clerk、Tailwind 等非开源许可文档）
- 服务条款（ToS）禁止再分发或禁止爬取的网站文档
- 未授权爬取内容
- 许可证不明或无法核实

**审核记录**：docset 的 `manifest.json` 的 `legal` 块必填：

- `license`：SPDX 标识符
- `copyright_holder`：版权方
- `attribution`：署名文本（CC-BY-4.0 必填）

## 重复侵权

策展制下，三次提交侵权 docset 的贡献者将被禁止再向 registry 提交。

## 法律联络

GWMM LLC（Wyoming, USA）
邮箱：`legal@gwmmai.com`
