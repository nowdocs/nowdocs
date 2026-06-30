---
title: generateMetadata
description: Learn how to add Metadata to your Next.js application for improved search engine optimization (SEO) and web shareability.
related:
  title: Next Steps
  description: View all the Metadata API options.
  links:
    - app/api-reference/file-conventions/metadata
    - app/api-reference/functions/generate-viewport
    - app/getting-started/caching
    - app/api-reference/config/next-config-js/cacheComponents
---

You can use the `metadata` object or the `generateMetadata` function to define metadata.

## The `metadata` object

To define static metadata, export a [`Metadata` object](#metadata-fields) from a `layout.js` or `page.js` file.

```tsx filename="layout.tsx | page.tsx" switcher
import type  from 'next'

export const metadata: Metadata = 

export default function Page() 
```

```jsx filename="layout.js | page.js" switcher
export const metadata = 

export default function Page() 
```

> See the [Metadata Fields](#metadata-fields) for a complete list of supported options.

## `generateMetadata` function

Dynamic metadata depends on **dynamic information**, such as the current route parameters, external data, or `metadata` in parent segments, can be set by exporting a `generateMetadata` function that returns a [`Metadata` object](#metadata-fields).

Resolving `generateMetadata` is part of rendering the page. If the page can be prerendered and `generateMetadata` doesn't introduce dynamic behavior, the resulting metadata is included in the page's initial HTML.

Otherwise the metadata resolved from `generateMetadata` [can be streamed](/docs/app/api-reference/functions/generate-metadata#streaming-metadata) after sending the initial UI.

```tsx filename="app/products/[id]/page.tsx" switcher
import type  from 'next'

type Props = {
  params: Promise<>
  searchParams: Promise<>
}

export async function generateMetadata(
  : Props,
  parent: ResolvingMetadata
): Promise
  )
}

export default function Page() 
```

The `DynamicMarker` component renders nothing but tells Next.js the page has intentional dynamic content. By wrapping it in Suspense, the static content still prerenders normally.

See [Next.js encountered runtime data in `generateMetadata()`](/docs/messages/blocking-prerender-metadata-runtime) and [Next.js encountered uncached data in `generateMetadata()`](/docs/messages/blocking-prerender-metadata-dynamic) for full fix options and trade-offs.

### Ordering

Metadata is evaluated in order, starting from the root segment down to the segment closest to the final `page.js` segment. For example:

1. `app/layout.tsx` (Root Layout)
2. `app/blog/layout.tsx` (Nested Blog Layout)
3. `app/blog/[slug]/page.tsx` (Blog Page)

### Merging

Following the [evaluation order](#ordering), Metadata objects exported from multiple segments in the same route are **shallowly** merged together to form the final metadata output of a route. Duplicate keys are **replaced** based on their ordering.

This means metadata with nested fields such as [`openGraph`](/docs/app/api-reference/functions/generate-metadata#opengraph) and [`robots`](/docs/app/api-reference/functions/generate-metadata#robots) that are defined in an earlier segment are **overwritten** by the last segment to define them.

#### Overwriting fields

```jsx filename="app/layout.js"
export const metadata = {
  title: 'Acme',
  openGraph: ,
}
```

```jsx filename="app/blog/page.js"
export const metadata = {
  title: 'Blog',
  openGraph: ,
}

// Output:
// Blog
// 
```

In the example above:

- `title` from `app/layout.js` is **replaced** by `title` in `app/blog/page.js`.
- All `openGraph` fields from `app/layout.js` are **replaced** in `app/blog/page.js` because `app/blog/page.js` sets `openGraph` metadata. Note the absence of `openGraph.description`.

If you'd like to share some nested fields between segments while overwriting others, you can pull them out into a separate variable:

```jsx filename="app/shared-metadata.js"
export const openGraphImage = 
```

```jsx filename="app/page.js"
import  from './shared-metadata'

export const metadata = {
  openGraph: ,
}
```

```jsx filename="app/about/page.js"
import  from '../shared-metadata'

export const metadata = {
  openGraph: ,
}
```

In the example above, the OG image is shared between `app/layout.js` and `app/about/page.js` while the titles are different.

#### Inheriting fields

```jsx filename="app/layout.js"
export const metadata = {
  title: 'Acme',
  openGraph: ,
}
```

```jsx filename="app/about/page.js"
export const metadata = 

// Output:
// About
// 
// 
```

**Notes**

- `title` from `app/layout.js` is **replaced** by `title` in `app/about/page.js`.
- All `openGraph` fields from `app/layout.js` are **inherited** in `app/about/page.js` because `app/about/page.js` doesn't set `openGraph` metadata.

## Version History

| Version   | Changes                                                                                                                                                 |
| --------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `v15.2.0` | Introduced streaming support to `generateMetadata`.                                                                                                     |
| `v13.2.0` | `viewport`, `themeColor`, and `colorScheme` deprecated in favor of the [`viewport` configuration](/docs/app/api-reference/functions/generate-viewport). |
| `v13.2.0` | `metadata` and `generateMetadata` introduced.                                                                                                           |
