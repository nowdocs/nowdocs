---
title: Incremental Static Regeneration with Cache Components
description: Learn how to prerender a subset of dynamic routes, serve App Shells for the rest, and upgrade them after the first visit.
nav_title: ISR with Cache Components
related:
  links:
    - app/api-reference/config/next-config-js/cacheComponents
    - app/api-reference/functions/generate-static-params
    - app/api-reference/file-conventions/loading
    - app/getting-started/caching
---

[Incremental Static Regeneration (ISR)](/docs/app/glossary#incremental-static-regeneration-isr) with [`cacheComponents`](/docs/app/api-reference/config/next-config-js/cacheComponents) gives every route an instant first visit, even for URLs that weren't included in the build.

During build, Partial Prerendering splits each render into two parts:

- The **App Shell**: the generic, reusable part of the page that doesn't depend on URL data
- The rest of the statically renderable content: the param-specific prerenders for the URLs you list in [`generateStaticParams`](/docs/app/api-reference/functions/generate-static-params)

For a visit to a URL whose params were included in `generateStaticParams`, Next.js serves the fully prerendered page from the cache. For a visit to a URL whose params weren't, Next.js serves the App Shell instantly, then upgrades it in the background with the now-known params. Subsequent visits to that URL get the upgraded result from the cache, skipping the App Shell entirely.

If you have used [ISR](/docs/app/guides/incremental-static-regeneration) or [`fallback: true`](https://nextjs.org/docs/pages/api-reference/functions/get-static-paths#fallback-true) in the Pages Router, this is the Cache Components equivalent.

Make sure [`cacheComponents`](/docs/app/api-reference/config/next-config-js/cacheComponents) is enabled in your project:

```ts filename="next.config.ts" highlight=
import type  from 'next'

const nextConfig: NextConfig = 

export default nextConfig
```

## Example

We'll build a product catalog with category layouts and product detail pages using the [Recipe API](https://next-recipe-api.vercel.dev/). You can find the resources used in this example here:

- [Demo](https://partial-fallbacks.labs.vercel.dev/)
- [Code](https://github.com/vercel-labs/partial-fallbacks)

### Prepare your routes

Use [`generateStaticParams`](/docs/app/api-reference/functions/generate-static-params) to define which param values to prerender. When rendering these routes, the param values are known at build time. The prerender process builds a static shell until it hits runtime APIs or uncached data. See [Prerendering](/docs/app/getting-started/caching#prerendering) for more details.

The category layout prerenders two categories:

```tsx filename="app/[category]/layout.tsx" highlight=
import Link from 'next/link'
import  from 'react'
import  from '../lib/data'

export async function generateStaticParams() {
  const categories = await getTopCategories()
  return categories.map((c) => ())
}

async function CategoryHeader(: Pick
      
      {data?.description && }
    
  )
}

export default function CategoryLayout(props: LayoutProps<'/[category]'>) {
  return (
    
      
      
    
  )
}
```

Notice that `CategoryLayout` does not `await props.params` itself. Instead, it passes the `params` promise to `CategoryHeader` inside `
      
      $
      
    </>
  )
}

export default function ProductPage(props: PageProps<'/[category]/[product]'>) 
```

> **Good to know:** You can also use [`loading.tsx`](/docs/app/api-reference/file-conventions/loading) files instead of `` in JSX. `loading.tsx` puts the boundary at the segment edge, while inline `` lets you place it anywhere in the tree.

The data helpers use `'use cache'` at the module level so all exported functions are cached. This lets their results be included in the static shell:

```ts filename="app/lib/data.ts" highlight=
'use cache'

const API = 'https://next-recipe-api.vercel.dev'

export async function getCategory(slug: string) {
  const res = await fetch(`$/categories/$`)
  if (!res.ok) return null
  return res.json()
}

export async function getProduct(category: string, slug: string) {
  const res = await fetch(`$/products/$/$`)
  if (!res.ok) return null
  return res.json()
}

export async function getTopCategories() {
  const res = await fetch(`$/categories`)
  const categories = await res.json()
  return categories.slice(0, 2)
}

export async function getPopularProducts(category: string) {
  const res = await fetch(`$/products?category=$`)
  const products = await res.json()
  return products.slice(0, 1)
}
```

If your components access runtime APIs like `cookies` or `headers`, wrap them in ``. Their fallback UI is included in the static shell instead.

### At build time

When you run `next build`, Next.js prerenders the layout for each known category (`tops`, `shorts`), plus one render where `await params` suspends, producing the App Shell for `[category]`.

It also prerenders the page for each known product under each category (`tee` under `tops`, `joggers` under `shorts`), plus one render where `await params` suspends, producing the App Shell for `[product]`.

These are combined into:

- `/tops/tee`, `/shorts/joggers`: fully static pages, both params known
- `/tops/[product]`, `/shorts/[product]`: category header rendered, product shows fallback
- `/[category]/[product]`: both show fallback

### At runtime

A visitor navigates to `/tops/tee`. Both params were prerendered. They get a fully static page.

The first visit to `/tops/overshirt`. The product `overshirt` is unknown, but the category `tops` was prerendered. Next.js serves the App Shell for `/tops/[product]` with the category header already rendered. The product streams in.

The first visit to `/shoes/basketball-shoes`. Neither param was prerendered. Next.js serves the generic App Shell for `/[category]/[product]`. Both the category and the product stream in.

After the first visit, Next.js renders these routes in the background with the now-known params. The next visitor to the same URLs gets the upgraded result.

### What the upgrade produces

After the first visit, Next.js renders the page in the background with the known params and tries to push the static boundary as far down the component tree as possible:

- If every data access is cached and all params are resolved, the upgrade produces a **fully static page**.
- If all params are resolved but the render still hits uncached data or runtime APIs (`cookies`, `headers`) wrapped in `` boundaries, the upgrade produces a **cached page with those fallbacks**. The uncached or runtime parts stream in at request time.
- Params are resolved in route order. A param value not returned by `generateStaticParams` stays unresolved and prevents any deeper params from upgrading.

> **Good to know**: Prefetching also triggers upgrades. When a [``](/docs/app/api-reference/components/link) enters the viewport or [`router.prefetch`](/docs/app/api-reference/functions/use-router) is called, Next.js can upgrade the App Shell in the background, so the next visitor gets the more specific version even before anyone actually navigates to the page.

## Coming from the Pages Router

If you are migrating from the Pages Router:

- `fallback: true` in `getStaticPaths` is now the default behavior with `cacheComponents`. Visitors get a `` fallback instantly and content streams in.
- `router.isFallback` is not needed. The prerendering step generates a static shell, and you can grow it further with `'use cache'`.
- `getStaticProps` with `revalidate` maps to `'use cache'` with [`cacheLife`](/docs/app/api-reference/functions/cacheLife).
- `getStaticPaths` maps to [`generateStaticParams`](/docs/app/api-reference/functions/generate-static-params).

## Next steps

- [Caching with Cache Components](/docs/app/getting-started/caching) for the full caching model
- [`generateStaticParams`](/docs/app/api-reference/functions/generate-static-params) for controlling which param combinations are prerendered
- [`loading.tsx`](/docs/app/api-reference/file-conventions/loading) for providing skeleton UI in App Shells
- [Streaming](/docs/app/guides/streaming) to learn how to progressively render UI as data becomes available
- [Self-hosting](/docs/app/guides/self-hosting#caching-and-isr) to keep an existing ISR [`cacheHandler`](/docs/app/api-reference/config/next-config-js/incrementalCacheHandlerPath) alongside [`cacheHandlers`](/docs/app/api-reference/config/next-config-js/cacheHandlers) for `'use cache'`
