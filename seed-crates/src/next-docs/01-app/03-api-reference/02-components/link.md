---
title: Link Component
description: Enable fast client-side navigation with the built-in `next/link` component.
---

`
}
```

```jsx filename="app/page.js" switcher
import Link from 'next/link'

export default function Page() 
```

}
```

```jsx filename="pages/index.js" switcher
import Link from 'next/link'

export default function Home() 
```

## Reference

The following props can be passed to the `

> **Good to know**: `` tag attributes such as `className` or `target="_blank"` can be added to `
  )
}
```

```jsx filename="app/page.js" switcher
import Link from 'next/link'

// Navigate to /about?name=test
export default function Page() 
```

  )
}
```

```jsx filename="pages/index.js" switcher
import Link from 'next/link'

// Navigate to /about?name=test
export default function Home() 
```

### `replace`

**Defaults to `false`.** When `true`, `next/link` will replace the current history state instead of adding a new URL into the [browser's history](https://developer.mozilla.org/docs/Web/API/History_API) stack.

  )
}
```

```jsx filename="app/page.js" switcher
import Link from 'next/link'

export default function Page() 
```

  )
}
```

```jsx filename="pages/index.js" switcher
import Link from 'next/link'

export default function Home() 
```

### `scroll`

**Defaults to `true`.** The default scrolling behavior of `
  )
}
```

```jsx filename="app/page.js" switcher
import Link from 'next/link'

export default function Page() 
```

  )
}
```

```jsx filename="pages/index.js" switcher
import Link from 'next/link'

export default function Home() 
```

### `prefetch`

  )
}
```

```jsx filename="app/page.js" switcher
import Link from 'next/link'

export default function Page() 
```

  )
}
```

```jsx filename="pages/index.js" switcher
import Link from 'next/link'

export default function Home() 
```

### `shallow`

Update the path of the current page without rerunning [`getStaticProps`](/docs/pages/building-your-application/data-fetching/get-static-props), [`getServerSideProps`](/docs/pages/building-your-application/data-fetching/get-server-side-props) or [`getInitialProps`](/docs/pages/api-reference/functions/get-initial-props). Defaults to `false`.

```tsx filename="pages/index.tsx" switcher
import Link from 'next/link'

export default function Home() 
```

```jsx filename="pages/index.js" switcher
import Link from 'next/link'

export default function Home() 
```

### `locale`

The active locale is automatically prepended. `locale` allows for providing a different locale. When `false` `href` has to include the locale as the default behavior is disabled.

```tsx filename="pages/index.tsx" switcher
import Link from 'next/link'

export default function Home() {
  return (
    <>
      
      

      
      

      
      
    </>
  )
}
```

```jsx filename="pages/index.js" switcher
import Link from 'next/link'

export default function Home() {
  return (
    <>
      
      

      
      

      
      
    </>
  )
}
```

### `as`

Optional decorator for the path that will be shown in the browser URL bar. Before Next.js 9.5.3 this was used for dynamic routes, check our [previous docs](https://github.com/vercel/next.js/blob/v9.5.2/docs/api-reference/next/link.md#dynamic-routes) to see how it worked.

When this path differs from the one provided in `href` the previous `href`/`as` behavior is used as shown in the [previous docs](https://github.com/vercel/next.js/blob/v9.5.2/docs/api-reference/next/link.md#dynamic-routes).

### `onNavigate`

An event handler called during client-side navigation. The handler receives an event object that includes a `preventDefault()` method, allowing you to cancel the navigation if needed.

```tsx filename="app/page.tsx" switcher
import Link from 'next/link'

export default function Page() 
```

```jsx filename="app/page.js" switcher
import Link from 'next/link'

export default function Page() 
```

> **Good to know**: While `onClick` and `onNavigate` may seem similar, they serve different purposes. `onClick` executes for all click events, while `onNavigate` only runs during client-side navigation. Some key differences:
>
> - When using modifier keys (`Ctrl`/`Cmd` + Click), `onClick` executes but `onNavigate` doesn't since Next.js prevents default navigation for new tabs.
> - External URLs won't trigger `onNavigate` since it's only for client-side and same-origin navigations.
> - Links with the `download` attribute will work with `onClick` but not `onNavigate` since the browser will treat the linked URL as a download.

### `transitionTypes`

  )
}
```

```jsx filename="app/page.js" switcher
import Link from 'next/link'

export default function Page() 
```

## Examples

The following examples demonstrate how to use the `
        
      ))}
    
  )
}
```

```jsx filename="app/blog/post-list.js" switcher
import Link from 'next/link'

export default function PostList() {
  return (
    
      
    
  )
}
```

### Checking active links

You can use [`usePathname()`](/docs/app/api-reference/functions/use-pathname) to determine if a link is active. For example, to add a class to the active link, you can check if the current `pathname` matches the `href` of the link:

```tsx filename="app/ui/nav-links.tsx" switcher
'use client'

import  from 'next/navigation'
import Link from 'next/link'

export function Links() 
```

```jsx filename="app/ui/nav-links.js" switcher
'use client'

import  from 'next/navigation'
import Link from 'next/link'

export function Links() 
```

        
      ))}
    
  )
}
```

```jsx filename="pages/blog/index.js" switcher
import Link from 'next/link'

function Posts() {
  return (
    
      
    
  )
}

export default Posts
```

### Scrolling to an `id`

