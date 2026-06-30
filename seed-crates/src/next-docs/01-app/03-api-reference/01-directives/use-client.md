---
title: use client
description: Learn how to use the use client directive to render a component on the client.
---

The `'use client'` directive declares an entry point for the components to be rendered on the **client side** and should be used when creating interactive user interfaces (UI) that require client-side JavaScript capabilities, such as state management, event handling, and access to browser APIs. This is a React feature.

> **Good to know:**
>
> You do not need to add the `'use client'` directive to every file that contains Client Components. You only need to add it to the files whose components you want to render directly within Server Components. The `'use client'` directive defines the client-server [boundary](https://nextjs.org/docs/app/building-your-application/rendering#network-boundary), and the components exported from such a file serve as entry points to the client.

## Usage

To declare an entry point for the Client Components, add the `'use client'` directive **at the top of the file**, before any imports:

```tsx filename="app/components/counter.tsx" highlight= switcher
'use client'

import  from 'react'

export default function Counter() {
  const [count, setCount] = useState(0)

  return (
    
      Count: 
       setCount(count + 1)}>Increment
    
  )
}
```

```jsx filename="app/components/counter.js" highlight= switcher
'use client'

import  from 'react'

export default function Counter() {
  const [count, setCount] = useState(0)

  return (
    
      Count: 
       setCount(count + 1)}>Increment
    
  )
}
```

When using the `'use client'` directive, the props of the Client Components must be [serializable](https://react.dev/reference/rsc/use-client#serializable-types). This means the props need to be in a format that React can serialize when sending data from the server to the client.

```tsx filename="app/components/counter.tsx" highlight= switcher
'use client'

export default function Counter() 
```

```jsx filename="app/components/counter.js" highlight= switcher
'use client'

export default function Counter() 
```

## Nesting Client Components within Server Components

Combining Server and Client Components allows you to build applications that are both performant and interactive:

1. **Server Components**: Use for static content, data fetching, and SEO-friendly elements.
2. **Client Components**: Use for interactive elements that require state, effects, or browser APIs.
3. **Component composition**: Nest Client Components within Server Components as needed for a clear separation of server and client logic.

In the following example:

- `Header` is a Server Component handling static content.
- `Counter` is a Client Component enabling interactivity within the page.

```tsx filename="app/page.tsx" highlight= switcher
import Header from './header'
import Counter from './counter' // This is a Client Component

export default function Page() 
```

```jsx filename="app/page.js" highlight= switcher
import Header from './header'
import Counter from './counter' // This is a Client Component

export default function Page() 
```

## Reference

See the [React documentation](https://react.dev/reference/rsc/use-client) for more information on `'use client'`.
