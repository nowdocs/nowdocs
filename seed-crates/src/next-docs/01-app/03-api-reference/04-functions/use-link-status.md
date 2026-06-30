---
title: useLinkStatus
description: API Reference for the useLinkStatus hook.
related:
  title: Next Steps
  description: Learn more about the features mentioned in this page by reading the API Reference.
  links:
    - app/api-reference/components/link
    - app/api-reference/file-conventions/loading
---

The `useLinkStatus` hook lets you track the **pending** state of a `
    
  )
}
```

```jsx filename="app/hint.js" switcher
'use client'

import Link from 'next/link'
import  from 'next/link'

function Hint() {
  const  = useLinkStatus()
  return (
    
  )
}

export default function Header() 
```

> **Good to know**:
>
> - `useLinkStatus` must be used within a descendant component of a `Link` component
> - The hook is most useful when `prefetch=` is set on the `Link` component
> - If the linked route has been prefetched, the pending state will be skipped
> - When clicking multiple links in quick succession, only the last link's pending state is shown
> - This hook is not supported in the Pages Router and always returns ``
> - Inline indicators can easily introduce layout shifts. Prefer a fixed-size, always-rendered hint element and toggle its opacity, or use an animation.

## You might not need `useLinkStatus`

Before adding inline feedback, consider if:

- The destination is static and prefetched in production, so the pending phase may be skipped.
- The route has a `loading.js` file, enabling instant transitions with a route-level fallback.

Navigation is typically fast. Use `useLinkStatus` as a quick patch when you identify a slow transition, then iterate to fix the root cause with prefetching or a `loading.js` fallback.

## Parameters

```tsx
const  = useLinkStatus()
```

`useLinkStatus` does not take any parameters.

## Returns

`useLinkStatus` returns an object with a single property:

| Property | Type    | Description                                  |
| -------- | ------- | -------------------------------------------- |
| pending  | boolean | `true` before history updates, `false` after |

## Example

### Inline link hint

Add a subtle, fixed-size hint that doesn’t affect layout to confirm a click when prefetching hasn’t completed.

```tsx filename="app/components/loading-indicator.tsx" switcher
'use client'

import  from 'next/link'

export default function LoadingIndicator() {
  const  = useLinkStatus()
  return (
    
  )
}
```

```jsx filename="app/components/loading-indicator.js" switcher
'use client'

import  from 'next/link'

export default function LoadingIndicator() {
  const  = useLinkStatus()
  return (
    
  )
}
```

```tsx filename="app/shop/layout.tsx" switcher
import Link from 'next/link'
import LoadingIndicator from './components/loading-indicator'

const links = [
  ,
  ,
  ,
]

function Menubar() {
  return (
    
      
    
  )
}

export default function Layout(: ) 
    
  )
}

export default function Layout() {
  return (
    
      
      
    
  )
}
```

## Gracefully handling fast navigation

If the navigation to a new route is fast, users may see an unnecessary flash of the hint. One way to improve the user experience and only show the hint when the navigation takes time to complete is to add an initial animation delay (e.g. 100ms) and start the animation as invisible (e.g. `opacity: 0`).

```css filename="app/styles/global.css"
.link-hint 

.link-hint.is-pending 

@keyframes fadeIn {
  to 
}
@keyframes pulse {
  50% 
}
```

## Version History

| Version   | Changes                     |
| --------- | --------------------------- |
| `v15.3.0` | `useLinkStatus` introduced. |
