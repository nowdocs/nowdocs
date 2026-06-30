---
title: use cache
description: Learn how to use the "use cache" directive to cache data in your Next.js application.
related:
  title: Related
  description: View related API references.
  links:
    - app/api-reference/directives/use-cache-private
    - app/api-reference/config/next-config-js/cacheComponents
    - app/api-reference/config/next-config-js/cacheLife
    - app/api-reference/config/next-config-js/cacheHandlers
    - app/api-reference/functions/cacheTag
    - app/api-reference/functions/cacheLife
    - app/api-reference/functions/revalidateTag
---

The `use cache` directive allows you to mark a route, React component, or a function as cacheable. It can be used at the top of a file to indicate that all exports in the file should be cached, or inline at the top of a function or component to cache the return value. Functions and components that use `use cache` must be async.

> **Good to know:**
>
> - To use cookies or headers, read them outside cached scopes and pass values as arguments. This is the preferred pattern.
> - If the in-memory cache isn't sufficient for runtime data, [`'use cache: remote'`](/docs/app/api-reference/directives/use-cache-remote) allows platforms to provide a dedicated cache handler, though it requires a network roundtrip to check the cache and typically incurs platform fees.
> - For compliance requirements or when you can't refactor to pass runtime data as arguments to a `use cache` scope, see [`'use cache: private'`](/docs/app/api-reference/directives/use-cache-private).

## Usage

`use cache` is a Cache Components feature. To enable it, add the [`cacheComponents`](/docs/app/api-reference/config/next-config-js/cacheComponents) option to your `next.config.ts` file:

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

Then, add `use cache` at the file, component, or function level. All functions and components using `use cache` must be async. When used at file level, every exported function becomes a cached function and must also be async:

```tsx
// File level
'use cache'

export default async function Page() 

// Component level
export async function MyComponent() 

// Function level
export async function getData() 
```

## How `use cache` works

### Cache keys

A cache entry's key is generated using a serialized version of its inputs, which includes:

1. **Build ID** - Unique per build, changing this invalidates all cache entries. If [`deploymentId`](/docs/app/api-reference/config/next-config-js/deploymentId) is configured, it overrides the build ID for cache key purposes.
2. **Function ID** - A secure hash of the function's location and signature in the codebase
3. **Serializable arguments** - Props (for components) or function arguments
4. **HMR refresh hash** (development only) - Invalidates cache on hot module replacement

When a cached function references variables from outer scopes, those variables are automatically captured and bound as arguments, making them part of the cache key.

```tsx filename="lib/data.ts"
async function Component(: ) {
  const getData = async (filter: string) => {
    'use cache'
    // Cache key includes both userId (from closure) and filter (argument)
    return fetch(`/api/users/$/data?filter=$`)
  }

  return getData('active')
}
```

In the snippet above, `userId` is captured from the outer scope and `filter` is passed as an argument, so both become part of the `getData` function's cache key. This means different user and filter combinations will have separate cache entries.

> **Good to know:** When a cached function reads [root parameters](/docs/app/api-reference/functions/next-root-params), only the ones it actually reads become part of its cache key.

## Serialization

Arguments to cached functions and their return values must be serializable.

For a complete reference, see:

