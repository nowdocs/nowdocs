# nowdocs 法务合规文件体系 spec

> **状态**：规划稿，供实施。基于现有合规资产盘点 + nowdocs 法务风险画像 + 用户拍板的三项关键决策。
> **日期**：2026-07-02
> **管辖区**：Wyoming, USA（GWMM LLC 注册地）
> **商标状态**：未注册（common-law 使用，™ 标记）
> **DMCA 路线**：GitHub 内置 takedown 流程（registry 托管于 GitHub）
> **核实方法**：勘察现有合规文件（`.github/SECURITY.md` / `NOTICE` / `deny.toml` / README 段落）+ 设计评审 spec §6.10 / §6.11 / D10 + 用户三项拍板。

---

## 0. 执行摘要

nowdocs 作为聚合第三方开发文档的本地 MCP server，法务风险集中在**第三方文档版权**（registry 上架审核 + DMCA 响应）。现有合规资产（LICENSE 双许可 / NOTICE / deny.toml / SECURITY.md）已覆盖代码侧；缺口在**内容侧与治理侧**——聚合文档的版权政策、本地隐私声称、商标、贡献流程均散落在 README 段落或未建档。

本 spec 规划 7 个待建/增强文件，分 P0/P1/P2 三优先级。**P0 四项为发布前法务阻断项**。

**三项已拍板决策**：
1. **管辖区 Wyoming**（US 联邦法 DMCA / Copyright Act / Lanham Act + Wyoming LLC 州法）
2. **商标未注册** → common-law ™（Lanham Act §43(a) 保护），不用 ®
3. **DMCA 走 GitHub 内置流程**（registry repo 托管于 GitHub，takedown 提交 GitHub 而非 nowdocs）

---

## 1. 现状盘点（已有资产，不重复造）

| 文件 | 路径 | 状态 | 处置 |
|---|---|---|---|
| 项目许可 | `LICENSE` + `LICENSE-MIT` + `LICENSE-APACHE` | ✅ MIT OR Apache-2.0 | 不动 |
| 第三方 notices | `NOTICE` | ✅ Apache-2.0 §4(a) 合规，616 crate 逐许可证分类 | 不动，Export Compliance 增补于此（§4.7） |
| 许可证审计配置 | `deny.toml` | ✅ 白名单 + advisory + bans + sources | 不动 |
| 安全披露 | `.github/SECURITY.md` | ✅ 完整（私密渠道 + 3天/30天窗口 + scope + supported versions） | P2 评估增强，现状已够（§4.8） |
| README 商标段 | `README.md:62-66` | ⚠️ 段落未建档 | 提升为 `docs/TRADEMARK.md`，README 留摘要 + 链接 |
| README 隐私段 | `README.md:68-81` | ⚠️ 段落未建档 | 提升为 `docs/PRIVACY.md`，README 留摘要 + 链接 |
| README Takedown 段 | `README.md:91-100` | ⚠️ 段落未建档 | 提升为 `docs/DMCA.md`，README 留摘要 + 链接 |

设计评审相关决策（已规划，本 spec 落地）：
- **§6.10**：Clerk/Tailwind 专有/禁爬文档禁入 registry → 写入 AUP + DMCA 策展 checklist
- **§6.11**：不签名分发避 F-1/OPT 签证风险 → 写入 CONTRIBUTING release 流程
- **D10**：CI 重建文本防向量投毒 → 写入 AUP + CONTRIBUTING

---

## 2. nowdocs 法务风险画像

| 风险域 | 风险描述 | 防线 |
|---|---|---|
| **第三方文档版权**（头号） | registry 聚合第三方文档，若收入 Clerk/Tailwind 等专有文档则侵权 | 事前：策展审核 checklist（§4.1 / §4.6）；事后：DMCA takedown（§4.1） |
| **本地隐私** | 用户 query/embedding 若出网则泄密 | Privacy Policy 明确三类网络行为，query 永不出网（§4.2） |
| **商标** | 衍生品冒用 "nowdocs" 名义 | Trademark Policy 明确代码许可 ≠ 商标许可（§4.3） |
| **出口管制** | release 二进制含 openssl 加密 | EAR §734.7 公开可用豁免声明（§4.7） |
| **向量投毒** | share 包注入恶意向量 | CI 重建文本（D10）+ AUP 禁止（§4.6） |
| **贡献者协议** | 贡献代码归属争议 | DCO（已 CI 强校验）+ CONTRIBUTING 文档化（§4.4） |

---

## 3. 待建文件清单（总览）

