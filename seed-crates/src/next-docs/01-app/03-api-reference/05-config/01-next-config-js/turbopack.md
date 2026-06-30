---
title: turbopack
description: Configure Next.js with Turbopack-specific options
---

The `turbopack` option lets you customize [Turbopack](/docs/app/api-reference/turbopack) to transform different files and change how modules are resolved.

> **Good to know**: The `turbopack` option was previously named `experimental.turbo` in Next.js versions 13.0.0 to 15.2.x. The `experimental.turbo` option will be removed in Next.js 16.
>
> If you are using an older version of Next.js, run `npx @next/codemod@latest next-experimental-turbo-to-turbopack .` to automatically migrate your configuration.

```ts filename="next.config.ts" switcher
import type  from 'next'

const nextConfig: NextConfig = {
  turbopack: ,
}

export default nextConfig
```

```js filename="next.config.js" switcher
/** @type  */
const nextConfig = {
  turbopack: ,
}

module.exports = nextConfig
```

> **Good to know**:
>
> - Turbopack for Next.js does not require loaders or loader configuration for built-in functionality. Turbopack has built-in support for CSS and compiling modern JavaScript, so there's no need for `css-loader`, `postcss-loader`, or `babel-loader` if you're using `@babel/preset-env`.

## Reference

### Options

The following options are available for the `turbopack` configuration:

