---
title: mdxRs
description: Use the new Rust compiler to compile MDX files in the App Router.
version: experimental
---

For experimental use with `@next/mdx`. Compiles MDX files using the new Rust compiler.

```js filename="next.config.js"
const withMDX = require('@next/mdx')()

/** @type  */
const nextConfig = {
  pageExtensions: ['ts', 'tsx', 'mdx'],
  experimental: ,
}

module.exports = withMDX(nextConfig)
```
