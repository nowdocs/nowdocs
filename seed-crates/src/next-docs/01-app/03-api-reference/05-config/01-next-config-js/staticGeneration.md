---
title: staticGeneration*
description: Learn how to configure static generation in your Next.js application.
version: experimental
---

The `staticGeneration*` options allow you to configure the Static Generation process for advanced use cases.

```ts filename="next.config.ts" switcher
import type  from 'next'

const nextConfig: NextConfig = {
  experimental: ,
}

export default nextConfig
```

```js filename="next.config.js" switcher
const nextConfig = {
  experimental: ,
}

export default nextConfig
```

## Config Options

The following options are available:

- `staticGenerationRetryCount`: The number of times to retry a failed page generation before failing the build.
- `staticGenerationMaxConcurrency`: The maximum number of pages to be processed per worker.
- `staticGenerationMinPagesPerWorker`: The minimum number of pages to be processed before starting a new worker.
