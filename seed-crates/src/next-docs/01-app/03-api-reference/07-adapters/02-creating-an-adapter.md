---
title: Creating an Adapter
description: Create an adapter module that implements the `NextAdapter` interface.
---

An adapter is a module that exports an object implementing the `NextAdapter` interface.

The interface can be imported from the `next` package:

```typescript
import type  from 'next'
```

The interface is defined as follows:

```typescript
type Route = 

export interface AdapterOutputs 

export interface NextAdapter {
  name: string
  modifyConfig?: (
    config: NextConfigComplete,
    ctx: 
  ) => Promise | NextConfigComplete
  onBuildComplete?: (ctx: {
    routing: 
    outputs: AdapterOutputs
    projectDir: string
    repoRoot: string
    distDir: string
    config: NextConfigComplete
    nextVersion: string
    buildId: string
  }) => Promise | void
}
```

## Basic Adapter Structure

Here's a minimal adapter example:

```js filename="my-adapter.js"
/** @type  */
const adapter = {
  name: 'my-custom-adapter',

  async modifyConfig(config, ) {
    // Modify the Next.js config based on the build phase
    if (phase === 'phase-production-build') {
      return 
    }
    return config
  },

  async onBuildComplete() {
    // Process the build output
    console.log('Build completed with', outputs.pages.length, 'pages')
    console.log('Build ID:', buildId)
    console.log('Dynamic routes:', routing.dynamicRoutes.length)

    // Access emitted output entries
    for (const page of outputs.pages) 

    for (const apiRoute of outputs.pagesApi) 

    for (const appPage of outputs.appPages) 

    for (const prerender of outputs.prerenders) 
  },
}

module.exports = adapter
```
