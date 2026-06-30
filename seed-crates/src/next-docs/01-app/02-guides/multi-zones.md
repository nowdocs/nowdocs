---
title: How to build micro-frontends using multi-zones and Next.js
nav_title: Multi-zones
description: Learn how to build micro-frontends using Next.js Multi-Zones to deploy multiple Next.js apps under a single domain.
---

  Examples

- [With Zones](https://github.com/vercel/next.js/tree/canary/examples/with-zones)

Multi-Zones are an approach to micro-frontends that separate a large application on a domain into smaller Next.js applications that each serve a set of paths. This is useful when there are collections of pages unrelated to the other pages in the application. By moving those pages to a separate zone (i.e., a separate application), you can reduce the size of each application which improves build times and removes code that is only necessary for one of the zones. Since applications are decoupled, Multi-Zones also allows other applications on the domain to use their own choice of framework.

For example, let's say you have the following set of pages that you would like to split up:

- `/blog/*` for all blog posts
- `/dashboard/*` for all pages when the user is logged-in to the dashboard
- `/*` for the rest of your website not covered by other zones

With Multi-Zones support, you can create three applications that all are served on the same domain and look the same to the user, but you can develop and deploy each of the applications independently.

