---
title: unauthorized.js
description: API reference for the unauthorized.js special file.
related:
  links:
    - app/api-reference/functions/unauthorized
version: experimental
---

The **unauthorized** file is used to render UI when the [`unauthorized`](/docs/app/api-reference/functions/unauthorized) function is invoked during authentication. Along with allowing you to customize the UI, Next.js will return a `401` status code.

```tsx filename="app/unauthorized.tsx" switcher
import Login from '@/app/components/Login'

export default function Unauthorized() 
```

```jsx filename="app/unauthorized.js" switcher
import Login from '@/app/components/Login'

export default function Unauthorized() 
```

## Reference

### Props

`unauthorized.js` components do not accept any props.

## Examples

### Displaying login UI to unauthenticated users

You can use [`unauthorized`](/docs/app/api-reference/functions/unauthorized) function to render the `unauthorized.js` file with a login UI.

```tsx filename="app/dashboard/page.tsx" switcher
import  from '@/app/lib/dal'
import  from 'next/navigation'

export default async function DashboardPage() {
  const session = await verifySession()

  if (!session) 

  return Dashboard
}
```

```jsx filename="app/dashboard/page.js" switcher
import  from '@/app/lib/dal'
import  from 'next/navigation'

export default async function DashboardPage() {
  const session = await verifySession()

  if (!session) 

  return Dashboard
}
```

```tsx filename="app/unauthorized.tsx" switcher
import Login from '@/app/components/Login'

export default function UnauthorizedPage() 
```

```jsx filename="app/unauthorized.js" switcher
import Login from '@/app/components/Login'

export default function UnauthorizedPage() 
```

## Version History

| Version   | Changes                       |
| --------- | ----------------------------- |
| `v15.1.0` | `unauthorized.js` introduced. |
