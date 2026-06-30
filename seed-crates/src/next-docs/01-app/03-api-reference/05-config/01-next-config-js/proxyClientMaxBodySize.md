---
title: proxyClientMaxBodySize
description: Configure the maximum request body size when using proxy.
version: experimental
---

When proxy is used, Next.js automatically clones the request body and buffers it in memory to enable multiple reads - both in proxy and the underlying route handler. To prevent excessive memory usage, this configuration option sets a size limit on the buffered body.

By default, the maximum body size is **10MB**. If a request body exceeds this limit, the body will only be buffered up to the limit, and a warning will be logged indicating which route exceeded the limit.

## Options

### String format (recommended)

Specify the size using a human-readable string format:

```ts filename="next.config.ts" switcher
import type  from 'next'

const nextConfig: NextConfig = {
  experimental: ,
}

export default nextConfig
```

```js filename="next.config.js" switcher
/** @type  */
const nextConfig = {
  experimental: ,
}

module.exports = nextConfig
```

Supported units: `b`, `kb`, `mb`, `gb`

### Number format

Alternatively, specify the size in bytes as a number:

```ts filename="next.config.ts" switcher
import type  from 'next'

const nextConfig: NextConfig = {
  experimental: ,
}

export default nextConfig
```

```js filename="next.config.js" switcher
/** @type  */
const nextConfig = {
  experimental: ,
}

module.exports = nextConfig
```

## Behavior

When a request body exceeds the configured limit:

1. Next.js will buffer only the first N bytes (up to the limit)
2. A warning will be logged to the console indicating the route that exceeded the limit
3. The request will continue processing normally, but only the partial body will be available
4. The request will **not** fail or return an error to the client

If your application needs to process the full request body, you should either:

- Increase the `proxyClientMaxBodySize` limit
- Handle the partial body gracefully in your application logic

## Example

```ts filename="proxy.ts"
import  from 'next/server'

export async function proxy(request: NextRequest) 
```

```ts filename="app/api/upload/route.ts"
import  from 'next/server'

export async function POST(request: NextRequest) {
  // ...and the body is still available in your route handler
  const body = await request.text()

  console.log('Body in route handler:', body.length)

  return NextResponse.json()
}
```

## Good to know

- This setting only applies when proxy is used in your application
- The default limit of 10MB is designed to balance memory usage and typical use cases
- The limit applies per-request, not globally across all concurrent requests
- For applications handling large file uploads, consider increasing the limit accordingly
