---
title: Codemods
description: Use codemods to upgrade your Next.js codebase when new features are released.
---

Codemods are transformations that run on your codebase programmatically. This allows a large number of changes to be programmatically applied without having to manually go through every file.

Next.js provides Codemod transformations to help upgrade your Next.js codebase when an API is updated or deprecated.

## Usage

In your terminal, navigate (`cd`) into your project's folder, then run:

```bash filename="Terminal"
npx @next/codemod  
```

Replacing `` and `` with appropriate values.

- `transform` - name of transform
- `path` - files or directory to transform
- `--dry` Do a dry-run, no code will be edited
- `--print` Prints the changed output for comparison

## Upgrade

Upgrades your Next.js application, automatically running codemods and updating Next.js, React, and React DOM.

```bash filename="Terminal"
npx @next/codemod upgrade [revision]
```

### Options

- `revision` (optional): Specify the upgrade type (`patch`, `minor`, `major`), an NPM dist tag (e.g. `latest`, `canary`, `rc`), or an exact version (e.g. `15.0.0`). Defaults to `minor` for stable versions.
- `--verbose`: Show more detailed output during the upgrade process.
- `-y, --yes`: Skip every interactive prompt and accept its default (upgrade React past 18, enable Turbopack, apply all recommended codemods, run the React 19 codemods). Also auto-enabled when stdin is not a TTY (CI, an AI coding agent, or any non-interactive shell), so you usually don't need to pass it explicitly.

For example:

```bash filename="Terminal"
# Upgrade to the latest patch (e.g. 16.0.7 -> 16.0.8)
npx @next/codemod upgrade patch

# Upgrade to the latest minor (e.g. 15.3.7 -> 15.4.8). This is the default.
npx @next/codemod upgrade minor

# Upgrade to the latest major (e.g. 15.5.7 -> 16.0.7)
npx @next/codemod upgrade major

# Upgrade to a specific version
npx @next/codemod upgrade 16

# Upgrade to the canary release
npx @next/codemod upgrade canary

# Run from an agent or CI: skip every prompt
npx @next/codemod upgrade canary --yes
```

