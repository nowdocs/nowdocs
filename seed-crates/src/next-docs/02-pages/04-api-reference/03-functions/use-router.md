---
title: useRouter
description: Learn more about the API of the Next.js Router, and access the router instance in your page with the useRouter hook.
---

If you want to access the [`router` object](#router-object) inside any function component in your app, you can use the `useRouter` hook, take a look at the following example:

```jsx
import  from 'next/router'

function ActiveLink() {
  const router = useRouter()
  const style = 

  const handleClick = (e) => 

  return (
    
      
    
  )
}

export default ActiveLink
```

> `useRouter` is a [React Hook](https://react.dev/learn#using-hooks), meaning it cannot be used with classes. You can either use [withRouter](#withrouter) or wrap your class in a function component.

## `router` object

The following is the definition of the `router` object returned by both [`useRouter`](#top) and [`withRouter`](#withrouter):

- `pathname`: `String` - The path for current route file that comes after `/pages`. Therefore, `basePath`, `locale` and trailing slash (`trailingSlash: true`) are not included.
- `query`: `Object` - The query string parsed to an object, including [dynamic route](/docs/pages/building-your-application/routing/dynamic-routes) parameters. It will be an empty object during prerendering if the page doesn't use [Server-side Rendering](/docs/pages/building-your-application/data-fetching/get-server-side-props). Defaults to ``
- `asPath`: `String` - The path as shown in the browser including the search params and respecting the `trailingSlash` configuration. `basePath` and `locale` are not included.
- `isFallback`: `boolean` - Whether the current page is in [fallback mode](/docs/pages/api-reference/functions/get-static-paths#fallback-true).
- `basePath`: `String` - The active [basePath](/docs/app/api-reference/config/next-config-js/basePath) (if enabled).
- `locale`: `String` - The active locale (if enabled).
- `locales`: `String[]` - All supported locales (if enabled).
- `defaultLocale`: `String` - The current default locale (if enabled).
- `domainLocales`: `Array<>` - Any configured domain locales.
- `isReady`: `boolean` - Whether the router fields are updated client-side and ready for use. Should only be used inside of `useEffect` methods and not for conditionally rendering on the server. See related docs for use case with [automatically statically optimized pages](/docs/pages/building-your-application/rendering/automatic-static-optimization)
- `isPreview`: `boolean` - Whether the application is currently in [preview mode](/docs/pages/guides/preview-mode).

> Using the `asPath` field may lead to a mismatch between client and server if the page is rendered using server-side rendering or [automatic static optimization](/docs/pages/building-your-application/rendering/automatic-static-optimization). Avoid using `asPath` until the `isReady` field is `true`.

The following methods are included inside `router`:

### router.push

Handles client-side transitions, this method is useful for cases where [`next/link`](/docs/pages/api-reference/components/link) is not enough.

```js
router.push(url, as, options)
```

- `url`: `UrlObject | String` - The URL to navigate to (see [Node.JS URL module documentation](https://nodejs.org/api/url.html#legacy-urlobject) for `UrlObject` properties).
- `as`: `UrlObject | String` - Optional decorator for the path that will be shown in the browser URL bar. Before Next.js 9.5.3 this was used for dynamic routes.
- `options` - Optional object with the following configuration options:
  - `scroll` - Optional boolean, controls scrolling to the top of the page after navigation. Defaults to `true`
  - [`shallow`](/docs/pages/building-your-application/routing/linking-and-navigating#shallow-routing): Update the path of the current page without rerunning [`getStaticProps`](/docs/pages/building-your-application/data-fetching/get-static-props), [`getServerSideProps`](/docs/pages/building-your-application/data-fetching/get-server-side-props) or [`getInitialProps`](/docs/pages/api-reference/functions/get-initial-props). Defaults to `false`
  - `locale` - Optional string, indicates locale of the new page

> You don't need to use `router.push` for external URLs. [window.location](https://developer.mozilla.org/docs/Web/API/Window/location) is better suited for those cases.

Navigating to `pages/about.js`, which is a predefined route:

```jsx
import  from 'next/router'

export default function Page() >
      Click me
    
  )
}
```

Navigating `pages/post/[pid].js`, which is a dynamic route:

```jsx
import  from 'next/router'

export default function Page() >
      Click me
    
  )
}
```

Redirecting the user to `pages/login.js`, useful for pages behind [authentication](/docs/pages/guides/authentication):

```jsx
import  from 'react'
import  from 'next/router'

// Here you would fetch and return the user
const useUser = () => ()

export default function Page() {
  const  = useUser()
  const router = useRouter()

  useEffect(() => {
    if (!(user || loading)) 
  }, [user, loading])

  return Redirecting...
}
```

#### Resetting state after navigation

When navigating to the same page in Next.js, the page's state **will not** be reset by default as React does not unmount unless the parent component has changed.

```jsx filename="pages/[slug].js"
import Link from 'next/link'
import  from 'react'
import  from 'next/router'

export default function Page(props) {
  const router = useRouter()
  const [count, setCount] = useState(0)
  return (
    
      Page: 
      Count: 
       setCount(count + 1)}>Increase count
       
    
  )
}
```

In the above example, navigating between `/one` and `/two` **will not** reset the count . The `useState` is maintained between renders because the top-level React component, `Page`, is the same.

If you do not want this behavior, you have a couple of options:

- Manually ensure each state is updated using `useEffect`. In the above example, that could look like:

  ```jsx
  useEffect(() => , [router.query.slug])
  ```

- Use a React `key` to [tell React to remount the component](https://react.dev/learn/rendering-lists#keeping-list-items-in-order-with-key). To do this for all pages, you can use a custom app:

  ```jsx filename="pages/_app.js"
  import  from 'next/router'

  export default function MyApp() 
  ```

#### With URL object

You can use a URL object in the same way you can use it for [`next/link`](/docs/pages/api-reference/components/link#passing-a-url-object). Works for both the `url` and `as` parameters:

```jsx
import  from 'next/router'

export default function ReadMore() {
  const router = useRouter()

  return (
     {
        router.push({
          pathname: '/post/[pid]',
          query: ,
        })
      }}
    >
      Click here to read more
    
  )
}
```

### router.replace

Similar to the `replace` prop in [`next/link`](/docs/pages/api-reference/components/link), `router.replace` will prevent adding a new URL entry into the `history` stack.

```js
router.replace(url, as, options)
```

- The API for `router.replace` is exactly the same as the API for [`router.push`](#routerpush).

Take a look at the following example:

```jsx
import  from 'next/router'

export default function Page() >
      Click me
    
  )
}
```

### router.prefetch

Prefetch pages for faster client-side transitions. This method is only useful for navigations without [`next/link`](/docs/pages/api-reference/components/link), as `next/link` takes care of prefetching pages automatically.

> This is a production only feature. Next.js doesn't prefetch pages in development.

```js
router.prefetch(url, as, options)
```

- `url` - The URL to prefetch, including explicit routes (e.g. `/dashboard`) and dynamic routes (e.g. `/product/[id]`)
- `as` - Optional decorator for `url`. Before Next.js 9.5.3 this was used to prefetch dynamic routes.
- `options` - Optional object with the following allowed fields:
  - `locale` - allows providing a different locale from the active one. If `false`, `url` has to include the locale as the active locale won't be used.

Let's say you have a login page, and after a login, you redirect the user to the dashboard. For that case, we can prefetch the dashboard to make a faster transition, like in the following example:

```jsx
import  from 'react'
import  from 'next/router'

export default function Login() {
  const router = useRouter()
  const handleSubmit = useCallback((e) => {
    e.preventDefault()

    fetch('/api/login', {
      method: 'POST',
      headers: ,
      body: JSON.stringify(),
    }).then((res) => )
  }, [])

  useEffect(() => , [router])

  return (
    
      
      Login
    
  )
}
```

### router.beforePopState

In some cases (for example, if using a [Custom Server](/docs/pages/guides/custom-server)), you may wish to listen to [popstate](https://developer.mozilla.org/docs/Web/API/Window/popstate_event) and do something before the router acts on it.

```js
router.beforePopState(cb)
```

- `cb` - The function to run on incoming `popstate` events. The function receives the state of the event as an object with the following props:
  - `url`: `String` - the route for the new state. This is usually the name of a `page`
  - `as`: `String` - the url that will be shown in the browser
  - `options`: `Object` - Additional options sent by [router.push](#routerpush)

If `cb` returns `false`, the Next.js router will not handle `popstate`, and you'll be responsible for handling it in that case. See [Disabling file-system routing](/docs/pages/guides/custom-server#disabling-file-system-routing).

You could use `beforePopState` to manipulate the request, or force a SSR refresh, as in the following example:

```jsx
import  from 'react'
import  from 'next/router'

export default function Page() {
  const router = useRouter()

  useEffect(() => {
    router.beforePopState(() => {
      // I only want to allow these two routes!
      if (as !== '/' && as !== '/other') 

      return true
    })
  }, [router])

  return Welcome to the page
}
```

### router.back

Navigate back in history. Equivalent to clicking the browser’s back button. It executes `window.history.back()`.

```jsx
import  from 'next/router'

export default function Page() >
      Click here to go back
    
  )
}
```

### router.reload

Reload the current URL. Equivalent to clicking the browser’s refresh button. It executes `window.location.reload()`.

```jsx
import  from 'next/router'

export default function Page() >
      Click here to reload
    
  )
}
```

### router.events

You can listen to different events happening inside the Next.js Router. Here's a list of supported events:

- `routeChangeStart(url, )` - Fires when a route starts to change
- `routeChangeComplete(url, )` - Fires when a route changed completely
- `routeChangeError(err, url, )` - Fires when there's an error when changing routes, or a route load is cancelled
  - `err.cancelled` - Indicates if the navigation was cancelled
- `beforeHistoryChange(url, )` - Fires before changing the browser's history
- `hashChangeStart(url, )` - Fires when the hash will change but not the page
- `hashChangeComplete(url, )` - Fires when the hash has changed but not the page

> **Good to know**: Here `url` is the URL shown in the browser, including the [`basePath`](/docs/app/api-reference/config/next-config-js/basePath).

For example, to listen to the router event `routeChangeStart`, open or create `pages/_app.js` and subscribe to the event, like so:

```jsx
import  from 'react'
import  from 'next/router'

export default function MyApp() {
  const router = useRouter()

  useEffect(() => {
    const handleRouteChange = (url, ) => {
      console.log(
        `App is changing to $ $ shallow routing`
      )
    }

    router.events.on('routeChangeStart', handleRouteChange)

    // If the component is unmounted, unsubscribe
    // from the event with the `off` method:
    return () => 
  }, [router])

  return 
}
```

> We use a [Custom App](/docs/pages/building-your-application/routing/custom-app) (`pages/_app.js`) for this example to subscribe to the event because it's not unmounted on page navigations, but you can subscribe to router events on any component in your application.

Router events should be registered when a component mounts ([useEffect](https://react.dev/reference/react/useEffect) or [componentDidMount](https://react.dev/reference/react/Component#componentdidmount) / [componentWillUnmount](https://react.dev/reference/react/Component#componentwillunmount)) or imperatively when an event happens.

If a route load is cancelled (for example, by clicking two links rapidly in succession), `routeChangeError` will fire. And the passed `err` will contain a `cancelled` property set to `true`, as in the following example:

```jsx
import  from 'react'
import  from 'next/router'

export default function MyApp() {
  const router = useRouter()

  useEffect(() => {
    const handleRouteChangeError = (err, url) => {
      if (err.cancelled) {
        console.log(`Route to $ was cancelled!`)
      }
    }

    router.events.on('routeChangeError', handleRouteChangeError)

    // If the component is unmounted, unsubscribe
    // from the event with the `off` method:
    return () => 
  }, [router])

  return 
}
```

## The `next/compat/router` export

This is the same `useRouter` hook, but can be used in both `app` and `pages` directories.

It differs from `next/router` in that it does not throw an error when the pages router is not mounted, and instead has a return type of `NextRouter | null`.
This allows developers to convert components to support running in both `app` and `pages` as they transition to the `app` router.

A component that previously looked like this:

```jsx
import  from 'next/router'
const MyComponent = () => {
  const  = useRouter()
  // ...
}
```

Will error when converted over to `next/compat/router`, as `null` cannot be destructured. Instead, developers will be able to take advantage of new hooks:

```jsx
import  from 'react'
import  from 'next/compat/router'
import  from 'next/navigation'
const MyComponent = () => {
  const router = useRouter() // may be null or a NextRouter instance
  const searchParams = useSearchParams()
  useEffect(() => {
    if (router && !router.isReady) 
    // In `app/`, searchParams will be ready immediately with the values, in
    // `pages/` it will be available after the router is ready.
    const search = searchParams.get('search')
    // ...
  }, [router, searchParams])
  // ...
}
```

This component will now work in both `pages` and `app` directories. When the component is no longer used in `pages`, you can remove the references to the compat router:

```jsx
import  from 'next/navigation'
const MyComponent = () => 
```

### Using `useRouter` outside of Next.js context in pages

Another specific use case is when rendering components outside of a Next.js application context, such as inside `getServerSideProps` on the `pages` directory. In this case, the compat router can be used to avoid errors:

```jsx
import  from 'react-dom/server'
import  from 'next/compat/router'
const MyComponent = () => 
export async function getServerSideProps() {
  const renderedComponent = renderToString()
  return {
    props: ,
  }
}
```

## Potential ESLint errors

Certain methods accessible on the `router` object return a Promise. If you have the ESLint rule, [no-floating-promises](https://typescript-eslint.io/rules/no-floating-promises) enabled, consider disabling it either globally, or for the affected line.

If your application needs this rule, you should either `void` the promise – or use an `async` function, `await` the Promise, then void the function call. **This is not applicable when the method is called from inside an `onClick` handler**.

The affected methods are:

- `router.push`
- `router.replace`
- `router.prefetch`

### Potential solutions

```jsx
import  from 'react'
import  from 'next/router'

// Here you would fetch and return the user
const useUser = () => ()

export default function Page() {
  const  = useUser()
  const router = useRouter()

  useEffect(() => {
    // disable the linting on the next line - This is the cleanest solution
    // eslint-disable-next-line no-floating-promises
    router.push('/login')

    // void the Promise returned by router.push
    if (!(user || loading)) 
    // or use an async function, await the Promise, then void the function call
    async function handleRouteChange() {
      if (!(user || loading)) 
    }
    void handleRouteChange()
  }, [user, loading])

  return Redirecting...
}
```

## withRouter

If [`useRouter`](#router-object) is not the best fit for you, `withRouter` can also add the same [`router` object](#router-object) to any component.

### Usage

```jsx
import  from 'next/router'

function Page() {
  return 
}

export default withRouter(Page)
```

### TypeScript

To use class components with `withRouter`, the component needs to accept a router prop:

```tsx
import React from 'react'
import  from 'next/router'

interface WithRouterProps 

interface MyComponentProps extends WithRouterProps 

class MyComponent extends React.Component {
  render() {
    return 
  }
}

export default withRouter(MyComponent)
```
