---
title: Error Handling
description: Learn how to display expected errors and handle uncaught exceptions.
related:
  title: API Reference
  description: Learn more about the features mentioned in this page by reading the API Reference.
  links:
    - app/api-reference/functions/redirect
    - app/api-reference/file-conventions/error
    - app/api-reference/functions/catchError
    - app/api-reference/functions/not-found
    - app/api-reference/file-conventions/not-found
---

Errors can be divided into two categories: [expected errors](#handling-expected-errors) and [uncaught exceptions](#handling-uncaught-exceptions). This page will walk you through how you can handle these errors in your Next.js application.

## Handling expected errors

Expected errors are those that can occur during the normal operation of the application, such as those from [server-side form validation](/docs/app/guides/forms) or failed requests. These errors should be handled explicitly and returned to the client.

### Server Functions

You can use the [`useActionState`](https://react.dev/reference/react/useActionState) hook to handle expected errors in [Server Functions](https://react.dev/reference/rsc/server-functions).

For these errors, avoid using `try`/`catch` blocks and throw errors. Instead, model expected errors as return values.

```ts filename="app/actions.ts" switcher
'use server'

export async function createPost(prevState: any, formData: FormData) {
  const title = formData.get('title')
  const content = formData.get('content')

  const res = await fetch('https://api.vercel.app/posts', {
    method: 'POST',
    body: ,
  })
  const json = await res.json()

  if (!res.ok) {
    return 
  }
}
```

```js filename="app/actions.js" switcher
'use server'

export async function createPost(prevState, formData) {
  const title = formData.get('title')
  const content = formData.get('content')

  const res = await fetch('https://api.vercel.app/posts', {
    method: 'POST',
    body: ,
  })
  const json = await res.json()

  if (!res.ok) {
    return 
  }
}
```

You can pass your action to the `useActionState` hook and use the returned `state` to display an error message.

```tsx filename="app/ui/form.tsx" highlight= switcher
'use client'

import  from 'react'
import  from '@/app/actions'

const initialState = 

export function Form() {
  const [state, formAction, pending] = useActionState(createPost, initialState)

  return (
    
      Title
      
      Content
      
      {state?.message && }
      Create Post
    
  )
}
```

```jsx filename="app/ui/form.js" highlight= switcher
'use client'

import  from 'react'
import  from '@/app/actions'

const initialState = 

export function Form() {
  const [state, formAction, pending] = useActionState(createPost, initialState)

  return (
    
      Title
      
      Content
      
      {state?.message && }
      Create Post
    
  )
}
```

### Server Components

When fetching data inside of a Server Component, you can use the response to conditionally render an error message or [`redirect`](/docs/app/api-reference/functions/redirect).

```tsx filename="app/page.tsx" switcher
export default async function Page() {
  const res = await fetch(`https://...`)
  const data = await res.json()

  if (!res.ok) 

  return '...'
}
```

```jsx filename="app/page.js" switcher
export default async function Page() {
  const res = await fetch(`https://...`)
  const data = await res.json()

  if (!res.ok) 

  return '...'
}
```

### Not found

You can call the [`notFound`](/docs/app/api-reference/functions/not-found) function within a route segment and use the [`not-found.js`](/docs/app/api-reference/file-conventions/not-found) file to show a 404 UI.

```tsx filename="app/blog/[slug]/page.tsx" switcher
import  from 'next/navigation'
import  from '@/lib/posts'

export default async function Page(: {
  params: Promise<>
}) {
  const  = await params
  const post = getPostBySlug(slug)

  if (!post) 

  return 
}
```

```jsx filename="app/blog/[slug]/page.js" switcher
import  from 'next/navigation'
import  from '@/lib/posts'

export default async function Page() {
  const  = await params
  const post = getPostBySlug(slug)

  if (!post) 

  return 
}
```

```tsx filename="app/blog/[slug]/not-found.tsx" switcher
export default function NotFound() 
```

```jsx filename="app/blog/[slug]/not-found.js" switcher
export default function NotFound() 
```

## Handling uncaught exceptions

Uncaught exceptions are unexpected errors that indicate bugs or issues that should not occur during the normal flow of your application. These should be handled by throwing errors, which will then be caught by error boundaries.

### Nested error boundaries

Next.js uses error boundaries to handle uncaught exceptions. Error boundaries catch errors in their child components and display a fallback UI instead of the component tree that crashed.

Create an error boundary by adding an [`error.js`](/docs/app/api-reference/file-conventions/error) file inside a route segment and exporting a React component:

```tsx filename="app/dashboard/error.tsx" switcher
'use client' // Error boundaries must be Client Components

import  from 'react'

export default function ErrorPage(: {
  error: Error & 
  retry: () => void
}) {
  useEffect(() => , [error])

  return (
    
      Something went wrong!
       retry()
        }
      >
        Try again
      
    
  )
}
```

```jsx filename="app/dashboard/error.js" switcher
'use client' // Error boundaries must be Client Components

import  from 'react'

export default function ErrorPage() {
  useEffect(() => , [error])

  return (
    
      Something went wrong!
       retry()
        }
      >
        Try again
      
    
  )
}
```

Errors will bubble up to the nearest parent error boundary. This allows for granular error handling by placing `error.tsx` files at different levels in the [route hierarchy](/docs/app/getting-started/project-structure#component-hierarchy).

}
```

```jsx filename="app/some-component.js" switcher
import ErrorBoundary from './custom-error-boundary'

export default function Component() 
```

Error boundaries don't catch errors inside event handlers. They're designed to catch errors [during rendering](https://react.dev/reference/react/Component#static-getderivedstatefromerror) to show a **fallback UI** instead of crashing the whole app.

In general, errors in event handlers or async code aren’t handled by error boundaries because they run after rendering.

To handle these cases, catch the error manually and store it using `useState` or `useReducer`, then update the UI to inform the user.

```tsx
'use client'

import  from 'react'

export function Button() {
  const [error, setError] = useState(null)

  const handleClick = () => {
    try  catch (reason) 
  }

  if (error) 

  return (
    
      Click me
    
  )
}
```

Note that unhandled errors inside `startTransition` from `useTransition`, will bubble up to the nearest error boundary.

```tsx
'use client'

import  from 'react'

export function Button() {
  const [pending, startTransition] = useTransition()

  const handleClick = () =>
    startTransition(() => )

  return (
    
      Click me
    
  )
}
```

### Global errors

While less common, you can handle errors in the root layout using the [`global-error.js`](/docs/app/api-reference/file-conventions/error#global-error) file, located in the root app directory, even when leveraging [internationalization](/docs/app/guides/internationalization). Global error UI must define its own `` and `` tags, since it is replacing the root layout or template when active.

```tsx filename="app/global-error.tsx" switcher
'use client' // Error boundaries must be Client Components

export default function GlobalError(: {
  error: Error & 
  retry: () => void
}) >Try again
      
    
  )
}
```

```jsx filename="app/global-error.js" switcher
'use client' // Error boundaries must be Client Components

export default function GlobalError() >Try again
      
    
  )
}
```
