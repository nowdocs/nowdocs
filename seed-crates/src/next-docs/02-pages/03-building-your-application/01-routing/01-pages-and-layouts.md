---
title: Pages and Layouts
description: Create your first page and shared layout with the Pages Router.
---

The Pages Router has a file-system based router built on the concept of pages.

When a file is added to the `pages` directory, it's automatically available as a route.

In Next.js, a **page** is a [React Component](https://react.dev/learn/your-first-component) exported from a `.js`, `.jsx`, `.ts`, or `.tsx` file in the `pages` directory. Each page is associated with a route based on its file name.

**Example**: If you create `pages/about.js` that exports a React component like below, it will be accessible at `/about`.

```jsx
export default function About() 
```

## Index routes

The router will automatically route files named `index` to the root of the directory.

- `pages/index.js` → `/`
- `pages/blog/index.js` → `/blog`

## Nested routes

The router supports nested files. If you create a nested folder structure, files will automatically be routed in the same way still.

- `pages/blog/first-post.js` → `/blog/first-post`
- `pages/dashboard/settings/username.js` → `/dashboard/settings/username`

## Pages with Dynamic Routes

Next.js supports pages with dynamic routes. For example, if you create a file called `pages/posts/[id].js`, then it will be accessible at `posts/1`, `posts/2`, etc.

> To learn more about dynamic routing, check the [Dynamic Routing documentation](/docs/pages/building-your-application/routing/dynamic-routes).

## Layout Pattern

The React model allows us to deconstruct a [page](/docs/pages/building-your-application/routing/pages-and-layouts) into a series of components. Many of these components are often reused between pages. For example, you might have the same navigation bar and footer on every page.

```jsx filename="components/layout.js"
import Navbar from './navbar'
import Footer from './footer'

export default function Layout() 
```

### Per-Page Layouts

If you need multiple layouts, you can add a property `getLayout` to your page, allowing you to return a React component for the layout. This allows you to define the layout on a _per-page basis_. Since we're returning a function, we can have complex nested layouts if desired.

```jsx filename="pages/index.js"

import Layout from '../components/layout'
import NestedLayout from '../components/nested-layout'

export default function Page() 

Page.getLayout = function getLayout(page) 
```

```jsx filename="pages/_app.js"
export default function MyApp() 

export default Page
```

```jsx filename="pages/index.js" switcher
import Layout from '../components/layout'
import NestedLayout from '../components/nested-layout'

const Page = () => 

Page.getLayout = function getLayout(page) 

export default Page
```

```tsx filename="pages/_app.tsx" switcher
import type  from 'react'
import type  from 'next'
import type  from 'next/app'

export type NextPageWithLayout = NextPage & 

type AppPropsWithLayout = AppProps & 

export default function MyApp(: AppPropsWithLayout) 
```

```jsx filename="pages/_app.js" switcher
export default function MyApp() 
```

### Data Fetching

Inside your layout, you can fetch data on the client-side using `useEffect` or a library like [SWR](https://swr.vercel.app/). Because this file is not a [Page](/docs/pages/building-your-application/routing/pages-and-layouts), you cannot use `getStaticProps` or `getServerSideProps` currently.

```jsx filename="components/layout.js"
import useSWR from 'swr'
import Navbar from './navbar'
import Footer from './footer'

export default function Layout() {
  const  = useSWR('/api/navigation', fetcher)

  if (error) return Failed to load
  if (!data) return Loading...

  return (
    <>
      
      
      
    </>
  )
}
```
