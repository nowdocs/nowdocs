---
title: generateViewport
description: API Reference for the generateViewport function.
related:
  title: Next Steps
  description: View all the Metadata API options.
  links:
    - app/api-reference/file-conventions/metadata
    - app/getting-started/caching
    - app/api-reference/config/next-config-js/cacheComponents
---

You can customize the initial viewport of the page with the static `viewport` object or the dynamic `generateViewport` function.

> **Good to know**:
>
> - The `viewport` object and `generateViewport` function exports are **only supported in Server Components**.
> - You cannot export both the `viewport` object and `generateViewport` function from the same route segment.
> - If you're coming from migrating `metadata` exports, you can use [metadata-to-viewport-export codemod](/docs/app/guides/upgrading/codemods#metadata-to-viewport-export) to update your changes.

## The `viewport` object

To define the viewport options, export a `viewport` object from a `layout.jsx` or `page.jsx` file.

```tsx filename="layout.tsx | page.tsx" switcher
import type  from 'next'

export const viewport: Viewport = 

export default function Page() 
```

```jsx filename="layout.jsx | page.jsx" switcher
export const viewport = 

export default function Page() 
```

## `generateViewport` function

`generateViewport` should return a [`Viewport` object](#viewport-fields) containing one or more viewport fields.

```tsx filename="layout.tsx | page.tsx" switcher
export function generateViewport() {
  return 
}
```

In TypeScript, the `params` argument can be typed via [`PageProps<'/route'>`](/docs/app/api-reference/file-conventions/page#page-props-helper) or [`LayoutProps<'/route'>`](/docs/app/api-reference/file-conventions/layout#layout-props-helper) depending on where `generateViewport` is defined.

```jsx filename="layout.js | page.js" switcher
export function generateViewport() {
  return 
}
```

> **Good to know**:
>
> - If the viewport doesn't depend on request information, it should be defined using the static [`viewport` object](#the-viewport-object) rather than `generateViewport`.

## Viewport Fields

### `themeColor`

Learn more about [`theme-color`](https://developer.mozilla.org/docs/Web/HTML/Element/meta/name/theme-color).

**Simple theme color**

```tsx filename="layout.tsx | page.tsx" switcher
import type  from 'next'

export const viewport: Viewport = 
```

```jsx filename="layout.jsx | page.jsx" switcher
export const viewport = 
```

```html filename=" output" hideLineNumbers

```

**With media attribute**

```tsx filename="layout.tsx | page.tsx" switcher
import type  from 'next'

export const viewport: Viewport = {
  themeColor: [
    ,
    ,
  ],
}
```

```jsx filename="layout.jsx | page.jsx" switcher
export const viewport = {
  themeColor: [
    ,
    ,
  ],
}
```

```html filename=" output" hideLineNumbers

```

### `width`, `initialScale`, `maximumScale` and `userScalable`

> **Good to know**: The `viewport` meta tag is automatically set, and manual configuration is usually unnecessary as the default is sufficient. However, the information is provided for completeness.

```tsx filename="layout.tsx | page.tsx" switcher
import type  from 'next'

export const viewport: Viewport = 
```

```jsx filename="layout.jsx | page.jsx" switcher
export const viewport = 
```

```html filename=" output" hideLineNumbers

```

### `colorScheme`

Learn more about [`color-scheme`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/meta/name#:~:text=color%2Dscheme%3A%20specifies,of%20the%20following%3A).

```tsx filename="layout.tsx | page.tsx" switcher
import type  from 'next'

export const viewport: Viewport = 
```

```jsx filename="layout.jsx | page.jsx" switcher
export const viewport = 
```

```html filename=" output" hideLineNumbers

```

## With Cache Components

When [Cache Components](/docs/app/getting-started/caching) is enabled, `generateViewport` follows the same rules as other components. If viewport accesses runtime data (`cookies()`, `headers()`, `params`, `searchParams`) or performs uncached data fetching, it defers to request time.

Unlike metadata, viewport cannot be streamed because it affects initial page load UI. If `generateViewport` defers to request time, the page would need to block until resolved.

If viewport depends on external data but not runtime data, use `use cache`:

```tsx filename="app/layout.tsx" highlight=
export async function generateViewport() {
  'use cache'
  const  = await db.query('viewport-size')
  return 
}
```

If viewport genuinely requires runtime data, wrap the document `` in a [`
  )
}
```

Alternatively, opt the segment out of instant-navigation validation with [`instant = false`](/docs/app/api-reference/file-conventions/route-segment-config/instant). The route renders on every request and the navigation blocks until the render completes:

```tsx filename="app/dashboard/layout.tsx" highlight=
import  from 'next/headers'

export const instant = false

export async function generateViewport() {
  const cookieJar = await cookies()
  return 
}

export default function DashboardLayout() {
  return 
}
```

Setting `instant = false` opts only the segment that exports it out. Descendant segments are still validated by the global default. See [Next.js encountered runtime data in `generateViewport()`](/docs/messages/blocking-prerender-viewport-runtime) for the full set of fix options, trade-offs, and the subtree-wide opt-out.

> **Good to know**: Use [multiple root layouts](/docs/app/api-reference/file-conventions/layout#root-layout) to isolate fully dynamic viewport to specific routes, while still letting other routes in your application generate a static shell.

## Types

You can add type safety to your viewport object by using the `Viewport` type. If you are using the [built-in TypeScript plugin](/docs/app/api-reference/config/typescript) in your IDE, you do not need to manually add the type, but you can still explicitly add it if you want.

### `viewport` object

```tsx
import type  from 'next'

export const viewport: Viewport = 
```

### `generateViewport` function

#### Regular function

```tsx
import type  from 'next'

export function generateViewport(): Viewport {
  return 
}
```

#### With segment props

```tsx
import type  from 'next'

type Props = {
  params: Promise<>
  searchParams: Promise<>
}

export function generateViewport(: Props): Viewport {
  return 
}

export default function Page(: Props) 
```

#### JavaScript Projects

For JavaScript projects, you can use JSDoc to add type safety.

```js
/** @type  */
export const viewport = 
```

## Version History

| Version   | Changes                                       |
| --------- | --------------------------------------------- |
| `v14.0.0` | `viewport` and `generateViewport` introduced. |
