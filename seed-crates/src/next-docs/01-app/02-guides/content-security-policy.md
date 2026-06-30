---
title: How to set a Content Security Policy (CSP) for your Next.js application
nav_title: Content Security Policy
description: Learn how to set a Content Security Policy (CSP) for your Next.js application.
related:
  links:
    - app/api-reference/file-conventions/proxy
    - app/api-reference/functions/headers
---

[Content Security Policy (CSP)](https://developer.mozilla.org/docs/Web/HTTP/CSP) is important to guard your Next.js application against various security threats such as cross-site scripting (XSS), clickjacking, and other code injection attacks.

By using CSP, developers can specify which origins are permissible for content sources, scripts, stylesheets, images, fonts, objects, media (audio, video), iframes, and more.

  Examples

- [Strict CSP](https://github.com/vercel/next.js/tree/canary/examples/with-strict-csp)

## Nonces

A [nonce](https://developer.mozilla.org/docs/Web/HTML/Global_attributes/nonce) is a unique, random string of characters created for a one-time use. It is used in conjunction with CSP to selectively allow certain inline scripts or styles to execute, bypassing strict CSP directives.

### Why use a nonce?

CSP can block both inline and external scripts to prevent attacks. A nonce lets you safely allow specific scripts to run—only if they include the matching nonce value.

If an attacker wanted to load a script into your page, they'd need to guess the nonce value. That's why the nonce must be unpredictable and unique for every request.

### Adding a nonce with Proxy

[Proxy](/docs/app/api-reference/file-conventions/proxy) enables you to add headers and generate nonces before the page renders.

Every time a page is viewed, a fresh nonce should be generated. This means that you **must use [dynamic rendering](/docs/app/glossary#dynamic-rendering) to add nonces**.

For example:

> **Good to know**: In development, `'unsafe-eval'` is required because React uses `eval` to provide enhanced debugging information, such as reconstructing server-side error stacks in the browser. `unsafe-eval` is not required for production. Neither React nor Next.js use `eval` in production by default.

```ts filename="proxy.ts" switcher
import  from 'next/server'

export function proxy(request: NextRequest) {
  const nonce = Buffer.from(crypto.randomUUID()).toString('base64')
  const isDev = process.env.NODE_ENV === 'development'
  const cspHeader = `
    default-src 'self';
    script-src 'self' 'nonce-$' 'strict-dynamic'$;
    style-src 'self' 'nonce-$';
    img-src 'self' blob: data:;
    font-src 'self';
    object-src 'none';
    base-uri 'self';
    form-action 'self';
    frame-ancestors 'none';
    upgrade-insecure-requests;
`
  // Replace newline characters and spaces
  const contentSecurityPolicyHeaderValue = cspHeader
    .replace(/\s/g, ' ')
    .trim()

  const requestHeaders = new Headers(request.headers)
  requestHeaders.set('x-nonce', nonce)

  requestHeaders.set(
    'Content-Security-Policy',
    contentSecurityPolicyHeaderValue
  )

  const response = NextResponse.next({
    request: ,
  })
  response.headers.set(
    'Content-Security-Policy',
    contentSecurityPolicyHeaderValue
  )

  return response
}
```

```js filename="proxy.js" switcher
import  from 'next/server'

export function proxy(request) {
  const nonce = Buffer.from(crypto.randomUUID()).toString('base64')
  const isDev = process.env.NODE_ENV === 'development'
  const cspHeader = `
    default-src 'self';
    script-src 'self' 'nonce-$' 'strict-dynamic'$;
    style-src 'self' 'nonce-$';
    img-src 'self' blob: data:;
    font-src 'self';
    object-src 'none';
    base-uri 'self';
    form-action 'self';
    frame-ancestors 'none';
    upgrade-insecure-requests;
`
  // Replace newline characters and spaces
  const contentSecurityPolicyHeaderValue = cspHeader
    .replace(/\s/g, ' ')
    .trim()

  const requestHeaders = new Headers(request.headers)
  requestHeaders.set('x-nonce', nonce)
  requestHeaders.set(
    'Content-Security-Policy',
    contentSecurityPolicyHeaderValue
  )

  const response = NextResponse.next({
    request: ,
  })
  response.headers.set(
    'Content-Security-Policy',
    contentSecurityPolicyHeaderValue
  )

  return response
}
```

By default, Proxy runs on all requests. You can filter Proxy to run on specific paths using a [`matcher`](/docs/app/api-reference/file-conventions/proxy#matcher).

We recommend ignoring matching prefetches (from `next/link`) and static assets that don't need the CSP header.

```ts filename="proxy.ts" switcher
export const config = {
  matcher: [
    /*
     * Match all request paths except for the ones starting with:
     * - api (API routes)
     * - _next/static (static files)
     * - _next/image (image optimization files)
     * - favicon.ico (favicon file)
     */
    {
      source: '/((?!api|_next/static|_next/image|favicon.ico).*)',
      missing: [
        ,
        ,
      ],
    },
  ],
}
```

```js filename="proxy.js" switcher
export const config = {
  matcher: [
    /*
     * Match all request paths except for the ones starting with:
     * - api (API routes)
     * - _next/static (static files)
     * - _next/image (image optimization files)
     * - favicon.ico (favicon file)
     */
    {
      source: '/((?!api|_next/static|_next/image|favicon.ico).*)',
      missing: [
        ,
        ,
      ],
    },
  ],
}
```

### How nonces work in Next.js

To use a nonce, your page must be **dynamically rendered**. This is because Next.js applies nonces during **server-side rendering**, based on the CSP header present in the request. Static pages are generated at build time, when no request or response headers exist—so no nonce can be injected.

Here’s how nonce support works in a dynamically rendered page:

1. **Proxy generates a nonce**: Your proxy creates a unique nonce for the request, adds it to your `Content-Security-Policy` header, and also sets it in a custom `x-nonce` header.
2. **Next.js extracts the nonce**: During rendering, Next.js parses the `Content-Security-Policy` header and extracts the nonce using the `'nonce-'` pattern.
3. **Nonce is applied automatically**: Next.js attaches the nonce to:
   - Framework scripts (React, Next.js runtime)
   - Page-specific JavaScript bundles
   - Inline styles and scripts generated by Next.js
   - Any `
    )
  }
}

export default MyDocument
```

```jsx filename="pages/_document.jsx" switcher
import Document,  from 'next/document'

class MyDocument extends Document {
  static async getInitialProps(ctx) {
    const initialProps = await Document.getInitialProps(ctx)
    const nonce = ctx.req?.headers?.['x-nonce']

    return 
  }

  render() {
    const  = this.props

    return (
      
    )
  }
}

export default MyDocument
```

## Static vs Dynamic Rendering with CSP

Using nonces has important implications for how your Next.js application renders:

### Dynamic Rendering Requirement

When you use nonces in your CSP, **all pages must be dynamically rendered**. This means:

- Pages will build successfully but may encounter runtime errors if not properly configured for dynamic rendering
- Each request generates a fresh page with a new nonce
- Static optimization and Incremental Static Regeneration (ISR) are disabled
- Pages cannot be cached by CDNs without additional configuration
- **Partial Prerendering (PPR) is incompatible** with nonce-based CSP since static shell scripts won't have access to the nonce

### Performance Implications

The shift from static to dynamic rendering affects performance:

- **Slower initial page loads**: Pages must be generated on each request
- **Increased server load**: Every request requires server-side rendering
- **No CDN caching**: Dynamic pages cannot be cached at the edge by default
- **Higher hosting costs**: More server resources needed for dynamic rendering

### When to use nonces

Consider nonces when:

- You have strict security requirements that prohibit `'unsafe-inline'`
- Your application handles sensitive data
- You need to allow specific inline scripts while blocking others
- Compliance requirements mandate strict CSP

## Without Nonces

For applications that do not require nonces, you can set the CSP header directly in your [`next.config.js`](/docs/app/api-reference/config/next-config-js) file:

```js filename="next.config.js"
const isDev = process.env.NODE_ENV === 'development'

const cspHeader = `
    default-src 'self';
    script-src 'self' 'unsafe-inline'$;
    style-src 'self' 'unsafe-inline';
    img-src 'self' blob: data:;
    font-src 'self';
    object-src 'none';
    base-uri 'self';
    form-action 'self';
    frame-ancestors 'none';
    upgrade-insecure-requests;
`

module.exports = {
  async headers() {
    return [
      {
        source: '/(.*)',
        headers: [
          ,
        ],
      },
    ]
  },
}
```

## Development vs Production Considerations

CSP implementation differs between development and production environments:

### Development Environment

In development, you will need to enable `'unsafe-eval'` because React uses `eval` to provide enhanced debugging information, such as reconstructing server-side error stacks in the browser to show you where errors originated on the server:

```ts filename="proxy.ts" switcher
export function proxy(request: NextRequest) {
  const nonce = Buffer.from(crypto.randomUUID()).toString('base64')
  const isDev = process.env.NODE_ENV === 'development'

  const cspHeader = `
    default-src 'self';
    script-src 'self' 'nonce-$' 'strict-dynamic' $;
    style-src 'self' ${isDev ? "'unsafe-inline'" : `'nonce-$'`};
    img-src 'self' blob: data:;
    font-src 'self';
    object-src 'none';
    base-uri 'self';
    form-action 'self';
    frame-ancestors 'none';
    upgrade-insecure-requests;
`

  // Rest of proxy implementation
}
```

```js filename="proxy.js" switcher
export function proxy(request) {
  const nonce = Buffer.from(crypto.randomUUID()).toString('base64')
  const isDev = process.env.NODE_ENV === 'development'

  const cspHeader = `
    default-src 'self';
    script-src 'self' 'nonce-$' 'strict-dynamic' $;
    style-src 'self' ${isDev ? "'unsafe-inline'" : `'nonce-$'`};
    img-src 'self' blob: data:;
    font-src 'self';
    object-src 'none';
    base-uri 'self';
    form-action 'self';
    frame-ancestors 'none';
    upgrade-insecure-requests;
`

  // Rest of proxy implementation
}
```

### Production Deployment

Common issues in production:

- **Nonce not applied**: Ensure your proxy runs on all necessary routes
- **Static assets blocked**: Verify your CSP allows Next.js static assets
- **Third-party scripts**: Add necessary domains to your CSP policy

## Troubleshooting

### Third-party Scripts

Update your CSP to allow third-party domains:

```ts filename="proxy.ts" switcher
const cspHeader = `
  default-src 'self';
  script-src 'self' 'nonce-$' 'strict-dynamic' https://www.googletagmanager.com;
  connect-src 'self' https://www.google-analytics.com;
  img-src 'self' data: https://www.google-analytics.com;
`
```

```js filename="proxy.js" switcher
const cspHeader = `
  default-src 'self';
  script-src 'self' 'nonce-$' 'strict-dynamic' https://www.googletagmanager.com;
  connect-src 'self' https://www.google-analytics.com;
  img-src 'self' data: https://www.google-analytics.com;
`
```

### Common CSP Violations

1. **Inline styles**: Use CSS-in-JS libraries that support nonces or move styles to external files
2. **Dynamic imports**: Ensure dynamic imports are allowed in your script-src policy
3. **WebAssembly**: Add `'wasm-unsafe-eval'` if using WebAssembly
4. **Service workers**: Add appropriate policies for service worker scripts

## Version History

| Version    | Changes                                                       |
| ---------- | ------------------------------------------------------------- |
| `v14.0.0`  | Experimental SRI support added for hash-based CSP             |
| `v13.4.20` | Recommended for proper nonce handling and CSP header parsing. |
