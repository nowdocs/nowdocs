---
title: 'use cache: remote'
description: 'Learn how to use the "use cache: remote" directive for persistent, shared caching using remote cache handlers.'
related:
  title: Related
  description: View related API references.
  links:
    - app/api-reference/directives/use-cache
    - app/api-reference/directives/use-cache-private
    - app/api-reference/config/next-config-js/cacheComponents
    - app/api-reference/config/next-config-js/cacheHandlers
    - app/api-reference/functions/cacheLife
    - app/api-reference/functions/cacheTag
    - app/api-reference/functions/connection
---

While the `use cache` directive is sufficient for most application needs, you might notice that cached operations are re-running more often than expected, or that your upstream services (CMS, databases, external APIs) are getting more hits than you'd expect. This can happen because `use cache` stores entries in-memory, which has inherent limitations:

- Cache entries being evicted to make room for new ones
- Memory constraints in your deployment environment
- Cache not persisting across requests or server restarts

Note that `use cache` still provides value beyond server-side caching: it informs Next.js what can be prefetched and defines stale times for client-side navigation.

The `'use cache: remote'` directive lets you declaratively specify that a cached output should be stored in a remote cache instead of in-memory, providing durable caching shared across all server instances. This comes with tradeoffs: infrastructure cost and network latency during cache lookups.

## Usage

To use `'use cache: remote'`, enable the [`cacheComponents`](/docs/app/api-reference/config/next-config-js/cacheComponents) flag in your `next.config.ts` file:

```ts filename="next.config.ts" switcher
import type  from 'next'

const nextConfig: NextConfig = 

export default nextConfig
```

```js filename="next.config.js" switcher
/** @type  */
const nextConfig = 

export default nextConfig
```

Then add `'use cache: remote'` to the functions or components where you've determined remote caching is justified. The handler implementation is configured via [`cacheHandlers`](/docs/app/api-reference/config/next-config-js/cacheHandlers), though hosting providers should typically provide this automatically. If you're self-hosting, see the `cacheHandlers` configuration reference to set up your cache storage.

### When to avoid remote caching

- If you already have a server-side cache key-value store wrapping your data layer, `use cache` may be sufficient to include data in the static shell without adding another caching layer
- If operations are already fast (< 50ms) due to proximity or local access, the remote cache lookup might not improve performance
- If cache keys have mostly unique values per request (search filters, price ranges, user-specific parameters), cache utilization will be near-zero
- If data changes frequently (seconds to minutes), cache hits will quickly go stale, leading to frequent misses and waiting for upstream revalidation

### When remote caching makes sense