| Option              | Description                                                                                                                              |
| ------------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| `root`              | Sets the application root directory. Should be an absolute path.                                                                         |
| `rules`             | List of supported webpack loaders to apply when running with Turbopack.                                                                  |
| `resolveAlias`      | Map aliased imports to modules to load in their place.                                                                                   |
| `resolveExtensions` | List of extensions to resolve when importing files.                                                                                      |
| `debugIds`          | Enable generation of [debug IDs](https://github.com/tc39/ecma426/blob/main/proposals/debug-id.md) in JavaScript bundles and source maps. |

### Supported loaders

The following loaders have been tested to work with Turbopack's webpack loader implementation, but many other webpack loaders should work as well even if not listed here:

- [`babel-loader`](https://www.npmjs.com/package/babel-loader) [_(Configured automatically if a Babel configuration file is found)_](/docs/app/api-reference/turbopack#language-features)
- [`@svgr/webpack`](https://www.npmjs.com/package/@svgr/webpack)
- [`svg-inline-loader`](https://www.npmjs.com/package/svg-inline-loader)
- [`yaml-loader`](https://www.npmjs.com/package/yaml-loader)
- [`string-replace-loader`](https://www.npmjs.com/package/string-replace-loader)
- [`raw-loader`](https://www.npmjs.com/package/raw-loader)
- [`sass-loader`](https://www.npmjs.com/package/sass-loader) [_(Configured automatically)_](/docs/app/api-reference/turbopack#css-and-styling)
- [`graphql-tag/loader`](https://www.npmjs.com/package/graphql-tag)

#### Missing Webpack loader features

Turbopack uses the [`loader-runner`](https://github.com/webpack/loader-runner) library to execute webpack loaders, which provides most of the standard loader API. However, some features are not supported:

**Module loading:**

- [`importModule`](https://webpack.js.org/api/loaders/#thisimportmodule) - No support
- [`loadModule`](https://webpack.js.org/api/loaders/#thisloadmodule) - No support

**File system and output:**

- [`fs`](https://webpack.js.org/api/loaders/#thisfs) - Partial support: only `fs.readFile` is currently implemented.
- [`emitFile`](https://webpack.js.org/api/loaders/#thisemitfile) - No support

**Context properties:**

- [`version`](https://webpack.js.org/api/loaders/#thisversion) - No support
- [`mode`](https://webpack.js.org/api/loaders/#thismode) - No support
- [`target`](https://webpack.js.org/api/loaders/#thistarget) - No support

**Utilities:**

- [`utils`](https://webpack.js.org/api/loaders/#thisutils) - No support
- [`resolve`](https://webpack.js.org/api/loaders/#thisresolve) - No support (use [`getResolve`](https://webpack.js.org/api/loaders/#thisgetresolve) instead)

If you have a loader that is critically dependent upon one of these features please file an issue.

## Examples

### Root directory

Turbopack uses the root directory to resolve modules. Files outside of the project root are not resolved.

The reason files are not resolved outside of the project root is to improve cache validation, reduce filesystem watching overhead, and reduce the number of resolving steps needed.

Next.js automatically detects the root directory of your project. It does so by looking for one of these files:

- `pnpm-lock.yaml`
- `package-lock.json`
- `yarn.lock`
- `bun.lock`
- `bun.lockb`

If you have a different project structure, for example if you don't use workspaces, you can manually set the `root` option:

```js filename="next.config.js"
const path = require('path')
module.exports = {
  turbopack: ,
}
```

To resolve files from linked dependencies outside the project root (via `npm link`, `yarn link`, `pnpm link`, etc.), you must configure the `turbopack.root` to the parent directory of both the project and the linked dependencies.

While this expands the scope of filesystem watching, it's typically only necessary during development when actively working on linked packages.

### Configuring webpack loaders

If you need loader support beyond what's built in, many webpack loaders already work with Turbopack. There are currently some limitations:

- Only a core subset of the webpack loader API is implemented. Currently, there is enough coverage for some popular loaders, and we'll expand our API support in the future.
- Only loaders that return JavaScript code are supported. Loaders that transform files like stylesheets or images are not currently supported.
- Options passed to webpack loaders must be plain JavaScript primitives, objects, and arrays. For example, it's not possible to pass `require()` plugin modules as option values.

To configure loaders, add the names of the loaders you've installed and any options in `next.config.js`, mapping file extensions to a list of loaders. Rules are evaluated in order.

Here is an example below using the [`@svgr/webpack`](https://www.npmjs.com/package/@svgr/webpack) loader, which enables importing `.svg` files and rendering them as React components.

```js filename="next.config.js"
module.exports = {
  turbopack: {
    rules: {
      '*.svg': ,
    },
  },
}
```

> **Good to know**: Globs used in the `rules` object match based on file name, unless the glob contains a `/` character, which will cause it to match based on the full project-relative file path. Windows file paths are normalized to use unix-style `/` path separators.
>
> Turbopack uses a modified version of the [Rust `globset` library](https://docs.rs/globset/latest/globset/).

For loaders that require configuration options, you can use an object format instead of a string:

```js filename="next.config.js"
module.exports = {
  turbopack: {
    rules: {
      '*.svg': {
        loaders: [
          {
            loader: '@svgr/webpack',
            options: ,
          },
        ],
        as: '*.js',
      },
    },
  },
}
```

> **Good to know**: Prior to Next.js version 13.4.4, `turbopack.rules` was named `turbo.loaders` and only accepted file extensions like `.mdx` instead of `*.mdx`.

### Advanced webpack loader conditions

You can further restrict where a loader runs using the advanced `condition` syntax:

```js filename="next.config.js"
module.exports = {
  turbopack: {
    rules: {
      // '*' will match all file paths, but we restrict where our
      // rule runs with a condition.
      '*': {
        condition: {
          all: [
            // 'foreign' is a built-in condition.
            ,
            // 'path' can be a RegExp or a glob string. A RegExp matches
            // anywhere in the full project-relative file path.
            { path: /^img\/[0-9]\// },
            {
              any: [
                ,
                // 'query' matches anywhere in the full query string,
                // which can be empty, or start with `?`.
                ,
                // 'content' is always a RegExp, and can match
                // anywhere in the file.
                { content: /\ **Good to know**: All matching rules are executed in order.

### Module types

You can set the module type directly without using a loader. This is useful for changing how files are processed, similar to webpack's [`type`](https://webpack.js.org/configuration/module/#ruletype) option.

```js filename="next.config.js"
module.exports = {
  turbopack: {
    rules: {
      '*.svg': ,
    },
  },
}
```

When using `type: 'asset'`, importing the file returns its URL:

```tsx filename="app/page.tsx"
import svgUrl from './icon.svg'

export default function Page() 
```

The `type` option can be combined with `loaders` - loaders run first, then the result is processed according to the specified type.

Available module types:

| Type         | Description                                              |
| ------------ | -------------------------------------------------------- |
| `asset`      | Emit file and return URL (like webpack `asset/resource`) |
| `ecmascript` | Process as JavaScript                                    |
| `typescript` | Process as TypeScript                                    |
| `css`        | Process as CSS                                           |
| `css-module` | Process as CSS module                                    |
| `wasm`       | Process as WebAssembly                                   |
| `raw`        | Return raw contents as string                            |
| `bytes`      | Inline contents as bytes                                 |

### Inline loader configuration with import attributes

You can apply a Turbopack loader to an individual import using the `with` clause (import attributes). This is specified per-import rather than globally via `turbopack.rules`.

This is useful when you want to apply a loader to a specific import without affecting all files of that type.

```tsx filename="app/page.tsx"
// Apply a raw loader to import a .txt file as a JavaScript module
import rawText from '../data.txt' with 

export default function Page() {
  return 
}
```

The following import attributes are supported:

| Attribute                | Description                                                                                                                   |
| ------------------------ | ----------------------------------------------------------------------------------------------------------------------------- |
| `turbopackLoader`        | The loader to apply (e.g. `'raw-loader'`).                                                                                    |
| `turbopackLoaderOptions` | JSON string of loader options (e.g. `''`).                                                        |
| `turbopackAs`            | Rename pattern for the output (same as `turbopack.rules[].as`). For example, `'*.js'` treats the loader output as JavaScript. |
| `turbopackModuleType`    | Set the module type for the output (same as `turbopack.rules[].type`).                                                        |

Loaders with options pass a JSON-encoded string via `turbopackLoaderOptions`:

```tsx filename="app/page.tsx"
import value from '../data.js' with { turbopackLoader: 'string-replace-loader', turbopackLoaderOptions: '' }
```

> **Good to know**: Import attributes with `turbopackLoader` are Turbopack-specific and are not supported by webpack. This feature requires the `with` keyword (not `assert`) in your import statements.

### Resolving aliases

Turbopack can be configured to modify module resolution through aliases, similar to webpack's [`resolve.alias`](https://webpack.js.org/configuration/resolve/#resolvealias) configuration.

To configure resolve aliases, map imported patterns to their new destination in `next.config.js`:

```js filename="next.config.js"
module.exports = {
  turbopack: {
    resolveAlias: {
      underscore: 'lodash',
      mocha: ,
    },
  },
}
```

This aliases imports of the `underscore` package to the `lodash` package. In other words, `import underscore from 'underscore'` will load the `lodash` module instead of `underscore`.

Turbopack also supports conditional aliasing through this field, similar to Node.js' [conditional exports](https://nodejs.org/docs/latest-v18.x/api/packages.html#conditional-exports). At the moment only the `browser` condition is supported. In the case above, imports of the `mocha` module will be aliased to `mocha/browser-entry.js` when Turbopack targets browser environments.

### Resolving custom extensions

Turbopack can be configured to resolve modules with custom extensions, similar to webpack's [`resolve.extensions`](https://webpack.js.org/configuration/resolve/#resolveextensions) configuration.

To configure resolve extensions, use the `resolveExtensions` field in `next.config.js`:

```js filename="next.config.js"
module.exports = {
  turbopack: ,
}
```

This overwrites the original resolve extensions with the provided list. Make sure to include the default extensions.

For more information and guidance for how to migrate your app to Turbopack from webpack, see [Turbopack's documentation on webpack compatibility](https://turbo.build/pack/docs/migrating-from-webpack).

### Debug IDs

Turbopack can be configured to generate [debug IDs](https://github.com/tc39/ecma426/blob/main/proposals/debug-id.md) in JavaScript bundles and source maps.

To configure debug IDs, use the `debugIds` field in `next.config.js`:

```js filename="next.config.js"
module.exports = {
  turbopack: ,
}
```

The option automatically adds a polyfill for debug IDs to the JavaScript bundle to ensure compatibility. The debug IDs are available in the `globalThis._debugIds` global variable.

## Version History

| Version  | Changes                                              |
| -------- | ---------------------------------------------------- |
| `16.2.0` | `turbopackLoader` import attributes were added.      |
| `16.2.0` | `turbopack.rules.*.type` was added.                  |
| `16.2.0` | `turbopack.rules.*.condition.contentType` was added. |
| `16.2.0` | `turbopack.rules.*.condition.query` was added.       |
| `16.0.0` | `turbopack.debugIds` was added.                      |
| `16.0.0` | `turbopack.rules.*.condition` was added.             |
| `15.3.0` | `experimental.turbo` is changed to `turbopack`.      |
| `13.0.0` | `experimental.turbo` introduced.                     |
