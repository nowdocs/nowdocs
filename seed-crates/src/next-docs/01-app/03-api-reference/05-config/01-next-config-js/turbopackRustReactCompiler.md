---
title: turbopackRustReactCompiler
description: Use the native Rust port of the React Compiler with Turbopack instead of the Babel transform.
version: experimental
---

The `experimental.turbopackRustReactCompiler` option enables the native Rust version of the [React Compiler](/docs/app/api-reference/config/next-config-js/reactCompiler), running it directly inside Turbopack as native code instead of through Node.js as it does with the standard Babel version. This typically results in a noticeable performance improvement.

This option is released as experimental to gather feedback before it becomes the default.

```ts filename="next.config.ts" switcher
import type  from 'next'

const nextConfig: NextConfig = {
  // Enable the React Compiler
  reactCompiler: true,
  experimental: ,
}

export default nextConfig
```

```js filename="next.config.js" switcher
/** @type  */
const nextConfig = {
  // Enable the React Compiler
  reactCompiler: true,
  experimental: ,
}

module.exports = nextConfig
```

## Good to know

> - This option requires [`reactCompiler`](/docs/app/api-reference/config/next-config-js/reactCompiler) to be enabled. It selects which implementation runs, but does not turn the compiler on by itself.
> - This option is only supported with Turbopack. Using it with webpack will throw an error.
> - When enabled, you do not need to install `babel-plugin-react-compiler`. The Rust compiler runs natively inside Turbopack.

See the [`reactCompiler` option documentation](/docs/app/api-reference/config/next-config-js/reactCompiler) for details on how to use the compiler.

## Version History

| Version   | Changes                                                                                             |
| --------- | --------------------------------------------------------------------------------------------------- |
| `v16.3.0` | Introduced the experimental `turbopackRustReactCompiler` option for the native Rust React Compiler. |
