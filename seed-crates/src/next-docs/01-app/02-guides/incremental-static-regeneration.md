---
title: How to implement Incremental Static Regeneration (ISR)
nav_title: ISR
description: Learn how to create or update static pages at runtime with Incremental Static Regeneration.
---

  Examples

- [Next.js Commerce](https://vercel.com/templates/next.js/nextjs-commerce)
- [On-Demand ISR](https://on-demand-isr.vercel.app)
- [Next.js Forms](https://github.com/vercel/next.js/tree/canary/examples/next-forms)

> **Good to know**: This guide covers ISR without Cache Components. If you are using [`cacheComponents`](/docs/app/api-reference/config/next-config-js/cacheComponents), see [ISR with Cache Components](/docs/app/guides/incremental-static-regeneration-cache-components) instead.

Incremental Static Regeneration (ISR) enables you to:

- Update static content without rebuilding the entire site
- Reduce server load by serving prerendered, static pages for most requests
- Ensure proper `cache-control` headers are automatically added to pages
- Handle large amounts of content pages without long `next build` times

Here's a minimal example:

## Reference

## Examples

### Handling uncaught exceptions

### Customizing the cache location

You can configure the Next.js cache location if you want to persist cached pages and data to durable storage, or share the cache across multiple containers or instances of your Next.js application. [Learn more](/docs/app/guides/self-hosting#caching-and-isr).

## Troubleshooting

### Debugging cached data in local development

If you are using the `fetch` API, you can add additional logging to understand which requests are cached or uncached. [Learn more about the `logging` option](/docs/app/api-reference/config/next-config-js/logging).

```jsx filename="next.config.js"
module.exports = {
  logging: {
    fetches: ,
  },
}
```

### Verifying correct production behavior

To verify your pages are cached and revalidated correctly in production, you can test locally by running `next build` and then `next start` to run the production Next.js server.

This will allow you to test ISR behavior as it would work in a production environment. For further debugging, add the following environment variable to your `.env` file:

```bash filename=".env"
NEXT_PRIVATE_DEBUG_CACHE=1
```

This will make the Next.js server console log ISR cache hits and misses. You can inspect the output to see which pages are generated during `next build`, as well as how pages are updated as paths are accessed on-demand.

## Caveats

## Platform Support

| Deployment Option                                                   | Supported         |
| ------------------------------------------------------------------- | ----------------- |
| [Node.js server](/docs/app/getting-started/deploying#nodejs-server) | Yes               |
| [Docker container](/docs/app/getting-started/deploying#docker)      | Yes               |
| [Static export](/docs/app/getting-started/deploying#static-export)  | No                |
| [Adapters](/docs/app/getting-started/deploying#adapters)            | Platform-specific |

Learn how to [configure ISR](/docs/app/guides/self-hosting#caching-and-isr) when self-hosting Next.js.

## Version history

| Version   | Changes                                                                             |
| --------- | ----------------------------------------------------------------------------------- |
| `v14.1.0` | Custom `cacheHandler` is stable.                                                    |
| `v13.0.0` | App Router is introduced.                                                           |
| `v12.2.0` | Pages Router: On-Demand ISR is stable                                               |
| `v12.0.0` | Pages Router: [Bot-aware ISR fallback](/blog/next-12#bot-aware-isr-fallback) added. |
| `v9.5.0`  | Pages Router: [Stable ISR introduced](/blog/next-9-5).                              |
