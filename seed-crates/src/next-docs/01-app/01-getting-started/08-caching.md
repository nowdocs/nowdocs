---
title: Caching
description: Learn how to cache data and UI in Next.js
related:
  title: Next Steps
  description: Learn more about revalidation and the APIs mentioned on this page.
  links:
    - app/getting-started/revalidating
    - app/api-reference/directives/use-cache
    - app/api-reference/config/next-config-js/cacheComponents
    - app/guides/instant-navigation
---

> This page covers caching with [Cache Components](/docs/app/api-reference/config/next-config-js/cacheComponents), enabled by setting [`cacheComponents: true`](/docs/app/api-reference/config/next-config-js/cacheComponents) in your `next.config.ts` file. If you're not using Cache Components, see the [Caching and Revalidating (Previous Model)](/docs/app/guides/caching-without-cache-components) guide.

Caching is a technique for storing the result of data fetching and other computations so that future requests for the same data can be served faster, without doing the work again.

## Enabling Cache Components

You can enable Cache Components by adding the [`cacheComponents`](/docs/app/api-reference/config/next-config-js/cacheComponents) option to your Next config file:

```ts filename="next.config.ts" switcher
import type  from 'next'

const nextConfig: NextConfig = 

export default nextConfig
```

```js filename="next.config.js" switcher
/** @type  */
const nextConfig = 

module.exports = nextConfig
```

> **Good to know:** When Cache Components is enabled, `GET` Route Handlers follow the same prerendering model as pages. See [Route Handlers with Cache Components](/docs/app/getting-started/route-handlers#with-cache-components) for details.

## Usage

The [`use cache`](/docs/app/api-reference/directives/use-cache) directive caches the return value of async functions and components. You can apply it at two levels:

- **Data-level**: Cache a function that fetches or computes data (e.g., `getProducts()`, `getUser(id)`)
- **UI-level**: Cache an entire component or page (e.g., `async function BlogPosts()`)

Two variants cover specific scenarios: [`"use cache: remote"`](/docs/app/api-reference/directives/use-cache-remote) for durable, shared remote storage, and [`"use cache: private"`](/docs/app/api-reference/directives/use-cache-private) for caching functions that read runtime data.

> Arguments and any closed-over values from parent scopes automatically become part of the [cache key](/docs/app/api-reference/directives/use-cache#cache-keys), which means different inputs will produce separate cache entries. This enables personalized or parameterized cached content. See [serialization requirements and constraints](/docs/app/api-reference/directives/use-cache#constraints) for details on what can be cached and how arguments work.

### Data-level caching

To cache an asynchronous function that fetches data, add the `use cache` directive at the top of the function body:

```tsx filename="app/lib/data.ts" highlight=
import  from 'next/cache'

export async function getUsers() 
```

Data-level caching is useful when the same data is used across multiple components, or when you want to cache the data independently from the UI.

### UI-level caching

To cache an entire component, page, or layout, add the `use cache` directive at the top of the component or page body:

```tsx filename="app/page.tsx" highlight=
import  from 'next/cache'

export default async function Page() {
  'use cache'
  cacheLife('hours')

  const users = await db.query('SELECT * FROM users')

  return (
    
      {users.map((user) => (
        
      ))}
    
  )
}
```

> If you add "`use cache`" at the top of a file, all exported functions in the file will be cached.

### Streaming uncached data

For components that fetch data from an asynchronous source such as an API, a database, or any other async operation, and require fresh data on every request, do not use `"use cache"`.

Instead, wrap the component in [`
    </>
  )
}
```

For example, `Loading posts...` is included in the static shell, and the posts stream in at request time.

Without a `', highlight: true },
    ]}
  />

> **Good to know:** Each fix card links to a detailed walkthrough with patterns, code samples, and trade-offs. Click a card to dive in.

`
    </>
  )
}
```

A runtime API access without `', highlight: true },
    ]}
  />

### Passing runtime values to cached functions

You can extract values from runtime APIs and pass them as arguments to cached functions:

```tsx filename="app/profile/page.tsx"
import  from 'next/headers'
import  from 'react'

export default function Page() 

// Component (not cached) reads runtime data
async function ProfileContent() {
  const session = (await cookies()).get('session')?.value
  return  | 
        
      

      
      
    </>
  )
}

type Post = 

// Everyone sees the same blog posts (revalidated every hour)
async function BlogPosts() {
  'use cache'
  cacheLife('hours')
  cacheTag('posts')

  const res = await fetch('https://api.vercel.app/blog')
  const posts: Post[] = await res.json()

  return (
    
      Latest Posts
      
        {posts.map((post) => (
          
            
            
              By  on 
            
          
        ))}
      
    
  )
}

// UI that depends on a value stored in cookies
async function UserPreferences() {
  const theme = (await cookies()).get('theme')?.value || 'light'
  const favoriteCategory = (await cookies()).get('category')?.value

  return (
    
      Your theme: 
      {favoriteCategory && Favorite category: }
    
  )
}
```

During prerendering, the header (static) and blog posts (cached with `use cache`) become part of the static shell, along with the fallback UI for user preferences. The UI preferences stored in cookies stream in at request time.

Reading `cookies()` here doesn't opt-in the whole route into dynamic rendering, the way the previous rendering model did. The Suspense boundary provides fallback UI where the runtime access streams, while static and cached content still ship in the initial HTML.

Just as `
  )
}
```

Alternatively, you can **cache the result** so all users see the same value until revalidation:

```tsx filename="page.tsx"
export default async function Page() {
  'use cache'
  const buildId = crypto.randomUUID()
  return Build ID: 
}
```

You don't need to memorize which operations behave this way. The dev overlay surfaces a **blocking-prerender-random**, **blocking-prerender-current-time**, or **blocking-prerender-crypto** insight (depending on the call) with these fixes:

## Synchronous I/O and pure computations

Unlike random or time-based APIs, synchronous I/O, module imports, and pure computations are predictable: the same inputs produce the same outputs. Components using only these operations are prerendered automatically, and their output becomes part of the static HTML at build time.

```tsx filename="page.tsx"
import fs from 'node:fs'

export default async function Page() {
  const constants = await import('./constants.json')
  const content = fs.readFileSync('./config.json', 'utf-8')
  const items = JSON.parse(content).items ?? []

  return (
    
      
      
        {items.map((item) => (
          
        ))}
      
    
  )
}
```

> **Good to know:** This includes queries to embedded databases with synchronous APIs, such as `better-sqlite3` or Node.js's built-in [`node:sqlite`](https://nodejs.org/api/sqlite.html). If you need per-request data from a synchronous source, call [`connection()`](/docs/app/api-reference/functions/connection) before the query.

## Prerendering

At build time, Next.js renders your route's component tree. How each component is handled depends on the APIs it uses:

- [`use cache`](#usage): the result is cached and included in the static shell
- [`
      
    
  )
}

function SlugHeading(: ) {
  return 
}
```

Now `
  )
}

async function Stats() {
  const session = (await cookies()).get('session')?.value
  if (!session) return Not signed in
  const stats = await getStats(session)
  return 
}

async function getStats(session: string) 
```

On a direct visit, `` streams in behind the fallback. When a user navigates to `/dashboard` from another route, the framework prefetches with their session cookie. `getStats` is cached, so its result joins the runtime prerender before the click.

See the [Runtime prefetching guide](/docs/app/guides/runtime-prefetching) for full patterns and the [`prefetch` reference](/docs/app/api-reference/file-conventions/route-segment-config/prefetch) for all modes.

