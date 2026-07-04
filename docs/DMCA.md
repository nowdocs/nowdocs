# DMCA Takedown Policy

> Copyright (c) 2026 GWMM LLC. Last updated: 2026-07-04
> Jurisdiction: Wyoming, USA (Digital Millennium Copyright Act, 17 U.S.C. § 512)

## Scope

This policy applies to docsets hosted on the public nowdocs registry:

- `github.com/nowdocs-registry/*` (hosted on GitHub) — current
- `registry.nowdocs.dev/*` (self-hosted mirror) — domain reserved, **not yet
  active**; a separate notice will be published when it launches

nowdocs is a local MCP server that aggregates third-party developer
documentation. We respect third-party copyright. The registry is **curated**:
licenses are reviewed before publication, and takedown notices are acted on
after publication.

## How to Report Infringement

Registry repositories are hosted on GitHub, so takedown requests go through
**GitHub's built-in DMCA process**:

- **Primary channel**: [GitHub DMCA Takedown](https://github.com/contact/dmca)
  — GitHub processes compliant notices for content it hosts; nowdocs
  maintainers cooperate with removal.
- **Secondary**: `legal@gwmmai.com` (subject line `[nowdocs DMCA]`) — we
  forward it into the GitHub process. This channel does not replace GitHub's
  official channel.

## Elements of a Valid Notice (17 U.S.C. § 512(c)(3))

A complete notice must include all of the following:

1. Identification of the copyrighted work claimed to have been infringed
   (e.g., URL of the original work or copyright registration number).
2. Identification of the allegedly infringing material: the docset name and
   its location in the registry (repository URL).
3. Your contact information: name, mailing address, telephone number, and
   email address.
4. A statement that you have a good-faith belief that use of the material in
   the manner complained of is not authorized by the copyright owner, its
   agent, or the law.
5. A statement, under penalty of perjury, that the information in the notice
   is accurate and that you are the copyright owner or authorized to act on
   the owner's behalf.
6. Your physical or electronic signature.

## Response Process

1. **Removal**: when GitHub issues a takedown determination, the affected
   docset is removed from the registry immediately (we follow GitHub's
   determination without a separate GWMM LLC review).
2. **Counter-notification** (§ 512(g)(3)): the contributor may submit a
   counter-notification through GitHub, containing the statutory elements.
3. **Restoration**: content is restored within 10–14 business days after a
   valid counter-notification, unless the original complainant notifies us
   that it has filed a court action seeking to restrain the activity.

Note: § 512(f) provides liability for knowingly and materially
misrepresenting that material is infringing, or that it was removed by
mistake.

## Preventive Review: Registry Admission Checklist

Under the curated model, every docset is reviewed by a GWMM LLC curator
before publication.

**✅ Accepted licenses**:
MIT / Apache-2.0 / CC-BY-4.0 / CC0 / BSD-2-Clause / BSD-3-Clause / ISC

**❌ Rejected**:

- Proprietary documentation (e.g., Clerk, Tailwind — documentation not
  published under an open license)
- Documentation from sites whose Terms of Service prohibit scraping or
  redistribution
- Content scraped without authorization
- ShareAlike-family licenses (CC-BY-SA, GFDL): **not accepted for now** —
  share-alike obligations propagate to derived docsets; this position may be
  re-evaluated in the future
- License unclear or unverifiable

**Review record**: the `legal` block in each docset's `manifest.json` is
mandatory:

- `license`: SPDX identifier
- `copyright_holder`: copyright owner
- `attribution`: attribution text (mandatory for CC-BY-4.0)

## Repeat Infringers

Contributors who submit infringing docsets three times are permanently barred
from further registry submissions.

## Self-Hosted Registry (Future)

The current registry relies on GitHub as the hosting service provider. If
GWMM LLC self-hosts the registry at `registry.nowdocs.dev` in the future,
GWMM LLC will register a DMCA designated agent with the U.S. Copyright Office
before launch, and this policy will be updated with the agent's contact
information.

## Legal Contact

GWMM LLC (Wyoming, USA)
Email: `legal@gwmmai.com`
