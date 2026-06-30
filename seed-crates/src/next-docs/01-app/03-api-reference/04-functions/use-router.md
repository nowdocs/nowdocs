---
title: useRouter
description: API reference for the useRouter hook.
---

The `useRouter` hook allows you to programmatically change routes inside [Client Components](/docs/app/getting-started/server-and-client-components).

> **Recommendation:** Use the [`
      
    
  )
}
```

> **Good to know**: `` is wrapped in a [`Suspense` boundary](/docs/app/api-reference/file-conventions/loading#examples) because[`useSearchParams()`](/docs/app/api-reference/functions/use-search-params) causes client-side rendering up to the closest `Suspense` boundary during [prerendering](/docs/app/glossary#prerendering). [Learn more](/docs/app/api-reference/functions/use-search-params#behavior).

### Disabling scroll to top

By default, Next.js will scroll to the top of the page when navigating to a new route. You can disable this behavior by passing `scroll: false` to `router.push()` or `router.replace()`.

```tsx filename="app/example-client-component.tsx" switcher
'use client'

import  from 'next/navigation'

export default function Page() {
  const router = useRouter()

  return (
     router.push('/dashboard', )}
    >
      Dashboard
    
  )
}
```

```jsx filename="app/example-client-component.jsx" switcher
'use client'

import  from 'next/navigation'

export default function Page() {
  const router = useRouter()

  return (
     router.push('/dashboard', )}
    >
      Dashboard
    
  )
}
```

### `bfcacheId`

`router.bfcacheId` is an opaque string identifier scoped to the current route segment. It changes when the surrounding segment is freshly created by a push or replace navigation, and stays the same for back/forward navigations, `router.refresh()`, and search-param- or hash-only navigations.

The recommended use is to pass it as a React `key` to opt out of state preservation on fresh navigations, while still restoring it during a back/forward navigation:

```tsx filename="app/example/page.tsx"
'use client'

import  from 'next/navigation'

export default function Page() {
  const  = useRouter()
  return 
}
```

When `cacheComponents` is enabled, the App Router preserves Client Component state across navigations using React ``. Keying a component on `bfcacheId` resets it on each fresh navigation while still preserving its state across browser back/forward navigations.

> **Good to know**:
> Instead of `bfcacheId`, prefer resetting state explicitly in an event handler (for example, `onSubmit`) or deriving a key from your data (for example, a draft id from the server). Use `bfcacheId` only as a last resort, like when migrating an existing codebase.

## Version History

| Version   | Changes                                                           |
| --------- | ----------------------------------------------------------------- |
| `v15.4.0` | Optional `onInvalidate` callback for `router.prefetch` introduced |
| `v13.0.0` | `useRouter` from `next/navigation` introduced.                    |
