---
title: Adopting Partial Prefetching
nav_title: Adopting Partial Prefetching
description: Learn how to enable Partial Prefetching and what changes for `
      
    
  )
}
```

## Adopting incrementally in an existing project

If you can't enable `partialPrefetching` for the entire app at once, opt routes in one at a time with the [`prefetch`](/docs/app/api-reference/file-conventions/route-segment-config/prefetch) route segment config:

```tsx filename="app/products/[slug]/page.tsx"
export const prefetch = 'partial'

export default function Page() 
```

A `           
      
    
```

After:

```tsx filename="app/nav.tsx"
                            
       
     
```

## Next steps

- [Runtime prefetching](/docs/app/guides/runtime-prefetching) for per-link runtime prefetches and App Shells in depth.
- [`partialPrefetching` API reference](/docs/app/api-reference/config/next-config-js/partialPrefetching) for the global config flag.
- [`prefetch` API reference](/docs/app/api-reference/file-conventions/route-segment-config/prefetch) for the per-segment prefetch config.
- [`` API reference](/docs/app/api-reference/components/link#prefetch) for the per-link `prefetch` prop.
- [Instant navigation](/docs/app/guides/instant-navigation) to validate that the routes you've marked actually navigate instantly.
