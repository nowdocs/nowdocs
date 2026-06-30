---
title: How Next.js preserves UI state with Activity
nav_title: Preserving UI state
description: Learn how React's Activity component preserves UI state across navigations in Next.js and how to control what resets.
related:
  title: Related
  description: Learn more about Cache Components and preserving UI state.
  links:
    - app/getting-started/caching
    - app/guides/migrating-to-cache-components
---

> **Good to know:** This guide assumes [Cache Components](/docs/app/getting-started/caching) is enabled. Enable it by setting [`cacheComponents: true`](/docs/app/api-reference/config/next-config-js/cacheComponents) in your Next config file.

Before Cache Components, preserving page-level state across navigations required workarounds like hoisting state to a [shared layout](/docs/app/getting-started/layouts-and-pages#nesting-layouts) or using an external store. With Cache Components, Next.js preserves state and DOM out of the box.

Instead of unmounting pages on navigation, Next.js hides them using React's [`
      
    </>
  )
}

function Comments(: ) {
  const comments = use(commentsPromise)
  return (
    
      {comments.map((c) => (
        
      ))}
    
  )
}

function CommentsSkeleton() 
```

The Server Component starts fetching comments immediately and passes the promise down. While hidden, the data streams at lower priority. When the user clicks "Show Comments", the `Comments` component resolves the promise with `use()` and the content appears instantly.

### Effect and media cleanup

When Activity hides content, React runs effect cleanup functions just like it does on unmount. This means timers, subscriptions, and media playback pause automatically if you have proper cleanup:

```tsx
'use client'

import  from 'react'

function LiveTimer() {
  const [count, setCount] = useState(0)

  useEffect(() => , [])

  return Count: 
}
```

For media elements like `` and ``, `display: none` does not stop playback. Add explicit cleanup with `useLayoutEffect`:

```tsx
'use client'

import  from 'react'

function VideoPlayer(: ) {
  const videoRef = useRef(null)

  useLayoutEffect(() => {
    const video = videoRef.current
    return () => 
  }, [])

  return 
}
```

When the component becomes visible again, effects re-run and playback position is preserved since the DOM node was never removed.

### Distinguishing first mount from re-show

Effects run on every hide-to-visible transition, not just the initial mount. If you need to distinguish the first mount from subsequent visibility changes, use a ref:

```tsx
'use client'

import  from 'react'

function TrackedComponent() {
  const hasMountedRef = useRef(false)

  useEffect(() => {
    if (!hasMountedRef.current)  else 
  }, [])

  return ...
}
```

The ref persists across hide/show cycles (refs aren't cleaned up), so `hasMountedRef.current` stays `true` after the first mount. Each time Activity becomes visible, the Effect runs again, but now it takes the `else` branch.

## Examples

The [Activity Patterns Demo](https://react-activity-patterns.labs.vercel.dev) ([source](https://github.com/vercel-labs/react-activity-patterns)) is a Next.js app with Cache Components enabled and three routes. Navigate between them to see state preservation in action:

- **Data** — sortable table and selectable list that keep their state across navigations, plus a reviews section that prerenders in the background
- **Forms** — filter panel with DOM state (``, checkboxes, text inputs) that persists, and a newsletter form that resets after submission using `useLayoutEffect` cleanup
- **Side Effects** — a live timer that pauses when you navigate away and resumes when you return, and a video player that auto-pauses with playback position preserved
