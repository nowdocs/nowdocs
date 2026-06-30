---
title: Dynamic Route Segments
nav_title: Dynamic Segments
description: Use Dynamic Segments to read URL path params and generate routes from dynamic data.
related:
  title: Next Steps
  description: For more information on what to do next, we recommend the following sections
  links:
    - app/api-reference/functions/generate-static-params
---

A URL path is a sequence of path segments. In the App Router, a segment may be **static** (a literal value matched exactly) or **dynamic** (a placeholder that captures a value from the URL). When you don't know a segment's value ahead of time, define a Dynamic Segment to create routes from dynamic data. Next.js passes the captured values to your page via the path `params` prop, either filled in at request time or prerendered at build time.

> **Good to know**: Dynamic Segments are often referred to as path params, route params, or URL params.

## Convention

A Dynamic Segment can be created by wrapping a folder's name in square brackets: `[folderName]`. For example, a blog could include the following route `app/blog/[slug]/page.js` where `[slug]` is the Dynamic Segment for blog posts.

```tsx filename="app/blog/[slug]/page.tsx" switcher
export default async function Page(: {
  params: Promise<>
}) {
  const  = await params
  return My Post: 
}
```

```jsx filename="app/blog/[slug]/page.js" switcher
export default async function Page() {
  const  = await params
  return My Post: 
}
```

