# Acceptable Use Policy

> Copyright (c) 2026 GWMM LLC.
> 最后更新：2026-07-04
> 适用范围：nowdocs 公共 registry 与 nowdocs 软件

## registry 性质

nowdocs 公共 registry 实行**策展制**（curated），非开放提交。GWMM LLC 决定
哪些 docset 上架。社区可提议，但上架须经策展人审核。

## 可入文档类型

- **开源许可文档**：MIT / Apache-2.0 / CC-BY-4.0
- **官方文档且其许可明确允许再分发**

## 禁入文档类型

- **专有文档**：如 Clerk、Tailwind 等非开源许可文档（设计评审 §6.10）
- **ToS 禁止爬取/再分发**的网站文档
- **未授权爬取内容**：无明确许可的抓取产物
- **ShareAlike 类许可（暂不收录）**：CC-BY-SA / GFDL 等（如 MDN）。
  ShareAlike 义务会传染到衍生 docset，处理成本高，暂缓收录，未来评估
- **恶意/误导/投毒内容**：向量投毒、误导性文本、植入性内容

## 贡献者行为

向 registry 贡献 docset 的贡献者须：

- 不提交**明知侵权**的 docset（版权或许可违规）
- 不**投毒**（恶意向量、误导文本、植入性内容）
- 如实填写 manifest `legal` 块（`license` / `copyright_holder` / `attribution`）

违规处理：三次违规者将被禁止再向 registry 提交。严重违规（如故意投毒）可
立即禁止。

## 防护机制

- **CI 重建文本**：share 包只含文本 + manifest，向量由 registry CI 用锁定模型
  重建（设计评审 D10），关闭向量投毒与模型漂移攻击面
- **许可证审核**：上架前策展人按 [DMCA.md](DMCA.md) checklist 审核

## nowdocs 软件使用边界

nowdocs 是工具，用户对其使用负责：

- **不得用于侵权**：例如用 `nowdocs ingest` 抓取专有文档后再分发至公共 registry
- **本地自用不限制**：用户本地导入任意文档自用，由用户自行承担合规责任，
  与 nowdocs 项目及 GWMM LLC 无关

## 联络

- 侵权举报：见 [DMCA.md](DMCA.md)
- 行为准则违规：见 [CODE_OF_CONDUCT.md](../CODE_OF_CONDUCT.md)
- 一般法务：`legal@gwmmai.com`
