---
title: How to optimize your Next.js application for production
nav_title: Production
description: Recommendations to ensure the best performance and user experience before taking your Next.js application to production.
---

Before taking your Next.js application to production, there are some optimizations and patterns you should consider implementing for the best user experience, performance, and security.

This page provides best practices that you can use as a reference when [building your application](#during-development) and [before going to production](#before-going-to-production), as well as the [automatic Next.js optimizations](#automatic-optimizations) you should be aware of.

## Automatic optimizations

These Next.js optimizations are enabled by default and require no configuration:

These defaults aim to improve your application's performance, and reduce the cost and amount of data transferred on each network request.

## During development

While building your application, we recommend using the following features to ensure the best performance and user experience:

### Routing and rendering

### Data fetching and caching

### UI and accessibility

- **[Font Module](/docs/app/api-reference/components/font):** Optimize fonts by using the Font Module, which automatically hosts your font files with other static assets, removes external network requests, and reduces [layout shift](https://web.dev/articles/cls).
- **[`

- **[Environment Variables](/docs/app/guides/environment-variables):** Ensure your `.env.*` files are added to `.gitignore` and only public variables are prefixed with `NEXT_PUBLIC_`.
- **[Content Security Policy](/docs/app/guides/content-security-policy):** Consider adding a Content Security Policy to protect your application against various security threats such as cross-site scripting, clickjacking, and other code injection attacks.

### Metadata and SEO

### Type safety

- **TypeScript and [TS Plugin](/docs/app/api-reference/config/typescript):** Use TypeScript and the TypeScript plugin for better type-safety, and to help you catch errors early.

## Before going to production

Before going to production, you can run `next build` to build your application locally and catch any build errors, then run `next start` to measure the performance of your application in a production-like environment.

### Core Web Vitals

- **[Lighthouse](https://developers.google.com/web/tools/lighthouse):** Run lighthouse in incognito to gain a better understanding of how your users will experience your site, and to identify areas for improvement. This is a simulated test and should be paired with looking at field data (such as Core Web Vitals).

### Analyzing bundles

Use the [`@next/bundle-analyzer` plugin](/docs/app/guides/package-bundling#nextbundle-analyzer-for-webpack) to analyze the size of your JavaScript bundles and identify large modules and dependencies that might be impacting your application's performance.

Additionally, the following tools can help you understand the impact of adding new dependencies to your application:

- [Import Cost](https://marketplace.visualstudio.com/items?itemName=wix.vscode-import-cost)
- [Package Phobia](https://packagephobia.com/)
- [Bundle Phobia](https://bundlephobia.com/)
- [bundlejs](https://bundlejs.com/)