Dynamic Segments are passed as the `params` prop to [`layout`](/docs/app/api-reference/file-conventions/layout), [`page`](/docs/app/api-reference/file-conventions/page), [`route`](/docs/app/api-reference/file-conventions/route), and [`generateMetadata`](/docs/app/api-reference/functions/generate-metadata#generatemetadata-function) functions.

| Route                     | Example URL | `params`        |
| ------------------------- | ----------- | --------------- |
| `app/blog/[slug]/page.js` | `/blog/a`   | `` |
| `app/blog/[slug]/page.js` | `/blog/b`   | `` |
| `app/blog/[slug]/page.js` | `/blog/c`   | `` |

Dynamic segments that appear before the [root layout](/docs/app/api-reference/file-conventions/layout#root-layout) are **root parameters**, which can additionally be read from any Server Component with [`next/root-params`](/docs/app/api-reference/functions/next-root-params).

### In Client Components

In a Client Component **page**, dynamic segments from props can be accessed using the [`use`](https://react.dev/reference/react/use) API.

```tsx filename="app/blog/[slug]/page.tsx" switcher
'use client'
import  from 'react'

export default function BlogPostPage(: {
  params: Promise<>
}) {
  const  = use(params)

  return (
    
      
    
  )
}
```

```jsx filename="app/blog/[slug]/page.js" switcher
'use client'
import  from 'react'

export default function BlogPostPage() {
  const  = use(params)

  return (
    
      
    
  )
}
```

Alternatively Client Components can use the [`useParams`](/docs/app/api-reference/functions/use-params) hook to access the `params` anywhere in the Client Component tree.

### Catch-all Segments

Dynamic Segments can be extended to **catch-all** subsequent segments by adding an ellipsis inside the brackets `[...folderName]`.

For example, `app/shop/[...slug]/page.js` will match `/shop/clothes`, but also `/shop/clothes/tops`, `/shop/clothes/tops/t-shirts`, and so on.

| Route                        | Example URL   | `params`                    |
| ---------------------------- | ------------- | --------------------------- |
| `app/shop/[...slug]/page.js` | `/shop/a`     | ``           |
| `app/shop/[...slug]/page.js` | `/shop/a/b`   | ``      |
| `app/shop/[...slug]/page.js` | `/shop/a/b/c` | `` |

### Optional Catch-all Segments

Catch-all Segments can be made **optional** by including the parameter in double square brackets: `[[...folderName]]`.

For example, `app/shop/[[...slug]]/page.js` will **also** match `/shop`, in addition to `/shop/clothes`, `/shop/clothes/tops`, `/shop/clothes/tops/t-shirts`.

The difference between **catch-all** and **optional catch-all** segments is that with optional, the route without the parameter is also matched (`/shop` in the example above).

| Route                          | Example URL   | `params`                    |
| ------------------------------ | ------------- | --------------------------- |
| `app/shop/[[...slug]]/page.js` | `/shop`       | ``       |
| `app/shop/[[...slug]]/page.js` | `/shop/a`     | ``           |
| `app/shop/[[...slug]]/page.js` | `/shop/a/b`   | ``      |
| `app/shop/[[...slug]]/page.js` | `/shop/a/b/c` | `` |

### TypeScript

When using TypeScript, you can add types for `params` depending on your configured route segment — use [`PageProps<'/route'>`](/docs/app/api-reference/file-conventions/page#page-props-helper), [`LayoutProps<'/route'>`](/docs/app/api-reference/file-conventions/layout#layout-props-helper), or [`RouteContext<'/route'>`](/docs/app/api-reference/file-conventions/route#route-context-helper) to type `params` in `page`, `layout`, and `route` respectively.

Route `params` values are typed as `string`, `string[]`, or `undefined` (for optional catch-all segments), because their values aren't known until runtime. Users can enter any URL into the address bar, and these broad types help ensure that your application code handles all these possible cases.

| Route                               | `params` Type Definition                 |
| ----------------------------------- | ---------------------------------------- |
| `app/blog/[slug]/page.js`           | ``                       |
| `app/shop/[...slug]/page.js`        | ``                     |
| `app/shop/[[...slug]]/page.js`      | ``                    |
| `app/[categoryId]/[itemId]/page.js` | `` |

If you're working on a route where `params` can only have a fixed number of valid values, such as a `[locale]` param with a known set of language codes, you can use runtime validation to handle any invalid params a user may enter, and let the rest of your application work with the narrower type from your known set.

```tsx filename="/app/[locale]/page.tsx"
import  from 'next/navigation'
import type  from '@i18n/types'
import  from '@i18n/utils'

function assertValidLocale(value: string): asserts value is Locale 

export default async function Page(props: PageProps<'/[locale]'>) {
  const  = await props.params // locale is typed as string
  assertValidLocale(locale)
  // locale is now typed as Locale
}
```

## Behavior

- Since the `params` prop is a promise. You must use `async`/`await` or React's use function to access the values.
  - In version 14 and earlier, `params` was a synchronous prop. To help with backwards compatibility, you can still access it synchronously in Next.js 15, but this behavior will be deprecated in the future.

### With Cache Components

When using [Cache Components](/docs/app/getting-started/caching) with dynamic route segments, how you handle params depends on whether you use [`generateStaticParams`](/docs/app/api-reference/functions/generate-static-params).

Without `generateStaticParams`, param values are unknown during prerendering, making params runtime data. You must wrap param access in `
    
  )
}

async function Content(: ) {
  const res = await fetch(`https://api.vercel.app/blog/$`)
  const post = await res.json()

  return (
    
      
      
    
  )
}
```

#### With `generateStaticParams`

Provide params ahead of time to prerender pages at build time. You can prerender all routes or a subset depending on your needs.

During the build process, the route is executed with each sample param to collect the HTML result. If dynamic content or runtime data are accessed incorrectly, the build will fail.

```tsx filename="app/blog/[slug]/page.tsx" highlight=
import  from 'react'

export async function generateStaticParams() {
  return [, , ]
}

export default async function Page(: PageProps<'/blog/[slug]'>) {
  const  = await params

  return (
    
      Blog Post
      
    )
  }

  return 
}

async function PrivatePost(: ) 
```

## Examples

### With `generateStaticParams`

The [`generateStaticParams`](/docs/app/api-reference/functions/generate-static-params) function can be used to [statically generate](/docs/app/glossary#prerendering) routes at build time instead of on-demand at request time.

```tsx filename="app/blog/[slug]/page.tsx" switcher
export async function generateStaticParams() {
  const posts = await fetch('https://.../posts').then((res) => res.json())

  return posts.map((post) => ())
}
```

```jsx filename="app/blog/[slug]/page.js" switcher
export async function generateStaticParams() {
  const posts = await fetch('https://.../posts').then((res) => res.json())

  return posts.map((post) => ())
}
```

When using `fetch` inside the `generateStaticParams` function, the requests are [automatically deduplicated](/docs/app/glossary#memoization). This avoids multiple network calls for the same data Layouts, Pages, and other `generateStaticParams` functions, speeding up build time.

### Dynamic GET Route Handlers with `generateStaticParams`

`generateStaticParams` also works with dynamic [Route Handlers](/docs/app/api-reference/file-conventions/route) to statically generate API responses at build time:

```ts filename="app/api/posts/[id]/route.ts" switcher
export async function generateStaticParams() {
  const posts: [] = await fetch(
    'https://api.vercel.app/blog'
  ).then((res) => res.json())

  return posts.map((post) => ({
    id: `$`,
  }))
}

export async function GET(
  request: Request,
  : RouteContext<'/api/posts/[id]'>
) {
  const  = await params
  const res = await fetch(`https://api.vercel.app/blog/$`)

  if (!res.ok) {
    return Response.json(, )
  }

  const post = await res.json()
  return Response.json(post)
}
```

```js filename="app/api/posts/[id]/route.js" switcher
export async function generateStaticParams() {
  const posts = await fetch('https://api.vercel.app/blog').then((res) =>
    res.json()
  )

  return posts.map((post) => ({
    id: `$`,
  }))
}

export async function GET(request, ) {
  const  = await params
  const res = await fetch(`https://api.vercel.app/blog/$`)

  if (!res.ok) {
    return Response.json(, )
  }

  const post = await res.json()
  return Response.json(post)
}
```

In this example, route handlers for all blog post IDs returned by `generateStaticParams` will be statically generated at build time. Requests to other IDs will be handled dynamically at request time.
