---
title: layout.js
description: API reference for the layout.js file.
---

The `layout` file is used to define a layout in your Next.js application.

```tsx filename="app/dashboard/layout.tsx" switcher
export default function DashboardLayout(: ) {
  return 
}
```

```jsx filename="app/dashboard/layout.js" switcher
export default function DashboardLayout() {
  return 
}
```

In the [component hierarchy](/docs/app/getting-started/project-structure#component-hierarchy), `layout.js` is the outermost component in a route segment. It wraps `template.js`, `error.js`, `loading.js`, `not-found.js`, and `page.js`.

A **root layout** is the top-most layout in the root `app` directory. It is used to define the `` and `` tags and other globally shared UI.

```tsx filename="app/layout.tsx" switcher
export default function RootLayout(: ) {
  return (
    
      
    
  )
}
```

```jsx filename="app/layout.js" switcher
export default function RootLayout() {
  return (
    
      
    
  )
}
```

## Reference

### Props

#### `children` (required)

Layout components should accept and use a `children` prop. During rendering, `children` will be populated with the route segments the layout is wrapping. These will primarily be the component of a child [Layout](/docs/app/api-reference/file-conventions/page) (if it exists) or [Page](/docs/app/api-reference/file-conventions/page), but could also be other special files like [Loading](/docs/app/api-reference/file-conventions/loading) or [Error](/docs/app/getting-started/error-handling) when applicable.

#### `params` (optional)

A promise that resolves to an object containing the [dynamic route parameters](/docs/app/api-reference/file-conventions/dynamic-routes) object from the root segment down to that layout.

```tsx filename="app/dashboard/[team]/layout.tsx" switcher
export default async function Layout(: {
  children: React.ReactNode
  params: Promise<>
}) {
  const  = await params
}
```

```jsx filename="app/dashboard/[team]/layout.js" switcher
export default async function Layout() {
  const  = await params
}
```

| Example Route                     | URL            | `params`                           |
| --------------------------------- | -------------- | ---------------------------------- |
| `app/dashboard/[team]/layout.js`  | `/dashboard/1` | `Promise<>`           |
| `app/shop/[tag]/[item]/layout.js` | `/shop/1/2`    | `Promise<>` |
| `app/blog/[...slug]/layout.js`    | `/blog/1/2`    | `Promise<>`    |

- Since the `params` prop is a promise. You must use `async/await` or React's [`use`](https://react.dev/reference/react/use) function to access the values.
  - In version 14 and earlier, `params` was a synchronous prop. To help with backwards compatibility, you can still access it synchronously in Next.js 15, but this behavior will be deprecated in the future.

### Layout Props Helper

You can type layouts with `LayoutProps` to get a strongly typed `params` and named slots inferred from your directory structure. `LayoutProps` is a globally available helper.

```tsx filename="app/dashboard/layout.tsx"
export default function Layout(props: LayoutProps<'/dashboard'>) {
  return (
    
      
      
      {/*  */}
    
  )
}
```

> **Good to know**:
>
> - Types are generated during `next dev`, `next build` or `next typegen`.
> - After type generation, the `LayoutProps` helper is globally available. It doesn't need to be imported.

### Root Layout

The `app` directory **must** include a **root layout**, which is the top-most layout in the root `app` directory. Typically, the root layout is `app/layout.js`.

```tsx filename="app/layout.tsx" switcher
export default function RootLayout(: ) {
  return (
    
      
    
  )
}
```

```jsx filename="app/layout.js" switcher
export default function RootLayout() {
  return (
    
      
    
  )
}
```

- The root layout **must** define `` and `` tags.
  - You should **not** manually add `` tags such as `` and `` to root layouts. Instead, you should use the [Metadata API](/docs/app/getting-started/metadata-and-og-images) which automatically handles advanced requirements such as streaming and de-duplicating `` elements.
- You can create **multiple root layouts**. Any layout without a `layout.js` above it is a root layout. Two common approaches:
  - Using [route groups](/docs/app/api-reference/file-conventions/route-groups) like `app/(shop)/layout.js` and `app/(marketing)/layout.js`
  - Omitting `app/layout.js` so layouts in subdirectories like `app/dashboard/layout.js` and `app/blog/layout.js` each become root layouts for their respective directories.
  - Navigating **across multiple root layouts** will cause a **full page load** (as opposed to a client-side navigation).
- The root layout can be under a **dynamic segment**, for example when implementing [internationalization](/docs/app/guides/internationalization) with `app/[lang]/layout.js`. Dynamic segments before the root layout are **root parameters** and can be read from any Server Component with [`next/root-params`](/docs/app/api-reference/functions/next-root-params).

## Caveats

### Request Object

Layouts are cached in the client during navigation to avoid unnecessary server requests.

[Layouts](/docs/app/api-reference/file-conventions/layout) do not rerender. They can be cached and reused to avoid unnecessary computation when navigating between pages. By restricting layouts from accessing the raw request, Next.js can prevent the execution of potentially slow or expensive user code within the layout, which could negatively impact performance.

To access the request object, you can use [`headers`](/docs/app/api-reference/functions/headers) and [`cookies`](/docs/app/api-reference/functions/cookies) APIs in [Server Components](/docs/app/getting-started/server-and-client-components) and Functions.

```tsx filename="app/shop/layout.tsx" switcher
import  from 'next/headers'

export default async function Layout() 
```

```jsx filename="app/shop/layout.js" switcher
import  from 'next/headers'

export default async function Layout() 
```

### Query params

Layouts do not rerender on navigation, so they cannot access search params which would otherwise become stale.

To access updated query parameters, you can use the Page [`searchParams`](/docs/app/api-reference/file-conventions/page#searchparams-optional) prop, or read them inside a Client Component using the [`useSearchParams`](/docs/app/api-reference/functions/use-search-params) hook. Since Client Components re-render on navigation, they have access to the latest query parameters.

```tsx filename="app/ui/search.tsx" switcher
'use client'

import  from 'next/navigation'

export default function Search() 
```

```jsx filename="app/ui/search.js" switcher
'use client'

import  from 'next/navigation'

export default function Search() 
```

```tsx filename="app/shop/layout.tsx" switcher
import Search from '@/app/ui/search'

export default function Layout() {
  return (
    <>
      
      
    </>
  )
}
```

```jsx filename="app/dashboard/layout.js" switcher highlight=
import  from 'react'
import  from './nav-skeleton'
import  from './dashboard-nav'

export default function Layout() {
  return (
    <>
      
      
    </>
  )
}
```

### Fetching Data

Layouts cannot pass data to their `children`. However, you can fetch the same data in a route more than once, and use React [`cache`](https://react.dev/reference/react/cache) to dedupe the requests without affecting performance.

Alternatively, when using [`fetch`](/docs/app/api-reference/functions/fetch)in Next.js, requests are automatically deduped.

```tsx filename="app/lib/data.ts" switcher
export async function getUser(id: string) {
  const res = await fetch(`https://.../users/$`)
  return res.json()
}
```

```tsx filename="app/dashboard/layout.tsx" switcher
import  from '@/app/lib/data'
import  from '@/app/ui/user-name'

export default async function Layout() {
  const user = await getUser('1')

  return (
    <>
      
        
        
  )
}
```

```jsx filename="app/ui/nav-link.js" switcher
'use client'

import Link from 'next/link'
import  from 'next/navigation'

export default function NavLinks() 
```

```tsx filename="app/blog/layout.tsx" switcher
import  from './nav-link'
import getPosts from './get-posts'

export default async function Layout(: ) {
  const featuredPosts = await getPosts()
  return (
    
      
      
    
  )
}
```

```jsx filename="app/blog/layout.js" switcher
import  from './nav-link'
import getPosts from './get-posts'

export default async function Layout() {
  const featuredPosts = await getPosts()
  return (
    
      
      
    
  )
}
```

## Examples

### Metadata

You can modify the `` HTML elements such as `title` and `meta` using the [`metadata` object](/docs/app/api-reference/functions/generate-metadata#the-metadata-object) or [`generateMetadata` function](/docs/app/api-reference/functions/generate-metadata#generatemetadata-function).

```tsx filename="app/layout.tsx" switcher
import type  from 'next'

export const metadata: Metadata = 

export default function Layout(: ) 
```

```jsx filename="app/layout.js" switcher
export const metadata = 

export default function Layout() 
```

> **Good to know**: You should **not** manually add `` tags such as `` and `` to root layouts. Instead, use the [Metadata APIs](/docs/app/api-reference/functions/generate-metadata) which automatically handles advanced requirements such as streaming and de-duplicating `` elements.

### Active Nav Links

You can use the [`usePathname`](/docs/app/api-reference/functions/use-pathname) hook to determine if a nav link is active.

Since `usePathname` is a client hook, you need to extract the nav links into a Client Component, which can be imported into your layout:

```tsx filename="app/ui/nav-links.tsx" switcher
'use client'

import  from 'next/navigation'
import Link from 'next/link'

export function NavLinks() 
```

```jsx filename="app/ui/nav-links.js" switcher
'use client'

import  from 'next/navigation'
import Link from 'next/link'

export function Links() 
```

```tsx filename="app/layout.tsx" switcher
import  from '@/app/ui/nav-links'

export default function Layout(: ) {
  return (
    
      
        
        
      
    
  )
}
```

```jsx filename="app/layout.js" switcher
import  from '@/app/ui/nav-links'

export default function Layout() {
  return (
    
      
        
        
      
    
  )
}
```

### Displaying content based on `params`

Using [dynamic route segments](/docs/app/api-reference/file-conventions/dynamic-routes), you can display or fetch specific content based on the `params` prop.

```tsx filename="app/dashboard/layout.tsx" switcher
export default async function DashboardLayout(: {
  children: React.ReactNode
  params: Promise<>
}) {
  const  = await params

  return (
    
      
        Welcome to 's Dashboard
      
      
    
  )
}
```

```jsx filename="app/dashboard/layout.js" switcher
export default async function DashboardLayout() {
  const  = await params

  return (
    
      
        Welcome to 's Dashboard
      
      
    
  )
}
```

### Reading `params` in Client Components

To use `params` in a Client Component (which cannot be `async`), you can use React's [`use`](https://react.dev/reference/react/use) function to read the promise:

```tsx filename="app/page.tsx" switcher
'use client'

import  from 'react'

export default function Page(: {
  params: Promise<>
}) {
  const  = use(params)
}
```

```js filename="app/page.js" switcher
'use client'

import  from 'react'

export default function Page() {
  const  = use(params)
}
```

## Version History

| Version      | Changes                                                                                       |
| ------------ | --------------------------------------------------------------------------------------------- |
| `v15.0.0-RC` | `params` is now a promise. A [codemod](/docs/app/guides/upgrading/codemods#150) is available. |
| `v13.0.0`    | `layout` introduced.                                                                          |
