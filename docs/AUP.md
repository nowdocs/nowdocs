# Acceptable Use Policy

> Copyright (c) 2026 GWMM LLC.
> Last updated: July 4, 2026.
> Scope: the nowdocs public registry and nowdocs software.

## Registry model

The nowdocs public registry is curated, not open-submission. GWMM LLC decides which docsets are admitted. Community members may propose docsets, but a curator must approve them before publication.

## Documentation that may be admitted

- Documentation released under MIT, Apache-2.0, or CC-BY-4.0.
- Official documentation whose license clearly permits redistribution.

## Documentation that may not be admitted

- Proprietary documentation, including documentation such as Clerk or Tailwind that is not released under a redistributable open license.
- Documentation from sites whose terms prohibit scraping or redistribution.
- Unauthorized scraped content without a clear license.
- ShareAlike licenses, including CC-BY-SA and GFDL. They are not currently admitted because their obligations can apply to derivative docsets; this may be revisited later.
- Malicious, deceptive, or poisoned content, including vector injection, misleading text, or embedded payloads.

## Contributor conduct

Registry contributors must:

- Not submit docsets they know infringe copyright or license terms.
- Not submit poisoned vectors, misleading text, or embedded malicious content.
- Complete the manifest `legal` block accurately: `license`, `copyright_holder`, and `attribution`.

After three violations, a contributor is barred from further registry submissions. Severe misconduct, such as deliberate poisoning, may result in an immediate ban.

## Safeguards

- **CI rebuilds vectors:** share bundles contain text and a manifest only. Registry CI rebuilds vectors with the pinned model, preventing vector injection and model drift.
- **License review:** a curator checks every candidate against the [DMCA.md](DMCA.md) checklist before publication.

## Software-use boundaries

nowdocs is a tool; users are responsible for their use of it.

- Do not use `nowdocs ingest` to collect proprietary documentation and redistribute it through the public registry.
- Local personal ingestion is not restricted by this policy. Users remain responsible for their own legal compliance, and local use does not make GWMM LLC responsible for the imported content.

## Contact

- Copyright reports: see [DMCA.md](DMCA.md).
- Code of Conduct reports: see [CODE_OF_CONDUCT.md](../CODE_OF_CONDUCT.md).
- General legal contact: `legal@gwmmai.com`.