> **Good to know**:
>
> - If the target version is the same as or lower than your current version, the command exits without making changes.
> - During the upgrade, you may be prompted to choose which Next.js codemods to apply and run React 19 codemods if upgrading React.
> - When invoked by an AI coding agent or in CI (anywhere stdin isn't a TTY), the upgrade runs non-interactively and accepts every default. Pass `--yes` to force this behavior even from a terminal.

## Codemods

### 16.3

#### Opt every route out of Cache Components validation

##### `cache-components-instant-false`

```bash filename="Terminal"
npx @next/codemod@canary cache-components-instant-false ./app
```

This codemod adds `export const instant = false` to every `app/**/` file that doesn't already export `instant`, so you can enable [`cacheComponents`](/docs/app/api-reference/config/next-config-js/cacheComponents) and then remove the opt-outs route by route. It skips Client Components (`"use client"`) and files that already declare `instant`.

```diff filename="app/page.tsx"
+ // TODO: Cache Components adoption. Refactor this route so this opt-out can be removed.
+ // See: https://nextjs.org/docs/app/guides/migrating-to-cache-components
+ export const instant = false
+
  export default function Page() 
```

See the [Migrating to Cache Components](/docs/app/guides/migrating-to-cache-components) guide for the full adoption path.

### 16.0

#### Remove `experimental_ppr` Route Segment Config from App Router pages and layouts

##### `remove-experimental-ppr`

```bash filename="Terminal"
npx @next/codemod@latest remove-experimental-ppr .
```

This codemod removes the `experimental_ppr` Route Segment Config from App Router pages and layouts.

```diff filename="app/page.tsx"
- export const experimental_ppr = true;
```

#### Remove `unstable_` prefix from stabilized API

##### `remove-unstable-prefix`

```bash filename="Terminal"
npx @next/codemod@latest remove-unstable-prefix .
```

This codemod removes the `unstable_` prefix from stabilized API.

For example:

```ts
import  from 'next/cache'

cacheTag()
```

Transforms into:

```ts
import  from 'next/cache'

cacheTag()
```

#### Migrate from deprecated `middleware` convention to `proxy`

##### `middleware-to-proxy`

```bash filename="Terminal"
npx @next/codemod@latest middleware-to-proxy .
```

This codemod migrates projects from using the deprecated `middleware` convention to using the `proxy` convention. It:

- Renames `middleware.` to `proxy.` (e.g. `middleware.ts` to `proxy.ts`)
- Renames named export `middleware` to `proxy`
- Renames Next.js config property `experimental.middlewarePrefetch` to `experimental.proxyPrefetch`
- Renames Next.js config property `experimental.middlewareClientMaxBodySize` to `experimental.proxyClientMaxBodySize`
- Renames Next.js config property `experimental.externalMiddlewareRewritesResolve` to `experimental.externalProxyRewritesResolve`
- Renames Next.js config property `skipMiddlewareUrlNormalize` to `skipProxyUrlNormalize`

For example:

```ts filename="middleware.ts"
import  from 'next/server'

export function middleware() 
```

Transforms into:

```ts filename="proxy.ts"
import  from 'next/server'

export function proxy() 
```

#### Migrate from `next lint` to ESLint CLI

##### `next-lint-to-eslint-cli`

```bash filename="Terminal"
npx @next/codemod@canary next-lint-to-eslint-cli .
```

This codemod migrates projects from using `next lint` to using the ESLint CLI with your local ESLint config. It:

- Creates an `eslint.config.mjs` file with Next.js recommended configurations
- Updates `package.json` scripts to use `eslint .` instead of `next lint`
- Adds necessary ESLint dependencies to `package.json`
- Preserves existing ESLint configurations when found

For example:

```json filename="package.json"
{
  "scripts": 
}
```

Becomes:

```json filename="package.json"
{
  "scripts": 
}
```

And creates:

```js filename="eslint.config.mjs"
import  from 'path'
import  from 'url'
import  from '@eslint/eslintrc'

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)

const compat = new FlatCompat()

const eslintConfig = [
  ...compat.extends('next/core-web-vitals', 'next/typescript'),
  ,
]

export default eslintConfig
```

### 15.0

#### Transform App Router Route Segment Config `runtime` value from `experimental-edge` to `edge`

##### `app-dir-runtime-config-experimental-edge`

> **Note**: This codemod is App Router specific.

```bash filename="Terminal"
npx @next/codemod@latest app-dir-runtime-config-experimental-edge .
```

This codemod transforms [Route Segment Config `runtime`](https://nextjs.org/docs/app/api-reference/file-conventions/route-segment-config/runtime) value `experimental-edge` to `edge`.

For example:

```ts
export const runtime = 'experimental-edge'
```

Transforms into:

```ts
export const runtime = 'edge'
```

#### Migrate to async Dynamic APIs

APIs that opted into dynamic rendering that previously supported synchronous access are now asynchronous. You can read more about this breaking change in the [upgrade guide](/docs/app/guides/upgrading/version-15).

##### `next-async-request-api`

```bash filename="Terminal"
npx @next/codemod@latest next-async-request-api .
```

This codemod will transform dynamic APIs (`cookies()`, `headers()` and `draftMode()` from `next/headers`) that are now asynchronous to be properly awaited or wrapped with `React.use()` if applicable.
When an automatic migration isn't possible, the codemod will either add a typecast (if a TypeScript file) or a comment to inform the user that it needs to be manually reviewed & updated.

For example:

```tsx
import  from 'next/headers'
const token = cookies().get('token')

function useToken() 

export default function Page() 

function getHeader() 
```

Transforms into:

```tsx
import  from 'react'
import  from 'next/headers'
const token = (cookies() as unknown as UnsafeUnwrappedCookies).get('token')

function useToken() 

export default async function Page() 

function getHeader() 
```

When we detect property access on the `params` or `searchParams` props in the page / route entries (`page.js`, `layout.js`, `route.js`, or `default.js`) or the `generateMetadata` / `generateViewport` APIs,
it will attempt to transform the callsite from a sync to an async function, and await the property access. If it can't be made async (such as with a Client Component), it will use `React.use` to unwrap the promise .

For example:

```tsx
// page.tsx
export default function Page(: {
  params: 
  searchParams: 
}) {
  const  = searchParams
  if (value === 'foo') 
}

export function generateMetadata(: { params:  }) {
  const  = params
  return {
    title: `My Page - $`,
  }
}
```

Transforms into:

```tsx
// page.tsx
export default async function Page(props: {
  params: Promise<>
  searchParams: Promise<>
}) {
  const searchParams = await props.searchParams
  const  = searchParams
  if (value === 'foo') 
}

export async function generateMetadata(props: {
  params: Promise<>
}) {
  const params = await props.params
  const  = params
  return {
    title: `My Page - $`,
  }
}
```

> **Good to know:** When this codemod identifies a spot that might require manual intervention, but we aren't able to determine the exact fix, it will add a comment or typecast to the code to inform the user that it needs to be manually updated. These comments are prefixed with **@next/codemod**, and typecasts are prefixed with `UnsafeUnwrapped`.
> Your build will error until these comments are explicitly removed. [Read more](/docs/messages/sync-dynamic-apis).

#### Replace `geo` and `ip` properties of `NextRequest` with `@vercel/functions`

##### `next-request-geo-ip`

```bash filename="Terminal"
npx @next/codemod@latest next-request-geo-ip .
```

This codemod installs `@vercel/functions` and transforms `geo` and `ip` properties of `NextRequest` with corresponding `@vercel/functions` features.

For example:

```ts
import type  from 'next/server'

export function GET(req: NextRequest) {
  const  = req
}
```

Transforms into:

```ts
import type  from 'next/server'
import  from '@vercel/functions'

export function GET(req: NextRequest) 
```

### 14.0

#### Migrate `ImageResponse` imports

##### `next-og-import`

```bash filename="Terminal"
npx @next/codemod@latest next-og-import .
```

This codemod moves transforms imports from `next/server` to `next/og` for usage of [Dynamic OG Image Generation](/docs/app/getting-started/metadata-and-og-images#generated-open-graph-images).

For example:

```js
import  from 'next/server'
```

Transforms into:

```js
import  from 'next/og'
```

#### Use `viewport` export

##### `metadata-to-viewport-export`

```bash filename="Terminal"
npx @next/codemod@latest metadata-to-viewport-export .
```

This codemod migrates certain viewport metadata to `viewport` export.

For example:

```js
export const metadata = {
  title: 'My App',
  themeColor: 'dark',
  viewport: ,
}
```

Transforms into:

```js
export const metadata = 

export const viewport = 
```

### 13.2

#### Use Built-in Font

##### `built-in-next-font`

```bash filename="Terminal"
npx @next/codemod@latest built-in-next-font .
```

This codemod uninstalls the `@next/font` package and transforms `@next/font` imports into the built-in `next/font`.

For example:

```js
import  from '@next/font/google'
```

Transforms into:

```js
import  from 'next/font/google'
```

### 13.0

#### Rename Next Image Imports

##### `next-image-to-legacy-image`

```bash filename="Terminal"
npx @next/codemod@latest next-image-to-legacy-image .
```

Safely renames `next/image` imports in existing Next.js 10, 11, or 12 applications to `next/legacy/image` in Next.js 13. Also renames `next/future/image` to `next/image`.

For example:

```jsx filename="pages/index.js"
import Image1 from 'next/image'
import Image2 from 'next/future/image'

export default function Home() {
  return (
    
      

For example:

```jsx

// transforms into

// transforms into

```

### 11

#### Migrate from CRA

##### `cra-to-next`

```bash filename="Terminal"
npx @next/codemod cra-to-next
```

Migrates a Create React App project to Next.js; creating a Pages Router and necessary config to match behavior. Client-side only rendering is leveraged initially to prevent breaking compatibility due to `window` usage during SSR and can be enabled seamlessly to allow the gradual adoption of Next.js specific features.

Please share any feedback related to this transform [in this discussion](https://github.com/vercel/next.js/discussions/25858).

### 10

#### Add React imports

##### `add-missing-react-import`

```bash filename="Terminal"
npx @next/codemod add-missing-react-import
```

Transforms files that do not import `React` to include the import in order for the new [React JSX transform](https://reactjs.org/blog/2020/09/22/introducing-the-new-jsx-transform.html) to work.

For example:

```jsx filename="my-component.js"
export default class Home extends React.Component {
  render() 
}
```

Transforms into:

```jsx filename="my-component.js"
import React from 'react'
export default class Home extends React.Component {
  render() 
}
```

### 9

#### Transform Anonymous Components into Named Components

##### `name-default-component`

```bash filename="Terminal"
npx @next/codemod name-default-component
```

**Versions 9 and above.**

Transforms anonymous components into named components to make sure they work with [Fast Refresh](https://nextjs.org/blog/next-9-4#fast-refresh).

For example:

```jsx filename="my-component.js"
export default function () 
```

Transforms into:

```jsx filename="my-component.js"
export default function MyComponent() 
```

The component will have a camel-cased name based on the name of the file, and it also works with arrow functions.

### 8

> **Note**: Built-in AMP support and this codemod have been removed in Next.js 16.

#### Transform AMP HOC into page config

##### `withamp-to-config`

```bash filename="Terminal"
npx @next/codemod withamp-to-config
```

Transforms the `withAmp` HOC into Next.js 9 page configuration.

For example:

```js
// Before
import  from 'next/amp'

function Home() 

export default withAmp(Home)
```

```js
// After
export default function Home() 

export const config = 
```

### 6

#### Use `withRouter`

##### `url-to-withrouter`

```bash filename="Terminal"
npx @next/codemod url-to-withrouter
```

Transforms the deprecated automatically injected `url` property on top level pages to using `withRouter` and the `router` property it injects. Read more here: [https://nextjs.org/docs/messages/url-deprecated](/docs/messages/url-deprecated)

For example:

```js filename="From"
import React from 'react'
export default class extends React.Component {
  render() {
    const  = this.props.url
    return Current pathname: 
  }
}
```

```js filename="To"
import React from 'react'
import  from 'next/router'
export default withRouter(
  class extends React.Component {
    render() {
      const  = this.props.router
      return Current pathname: 
    }
  }
)
```

This is one case. All the cases that are transformed (and tested) can be found in the [`__testfixtures__` directory](https://github.com/vercel/next.js/tree/canary/packages/next-codemod/transforms/__testfixtures__/url-to-withrouter).
