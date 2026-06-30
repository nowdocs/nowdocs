---
title: How to build single-page applications with Next.js
nav_title: SPAs
description: Next.js fully supports building Single-Page Applications (SPAs).
---

Next.js fully supports building Single-Page Applications (SPAs).

This includes fast route transitions with prefetching, client-side data fetching, using browser APIs, integrating with third-party client libraries, creating static routes, and more.

If you have an existing SPA, you can migrate to Next.js without large changes to your code. Next.js then allows you to progressively add server features as needed.

## What is a Single-Page Application?

The definition of a SPA varies. We’ll define a “strict SPA” as:

- **Client-side rendering (CSR)**: The app is served by one HTML file (e.g. `index.html`). Every route, page transition, and data fetch is handled by JavaScript in the browser.
- **No full-page reloads**: Rather than requesting a new document for each route, client-side JavaScript manipulates the current page’s DOM and fetches data as needed.

Strict SPAs often require large amounts of JavaScript to load before the page can be interactive. Further, client data waterfalls can be challenging to manage. Building SPAs with Next.js can address these issues.

## Why use Next.js for SPAs?

Next.js can automatically code split your JavaScript bundles, and generate multiple HTML entry points into different routes. This avoids loading unnecessary JavaScript code on the client-side, reducing the bundle size and enabling faster page loads.

