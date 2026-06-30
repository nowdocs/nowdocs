---
title: Layouts and Pages
description: Learn how to create your first pages and layouts, and link between them with the Link component.
related:
  title: API Reference
  description: Learn more about the features mentioned in this page by reading the API Reference.
  links:
    - app/getting-started/linking-and-navigating
    - app/api-reference/file-conventions/layout
    - app/api-reference/file-conventions/page
    - app/api-reference/components/link
    - app/api-reference/file-conventions/dynamic-routes
---

Next.js uses **file-system based routing**, meaning you can use folders and files to define routes. This page will guide you through how to create layouts and pages, and link between them.

## Creating a page

A **page** is UI that is rendered on a specific route. To create a page, add a [`page` file](/docs/app/api-reference/file-conventions/page) inside the `app` directory and default export a React component. For example, to create an index page (`/`):

        
      ))}
    
  )
}
```

```jsx filename="app/ui/post.js" highlight= switcher
import Link from 'next/link'
import  from '@/lib/posts'

export default async function Posts() {
  const posts = await getPosts()

  return (
    
      
    
  )
}
```

> **Good to know**: `` is the primary way to navigate between routes in Next.js. You can also use the [`useRouter` hook](/docs/app/api-reference/functions/use-router) for more advanced navigation.

## Route Props Helpers

Next.js exposes utility types that infer `params` and named slots from your route structure:

- [**PageProps**](/docs/app/api-reference/file-conventions/page#page-props-helper): Props for `page` components, including `params` and `searchParams`.
- [**LayoutProps**](/docs/app/api-reference/file-conventions/layout#layout-props-helper): Props for `layout` components, including `children` and any named slots (e.g. folders like `@analytics`).

These are globally available helpers, generated when running either `next dev`, `next build` or [`next typegen`](/docs/app/api-reference/cli/next#next-typegen-options).

```tsx filename="app/blog/[slug]/page.tsx"
export default async function Page(props: PageProps<'/blog/[slug]'>) {
  const  = await props.params
  return Blog post: 
}
```

```tsx filename="app/dashboard/layout.tsx"
export default function Layout(props: LayoutProps<'/dashboard'>) {
  return (
    
      
      
      {/*  */}
    
  )
}
```

> **Good to know**
>
> - Static routes resolve `params` to ``.
> - `PageProps`, `LayoutProps` are global helpers — no imports required.
> - Types are generated during `next dev`, `next build` or `next typegen`.