If you'd like to scroll to a specific `id` on navigation, you can append your URL with a `#` hash link or just pass a hash link to the `href` prop. This is possible since `

// Output
Settings
```

      
      
        
      
    
  )
}

export default Home
```

```jsx filename="pages/index.js" switcher
import Link from 'next/link'

function Home() 

export default Home
```

The above example has a link to:

- A predefined route: `/about?name=test`
- A [dynamic route](/docs/pages/building-your-application/routing/dynamic-routes#convention): `/blog/my-post`

You can use every property as defined in the [Node.js URL module documentation](https://nodejs.org/api/url.html#url_url_strings_and_url_objects).

### Replace the URL instead of push

The default behavior of the `Link` component is to `push` a new URL into the `history` stack. You can use the `replace` prop to prevent adding a new entry, as in the following example:

  )
}
```

```jsx filename="app/page.js" switcher
import Link from 'next/link'

export default function Page() 
```

  )
}
```

```jsx filename="pages/index.js" switcher
import Link from 'next/link'

export default function Home() 
```

### Disable scrolling to the top of the page

  )
}
```

```tsx filename="app/page.tsx" switcher
import Link from 'next/link'

export default function Page() 
```

Using `router.push()` or `router.replace()`:

```jsx
// useRouter
import  from 'next/navigation'

const router = useRouter()

router.push('/dashboard', )
```

  )
}
```

```tsx filename="pages/index.tsx" switcher
import Link from 'next/link'

export default function Home() 
```

### Scroll offset with sticky headers

Because Next.js skips sticky and fixed positioned elements when finding the scroll target, content may end up behind a sticky header after navigation. For example, if your layout has a sticky header:

```tsx filename="app/layout.tsx" switcher
import './globals.css'

export default function RootLayout(: ) {
  return (
    
      
        
          
        
        
      
    
  )
}
```

```jsx filename="app/layout.js" switcher
import './globals.css'

export default function RootLayout() {
  return (
    
      
        
          
        
        
      
    
  )
}
```

You can account for its height using [`scroll-padding-top`](https://developer.mozilla.org/en-US/docs/Web/CSS/scroll-padding-top) on the scroll container:

```css filename="app/globals.css"
html 
```

This is a browser CSS property that offsets scroll-based positioning. It applies whenever Next.js uses the native [`scrollIntoView()`](https://developer.mozilla.org/en-US/docs/Web/API/Element/scrollIntoView) API, including hash fragment (`#id`) navigation. Alternatively, you can use [`scroll-margin-top`](https://developer.mozilla.org/en-US/docs/Web/CSS/scroll-margin-top) on individual target elements instead of setting a global offset.

### Prefetching links in Proxy

It's common to use [Proxy](/docs/app/api-reference/file-conventions/proxy) for authentication or other purposes that involve rewriting the user to a different page. In order for the `
  )
}
```

```js filename="app/page.js" switcher
'use client'

import Link from 'next/link'
import useIsAuthed from './hooks/useIsAuthed' // Your auth hook

export default function Page() 
```

  )
}
```

```js filename="pages/index.js" switcher
'use client'

import Link from 'next/link'
import useIsAuthed from './hooks/useIsAuthed' // Your auth hook

export default function Home() 
```

> **Good to know**: If you're using [Dynamic Routes](/docs/pages/building-your-application/routing/dynamic-routes#convention), you'll need to adapt your `as` and `href` props. For example, if you have a Dynamic Route like `/dashboard/authed/[user]` that you want to present differently via proxy, you would write: ``.

  )
}
```

```jsx filename="app/components/custom-link.js" switcher
'use client'

import Link from 'next/link'
import  from '../contexts/navigation-blocker'

export function CustomLink() {
  const  = useNavigationBlocker()

  return (
    
  )
}
```

Create a navigation component:

```tsx filename="app/components/nav.tsx" switcher
'use client'

import  from './custom-link'

export default function Nav() 
```

```jsx filename="app/components/nav.js" switcher
'use client'

import  from './custom-link'

export default function Nav() 
```

Finally, wrap your app with the `NavigationBlockerProvider` in the root layout and use the components in your page:

```tsx filename="app/layout.tsx" switcher
import  from './contexts/navigation-blocker'

export default function RootLayout(: ) 
```

```jsx filename="app/layout.js" switcher
import  from './contexts/navigation-blocker'

export default function RootLayout() 
```

Then, use the `Nav` and `Form` components in your page:

```tsx filename="app/page.tsx" switcher
import Nav from './components/nav'
import Form from './components/form'

export default function Page() {
  return (
    
      

## Version history

| Version   | Changes                                                                                                                                                                      |
| --------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `v16.2.0` | Add `transitionTypes` prop.                                                                                                                                                  |
| `v15.4.0` | Add `auto` as an alias to the default `prefetch` behavior.                                                                                                                   |
| `v15.3.0` | Add `onNavigate` API                                                                                                                                                         |
| `v13.0.0` | No longer requires a child `` tag. A [codemod](/docs/app/guides/upgrading/codemods#remove-a-tags-from-link-components) is provided to automatically update your codebase. |
| `v10.0.0` | `href` props pointing to a dynamic route are automatically resolved and no longer require an `as` prop.                                                                      |
| `v8.0.0`  | Improved prefetching performance.                                                                                                                                            |
| `v1.0.0`  | `next/link` introduced.                                                                                                                                                      |