| # | 文件 | 路径 | 优先级 | 依据 |
|---|---|---|---|---|
| 1 | DMCA Takedown Policy | `docs/DMCA.md` | **P0** | 聚合文档头号风险 |
| 2 | Privacy Policy | `docs/PRIVACY.md` | **P0** | 本地离线卖点合规声称 |
| 3 | Trademark Policy | `docs/TRADEMARK.md` | **P0** | 商标保护 |
| 4 | CONTRIBUTING.md | `CONTRIBUTING.md` | **P0** | DCO + 策展审核文档化 |
| 5 | CODE_OF_CONDUCT.md | `CODE_OF_CONDUCT.md` | P1 | 社区成熟度 |
| 6 | Acceptable Use Policy | `docs/AUP.md` | P1 | registry 策展准入 |
| 7 | Export Compliance | `NOTICE` 增补 | P2 | openssl 加密声明 |
| 8 | SECURITY.md 增强 | `.github/SECURITY.md` | P2 | 评估加 PGP key |

---

## 4. 各文件详细规划

### 4.1 DMCA Takedown Policy（P0）

**路径**：`docs/DMCA.md`
**目的**：侵权下架响应流程 + registry 上架前许可证审核 checklist（事前 + 事后双防线）
**内容大纲**：

1. **政策适用范围**：nowdocs 公共 registry（当前 `github.com/nowdocs-registry/*`，未来 `registry.nowdocs.dev/*`）
2. **举报渠道**（走 GitHub 内置）：
   - **主渠道**：GitHub DMCA takedown（`https://github.com/contact/dmca`）—— registry repo 托管于 GitHub，GitHub 直接处理
   - **备用**：`legal@gwmmai.com`（标题 `[nowdocs DMCA]`），收到后转交 GitHub 流程
3. **512(c)(3) 要件**（举报须包含）：
   - 被侵权作品权属证明
   - 被指控 docset 名称及位置（repo URL）
   - 善意声明（善意确信未授权）
   - 伪证处罚声明（17 U.S.C. §512(c)(3)(A)(vi)）
   - 物理或电子签名
4. **响应流程**：
   - GitHub 下达 takedown notice → 立即下架涉事 docset（不等待 GWMM LLC 复核，遵循 GitHub 判定）
   - 反通知（counter-notification）：贡献者可经 GitHub 提交反通知（§512(g)(3) 要件）
   - 反通知后 10-14 个工作日内决定是否恢复（除非原举报方声明已提起诉讼）
5. **事前防线：registry 上架许可证审核 checklist**（策展制）：
   - ✅ 可入：MIT / Apache-2.0 / CC-BY-4.0 / CC0 / BSD / ISC
   - ❌ 禁入：专有文档、ToS 禁止再分发、未授权爬取、Clerk/Tailwind 类专有（设计评审 §6.10）
   - 审核者：GWMM LLC 策展人
   - 审核记录：docset manifest 的 `legal` 块（`license` / `copyright_holder` / `attribution`）必填
6. **重复侵权者**：策展制下，三次侵权贡献者禁止再提交
7. **法律联络**：`legal@gwmmai.com`（GWMM LLC，Wyoming）

**依据**：设计评审 §6.10；DMCA 17 U.S.C. §512(c)(3) / §512(g) safe harbor

---

### 4.2 Privacy Policy（P0）

**路径**：`docs/PRIVACY.md`
**目的**：明确本地离线工具的隐私边界，区分三类网络行为
**内容大纲**：

1. **核心承诺**：nowdocs 本地运行，**query、embedding、文档内容永不出网**
2. **三类网络行为**（用户主动触发）：
   - `nowdocs ingest`：**不联网**（读本地目录，用户自行 clone 源 repo，连接源站属用户行为）
   - `nowdocs install` / `update`：从 registry 下载 docset（白名单 `github.com/nowdocs-registry/*`、`registry.nowdocs.dev/*`）
   - 首��� `nowdocs serve`：从 HuggingFace 下载 embedder 模型（`hf-hub`，jina-v2-small，66MB），之后本地缓存
3. **不收集**：无遥测、无分析、无 tracking、无账号、无云服务
4. **本地存储**：
   - docset 数据：`~/.local/share/nowdocs/`（或平台等价路径）
   - embedder 缓存：`~/.cache/huggingface/`
5. **GDPR / CCPA 声明**：nowdocs 不收集个人数据，上述法规不适用；声明以示透明
6. **registry 网站隐私**（未来）：若 `registry.nowdocs.dev` 提供网页，将另行声明（当前 registry 走 GitHub，适用 GitHub 隐私政策）

**依据**：README:68-81 现有段落；`src/registry.rs` URL 白名单（`is_allowed_registry_url`）；`src/embedder.rs` hf-hub 下载

---

### 4.3 Trademark Policy（P0）

