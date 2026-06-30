---
title: useSearchParams
description: API Reference for the useSearchParams hook in the Pages Router.
---

`useSearchParams` is a hook that lets you read the current URL's **query string**.

`useSearchParams` returns a **read-only** version of the [`URLSearchParams`](https://developer.mozilla.org/docs/Web/API/URLSearchParams) interface.

```tsx filename="pages/dashboard.tsx" switcher
import  from 'next/navigation'

export default function Dashboard() {
  const searchParams = useSearchParams()

  if (!searchParams) 

  const search = searchParams.get('search')

  // URL -> `/dashboard?search=my-project`
  // `search` -> 'my-project'
  return <>Search: </>
}
```

```jsx filename="pages/dashboard.js" switcher
import  from 'next/navigation'

export default function Dashboard() {
  const searchParams = useSearchParams()

  if (!searchParams) 

  const search = searchParams.get('search')

  // URL -> `/dashboard?search=my-project`
  // `search` -> 'my-project'
  return <>Search: </>
}
```

## Parameters

```tsx
const searchParams = useSearchParams()
```

`useSearchParams` does not take any parameters.

## Returns

`useSearchParams` returns a **read-only** version of the [`URLSearchParams`](https://developer.mozilla.org/docs/Web/API/URLSearchParams) interface, or `null` during [prerendering](#behavior-during-prerendering).

The interface includes utility methods for reading the URL's query string:

- [`URLSearchParams.get()`](https://developer.mozilla.org/docs/Web/API/URLSearchParams/get): Returns the first value associated with the search parameter. For example:

  | URL                  | `searchParams.get("a")`                                                                                         |
  | -------------------- | --------------------------------------------------------------------------------------------------------------- |
  | `/dashboard?a=1`     | `'1'`                                                                                                           |
  | `/dashboard?a=`      | `''`                                                                                                            |
  | `/dashboard?b=3`     | `null`                                                                                                          |
  | `/dashboard?a=1&a=2` | `'1'` _- use [`getAll()`](https://developer.mozilla.org/docs/Web/API/URLSearchParams/getAll) to get all values_ |

- [`URLSearchParams.has()`](https://developer.mozilla.org/docs/Web/API/URLSearchParams/has): Returns a boolean value indicating if the given parameter exists. For example:

  | URL              | `searchParams.has("a")` |
  | ---------------- | ----------------------- |
  | `/dashboard?a=1` | `true`                  |
  | `/dashboard?b=3` | `false`                 |

- Learn more about other **read-only** methods of [`URLSearchParams`](https://developer.mozilla.org/docs/Web/API/URLSearchParams), including the [`getAll()`](https://developer.mozilla.org/docs/Web/API/URLSearchParams/getAll), [`keys()`](https://developer.mozilla.org/docs/Web/API/URLSearchParams/keys), [`values()`](https://developer.mozilla.org/docs/Web/API/URLSearchParams/values), [`entries()`](https://developer.mozilla.org/docs/Web/API/URLSearchParams/entries), [`forEach()`](https://developer.mozilla.org/docs/Web/API/URLSearchParams/forEach), and [`toString()`](https://developer.mozilla.org/docs/Web/API/URLSearchParams/toString).

> **Good to know**: `useSearchParams` is a [React Hook](https://react.dev/learn#using-hooks) and cannot be used with classes.

## Behavior

### Behavior during prerendering

For pages that are [statically optimized](/docs/pages/building-your-application/rendering/automatic-static-optimization) (not using `getServerSideProps`), `useSearchParams` will return `null` during prerendering. After hydration, the value will be updated to the actual search params.

This is because search params cannot be known during static generation as they depend on the request.

```tsx filename="pages/dashboard.tsx" switcher
import  from 'next/navigation'

export default function Dashboard() {
  const searchParams = useSearchParams()

  if (!searchParams) 

  const search = searchParams.get('search')

  return <>Search: </>
}
```

```jsx filename="pages/dashboard.js" switcher
import  from 'next/navigation'

export default function Dashboard() {
  const searchParams = useSearchParams()

  if (!searchParams) 

  const search = searchParams.get('search')

  return <>Search: </>
}
```

### Using with `getServerSideProps`

When using [`getServerSideProps`](/docs/pages/building-your-application/data-fetching/get-server-side-props), the page is server-rendered on each request and `useSearchParams` will return the actual search params immediately:

```tsx filename="pages/dashboard.tsx" switcher
import  from 'next/navigation'

export default function Dashboard() {
  const searchParams = useSearchParams()

  // With getServerSideProps, this fallback is never rendered because
  // searchParams is always available on the server. However, keeping
  // the fallback allows this component to be reused on other pages
  // that may not use getServerSideProps.
  if (!searchParams) 

  const search = searchParams.get('search')

  return <>Search: </>
}

export async function getServerSideProps() {
  return { props:  }
}
```

```jsx filename="pages/dashboard.js" switcher
import  from 'next/navigation'

export default function Dashboard() {
  const searchParams = useSearchParams()

  // With getServerSideProps, this fallback is never rendered because
  // searchParams is always available on the server. However, keeping
  // the fallback allows this component to be reused on other pages
  // that may not use getServerSideProps.
  if (!searchParams) 

  const search = searchParams.get('search')

  return <>Search: </>
}

export async function getServerSideProps() {
  return { props:  }
}
```

## Examples

### Updating search params

You can use the [`useRouter`](/docs/pages/api-reference/functions/use-router) hook to update search params:

```tsx filename="pages/dashboard.tsx" switcher
import  from 'next/router'
import  from 'next/navigation'
import  from 'react'

export default function Dashboard() {
  const router = useRouter()
  const searchParams = useSearchParams()

  const createQueryString = useCallback(
    (name: string, value: string) => ,
    [searchParams]
  )

  if (!searchParams) 

  return (
    <>
      Sort By
       }
      >
        ASC
      
       }
      >
        DESC
      
    </>
  )
}
```

```jsx filename="pages/dashboard.js" switcher
import  from 'next/router'
import  from 'next/navigation'
import  from 'react'

export default function Dashboard() {
  const router = useRouter()
  const searchParams = useSearchParams()

  const createQueryString = useCallback(
    (name, value) => ,
    [searchParams]
  )

  if (!searchParams) 

  return (
    <>
      Sort By
       }
      >
        ASC
      
       }
      >
        DESC
      
    </>
  )
}
```

### Sharing components with App Router

`useSearchParams` from `next/navigation` works in both the Pages Router and App Router. This allows you to create shared components that work in either context:

```tsx filename="components/search-bar.tsx" switcher
import  from 'next/navigation'

// This component works in both pages/ and app/
export function SearchBar() {
  const searchParams = useSearchParams()

  if (!searchParams) 

  const search = searchParams.get('search') ?? ''

  return 
}
```

```jsx filename="components/search-bar.js" switcher
import  from 'next/navigation'

// This component works in both pages/ and app/
export function SearchBar() {
  const searchParams = useSearchParams()

  if (!searchParams) 

  const search = searchParams.get('search') ?? ''

  return 
}
```

> **Good to know**: When using this component in the App Router, wrap it in a `` boundary for [prerendering](/docs/app/api-reference/functions/use-search-params#prerendering) support.

## Version History

| Version   | Changes                       |
| --------- | ----------------------------- |
| `v13.0.0` | `useSearchParams` introduced. |
