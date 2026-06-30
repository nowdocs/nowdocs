---
title: cacheLife
description: Learn how to use the cacheLife function to set the cache expiration time for a cached function or component.
related:
  title: Related
  description: View related API references.
  links:
    - app/api-reference/config/next-config-js/cacheComponents
    - app/api-reference/directives/use-cache
    - app/api-reference/functions/revalidateTag
    - app/api-reference/functions/cacheTag
---

The `cacheLife` function is used to set the cache lifetime of a function or component. It should be used alongside the [`use cache`](/docs/app/api-reference/directives/use-cache) directive, and within the scope of the function or component.

## Usage

### Basic setup

To use `cacheLife`, first enable the [`cacheComponents` flag](/docs/app/api-reference/config/next-config-js/cacheComponents) in your `next.config.js` file:

```ts filename="next.config.ts" switcher
import type  from 'next'

const nextConfig: NextConfig = 

export default nextConfig
```

```js filename="next.config.js" switcher
const nextConfig = 

export default nextConfig
```

`cacheLife` can only be used within a cache directive scope.

Add a cache directive (for example, `use cache`) at the file level or at the top of an async function or component.

`cacheLife` cannot be used at module scope. Calling it at the top level of a file will throw an error.

> **Good to know**:
>
> - Calling `cacheLife` is optional. If omitted, a default cache lifetime is applied.
> - Call `cacheLife` in the same function or component where caching is defined. Avoid abstracting it into shared utilities so the cache behavior remains explicit and easy to reason about.
> - If you call `cacheLife`, ensure only one call executes per function invocation. You can call it in different control flow branches, but only one should run per request. See the [conditional cache lifetimes](#conditional-cache-lifetimes) example.

### Using preset profiles

Next.js provides preset cache profiles that cover common caching needs. Each profile balances three factors:

- How long users see cached content without checking for updates (client-side)
- How often fresh content is generated on the server
- When old content expires completely

Choose a profile based on how frequently your content changes:

- **`seconds`** - Real-time data (stock prices, live scores)
- **`minutes`** - Frequently updated (social feeds, news)
- **`hours`** - Multiple daily updates (product inventory, weather)
- **`days`** - Daily updates (blog posts, articles)
- **`weeks`** - Weekly updates (podcasts, newsletters)
- **`max`** - Rarely changes (legal pages, archived content)

Import `cacheLife` and pass a profile name:

```tsx filename="app/blog/page.tsx" highlight=
'use cache'
import  from 'next/cache'

export default async function BlogPage() {
  cacheLife('days') // Blog content updated daily

  const posts = await getBlogPosts()
  return 
}
```

The profile name tells Next.js how to cache the entire function's output. If you don't call `cacheLife`, the `default` profile is used. See [preset cache profiles](#preset-cache-profiles) for timing details.

## Reference

### Cache profile properties

Cache profiles control caching behavior through three timing properties:

- **[`stale`](#stale)**: How long the client can use cached data without checking the server
- **[`revalidate`](#revalidate)**: After this time, the next request will trigger a background refresh
- **[`expire`](#expire)**: After this time with no requests, the next one waits for fresh content

#### `stale`

**Client-side:** How long the client can use cached data without checking the server.

During this time, the client-side router displays cached content immediately without any network request. After this period expires, the router must check with the server on the next navigation or request. This provides instant page loads from the client cache, but data may be outdated.

- If omitted, defaults to the `default` profile's `stale` value (5 minutes, see [`staleTimes`](/docs/app/api-reference/config/next-config-js/staleTimes))

```tsx
cacheLife() // 5 minutes
```

#### `revalidate`

How often the server regenerates cached content in the background.

- When a request arrives after this period, the server:
  1. Serves the cached version immediately (if available)
  2. Regenerates content in the background
  3. Updates the cache with fresh content
- Similar to [Incremental Static Regeneration (ISR)](/docs/app/guides/incremental-static-regeneration)
- If omitted, defaults to the `default` profile's `revalidate` value (15 minutes)

```tsx
cacheLife() // 15 minutes
```

#### `expire`

Maximum time before the server must regenerate cached content.

- After this period with no traffic, the server regenerates content synchronously on the next request
- When you set both `revalidate` and `expire`, `expire` must be longer than `revalidate`. Next.js validates this and raises an error for invalid configurations.
- If omitted, defaults to the `default` profile's `expire` value (never expires)

```tsx
cacheLife() // 1 hour
```

### Preset cache profiles

If you don't specify a profile, Next.js uses the `default` profile. We recommend explicitly setting a profile to make caching behavior clear.

| **Profile** | **Use Case**                           | `stale`    | `revalidate` | `expire` |
| ----------- | -------------------------------------- | ---------- | ------------ | -------- |
| `default`   | Standard content                       | 5 minutes  | 15 minutes   | never    |
| `seconds`   | Real-time data                         | 30 seconds | 1 second     | 1 minute |
| `minutes`   | Frequently updated content             | 5 minutes  | 1 minute     | 1 hour   |
| `hours`     | Content updated multiple times per day | 5 minutes  | 1 hour       | 1 day    |
| `days`      | Content updated daily                  | 5 minutes  | 1 day        | 1 week   |
| `weeks`     | Content updated weekly                 | 5 minutes  | 1 week       | 30 days  |
| `max`       | Stable content that rarely changes     | 5 minutes  | 30 days      | 1 year   |

### Custom cache profiles

Define reusable cache profiles in your `next.config.ts` file:

```ts filename="next.config.ts"
import type  from 'next'

const nextConfig: NextConfig = {
  cacheComponents: true,
  cacheLife: {
    biweekly: ,
  },
}

export default nextConfig
```

```js filename="next.config.js" switcher
const nextConfig = {
  cacheComponents: true,
  cacheLife: {
    biweekly: ,
  },
}

module.exports = nextConfig
```

The example above caches for 14 days, checks for updates daily, and expires the cache after 14 days. You can then reference this profile throughout your application by its name:

> **Good to know**: Any omitted properties in a custom profile inherit from the `default` profile. This also applies to inline profile objects passed directly to `cacheLife()`.

```tsx filename="app/page.tsx" highlight=
'use cache'
import  from 'next/cache'

export default async function Page() 
```

### Overriding the default cache profiles

While the default cache profiles provide a useful way to think about how fresh or stale any given part of cacheable output can be, you may prefer different named profiles to better align with your applications caching strategies.

You can override the default named cache profiles by creating a new configuration with the same name as the defaults.

The example below shows how to override the default `"days"` cache profile:

```ts filename="next.config.ts"
const nextConfig = {
  cacheComponents: true,
  cacheLife: {
    // Override the 'days' profile
    days: ,
  },
}

export default nextConfig
```

### Inline cache profiles

For one-off cases, pass a profile object directly to `cacheLife`:

```tsx filename="app/page.tsx"
'use cache'
import  from 'next/cache'

export default async function Page() {
  cacheLife()

  return Page
}
```

Inline profiles apply only to the specific function or component. For reusable configurations, define custom profiles in `next.config.ts`.

Using `cacheLife()` with an empty object applies the `default` profile values.

### Client cache behavior

The `stale` property controls the [Client Cache](/docs/app/glossary#client-cache), not the `Cache-Control` header:

- The server sends the stale time via the `x-nextjs-stale-time` response header
- The client router uses this value to determine when to revalidate
- **Minimum of 30 seconds is enforced** to ensure prefetched links remain usable

This 30-second minimum prevents prefetched data from expiring before users can click on links. It only applies to time-based expiration.

When you call revalidation functions from a Server Action ([`revalidateTag`](/docs/app/api-reference/functions/revalidateTag), [`revalidatePath`](/docs/app/api-reference/functions/revalidatePath), [`updateTag`](/docs/app/api-reference/functions/updateTag), or [`refresh`](/docs/app/api-reference/functions/refresh)), the entire client cache is immediately cleared, bypassing the stale time.

> **Good to know**: The `stale` property in `cacheLife` differs from [`staleTimes`](/docs/app/api-reference/config/next-config-js/staleTimes). While `staleTimes` is a global setting affecting all routes, `cacheLife` allows per-function or per-route configuration. Updating `staleTimes.static` also updates the `stale` value of the `default` cache profile.

### Prerendering behavior

Caches with very short lifetimes — zero `revalidate` or `expire` under 5 minutes — are automatically excluded from prerenders and become "dynamic holes" instead. This includes the `seconds` profile.

This behavior allows you to mix static and dynamic content within the same page. Static parts are prerendered, while short-lived caches create boundaries where data is fetched at request time rather than build time. Use a `
    
  )
}
```

> **Note:** This example uses `"use cache: remote"` because runtime caching in serverless deployments doesn't persist across requests with the default in-memory cache. For self-hosted environments, `"use cache"` may be sufficient. See [Runtime caching considerations](/docs/app/api-reference/directives/use-cache#runtime-caching-considerations) for more details.

### Conditional cache lifetimes

You can call `cacheLife` conditionally in different code paths to set different cache durations based on your application logic:

```tsx filename="lib/posts.ts" highlight=
import  from 'next/cache'

async function getPostContent(slug: string) {
  'use cache'

  const post = await fetchPost(slug)

  // Tag the cache entry for targeted revalidation
  cacheTag(`post-$`)

  if (!post) 

  // Published content can be cached longer
  cacheLife('days')

  // Return only the necessary data to keep cache size minimal
  return post.data
}
```

This pattern is useful when different outcomes need different cache durations, for example, when an item is missing but is likely to be available later.

#### Using dynamic cache lifetimes from data

If you want to calculate cache lifetime at runtime, for example by reading it from the fetched data, use an [inline cache profile](#inline-cache-profiles) object:

```tsx filename="lib/posts.ts" highlight=
import  from 'next/cache'

async function getPostContent(slug: string) {
  'use cache'

  const post = await fetchPost(slug)
  cacheTag(`post-$`)

  if (!post) 

  // Use cache timing from CMS data directly as an object
  cacheLife()

  return post.data
}
```