**路径**：`docs/TRADEMARK.md`
**目的**：明确 "nowdocs" 名称与 logo 的商标保护边界
**内容大纲**：

1. **商标权属**："nowdocs" 名称及 nowdocs logo 为 GWMM LLC（Wyoming）的商标
2. **商标状态**：**未在美国专利商标局（USPTO）注册**，依 common-law 商标权保护（Lanham Act §43(a)），使用 **™** 标记，不使用 ®
3. **代码许可 ≠ 商标许可**：MIT/Apache-2.0 授予代码使用权，**不授予商标权**
4. **禁止**：
   - 用 "nowdocs" 命名、推广或标识衍生产品
   - 用 nowdocs logo 作为项目/产品标识
   - 暗示与 GWMM LLC 有官方关联或背书
5. **合理使用**（允许）：
   - 描述性引用："基于 nowdocs"、"兼容 nowdocs"
   - 指代本上游项目时的事实性使用
6. **未来注册**：GWMM LLC 保留未来向 USPTO 申请注册的权利；注册后将更新本政策与标记
7. **侵权报告**：`legal@gwmmai.com`（标题 `[nowdocs trademark]`）

**依据**：README:62-66 现有段；Lanham Act §43(a) common-law 保护；用户拍板"未注册"

---

### 4.4 CONTRIBUTING.md（P0）

**路径**：`CONTRIBUTING.md`
**目的**：DCO 流程 + 策展审核 + 代码/文档贡献路径文档化
**内容大纲**：

1. **贡献者协议**：DCO（Developer Certificate of Origin），非 CLA
   - 每个 commit 须 `Signed-off-by:`（`git commit -s`）
   - CI 由 `scripts/ci-check-dco.sh` 强制校验
   - DCO 全文链接（developercertificate.org）
2. **代码贡献流程**：
   - fork → 特性分支（**严禁直接 main**）→ `commit -s` → PR
   - L1-L4 门禁：pre-commit（ruff/biome/gitleaks/semgrep）→ pre-push（tsc/pyright + 单测）→ CI（playwright + build）→ weekly audit
   - 分支命名约定：`feat/` `fix/` `docs/` `chore/`
3. **文档/docset 贡献流程**：
   - `nowdocs share <docset>` 打文本包（text + manifest，**不含向量**）
   - CI 重建向量（D10，防向量投毒 + 模型漂移）
   - 提交至 `nowdocs-registry/<docset>` repo
4. **registry 策展审核 checklist**（引用 §4.1 DMCA 的许可证 checklist）：
   - 许可证可入清单 / 禁入清单
   - manifest `legal` 块必填
   - 策展人：GWMM LLC
5. **不签名分发**（设计评审 §6.11）：release 二进制不代码签名，避 F-1/OPT 签证风险；贡献者不涉及签名
6. **commit 规范**：DCO signed-off，commit message 简明（项目约定：仅 `Signed-off-by:`，无 Co-Authored-By）
7. **行为准则**：引用 `CODE_OF_CONDUCT.md`

**依据**：现有 `scripts/ci-check-dco.sh`；设计评审 §6.11 / D10；`deny.toml` 白名单

---

### 4.5 CODE_OF_CONDUCT.md（P1）

**路径**：`CODE_OF_CONDUCT.md`
**目的**：社区行为准则
**内容大纲**：
- 采用 **Contributor Covenant v2.1** 标准版（行业惯例，开源社区广泛认可）
- 英文版（国际社区惯例）
- 报告渠道：`legal@gwmmai.com`（标题 `[nowdocs CoC]`）
- 执行：GWMM LLC 维护者团队

**依据**：Contributor Covenant 2.1（https://www.contributor-covenant.org/version/2/1/code_of_conduct/）

---

### 4.6 Acceptable Use Policy（P1）

**路径**：`docs/AUP.md`
**目的**：registry 策展准入规则 + 可接受使用边界
**内容大纲**：

1. **registry 性质**：策展制（curated），非开放提交；GWMM LLC 决定上架
2. **可入文档类型**：
   - 开源许可文档（MIT / Apache / CC-BY / CC0 / BSD / ISC）
   - 官方文档且许可再分发
3. **禁入文档类型**：
   - 专有文档（Clerk/Tailwind 类，设计评审 §6.10）
   - ToS 禁止爬取/再分发的网站文档
   - 未授权爬取内容
   - 恶意/误导/投毒内容（向量投毒，D10 防护）
4. **贡献者行为**：
   - 不得提交明知侵权的 docset
   - 不得投毒（恶意向量/误导文本）
   - 三次违规禁提交
5. **nowdocs 软件使用边界**：
   - 不得用于侵权（用 ingest 抓取专有文档再分发至 registry）
   - 本地自用不限制（用户自行承担合规责任）

