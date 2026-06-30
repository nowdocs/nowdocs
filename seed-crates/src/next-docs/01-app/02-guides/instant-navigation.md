---
title: Ensuring instant navigations
description: Learn how to structure your app to prefetch and prerender more content, providing instant page loads and client navigations.
nav_title: Instant navigation
related:
  title: Learn more
  description: Explore the full instant API, caching, and revalidation.
  links:
    - app/api-reference/file-conventions/route-segment-config/instant
    - app/guides/runtime-prefetching
    - app/getting-started/caching
    - app/getting-started/revalidating
    - app/guides/prefetching
---

This guide walks through understanding instant navigations, writing a route that navigates instantly, visualizing what's in the initial UI, and locking the behavior in with end-to-end tests.

## What "instant" means

A navigation is **instant** when the browser can start rendering the new page the moment the user clicks, with static, cached, and fallback content showing up right away, while the server streams the remaining content into its fallbacks.

> **Good to know:** This definition assumes caches are warm. Cold caches still require the server to compute the cached result once, so the first navigation to a route may still wait.

A direct visit and a client navigation to the same route can produce different initial UI. **Direct visits** get the [**static shell**](/docs/app/glossary#static-shell) as HTML, typically from a CDN. **Client navigations** only re-render below the layout the current and destination routes share, so the fallback UI defined by a `
```

To prefetch with the user's session (cookies, headers, the full URL), opt the destination segment into [runtime prefetching](/docs/app/guides/runtime-prefetching) with `export const prefetch = 'allow-runtime'`.

### Validate instant navigation

By **default** (`validationLevel: 'warning'`), Cache Components apps validate every Page and Default segment in development. Validation surfaces what would keep navigations into a segment from being instant — which navigations would block, where a `
      
    
  )
}

type Params = PageProps<'/store/[slug]'>['params']

async function ProductInfo(: ) {
  const  = await params
  const product = await getProduct(slug)
  return (
    <>
      
      $
    </>
  )
}

async function getProduct(slug: string) 

async function Inventory(: ) {
  const  = await params
  const item = await db.inventory.findBySlug(slug)
  return  in stock
}
```

Cache Components validates this route automatically in development. If something would block a navigation, the dev overlay surfaces a **blocking-route** insight that names the offending component and points at these fixes:

', highlight: true },
    ]}
  />

> **Good to know:** Each fix card links to a detailed walkthrough with patterns, code samples, and trade-offs. Click a card to dive in.

Validation runs on every page load using the real request from your browser, so dynamic params like `[slug]` are checked against actual values as you navigate.

## Visualize loading states with the Next.js DevTools

As you develop a route, the Next.js DevTools let you see what your users see on page loads and client navigations before dynamic data streams in. Use it to verify that your loading states look right, confirm the content you expect appears immediately, and iterate on where to place `
    
  )
}
```

Now `await props.params` and the product fetch suspend together. The product-fetch error clears, and validation moves on to the next blocker.

### Step 2: Cache the featured fetch

Validation now fires on `getFeatured()`.

  

Set `instant = false` on the page or layout file. This opts the segment out of validation feedback. The segment may still navigate instantly if its structure supports it; the framework just won't surface insights for it. Navigations between sibling segments below are still validated.

```tsx filename="app/dashboard/layout.tsx"
export const instant = false
```

With `false` on `/dashboard/layout.tsx`, validation no longer flags navigations into `/dashboard` from outside; navigations between `/dashboard/a` and `/dashboard/b` are still checked.

For opted-out segments, the navigation blocks on the server. If the content depends on cookies or headers but has a known cache lifetime, [runtime prefetching](/docs/app/guides/runtime-prefetching) can prerender it ahead of click instead of opting out.

## Next steps

- [Adopting Partial Prefetching](/docs/app/guides/adopting-partial-prefetching) for the recommended `` defaults and the migration path off `unstable_eager`
- [`instant` API reference](/docs/app/api-reference/file-conventions/route-segment-config/instant) for the full configuration
- [Runtime prefetching](/docs/app/guides/runtime-prefetching) when parts of your route depend on cookies or headers and you want those in the shell
- [Caching](/docs/app/getting-started/caching) for background on `use cache`, Suspense, and Partial Prerendering
- [Revalidating](/docs/app/getting-started/revalidating) for how to expire cached data with `cacheLife` and `updateTag`
