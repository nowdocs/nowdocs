---
title: route.js
description: API reference for the route.js special file.
---

Route Handlers allow you to create custom request handlers for a given route using the Web [Request](https://developer.mozilla.org/docs/Web/API/Request) and [Response](https://developer.mozilla.org/docs/Web/API/Response) APIs.

```ts filename="route.ts" switcher
export async function GET() {
  return Response.json()
}
```

```js filename="route.js" switcher
export async function GET() {
  return Response.json()
}
```

## Reference

### HTTP Methods

A **route** file allows you to create custom request handlers for a given route. The following [HTTP methods](https://developer.mozilla.org/docs/Web/HTTP/Methods) are supported: `GET`, `POST`, `PUT`, `PATCH`, `DELETE`, `HEAD`, and `OPTIONS`.

```ts filename="route.ts" switcher
export async function GET(request: Request) 

export async function HEAD(request: Request) 

export async function POST(request: Request) 

export async function PUT(request: Request) 

export async function DELETE(request: Request) 

export async function PATCH(request: Request) 

// If `OPTIONS` is not defined, Next.js will automatically implement `OPTIONS` and set the appropriate Response `Allow` header depending on the other methods defined in the Route Handler.
export async function OPTIONS(request: Request) 
```

```js filename="route.js" switcher
export async function GET(request) 

export async function HEAD(request) 

export async function POST(request) 

export async function PUT(request) 

export async function DELETE(request) 

export async function PATCH(request) 

// If `OPTIONS` is not defined, Next.js will automatically implement `OPTIONS` and set the appropriate Response `Allow` header depending on the other methods defined in the Route Handler.
export async function OPTIONS(request) 
```

### Parameters

#### `request` (optional)

The `request` object is a [NextRequest](/docs/app/api-reference/functions/next-request) object, which is an extension of the Web [Request](https://developer.mozilla.org/docs/Web/API/Request) API. `NextRequest` gives you further control over the incoming request, including easily accessing `cookies` and an extended, parsed, URL object `nextUrl`.

```ts filename="route.ts" switcher
import type  from 'next/server'

export async function GET(request: NextRequest) 
```

```js filename="route.js" switcher
export async function GET(request) 
```

#### `context` (optional)

- **`params`**: a promise that resolves to an object containing the [dynamic route parameters](/docs/app/api-reference/file-conventions/dynamic-routes) for the current route.

```ts filename="app/dashboard/[team]/route.ts" switcher
export async function GET(
  request: Request,
  : { params: Promise<> }
) {
  const  = await params
}
```

```js filename="app/dashboard/[team]/route.js" switcher
export async function GET(request, ) {
  const  = await params
}
```

| Example                          | URL            | `params`                           |
| -------------------------------- | -------------- | ---------------------------------- |
| `app/dashboard/[team]/route.js`  | `/dashboard/1` | `Promise<>`           |
| `app/shop/[tag]/[item]/route.js` | `/shop/1/2`    | `Promise<>` |
| `app/blog/[...slug]/route.js`    | `/blog/1/2`    | `Promise<>`    |

### Route Context Helper

You can type the Route Handler context using `RouteContext` to get strongly typed `params` from a route literal. `RouteContext` is a globally available helper.

```ts filename="app/users/[id]/route.ts"
import type  from 'next/server'

export async function GET(_req: NextRequest, ctx: RouteContext<'/users/[id]'>) {
  const  = await ctx.params
  return Response.json()
}
```

> **Good to know**
>
> - Types are generated during `next dev`, `next build` or `next typegen`.
> - After type generation, the `RouteContext` helper is globally available. It doesn't need to be imported.

## Examples

### Cookies

You can read or set cookies with [`cookies`](/docs/app/api-reference/functions/cookies) from `next/headers`.

```ts filename="route.ts" switcher
import  from 'next/headers'

export async function GET(request: NextRequest) 
```

```js filename="route.js" switcher
import  from 'next/headers'

export async function GET(request) 
```

Alternatively, you can return a new `Response` using the [`Set-Cookie`](https://developer.mozilla.org/docs/Web/HTTP/Headers/Set-Cookie) header.

```ts filename="app/api/route.ts" switcher
import  from 'next/headers'

export async function GET(request: Request) {
  const cookieStore = await cookies()
  const token = cookieStore.get('token')

  return new Response('Hello, Next.js!', {
    status: 200,
    headers: { 'Set-Cookie': `token=$` },
  })
}
```

```js filename="app/api/route.js" switcher
import  from 'next/headers'

export async function GET(request) {
  const cookieStore = await cookies()
  const token = cookieStore.get('token')

  return new Response('Hello, Next.js!', {
    status: 200,
    headers: { 'Set-Cookie': `token=$` },
  })
}
```

You can also use the underlying Web APIs to read cookies from the request ([`NextRequest`](/docs/app/api-reference/functions/next-request)):

```ts filename="app/api/route.ts" switcher
import  from 'next/server'

export async function GET(request: NextRequest) 
```

```js filename="app/api/route.js" switcher
export async function GET(request) 
```

### Headers

You can read headers with [`headers`](/docs/app/api-reference/functions/headers) from `next/headers`.

```ts filename="route.ts" switcher
import  from 'next/headers'
import type  from 'next/server'

export async function GET(request: NextRequest) 
```

```js filename="route.js" switcher
import  from 'next/headers'

export async function GET(request) 
```

This `headers` instance is read-only. To set headers, you need to return a new `Response` with new `headers`.

```ts filename="app/api/route.ts" switcher
import  from 'next/headers'

export async function GET(request: Request) {
  const headersList = await headers()
  const referer = headersList.get('referer')

  return new Response('Hello, Next.js!', {
    status: 200,
    headers: ,
  })
}
```

```js filename="app/api/route.js" switcher
import  from 'next/headers'

export async function GET(request) {
  const headersList = await headers()
  const referer = headersList.get('referer')

  return new Response('Hello, Next.js!', {
    status: 200,
    headers: ,
  })
}
```

You can also use the underlying Web APIs to read headers from the request ([`NextRequest`](/docs/app/api-reference/functions/next-request)):

```ts filename="app/api/route.ts" switcher
import  from 'next/server'

export async function GET(request: NextRequest) 
```

```js filename="app/api/route.js" switcher
export async function GET(request) 
```

### Revalidating Cached Data

You can [revalidate cached data](/docs/app/guides/incremental-static-regeneration) using the `revalidate` route segment config option.

```ts filename="app/posts/route.ts" switcher
export const revalidate = 60

export async function GET() 
```

```js filename="app/posts/route.js" switcher
export const revalidate = 60

export async function GET() 
```

### Redirects

```ts filename="app/api/route.ts" switcher
import  from 'next/navigation'

export async function GET(request: Request) 
```

```js filename="app/api/route.js" switcher
import  from 'next/navigation'

export async function GET(request) 
```

### Dynamic Route Segments

Route Handlers can use [Dynamic Segments](/docs/app/api-reference/file-conventions/dynamic-routes) to create request handlers from dynamic data.

```ts filename="app/items/[slug]/route.ts" switcher
export async function GET(
  request: Request,
  : { params: Promise<> }
) {
  const  = await params // 'a', 'b', or 'c'
}
```

```js filename="app/items/[slug]/route.js" switcher
export async function GET(request, ) {
  const  = await params // 'a', 'b', or 'c'
}
```

| Route                       | Example URL | `params`                 |
| --------------------------- | ----------- | ------------------------ |
| `app/items/[slug]/route.js` | `/items/a`  | `Promise<>` |
| `app/items/[slug]/route.js` | `/items/b`  | `Promise<>` |
| `app/items/[slug]/route.js` | `/items/c`  | `Promise<>` |

#### Static Generation with `generateStaticParams`

You can use [`generateStaticParams`](/docs/app/api-reference/functions/generate-static-params) with dynamic Route Handlers to statically generate responses at build time for specified params, while handling other params dynamically at request time.

When using [Cache Components](/docs/app/getting-started/caching), you can combine `generateStaticParams` with `use cache` to enable data caching for both prerendered and runtime params.

See the [generateStaticParams with Route Handlers](/docs/app/api-reference/functions/generate-static-params#with-route-handlers) documentation for examples and details.

### URL Query Parameters

The request object passed to the Route Handler is a `NextRequest` instance, which includes [some additional convenience methods](/docs/app/api-reference/functions/next-request#nexturl), such as those for more easily handling query parameters.

```ts filename="app/api/search/route.ts" switcher
import  from 'next/server'

export function GET(request: NextRequest) 
```

```js filename="app/api/search/route.js" switcher
export function GET(request) 
```

### Streaming

Streaming is commonly used in combination with Large Language Models (LLMs), such as OpenAI, for AI-generated content. Learn more about the [AI SDK](https://sdk.vercel.ai/docs/introduction).

```ts filename="app/api/chat/route.ts" switcher
import  from '@ai-sdk/openai'
import  from 'ai'

export async function POST(req: Request) {
  const  = await req.json()
  const result = await streamText()

  return new StreamingTextResponse(result.toAIStream())
}
```

```js filename="app/api/chat/route.js" switcher
import  from '@ai-sdk/openai'
import  from 'ai'

export async function POST(req) {
  const  = await req.json()
  const result = await streamText()

  return new StreamingTextResponse(result.toAIStream())
}
```

These abstractions use the Web APIs to create a stream. You can also use the underlying Web APIs directly.

```ts filename="app/api/route.ts" switcher
// https://developer.mozilla.org/docs/Web/API/ReadableStream#convert_async_iterator_to_stream
function iteratorToStream(iterator: any) {
  return new ReadableStream({
    async pull(controller) {
      const  = await iterator.next()

      if (done)  else 
    },
  })
}

function sleep(time: number) {
  return new Promise((resolve) => )
}

const encoder = new TextEncoder()

async function* makeIterator() 

export async function GET() 
```

```js filename="app/api/route.js" switcher
// https://developer.mozilla.org/docs/Web/API/ReadableStream#convert_async_iterator_to_stream
function iteratorToStream(iterator) {
  return new ReadableStream({
    async pull(controller) {
      const  = await iterator.next()

      if (done)  else 
    },
  })
}

function sleep(time) {
  return new Promise((resolve) => )
}

const encoder = new TextEncoder()

async function* makeIterator() 

export async function GET() 
```

### Request Body

You can read the `Request` body using the standard Web API methods:

```ts filename="app/items/route.ts" switcher
export async function POST(request: Request) {
  const res = await request.json()
  return Response.json()
}
```

```js filename="app/items/route.js" switcher
export async function POST(request) {
  const res = await request.json()
  return Response.json()
}
```

### Request Body FormData

You can read the `FormData` using the `request.formData()` function:

```ts filename="app/items/route.ts" switcher
export async function POST(request: Request) {
  const formData = await request.formData()
  const name = formData.get('name')
  const email = formData.get('email')
  return Response.json()
}
```

```js filename="app/items/route.js" switcher
export async function POST(request) {
  const formData = await request.formData()
  const name = formData.get('name')
  const email = formData.get('email')
  return Response.json()
}
```

Since `formData` data are all strings, you may want to use [`zod-form-data`](https://www.npmjs.com/zod-form-data) to validate the request and retrieve data in the format you prefer (e.g. `number`).

### CORS

You can set CORS headers for a specific Route Handler using the standard Web API methods:

```ts filename="app/api/route.ts" switcher
export async function GET(request: Request) {
  return new Response('Hello, Next.js!', {
    status: 200,
    headers: ,
  })
}
```

```js filename="app/api/route.js" switcher
export async function GET(request) {
  return new Response('Hello, Next.js!', {
    status: 200,
    headers: ,
  })
}
```

> **Good to know**:
>
> - To add CORS headers to multiple Route Handlers, you can use [Proxy](/docs/app/api-reference/file-conventions/proxy#cors) or the [`next.config.js` file](/docs/app/api-reference/config/next-config-js/headers#cors).

### Webhooks

You can use a Route Handler to receive webhooks from third-party services:

```ts filename="app/api/route.ts" switcher
export async function POST(request: Request) {
  try  catch (error) {
    return new Response(`Webhook error: $`, )
  }

  return new Response('Success!', )
}
```

```js filename="app/api/route.js" switcher
export async function POST(request) {
  try  catch (error) {
    return new Response(`Webhook error: $`, )
  }

  return new Response('Success!', )
}
```

Notably, unlike API Routes with the Pages Router, you do not need to use `bodyParser` to use any additional configuration.

### Non-UI Responses

You can use Route Handlers to return non-UI content. Note that [`sitemap.xml`](/docs/app/api-reference/file-conventions/metadata/sitemap#generating-a-sitemap-using-code-js-ts), [`robots.txt`](/docs/app/api-reference/file-conventions/metadata/robots#generate-a-robots-file), [`app icons`](/docs/app/api-reference/file-conventions/metadata/app-icons#generate-icons-using-code-js-ts-tsx), and [open graph images](/docs/app/api-reference/file-conventions/metadata/opengraph-image) all have built-in support.

```ts filename="app/rss.xml/route.ts" switcher
export async function GET() {
  return new Response(
    `<?xml version="1.0" encoding="UTF-8" ?>

  Next.js Documentation
  https://nextjs.org/docs
  The React Framework for the Web

`,
    {
      headers: ,
    }
  )
}
```

```js filename="app/rss.xml/route.js" switcher
export async function GET() 
```

### Segment Config Options

Route Handlers use the same [route segment configuration](/docs/app/api-reference/file-conventions/route-segment-config) as pages and layouts.

```ts filename="app/items/route.ts" switcher
export const dynamic = 'auto'
export const dynamicParams = true
export const revalidate = false
export const fetchCache = 'auto'
export const runtime = 'nodejs'
export const preferredRegion = 'auto' // deprecated
```

```js filename="app/items/route.js" switcher
export const dynamic = 'auto'
export const dynamicParams = true
export const revalidate = false
export const fetchCache = 'auto'
export const runtime = 'nodejs'
export const preferredRegion = 'auto' // deprecated
```

See the [API reference](/docs/app/api-reference/file-conventions/route-segment-config) for more details.

## Version History

| Version      | Changes                                                                                              |
| ------------ | ---------------------------------------------------------------------------------------------------- |
| `v15.0.0-RC` | `context.params` is now a promise. A [codemod](/docs/app/guides/upgrading/codemods#150) is available |
| `v15.0.0-RC` | The default caching for `GET` handlers was changed from static to dynamic                            |
| `v13.2.0`    | Route Handlers are introduced.                                                                       |
