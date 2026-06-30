---
title: Migrating to Cache Components
nav_title: Migrating to Cache Components
description: Learn how to migrate from route segment configs to Cache Components in Next.js.
related:
  title: Next Steps
  description: Learn about other behavior changes when Cache Components is enabled.
  links:
    - app/guides/instant-navigation
    - app/getting-started/caching
    - app/guides/preserving-ui-state
    - app/guides/incremental-static-regeneration-cache-components
    - app/api-reference/functions/generate-static-params
    - app/api-reference/config/next-config-js/cacheComponents
---

When [Cache Components](/docs/app/api-reference/config/next-config-js/cacheComponents) is enabled, route segment configs like `dynamic`, `revalidate`, and `fetchCache` are replaced by [`use cache`](/docs/app/api-reference/directives/use-cache) and [`cacheLife`](/docs/app/api-reference/functions/cacheLife).

The migration is driven by **instant navigation validation**. With Cache Components enabled, Next.js validates in development whether navigating into each route renders instantly, and surfaces the code that would block it as an error or insight.

## Use the adoption skill (recommended)

The [`next-cache-components-adoption`](https://github.com/vercel/next.js/tree/canary/skills/next-cache-components-adoption) skill drives this migration with a coding agent, one feature at a time, checking in at every feature boundary. It supports two modes:

- **Incremental.** Opens a single mechanical PR that opts every route out of validation, then ships each feature as a follow-up PR.
- **Direct.** Adopts every route in place on one branch.

Install the skill:

```bash filename="Terminal"
npx skills add vercel/next.js --skill next-cache-components-adoption
```

Prompt the agent:

```text
Adopt Cache Components in this project using the next-cache-components-adoption skill.
```

## Or migrate by hand

Start by removing the route segment configs (`dynamic`, `revalidate`, `fetchCache`), then follow the validation insights and errors. Each one names the code to fix, most often uncached data to cache with [`use cache`](/docs/app/api-reference/directives/use-cache) or runtime data to wrap in [`', highlight: true },
    ]}
  />

Each card is clickable and opens a page with patterns, code samples, and trade-offs. Work through the insights and errors until they're gone. For the full validation workflow, the DevTools, and CI testing, see the [instant navigation guide](/docs/app/guides/instant-navigation).

Insights don't show up in the HTTP response. An offending route still returns `200` with rendered HTML in dev. The insight only appears in the dev overlay, the dev-server log, or the [MCP `get_errors` tool](/docs/app/guides/mcp). To see them, read the overlay (or query the MCP).

## Opting out of validation

A validation insight or error tells you a route won't render instantly. Resolve it by following the recommendation: cache the data with [`use cache`](/docs/app/api-reference/directives/use-cache) or wrap it in [`
  )
}

async function Post(: Pick
  )
}

async function Post() {
  const  = await params
  // ...
}
```

The same applies to client hooks that read the route. When the route's pathname is fully known, they resolve during prerendering and need no boundary. When it depends on dynamic params not yet known, they suspend, wherever the component sits. A nav or breadcrumb in a shared layout, for instance, suspends while Next.js generates the App Shell for any route below it that has dynamic params. Wrap the component that reads the hook in `
  )
}

async function Dashboard() 
```

```jsx filename="app/page.js" switcher
import  from 'next/headers'
import  from 'react'

// After - the page prerenders; only Dashboard streams at request time
export default function Page() 

async function Dashboard() 
```

Your page receives `params` and `searchParams` as props, and both are promises. Apply the same pattern: pass the promise straight through to the `
  )
}

async function Results(: Pick
  )
}

async function Results() {
  const  = await searchParams
  // ...
}
```

> **Good to know**: When a cookie or header value drives an attribute on the `` element in the root layout (`lang`, `dir`, `data-theme`, etc.), reading it on the server makes the whole subtree request-bound, so there's no child to wrap in `
  )
}

async function Connection() 

export default function Page() 
```

See [`generateMetadata` with Cache Components](/docs/app/api-reference/functions/generate-metadata#with-cache-components) and [`generateViewport` with Cache Components](/docs/app/api-reference/functions/generate-viewport#with-cache-components) for the full set of fix options and trade-offs.

## `runtime = 'edge'`

**Not supported.** Cache Components requires the Node.js runtime. Switch to the Node.js runtime (the default) by removing the [deprecated](/docs/messages/edge-runtime-deprecated) `runtime = 'edge'` export. If you need edge behavior for specific routes, use [Proxy](/docs/app/api-reference/file-conventions/proxy) instead.

## `experimental_ppr`

**Removed. Enable `cacheComponents` instead.** Next.js 16 removes the experimental Partial Prerendering flag (`experimental.ppr`) and the `experimental_ppr` route segment config. Partial Prerendering is now part of [Cache Components](/docs/app/api-reference/config/next-config-js/cacheComponents), so remove `experimental.ppr` from `next.config` and `experimental_ppr` from your segments. A [codemod](/docs/app/guides/upgrading/codemods#remove-experimental_ppr-route-segment-config-from-app-router-pages-and-layouts) removes the segment config for you.

```tsx filename="app/page.tsx" switcher
// Before - no longer needed
export const experimental_ppr = true

export default function Page() 
```

```jsx filename="app/page.js" switcher
// Before - no longer needed
export const experimental_ppr = true

export default function Page() 
```

```tsx filename="app/page.tsx" switcher
// After - remove it; cacheComponents enables Partial Prerendering
export default function Page() 
```

```jsx filename="app/page.js" switcher
// After - remove it; cacheComponents enables Partial Prerendering
export default function Page() 
```

## UI state preservation

**Component state now persists across navigations.** With Cache Components, Next.js preserves routes using React's [``](https://react.dev/reference/react/Activity) component in [`"hidden"`](https://react.dev/reference/react/Activity#activity) mode instead of unmounting them. Effects clean up and re-run normally, but `useState` values, form inputs, and scroll position are no longer reset when navigating away and back.

If your code relied on unmounting to clear state, you may need to add explicit reset logic:

- **Dropdowns and popovers**: stay open when navigating back. Close them in a `useLayoutEffect` cleanup function.
- **Dialogs with initialization logic**: Effects that depend on dialog state (like focusing an input) won't re-fire if the state was preserved. Derive dialog state from the URL instead.
- **Forms after submission**: input values and `useActionState` results (success/error messages) persist when returning. Reset in the submit handler or user action when possible, otherwise use a cleanup effect.

See [Preserving UI state across navigations](/docs/app/guides/preserving-ui-state) for detailed examples of each pattern.