- [Serializable arguments](https://react.dev/reference/rsc/use-server#serializable-parameters-and-return-values) - Uses **React Server Components** serialization
- [Serializable return types](https://react.dev/reference/rsc/use-client#serializable-types) - Uses **React Client Components** serialization

> **Good to know:** Arguments and return values use different serialization systems. Server Component serialization (for arguments) is more restrictive than Client Component serialization (for return values). This means you can return JSX elements but cannot accept them as arguments unless using pass-through patterns.

### Supported types

**Arguments:**

- Primitives: `string`, `number`, `boolean`, `null`, `undefined`
- Plain objects: ``
- Arrays: `[1, 2, 3]`
- Dates, Maps, Sets, TypedArrays, ArrayBuffers
- React elements (as pass-through only)

**Return values:**

- Same as arguments, plus JSX elements

### Unsupported types

- Class instances
- Functions (except as pass-through)
- Symbols, WeakMaps, WeakSets
- URL instances

```tsx filename="app/components/user-card.tsx"
// Valid - primitives and plain objects
async function UserCard(: {
  id: string
  config: 
}) {
  'use cache'
  return 
}

// Invalid - class instance
async function UserProfile(: ) {
  'use cache'
  // Error: Cannot serialize class instance
  return 
}
```

### Pass-through (non-serializable arguments)

You can accept non-serializable values **as long as you don't introspect them**. This enables composition patterns with `children` and Server Actions:

```tsx filename="app/components/cached-wrapper.tsx"
async function CachedWrapper(: ) {
  'use cache'
  // Don't read or modify children - just pass it through
  return (
    
      Cached Header
      
    
  )
}

// Usage: children can be dynamic
export default function Page() 
```

You can also pass Server Actions through cached components:

```tsx filename="app/components/cached-form.tsx"
async function CachedForm(: ) {
  'use cache'
  // Don't call action here - just pass it through
  return 
}
```

## Constraints

Cached functions execute in an isolated environment. The following constraints ensure cache behavior remains predictable and secure.

### Request-time APIs

Cached functions and components **cannot** directly access runtime APIs like `cookies()`, `headers()`, or `searchParams`. Instead, read these values outside the cached scope and pass them as arguments.

### Runtime caching considerations

While `use cache` is designed primarily to include uncached data in the static shell, it can also cache data at runtime using in-memory LRU (Least Recently Used) storage.

With the default in-memory handler, runtime cache behavior depends on your hosting environment:

| Environment     | Runtime Caching Behavior                                                                                                                                          |
| --------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Serverless**  | Cache entries typically don't persist across requests (each request can be a different instance), or during revalidation. Build-time caching works normally.      |
| **Self-hosted** | Cache entries persist across requests. Control cache size with [`cacheMaxMemorySize`](/docs/app/api-reference/config/next-config-js/incrementalCacheHandlerPath). |

For example, in a serverless environment, a cached function shared by two pages executes on each static shell revalidation, whereas in self-hosted or environments with persistent memory, the cached output is reused if it's still fresh.

If the default in-memory cache isn't enough, consider **[`use cache: remote`](/docs/app/api-reference/directives/use-cache-remote)** which allows platforms to provide a dedicated cache handler (like Redis or KV database). This helps reduce hits against data sources not scaled to your total traffic, though it comes with costs (storage, network latency, platform fees).

> [!NOTE]
> With the default in-memory handler, serverless instances are ephemeral, so entries may not be reused between requests, unlike with `use cache: remote`. Neither caching directive carries over to a new deploy, because the [cache key](#cache-keys) includes the build (or `deploymentId`) ID.
>
> For data that needs to persist across deploys, use [`unstable_cache`](/docs/app/api-reference/functions/unstable_cache) for non-`fetch` functions or the [`fetch`](/docs/app/api-reference/functions/fetch) cache.

Very rarely, for compliance requirements or when you can't refactor your code to pass runtime data as arguments to a `use cache` scope, you might need [`use cache: private`](/docs/app/api-reference/directives/use-cache-private).

### Draft Mode

When [Draft Mode](/docs/app/guides/draft-mode) is enabled, all cached functions and components re-execute on every request, and results are not saved to the cache. This ensures draft content is always fresh without requiring any changes to your caching code.

You can read `isEnabled` from [`draftMode()`](/docs/app/api-reference/functions/draft-mode) inside a `use cache` scope, however, other runtime APIs like `cookies()` and `headers()` are not allowed, even when Draft Mode is active. See [Passing runtime values to cached functions](/docs/app/getting-started/caching#passing-runtime-values-to-cached-functions) for the recommended pattern.

```tsx filename="app/components/content.tsx"
import  from 'next/headers'

async function Content() {
  'use cache'

  const  = await draftMode()
  const url = isEnabled
    ? 'https://draft.example.com/content'
    : 'https://production.example.com/content'

  const data = await fetch(url)
  return 
}
```

Calling `enable()` or `disable()` inside a caching directive scope will also throw an error. Draft Mode can only be toggled in [Route Handlers](/docs/app/api-reference/file-conventions/route) or [Server Actions](/docs/app/getting-started/mutating-data).

### React.cache isolation

[`React.cache`](https://react.dev/reference/react/cache) operates in an isolated scope inside `use cache` boundaries. Values stored via `React.cache` outside a `use cache` function are not visible inside it.

This means you cannot use `React.cache` to pass data into a `use cache` scope:

```tsx
import  from 'react'

const store = cache(() => ())

function Parent() 

async function CacheComponent(: ) {
  'use cache'
  const cachedData = await fetch('/api/cached-data')
  return (
    
      
      
  )
}

async function CacheComponent() {
  'use cache'
  const cachedData = await fetch('/api/cached-data')
  return (
    
      
      
  )
}

async function Dynamic() 

async function Dynamic(: ) {
  // Stores dynamic Promise in shared Map
  cache.set(
    id,
    fetch(`https://api.example.com/$`).then((r) => r.text())
  )
  return Dynamic
}

async function Cached(: ) {
  'use cache'
  return  // Build hangs - retrieves dynamic Promise
}
```

Use Next.js's built-in `fetch()` deduplication or use separate Maps for cached and uncached contexts.

## Platform Support

| Deployment Option                                                   | Supported         |
| ------------------------------------------------------------------- | ----------------- |
| [Node.js server](/docs/app/getting-started/deploying#nodejs-server) | Yes               |
| [Docker container](/docs/app/getting-started/deploying#docker)      | Yes               |
| [Static export](/docs/app/getting-started/deploying#static-export)  | No                |
| [Adapters](/docs/app/getting-started/deploying#adapters)            | Platform-specific |

Learn how to [configure caching](/docs/app/guides/self-hosting#caching-and-isr) when self-hosting Next.js.

## Version History

| Version   | Changes                                                     |
| --------- | ----------------------------------------------------------- |
| `v16.0.0` | `"use cache"` is enabled with the Cache Components feature. |
| `v15.0.0` | `"use cache"` is introduced as an experimental feature.     |
