---
title: Runtime prefetching
description: Extend the App Shell with personalized content using the prefetch segment config and per-session caching directives.
nav_title: Runtime prefetching
version: experimental
related:
  title: Learn more
  description: Validate your structure and dive into caching primitives.
  links:
    - app/api-reference/file-conventions/route-segment-config/prefetch
    - app/api-reference/file-conventions/route-segment-config/instant
    - app/api-reference/directives/use-cache-private
    - app/getting-started/caching
    - app/guides/instant-navigation
---

The router prefetches an [App Shell](/docs/app/glossary#app-shell) per route as visible `
        
        
      
    
  )
}
```

```tsx filename="app/courses/page.tsx"
import  from 'react'

export const prefetch = 'allow-runtime'

export default function CoursesPage() 
```

Without `prefetch = 'allow-runtime'`, the App Shell renders `` and `
      
    
  )
}
```

`` reads a cookie, then looks up data based on it. The challenge is that `"use cache"` can't read `cookies()` inside the cached function. Two patterns handle this: **extract and pass** when the lookup result is shared across many users, and `"use cache: private"` when it's tied to a specific user.

### Extract and pass

Read the cookie outside the cached function and pass the value in as an argument. `cookies()` stays outside the cache scope, the argument crosses the boundary, and the cached function has a deterministic signature. The cache entry is keyed on that argument; if many users share the value, they share the entry.

```tsx filename="app/dashboard/user-nav.tsx"
import  from 'next/headers'

async function UserNav() {
  const team = (await cookies()).get('team')?.value
  const topics = await getTopics(team)
  return (
    
      {topics.map((topic) => (
        
          
        
      ))}
    
  )
}

async function getTopics(team: string | undefined) 
```

On a direct visit, `` streams in behind the fallback. With runtime prefetching, the prerender resolves `` before the click because the team cookie is available at prefetch time. Users on the same team share the cache entry, so traffic to the underlying data scales with team count, not user count.

Anything without a caching directive still streams in after navigation. The runtime prerender is not a full server render. It advances only as far as the caching structure allows.

### `"use cache: private"`

When the lookup is tied to a specific user, use [`"use cache: private"`](/docs/app/api-reference/directives/use-cache-private). It assigns a cache lifetime to a function that reads cookies, headers, or other runtime data directly. Results are cached in the browser only, so the cache is per-user by definition.

```tsx filename="app/dashboard/user-nav.tsx"
import  from 'next/headers'

async function UserNav() {
  const user = await getUser()
  return 
}

async function getUser() 
```

`cookies()` lives inside the cached function, which only works under `"use cache: private"`. This is also the pattern when you can't extract the runtime data from the outside: auth helpers that check `Date.now()` against a token's expiry, or session helpers that read cookies deep inside their own code, can't be wrapped at the call site.

Everything inside the scope shares the same lifetime, so colocate `"use cache: private"` as close to the runtime data access as possible.

## When to reach for runtime prefetching

Use it on routes where:

- A useful chunk of the page depends on request data: cookies, headers, the full URL, `searchParams`, or `params` not resolved by [`generateStaticParams`](/docs/app/api-reference/functions/generate-static-params)
- That chunk has a known cache lifetime (it can be expressed with `"use cache"` or `"use cache: private"`)
- The traffic justifies the per-link server invocation

Skip it when the prefetch can't produce a better UI than the App Shell. Each visible `` to a route with `'allow-runtime'` wakes a server, and that cost only pays off if more of the page is ready before the click:

- The route has little or no runtime-data dependency. The App Shell already gets you instant.
- The dependent content has to be fresh on every request. The prerender stops at the same `` fallback, so the user sees the same UI either way.
- The route is rarely navigated to. You pay per visible link, regardless of click-through.

## App Shells

A per-link runtime prefetch only helps the navigations where it fires before the click and completes before the click. On a slow connection, on a feed of many links, or on a direct visit, the per-link prefetch may not yet exist when the user navigates. Without something to fall back on, the navigation blocks until the server responds.

The [**App Shell**](/docs/app/glossary#app-shell) closes that gap. It's a per-route prerender, deduped across every link to the same route, generated and prefetched once per route rather than once per visible link.

App Shells are on by default with Cache Components. Pairing with `partialPrefetching: true` makes them the prefetch baseline for every route:

```ts filename="next.config.ts" highlight=
import type  from 'next'

const nextConfig: NextConfig = 

export default nextConfig
```

N links to the same route share one prefetched App Shell, so rendering a `` is effectively free. To prefetch more than the shell (including request data like cookies, headers, params, and `searchParams`), opt the destination into [`prefetch = 'allow-runtime'`](/docs/app/api-reference/file-conventions/route-segment-config/prefetch#allow-runtime) and use `` on the link side.

> **Good to know**: Routes that read `cookies()` or `headers()` produce an App Shell that includes session data. The framework auto-detects this, and the shell is cached per session on the client, not shared across users.

App Shells vs runtime prefetches:

|         | App Shell                                    | Per-link runtime prefetch (`allow-runtime`) |
| ------- | -------------------------------------------- | ------------------------------------------- |
| Scope   | One per route                                | One per visible ``    |
| Content | Route's rendered output minus per-link data  | Same, plus request data resolved            |
| Cost    | Bounded by route count                       | Bounded by visible-link count               |
| Role    | Every Cache Components route's instant floor | Upgrade: more rendered before click         |

## Next steps

- [Adopting Partial Prefetching](/docs/app/guides/adopting-partial-prefetching) for how `` behaves under the new model and how to migrate existing apps.
- [`prefetch` API reference](/docs/app/api-reference/file-conventions/route-segment-config/prefetch) for all prefetch modes.
- [`use cache: private` reference](/docs/app/api-reference/directives/use-cache-private) for per-user caching specifics.
- [Instant navigation guide](/docs/app/guides/instant-navigation) for validating the route's caching structure.
- [Caching](/docs/app/getting-started/caching) for background on `use cache`, Suspense, and Partial Prerendering.
