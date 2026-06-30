---
title: Parallel Routes
description: Simultaneously render one or more pages in the same view that can be navigated independently. A pattern for highly dynamic applications.
related:
  links:
    - app/api-reference/file-conventions/default
---

Parallel Routes allows you to simultaneously or conditionally render one or more pages within the same layout. They are useful for highly dynamic sections of an app, such as dashboards and feeds on social sites.

For example, considering a dashboard, you can use parallel routes to simultaneously render the `team` and `analytics` pages:

        
      
      
    </>
  )
}
```

```jsx filename="app/@analytics/layout.js" switcher
import Link from 'next/link'

export default function Layout() {
  return (
    <>
      
        
        
      
      
    </>
  )
}
```

### Modals

Parallel Routes can be used together with [Intercepting Routes](/docs/app/api-reference/file-conventions/intercepting-routes) to create modals that support deep linking. This allows you to solve common challenges when building modals, such as:

- Making the modal content **shareable through a URL**.
- **Preserving context** when the page is refreshed, instead of closing the modal.
- **Closing the modal on backwards navigation** rather than going to the previous route.
- **Reopening the modal on forwards navigation**.

Consider the following UI pattern, where a user can open a login modal from a layout using client-side navigation, or access a separate `/login` page:

  )
}
```

```jsx filename="app/@auth/(.)login/page.js" switcher
import  from '@/app/ui/modal'
import  from '@/app/ui/login'

export default function Page() 
```

> **Good to know:**
>
> - The convention `(.)` is used for intercepting routes. See [Intercepting Routes](/docs/app/api-reference/file-conventions/intercepting-routes#convention) docs for more information.
> - By separating the `
      
      
      
    </>
  )
}
```

```jsx filename="app/layout.js" switcher
import Link from 'next/link'

export default function Layout() {
  return (
    <>
      
        
      
      
      
    </>
  )
}
```

When the user clicks the `
      
    </>
  )
}
```

```jsx filename="app/ui/modal.js" switcher
import Link from 'next/link'

export function Modal() {
  return (
    <>
      
      
    </>
  )
}
```

```tsx filename="app/@auth/page.tsx" switcher
export default function Page() 
```

```jsx filename="app/@auth/page.js" switcher
export default function Page() 
```

Or if navigating to any other page (such as `/foo`, `/foo/bar`, etc), you can use a catch-all slot:

```tsx filename="app/@auth/[...catchAll]/page.tsx" switcher
export default function CatchAll() 
```

```jsx filename="app/@auth/[...catchAll]/page.js" switcher
export default function CatchAll() 
```

> **Good to know:**
>
> - We use a catch-all route in our `@auth` slot to close the modal because of how parallel routes behave. Since client-side navigations to a route that no longer match the slot will remain visible, we need to match the slot to a route that returns `null` to close the modal.
> - Other examples could include opening a photo modal in a gallery while also having a dedicated `/photo/[id]` page, or opening a shopping cart in a side modal.
> - [View an example](https://github.com/vercel-labs/nextgram) of modals with Intercepted and Parallel Routes.

### Loading and Error UI

Parallel Routes can be streamed independently, allowing you to define independent error and loading states for each route:

See the [Loading UI](/docs/app/api-reference/file-conventions/loading) and [Error Handling](/docs/app/getting-started/error-handling) documentation for more information.