Remote caching provides the most value when content is deferred to request time (outside the static shell). This typically happens when a component accesses request values like [`cookies()`](/docs/app/api-reference/functions/cookies), [`headers()`](/docs/app/api-reference/functions/headers), or [`searchParams`](/docs/app/api-reference/file-conventions/page#searchparams-optional), placing it inside a Suspense boundary. In this context:

- Each request executes the component and looks up the cache
- In serverless environments, each instance has its own ephemeral memory with low cache hit rates
- Remote caching provides a shared cache across all instances, improving hit rates and reducing backend load

Compelling scenarios for `'use cache: remote'`:

- **Rate-limited APIs**: Your upstream service has rate limits or request quotas that you risk hitting
- **Protecting slow backends**: Your database or API becomes a bottleneck under high traffic
- **Expensive operations**: Database queries or computations that are costly to run repeatedly
- **Flaky or unreliable services**: External services that occasionally fail or have availability issues

In these cases, the cost and latency of remote caching is justified by avoiding worse outcomes (rate limit errors, backend overload, high compute bills, or degraded user experience).

For static shell content, `use cache` is usually sufficient. If your upstream source can't handle concurrent revalidation requests (like a rate-limited CMS), `use cache: remote` acts as a shared cache layer. This is the same pattern as putting a key-value store in front of a database, but declared in code.

### How `use cache: remote` differs from `use cache` and `use cache: private`

Next.js provides three caching directives, each designed for different use cases:

| Feature                                 | `use cache`                     | `'use cache: remote'`             | `'use cache: private'` |
| --------------------------------------- | ------------------------------- | --------------------------------- | ---------------------- |
| **Server-side caching**                 | In-memory or cache handler      | Remote cache handler              | None                   |
| **Cache scope**                         | Shared across all users         | Shared across all users           | Per-client (browser)   |
| **Can access cookies/headers directly** | No (must pass as arguments)     | No (must pass as arguments)       | Yes                    |
| **Server cache utilization**            | May be low outside static shell | High (shared across instances)    | N/A                    |
| **Additional costs**                    | None                            | Infrastructure (storage, network) | None                   |
| **Latency impact**                      | None                            | Cache handler lookup              | None                   |
| **Persists across deploys**             | No                              | No                                | N/A                    |

### Persistence across deploys

Remote cache entries do not persist across deploys. The cache key includes the `deploymentId` (when configured) or the `buildId`, so a new build produces new keys and the previous build's entries are no longer reachable. See [Cache keys](/docs/app/api-reference/directives/use-cache#cache-keys) for the full key composition.

This is intentional. Between builds, the function's identity hash or the shape of its return value can change. Upgrading a CMS client, refactoring a cached function, or changing a dependency could produce a value that doesn't match what older callers expect, so reusing entries across deploys risks serving stale or malformed data.

If you need entries that persist across deploys, use [`unstable_cache`](/docs/app/api-reference/functions/unstable_cache) for non-`fetch` functions, or rely on the [`fetch`](/docs/app/api-reference/functions/fetch) cache.

### Caching with runtime data

Both `use cache` and `'use cache: remote'` can't access runtime values like cookies or search params directly. You can extract these values and pass them as arguments to cached functions. See [with runtime data](/docs/app/getting-started/caching#working-with-runtime-apis) for this pattern.

> **Good to know**: `use cache` stores entries in-memory. In serverless environments, memory is not shared between instances and is typically destroyed after serving a request, leading to frequent cache misses for runtime caching.

### Cache key considerations

Be thoughtful about which values you include in cache keys. Each unique value creates a separate cache entry, reducing cache utilization. Consider this example with search filters:

```tsx filename="app/products/[category]/page.tsx"
import  from 'react'

export default async function ProductsPage(: {
  params: Promise<>
  searchParams: Promise<>
}) 

async function ProductList(: {
  params: Promise<>
  searchParams: Promise<>
}) {
  const  = await params

  const  = await searchParams

  // Cache only on category (few unique values)
  // Don't include price filter (many unique values)
  const products = await getProductsByCategory(category)

  // Filter price in memory instead of creating cache entries
  // for every price value
  const filtered = minPrice
    ? products.filter((p) => p.price >= parseFloat(minPrice))
    : products

  return 
}

async function getProductsByCategory(category: string) 
```

In this example, the remote handler stores more data per cache entry (all products in a category) to achieve better cache hit rates. This is worth it when the cost of cache misses (hitting your backend) outweighs the storage cost of larger entries.

The same principle applies to user-specific data. Rather than caching per-user data directly, use user preferences to determine what shared data to cache.

For example, if users have a language preference in their session, extract that preference and use it to cache shared content:

- Instead of remote caching `getUserProfile(sessionID)`, which creates one entry per user
- Remote cache `getCMSContent(language)` to create one entry per language

```tsx filename="app/components/welcome-message.tsx"
import  from 'next/headers'
import  from 'next/cache'

export async function WelcomeMessage() {
  // Extract the language preference (not unique per user)
  const language = (await cookies()).get('language')?.value || 'en'

  // Cache based on language (few unique values: en, es, fr, de, etc.)
  // All users who prefer 'en' share the same cache entry
  const content = await getCMSContent(language)

  return 
}

async function getCMSContent(language: string) {
  'use cache: remote'
  cacheLife()
  // Creates ~10-50 cache entries (one per language)
  // instead of thousands (one per user)
  return cms.getHomeContent(language)
}
```

This way all users who prefer the same language share a cache entry, improving cache utilization and reducing load on your CMS.

The pattern is the same in both examples: find the dimension with fewer unique values (category vs. price, language vs. user ID), cache on that dimension, and filter or select the rest in memory.

If the service used by `getUserProfile` cannot scale with your frontend load, you may still be able to use the `use cache` directive with a short `cacheLife` for in-memory caching. However, for most user data, you likely want to fetch directly from the source (which might already be wrapped in a key/value store as mentioned in the guidelines above).

Only use [`'use cache: private'`](/docs/app/api-reference/directives/use-cache-private) if you have compliance requirements or can't refactor to pass runtime data as arguments.

### Nesting rules

Remote caches have specific nesting rules:

- Remote caches **can** be nested inside other remote caches (`'use cache: remote'`)
- Remote caches **can** be nested inside regular caches (`'use cache'`)
- Remote caches **cannot** be nested inside private caches (`'use cache: private'`)
- Private caches **cannot** be nested inside remote caches

```tsx
// VALID: Remote inside remote
async function outerRemote() 

async function innerRemote() 

// VALID: Remote inside regular cache
async function outerCache() 

async function innerRemote() 

// INVALID: Remote inside private
async function outerPrivate() 

async function innerRemote() 

// INVALID: Private inside remote
async function outerRemote() 

async function innerPrivate() 
```

## Examples

The following examples demonstrate common patterns for using `'use cache: remote'`. For details about `cacheLife` parameters (`stale`, `revalidate`, `expire`), see the [`cacheLife` API reference](/docs/app/api-reference/functions/cacheLife).

### With user preferences

Cache product pricing based on the user's currency preference. Since the currency is stored in a cookie, this component renders at request time. Remote caching is valuable here because all users with the same currency share the cached price, and in serverless environments, all instances share the same remote cache.

```tsx filename="app/product/[id]/page.tsx" switcher
import  from 'react'
import  from 'next/headers'
import  from 'next/cache'

export async function generateStaticParams() {
  return [, , ]
}

export default async function ProductPage(: {
  params: Promise<>
}) {
  const  = await params

  return (
    
      
    
  )
}

function ProductDetails(: ) {
  return Product: 
}

async function ProductPrice(: ) {
  // Reading cookies defers this component to request time
  const currency = (await cookies()).get('currency')?.value ?? 'USD'

  // Cache the price per product and currency combination
  // All users with the same currency share this cache entry
  const price = await getProductPrice(productId, currency)

  return (
    
      Price:  
    
  )
}

async function getProductPrice(productId: string, currency: string) {
  'use cache: remote'
  cacheTag(`product-price-$`)
  cacheLife() // 1 hour

  // Cached per (productId, currency) - few currencies means high cache utilization
  return db.products.getPrice(productId, currency)
}
```

```jsx filename="app/product/[id]/page.js" switcher
import  from 'react'
import  from 'next/headers'
import  from 'next/cache'

export async function generateStaticParams() {
  return [, , ]
}

export default async function ProductPage() {
  const  = await params

  return (
    
      
    
  )
}

function ProductDetails() {
  return Product: 
}

async function ProductPrice() {
  // Reading cookies defers this component to request time
  const currency = (await cookies()).get('currency')?.value ?? 'USD'

  // Cache the price per product and currency combination
  // All users with the same currency share this cache entry
  const price = await getProductPrice(productId, currency)

  return (
    
      Price:  
    
  )
}

async function getProductPrice(productId, currency) {
  'use cache: remote'
  cacheTag(`product-price-$`)
  cacheLife() // 1 hour

  // Cached per (productId, currency) - few currencies means high cache utilization
  return db.products.getPrice(productId, currency)
}
```

### Reducing database load

Cache expensive database queries, reducing load on your database. In this example, we don't access `cookies()`, `headers()`, or `searchParams`. If we had a requirement to not include these stats in the static shell, we could use [`connection()`](/docs/app/api-reference/functions/connection) to explicitly defer to request time:

```tsx filename="app/dashboard/page.tsx"
import  from 'react'
import  from 'next/server'
import  from 'next/cache'

export default function DashboardPage() 

async function DashboardStats() 

async function FeedItems() {
  // Defer to request time
  await connection()

  const items = await getFeedItems()

  return items.map((item) => 

      
      
    
  )
}

function ProductDetails() {
  return (
    
      
      
    
  )
}

async function ProductPriceComponent() {
  // Defer to request time
  await connection()

  const price = await getProductPrice(productId)
  return Price: $
}

async function ProductRecommendations() 

function PriceSkeleton() 

function RecommendationsSkeleton() 

function RecommendationsList() {
  return (
    
      {items.map((item) => (
        
      ))}
    
  )
}
```

> **Good to know**:
>
> - Remote caches are stored in server-side cache handlers and shared across all users
> - `'use cache: remote'` works outside the static shell where [`use cache`](/docs/app/api-reference/directives/use-cache) may not provide server-side cache hits
> - Use [`cacheTag()`](/docs/app/api-reference/functions/cacheTag) and [`revalidateTag()`](/docs/app/api-reference/functions/revalidateTag) to invalidate remote caches on-demand
> - Use [`cacheLife()`](/docs/app/api-reference/functions/cacheLife) to configure cache expiration
> - For user-specific data, use [`'use cache: private'`](/docs/app/api-reference/directives/use-cache-private) instead of `'use cache: remote'`
> - Remote caches reduce origin load by storing computed or fetched data server-side

## Platform Support

| Deployment Option                                                   | Supported |
| ------------------------------------------------------------------- | --------- |
| [Node.js server](/docs/app/getting-started/deploying#nodejs-server) | Yes       |
| [Docker container](/docs/app/getting-started/deploying#docker)      | Yes       |
| [Static export](/docs/app/getting-started/deploying#static-export)  | No        |
| [Adapters](/docs/app/getting-started/deploying#adapters)            | Yes       |

## Version History

| Version   | Changes                                                             |
| --------- | ------------------------------------------------------------------- |
| `v16.0.0` | `"use cache: remote"` is enabled with the Cache Components feature. |