The [`next/link`](/docs/app/api-reference/components/link) component automatically [prefetches](/docs/app/api-reference/components/link#prefetch) routes, giving you the fast page transitions of a strict SPA, but with the advantage of persisting application routing state to the URL for linking and sharing.

Next.js can start as a static site or even a strict SPA where everything is rendered client-side. If your project grows, Next.js allows you to progressively add more server features (e.g. [React Server Components](/docs/app/getting-started/server-and-client-components), [Server Actions](/docs/app/guides/server-actions), and more) as needed.

## Examples

Let's explore common patterns used to build SPAs and how Next.js solves them.

### Using React’s `use` within a Context Provider

We recommend fetching data in a parent component (or layout), returning the Promise, and then unwrapping the value in a Client Component with React’s [`use` API](https://react.dev/reference/react/use).

Next.js can start data fetching early on the server. In this example, that’s the root layout — the entry point to your application. The server can immediately begin streaming a response to the client.

By “hoisting” your data fetching to the root layout, Next.js starts the specified requests on the server early before any other components in your application. This eliminates client waterfalls and prevents having multiple roundtrips between client and server. It can also significantly improve performance, as your server is closer (and ideally colocated) to where your database is located.

For example, update your root layout to call the Promise, but do _not_ await it.

```tsx filename="app/layout.tsx" switcher
import  from './user-provider'
import  from './user' // some server-side function

export default function RootLayout(: ) 
```

```jsx filename="app/layout.js" switcher
import  from './user-provider'
import  from './user' // some server-side function

export default function RootLayout() 
```

While you can [defer and pass a single Promise](/docs/app/getting-started/fetching-data#streaming-data-with-the-use-api) as a prop to a Client Component, we generally see this pattern paired with a React context provider. This enables easier access from Client Components with a custom React Hook.

You can forward a Promise to the React context provider:

```ts filename="app/user-provider.ts" switcher
'use client';

import  from 'react';

type User = any;
type UserContextType = 
```

```js filename="app/layout.js" switcher
import  from 'swr'
import  from './user' // some server-side function

export default function RootLayout() 
```

Because this is a Server Component, `getUser()` can securely read cookies, headers, or talk to your database. No separate API route is needed. Client components below the `` can call `useSWR()` with the same key to retrieve the user data. The component code with `useSWR` **does not require any changes** from your existing client-fetching solution.

```tsx filename="app/profile.tsx" switcher
'use client'

import useSWR from 'swr'

export function Profile() {
  const fetcher = (url) => fetch(url).then((res) => res.json())
  // The same SWR pattern you already know
  const  = useSWR('/api/user', fetcher)

  return '...'
}
```

```jsx filename="app/profile.js" switcher
'use client'

import useSWR from 'swr'

export function Profile() {
  const fetcher = (url) => fetch(url).then((res) => res.json())
  // The same SWR pattern you already know
  const  = useSWR('/api/user', fetcher)

  return '...'
}
```

The `fallback` data can be prerendered and included in the initial HTML response, then immediately read in the child components using `useSWR`. SWR’s polling, revalidation, and caching still run **client-side only**, so it preserves all the interactivity you rely on for an SPA.

Since the initial `fallback` data is automatically handled by Next.js, you can now delete any conditional logic previously needed to check if `data` was `undefined`. When the data is loading, the closest `` boundary will be suspended.

|                      | SWR                 | RSC                 | RSC + SWR           |
| -------------------- | ------------------- | ------------------- | ------------------- |
| SSR data             |  |  |  |
| Streaming while SSR  |  |  |  |
| Deduplicate requests |  |  |  |
| Client-side features |  |  |  |

### SPAs with React Query

You can use React Query with Next.js on both the client and server. This enables you to build both strict SPAs, as well as take advantage of server features in Next.js paired with React Query.

Learn more in the [React Query documentation](https://tanstack.com/query/latest/docs/framework/react/guides/advanced-ssr).

### Rendering components only in the browser

Client components are [prerendered](https://github.com/reactwg/server-components/discussions/4) during `next build`. If you want to disable prerendering for a Client Component and only load it in the browser environment, you can use [`next/dynamic`](/docs/app/guides/lazy-loading#nextdynamic):

```jsx
import dynamic from 'next/dynamic'

const ClientOnlyComponent = dynamic(() => import('./component'), )
```

This can be useful for third-party libraries that rely on browser APIs like `window` or `document`. You can also add a `useEffect` that checks for the existence of these APIs, and if they do not exist, return `null` or a loading state which would be prerendered.

### Shallow routing on the client

If you are migrating from a strict SPA like [Create React App](/docs/app/guides/migrating/from-create-react-app) or [Vite](/docs/app/guides/migrating/from-vite), you might have existing code which shallow routes to update the URL state. This can be useful for manual transitions between views in your application _without_ using the default Next.js file-system routing.

Next.js allows you to use the native [`window.history.pushState`](https://developer.mozilla.org/en-US/docs/Web/API/History/pushState) and [`window.history.replaceState`](https://developer.mozilla.org/en-US/docs/Web/API/History/replaceState) methods to update the browser's history stack without reloading the page.

`pushState` and `replaceState` calls integrate into the Next.js Router, allowing you to sync with [`usePathname`](/docs/app/api-reference/functions/use-pathname) and [`useSearchParams`](/docs/app/api-reference/functions/use-search-params).

```tsx fileName="app/ui/sort-products.tsx" switcher
'use client'

import  from 'next/navigation'

export default function SortProducts() {
  const searchParams = useSearchParams()

  function updateSorting(sortOrder: string) {
    const urlSearchParams = new URLSearchParams(searchParams.toString())
    urlSearchParams.set('sort', sortOrder)
    window.history.pushState(null, '', `?$`)
  }

  return (
    <>
       updateSorting('asc')}>Sort Ascending
       updateSorting('desc')}>Sort Descending
    </>
  )
}
```

```jsx fileName="app/ui/sort-products.js" switcher
'use client'

import  from 'next/navigation'

export default function SortProducts() {
  const searchParams = useSearchParams()

  function updateSorting(sortOrder) {
    const urlSearchParams = new URLSearchParams(searchParams.toString())
    urlSearchParams.set('sort', sortOrder)
    window.history.pushState(null, '', `?$`)
  }

  return (
    <>
       updateSorting('asc')}>Sort Ascending
       updateSorting('desc')}>Sort Descending
    </>
  )
}
```

Learn more about how [routing and navigation](/docs/app/getting-started/linking-and-navigating#how-navigation-works) work in Next.js.

### Using Server Actions in Client Components

You can progressively adopt Server Actions while still using Client Components. This allows you to remove boilerplate code to call an API route, and instead use React features like `useActionState` to handle loading and error states. See the [Building interactive apps](/docs/app/guides/interactive-apps) guide for a deeper walkthrough, including pending feedback, optimistic UI, transitions, and error handling.

For example, create your first Server Action:

```tsx filename="app/actions.ts" switcher
'use server'

export async function create() 
```

```js filename="app/actions.js" switcher
'use server'

export async function create() 
```

You can import and use a Server Action from the client, similar to calling a JavaScript function. You do not need to create an API endpoint manually:

```tsx filename="app/button.tsx" switcher
'use client'

import  from './actions'

export function Button() >Create
}
```

```jsx filename="app/button.js" switcher
'use client'

import  from './actions'

export function Button() >Create
}
```

Learn more about [mutating data with Server Actions](/docs/app/getting-started/mutating-data).

## Static export (optional)

Next.js also supports generating a fully [static site](/docs/app/guides/static-exports). This has some advantages over strict SPAs:

- **Automatic code-splitting**: Instead of shipping a single `index.html`, Next.js will generate an HTML file per route, so your visitors get the content faster without waiting for the client JavaScript bundle.
- **Improved user experience:** Instead of a minimal skeleton for all routes, you get fully rendered pages for each route. When users navigate client side, transitions remain instant and SPA-like.

To enable a static export, update your configuration:

```ts filename="next.config.ts"
import type  from 'next'

const nextConfig: NextConfig = 

export default nextConfig
```

After running `next build`, Next.js will create an `out` folder with the HTML/CSS/JS assets for your application.

> **Note:** Next.js server features are not supported with static exports. [Learn more](/docs/app/guides/static-exports#unsupported-features).

## Migrating existing projects to Next.js

You can incrementally migrate to Next.js by following our guides:

- [Migrating from Create React App](/docs/app/guides/migrating/from-create-react-app)
- [Migrating from Vite](/docs/app/guides/migrating/from-vite)

If you are already using a SPA with the Pages Router, you can learn how to [incrementally adopt the App Router](/docs/app/guides/migrating/app-router-migration).
