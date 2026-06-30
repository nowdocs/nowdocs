---
title: sassOptions
description: Configure Sass options.
---

`sassOptions` allow you to configure the Sass compiler.

```ts filename="next.config.ts" switcher
import type  from 'next'

const sassOptions = 

const nextConfig: NextConfig = {
  sassOptions: ,
}

export default nextConfig
```

```js filename="next.config.js" switcher
/** @type  */

const sassOptions = 

const nextConfig = {
  sassOptions: ,
}

module.exports = nextConfig
```

> **Good to know:**
>
> - `sassOptions` are not typed outside of `implementation` because Next.js does not maintain the other possible properties.
> - The `functions` property for defining custom Sass functions is only supported with webpack. When using Turbopack, custom Sass functions are not available because Turbopack's Rust-based architecture cannot directly execute JavaScript functions passed through this option.
