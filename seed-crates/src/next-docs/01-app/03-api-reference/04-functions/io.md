---
title: io
description: API Reference for the io function.
---

When [Cache Components](/docs/app/api-reference/config/next-config-js/cacheComponents) is enabled, Next.js needs you to decide whether to capture a synchronous value like `new Date()` or `Math.random()` once and reuse it for every visitor, or produce it fresh for each request.

To capture the value in the [static shell](/docs/app/glossary#static-shell), wrap it in [`"use cache"`](/docs/app/api-reference/directives/use-cache). To keep it out of the static shell, use `await io()`, which suspends during prerendering.

During a request, inside cached scopes, in the browser, and apps without Cache Components (including the Pages Router), calling `io()` resolves immediately.

## Usage

Call `io()` to inform Next.js that an IO operation follows.

In a Server Component, call `await io()` before reading a value like `new Date()`, `Math.random()`, `crypto.randomUUID()`, or a synchronous database driver such as [`node:sqlite`](https://nodejs.org/api/sqlite.html):

```tsx filename="app/page.tsx" highlight= switcher
import  from 'react'
import  from 'next/cache'

export default function Page() 

async function CurrentTime() {
  await io()
  return 
}
```

```jsx filename="app/page.js" highlight= switcher
import  from 'react'
import  from 'next/cache'

export default function Page() 

async function CurrentTime() {
  await io()
  return 
}
```

In this route `CurrentTime` is wrapped in a `` boundary, during prerender, the `await io()` suspends and the fallback ships in the static shell. If `CurrentTime` were rendered inside a [`"use cache"`](/docs/app/api-reference/directives/use-cache) scope instead, `io()` would be a no-op, the value is captured into the static shell and no `` boundary is required.

In a Client Component, call `io()` with React's [`use`](https://react.dev/reference/react/use) hook before reading a synchronous source like `Date.now()`. Client Components prerender on the server during SSR, where the read would otherwise be included in the static shell:

```tsx filename="app/components.tsx" highlight= switcher
'use client'
import  from 'react'
import  from 'next/cache'

export function CurrentTime() {
  use(io())
  return 
}
```

```jsx filename="app/components.js" highlight= switcher
'use client'
import  from 'react'
import  from 'next/cache'

export function CurrentTime() {
  use(io())
  return 
}
```

## When you don't need `io()`

- The component already uses a [Request-time API](/docs/app/glossary#request-time-apis) like `cookies()` or `headers()`. The request-time API is itself the suspension point.
- The data comes from an awaited `fetch` or async database query wrapped in ``. The `await` is the suspension point.

## How `io()` differs from `connection()`

The [`connection()`](/docs/app/api-reference/functions/connection) function excludes the code that follows it from the static shell, but it stays suspended until a full user navigation reaches the server, so it also blocks [prefetches](/docs/app/guides/prefetching). `io()` suspends like any other asynchronous function, so the code after it can be wrapped in [`"use cache"`](/docs/app/api-reference/directives/use-cache) and prefetched and cached on the client. Prefer `io()` over `connection()`, and reach for `connection()` only when you need to wait for a real user request.

## Reference

### Type

```ts
function io(): Promise
```

### Parameters

- The function does not accept any parameters.

### Returns

A `Promise`. With Cache Components enabled, awaiting this promise stops prerendering so the code that follows is excluded from the prerender output. In every other context (real requests, cache scopes, [`generateStaticParams`](/docs/app/api-reference/functions/generate-static-params), the browser, and routes without Cache Components), it resolves immediately.

## Version History

| Version   | Changes     |
| --------- | ----------- |
| `v16.3.0` | `io` added. |
