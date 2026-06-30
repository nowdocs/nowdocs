---
title: not-found.js
description: API reference for the not-found.js file.
---

Next.js provides two conventions to handle not found cases:

- **`not-found.js`**: Used when you call the [`notFound`](/docs/app/api-reference/functions/not-found) function in a route segment.
- **`global-not-found.js`**: Used to define a global 404 page for unmatched routes across your entire app. This is handled at the routing level and doesn't depend on rendering a layout or page.

## `not-found.js`

The **not-found** file is used to render UI when the [`notFound`](/docs/app/api-reference/functions/not-found) function is thrown within a route segment. Along with serving a custom UI, Next.js will return a `200` HTTP status code for streamed responses, and `404` for non-streamed responses (see [Status Codes](/docs/app/api-reference/file-conventions/loading#status-codes) for details about SEO).

```tsx filename="app/not-found.tsx" switcher
import Link from 'next/link'

export default function NotFound() 
```

```jsx filename="app/blog/not-found.js" switcher
import Link from 'next/link'

export default function NotFound() 
```

In the [component hierarchy](/docs/app/getting-started/project-structure#component-hierarchy), `not-found.js` renders between `loading.js` and `page.js`. It is wrapped by the `
      
    
  )
}
```

```jsx filename="app/not-found.jsx" switcher
import Link from 'next/link'
import  from 'next/headers'

export default async function NotFound() {
  const headersList = await headers()
  const domain = headersList.get('host')
  const data = await getSiteData(domain)
  return (
    
      Not Found: 
      Could not find requested resource
      
        View 
      
    
  )
}
```

If you need to use Client Component hooks like `usePathname` to display content based on the path, you must fetch data on the client-side instead.

### Metadata

For `global-not-found.js`, you can export a `metadata` object or a [`generateMetadata`](/docs/app/api-reference/functions/generate-metadata) function to customize the ``, ``, and other head tags for your 404 page:

> **Good to know**: Next.js automatically injects `` for pages that return a 404 status code, including `global-not-found.js` pages.

```tsx filename="app/global-not-found.tsx" switcher
import type  from 'next'

export const metadata: Metadata = 

export default function GlobalNotFound() 
```

```jsx filename="app/global-not-found.js" switcher
export const metadata = 

export default function GlobalNotFound() 
```

## Version History

| Version   | Changes                                             |
| --------- | --------------------------------------------------- |
| `v15.4.0` | `global-not-found.js` introduced (experimental).    |
| `v13.3.0` | Root `app/not-found` handles global unmatched URLs. |
| `v13.0.0` | `not-found` introduced.                             |
