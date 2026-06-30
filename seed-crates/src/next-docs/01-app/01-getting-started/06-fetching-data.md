---
title: Fetching Data
description: Learn how to fetch data and stream content that depends on data.
related:
  title: API Reference
  description: Learn more about the features mentioned in this page by reading the API Reference.
  links:
    - app/guides/data-security
    - app/api-reference/functions/fetch
    - app/api-reference/file-conventions/loading
    - app/api-reference/config/next-config-js/logging
    - app/api-reference/config/next-config-js/taint
---

This page will walk you through how you can fetch data in [Server](#server-components) and [Client](#client-components) Components, and how to [stream](#streaming) components that depend on uncached data.

## Fetching data

### Server Components

You can fetch data in Server Components using any asynchronous I/O, such as:

1. The [`fetch` API](#with-the-fetch-api)
2. An [ORM or database](#with-an-orm-or-database)

#### With the `fetch` API

To fetch data with the `fetch` API, turn your component into an asynchronous function, and await the `fetch` call. For example:

```tsx filename="app/blog/page.tsx" switcher
export default async function Page() {
  const data = await fetch('https://api.vercel.app/blog')
  const posts = await data.json()
  return (
    
      {posts.map((post) => (
        
      ))}
    
  )
}
```

```jsx filename="app/blog/page.js" switcher
export default async function Page() {
  const data = await fetch('https://api.vercel.app/blog')
  const posts = await data.json()
  return (
    
      {posts.map((post) => (
        
      ))}
    
  )
}
```

> **Good to know:**
>
> - Identical `fetch` requests in a React component tree are [memoized](/docs/app/glossary#memoization) by default, so you can fetch data in the component that needs it instead of drilling props.
> - `fetch` requests are not cached by default and will block the page from rendering until the request is complete. Use the [`use cache`](/docs/app/api-reference/directives/use-cache) directive to cache results, or wrap the fetching component in [`
      
    
  )
}
```

```jsx filename="app/blog/page.js" switcher
import  from 'react'
import BlogList from '@/components/BlogList'
import BlogListSkeleton from '@/components/BlogListSkeleton'

export default function BlogPage() {
  return (
    
      
      
        Welcome to the Blog
        Read the latest posts below.
      
      
        
        
      
    
  )
}
```

#### Creating meaningful loading states

An instant loading state is fallback UI that is shown immediately to the user after navigation. For the best user experience, we recommend designing loading states that are meaningful and help users understand the app is responding. For example, you can use skeletons and spinners, or a small but meaningful part of future screens such as a cover photo, title, etc.

In development, you can preview and inspect the loading state of your components using the [React Devtools](https://react.dev/learn/react-developer-tools).

### Client Components

There are two ways to fetch data in Client Components, using:

1. React's [`use` API](https://react.dev/reference/react/use)
2. A community library like [SWR](https://swr.vercel.app/) or [React Query](https://tanstack.com/query/latest)

#### Streaming data with the `use` API

You can use React's [`use` API](https://react.dev/reference/react/use) to [stream](#streaming) data from the server to client. Start by fetching data in your Server component, and pass the promise to your Client Component as prop:

```tsx filename="app/blog/page.tsx" switcher
import Posts from '@/app/ui/posts'
import  from 'react'

export default function Page() 
```

```jsx filename="app/blog/page.js" switcher
import Posts from '@/app/ui/posts'
import  from 'react'

export default function Page() 
```

Then, in your Client Component, use the `use` API to read the promise:

```tsx filename="app/ui/posts.tsx" switcher
'use client'
import  from 'react'

export default function Posts(: {
  posts: Promise<[]>
}) {
  const allPosts = use(posts)

  return (
    
      {allPosts.map((post) => (
        
      ))}
    
  )
}
```

```jsx filename="app/ui/posts.js" switcher
'use client'
import  from 'react'

export default function Posts() {
  const allPosts = use(posts)

  return (
    
      {allPosts.map((post) => (
        
      ))}
    
  )
}
```

In the example above, the `
    </>
  )
}

async function Playlists(: ) {
  // Use the artist ID to fetch playlists
  const playlists = await getArtistPlaylists(artistID)

  return (
    
      {playlists.map((playlist) => (
        
      ))}
    
  )
}
```

```jsx filename="app/artist/[username]/page.js" switcher
export default async function Page() {
  const  = await params
  // Get artist information
  const artist = await getArtist(username)

  return (
    <>
      
      
      
    </>
  )
}

async function Playlists() {
  // Use the artist ID to fetch playlists
  const playlists = await getArtistPlaylists(artistID)

  return (
    
      {playlists.map((playlist) => (
        
      ))}
    
  )
}
```

In this example, `
}
```

```jsx filename="app/user-provider.js" switcher
'use client'

import  from 'react'

export const UserContext = createContext(null)

export default function UserProvider() 
```

In a layout, pass the promise to the provider without awaiting:

```tsx filename="app/layout.tsx" switcher
import UserProvider from './user-provider'
import  from './lib/user'

export default function RootLayout(: ) 
```

```jsx filename="app/layout.js" switcher
import UserProvider from './user-provider'
import  from './lib/user'

export default function RootLayout() 
```

Client Components use [`use()`](https://react.dev/reference/react/use) to resolve the promise from context, wrapped in `
  )
}
```

```jsx filename="app/page.js" switcher
import  from 'react'
import  from './ui/profile'

export default function Page() 
```

Server Components can also call `getUser()` directly:

```tsx filename="app/dashboard/page.tsx" switcher
import  from '../lib/user'

export default async function DashboardPage() {
  const user = await getUser() // Cached - same request, no duplicate fetch
  return Dashboard for 
}
```

```jsx filename="app/dashboard/page.js" switcher
import  from '../lib/user'

export default async function DashboardPage() {
  const user = await getUser() // Cached - same request, no duplicate fetch
  return Dashboard for 
}
```

Since `getUser` is wrapped with `React.cache`, multiple calls within the same request return the same memoized result, whether called directly in Server Components or resolved via context in Client Components.

> **Good to know**: `React.cache` is scoped to the current request only. Each request gets its own memoization scope with no sharing between requests.
