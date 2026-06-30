---
title: Prefetching
description: Learn how to configure prefetching in Next.js
---

Prefetching makes navigating between different routes in your application feel instant. Next.js tries to intelligently prefetch by default, based on the links used in your application code.

This guide will explain how prefetching works and show common implementation patterns:

- [Automatic prefetch](#automatic-prefetch)
- [Manual prefetch](#manual-prefetch)
- [Hover-triggered prefetch](#hover-triggered-prefetch)
- [Extending or ejecting link](#extending-or-ejecting-link)
- [Disabled prefetch](#disabled-prefetch)

> **Using Cache Components?** With [`cacheComponents`](/docs/app/api-reference/config/next-config-js/cacheComponents) enabled, `
}
```

```jsx filename="app/ui/nav-link.js" switcher
import Link from 'next/link'

export default function NavLink() 
```

| **Context**       | **Prefetched payload**           | **Client Cache TTL**                                                                              |
| ----------------- | -------------------------------- | ------------------------------------------------------------------------------------------------- |
| No `loading.js`   | Entire page                      | 5 min ([`staleTimes.static`](/docs/app/api-reference/config/next-config-js/staleTimes))           |
| With `loading.js` | Layout to first loading boundary | Off by default ([`staleTimes.dynamic`](/docs/app/api-reference/config/next-config-js/staleTimes)) |

Automatic prefetching runs only in production. Disable with `prefetch=` or use the wrapper in [Disabled Prefetch](#disabled-prefetch).

## Manual prefetch

To do manual prefetching, import the `useRouter` hook from `next/navigation`, and call `router.prefetch()` to warm routes outside the viewport or in response to analytics, hover, scroll, etc.

```tsx
'use client'

import  from 'next/navigation'
import  from '@components/link'

export function PricingCard() >
      
      
    
  )
}
```

To prefetch a URL when a component loads, see [Extending or ejecting link](#extending-or-ejecting-link).

## Hover-triggered prefetch

> **Proceed with caution:** Extending `Link` opts you into maintaining prefetching, cache invalidation, and accessibility concerns. Proceed only if defaults are insufficient.

Next.js tries to do the right prefetching by default, but power users can eject and modify based on their needs. You have the control between performance and resource consumption.

For example, you might have to only trigger prefetches on hover, instead of when entering the viewport (the default behavior):

```tsx
'use client'

import Link from 'next/link'
import  from 'react'

export function HoverPrefetchLink(: ) 
```

`prefetch=` restores default (static) prefetching once the user shows intent.

## Extending or ejecting link

You can extend the `
```

However, this means static routes will only be fetched on click, and dynamic routes will wait for the server to render before navigating.

To reduce resource usage without disabling prefetch entirely, you can defer prefetching until the user hovers over a link. This targets only links the user is likely to visit.

```tsx filename="app/ui/hover-prefetch-link.tsx" switcher
'use client'

import Link from 'next/link'
import  from 'react'

export function HoverPrefetchLink(: ) 
```

```jsx filename="app/ui/hover-prefetch-link.js" switcher
'use client'

import Link from 'next/link'
import  from 'react'

export function HoverPrefetchLink() 
```