**依据**：设计评审 §6.10；D10（CI 重建文本防投毒）

---

### 4.7 Export Compliance（P2）

**路径**：`NOTICE` 增补（Export Compliance 段）或 `docs/EXPORT.md` 独立
**目的**：openssl 加密出口管制声明
**内容大纲**：

1. **加密组件**：nowdocs 静态链接 openssl（`openssl = { features = ["vendored"] }`，Cargo.toml:46），含加密功能
2. **EAR 豁免声明**：
   - nowdocs 源码公开可用，依 EAR §734.7 "已公开可用"豁免，不受 EAR 管制
   - release 二进制为公开可用开源软件编译产物，依 §740.13(e) 或 §734.7 豁免
3. **ECCN**：含加密功能，潜在 ECCN 5D002，但公开可用豁免适用
4. **用户责任**：终端用户自行遵守其所在法域的加密进出口管制

**依据**：`Cargo.toml:46` openssl vendored；EAR §734.7 / §740.13(e)；BIS 公开可用开源豁免惯例

---

### 4.8 SECURITY.md 增强（P2 评估）

**路径**：`.github/SECURITY.md`
**现状**：已完整（私密渠道 + 3天/30天窗口 + scope + supported versions）
**评估增强项**：
- PGP 公钥（加密敏感漏洞报告）—— 可选，当前 GitHub Security Advisories 已是私密渠道
- CVE 发布流程 —— 可选，当前 "credit reporters in public advisory" 已覆盖
**结论**：现状已够，增强非必须。发布前不动，后续按需。

---

## 5. README 段落提升策略

三个 README 段落（商标 / 隐私 / Takedown）提升为独立政策文件后，README 各保留：
- 一句话摘要
- 链接到独立文件

例：README 隐私段改为：
```
## 隐私
nowdocs 本地运行，query 永不出网，无遥测。完整隐私政策见 [PRIVACY.md](docs/PRIVACY.md)。
```

README 新增"法务与合规"索引段，链接所有政策文件（DMCA / PRIVACY / TRADEMARK / AUP / SECURITY / CONTRIBUTING / CoC）。

---

## 6. 实施顺序与依赖

```
Step 1（P0，发布前法务阻断）：
  4.1 DMCA.md
  4.2 PRIVACY.md
  4.3 TRADEMARK.md
  4.4 CONTRIBUTING.md
  → README 段落提升（§5）
  → 这 4 项互不依赖，可并行写

Step 2（P1，发布前应做）：
  4.5 CODE_OF_CONDUCT.md
  4.6 AUP.md
  → AUP 依赖 4.1（引用 DMCA checklist）

Step 3（P2，可延后至 v0.2）：
  4.7 Export Compliance（NOTICE 增补）
  4.8 SECURITY.md 增强（评估后决定）
```

**与发布路径的关系**：P0 四项是 v0.1.0 发布前法务阻断项（与 README 重写、limit bug 修复、release 产物并列）。P1 可同批或紧随。P2 延后 v0.2。

---

## 7. Open Questions

**已解决**：
- ✅ 管辖区：Wyoming, USA（GWMM LLC）
- ✅ 商标：未注册，common-law ™
- ✅ DMCA：GitHub 内置流程

**保留（实现阶段定）**：
1. `legal@gwmmai.com` 是否已设邮箱别名/收件人？（影响所有政策文件的联络点真实性）
2. registry 策展权限：GWMM LLC 独家，还是未来开放社区策展人？（影响 AUP / CONTRIBUTING 措辞）
3. ✅ 已解决：改用已持有的 `registry.nowdocs.dev` 域名（commit 5fb0c35；见 PRIVACY.md / AUP.md / DMCA.md "Self-Hosted Registry (Future)"）。
4. PGP key 是否要生成？（影响 SECURITY.md 增强，P2）
5. Wyoming LLC 是否已实际注册完成？（影响所有文件的"GWMM LLC, Wyoming"实体声明）

---

## 8. 核实清单（实现时）

- [ ] 现有 `.github/SECURITY.md` 内容不动（除非 P2 增强）
- [ ] `NOTICE` / `deny.toml` / `LICENSE*` 不动
- [ ] README 三段提升后保留摘要 + 链接
- [ ] 所有政策文件联络邮箱统一 `legal@gwmmai.com`
- [ ] 所有文件标注版权 `Copyright (c) 2026 GWMM LLC`
- [ ] 商标统一用 ™ 不用 ®
- [ ] DMCA 流程指向 GitHub 内置，不自建
- [ ] 实现完成后，本 spec 移入 `docs/superpowers/specs/`（已在此路径）并随实现 commit
