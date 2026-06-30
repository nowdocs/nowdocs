---
title: headers
description: Add custom HTTP headers to your Next.js app.
---

Headers allow you to set custom HTTP headers on the response to an incoming request on a given path.

To set custom HTTP headers you can use the `headers` key in `next.config.js`:

```js filename="next.config.js"
module.exports = {
  headers() {
    return [
      {
        source: '/about',
        headers: [
          ,
          ,
        ],
      },
    ]
  },
}
```

`headers` can be defined as a synchronous or async function. It should return, or resolve to, an array of objects with `source` and `headers` properties:

- `source` is the incoming request path pattern.
- `headers` is an array of response header objects, with `key` and `value` properties.
- `basePath`: `false` or `undefined` - if false the basePath won't be included when matching, can be used for external rewrites only.
- `locale`: `false` or `undefined` - whether the locale should not be included when matching.
- `has` is an array of [has objects](#header-cookie-and-query-matching) with the `type`, `key` and `value` properties.
- `missing` is an array of [missing objects](#header-cookie-and-query-matching) with the `type`, `key` and `value` properties.

Headers are checked before the filesystem which includes pages and `/public` files.

## Header Overriding Behavior

If two headers match the same path and set the same header key, the last header key will override the first. Using the below headers, the path `/hello` will result in the header `x-hello` being `world` due to the last header value set being `world`.

```js filename="next.config.js"
module.exports = {
  headers() {
    return [
      {
        source: '/:path*',
        headers: [
          ,
        ],
      },
      {
        source: '/hello',
        headers: [
          ,
        ],
      },
    ]
  },
}
```

## Path Matching

Path matches are allowed, for example `/blog/:slug` will match `/blog/first-post` (no nested paths):

```js filename="next.config.js"
module.exports = {
  headers() {
    return [
      {
        source: '/blog/:slug',
        headers: [
          ,
          ,
        ],
      },
    ]
  },
}
```

The pattern `/blog/:slug` matches `/blog/first-post` and `/blog/post-1` but not a nested path like `/blog/a/b`. Patterns are anchored to the start, `/blog/:slug` will not match `/archive/blog/first-post`.

You can use modifiers on parameters: `*` (zero or more), `+` (one or more), `?` (zero or one). For example, `/blog/:slug*` matches `/blog`, `/blog/a`, and `/blog/a/b/c`.

Read more details on [path-to-regexp](https://github.com/pillarjs/path-to-regexp) documentation.

### Wildcard Path Matching

To match a wildcard path you can use `*` after a parameter, for example `/blog/:slug*` will match `/blog/a/b/c/d/hello-world`:

```js filename="next.config.js"
module.exports = {
  headers() {
    return [
      {
        source: '/blog/:slug*',
        headers: [
          ,
          ,
        ],
      },
    ]
  },
}
```

### Regex Path Matching

To match a regex path you can wrap the regex in parenthesis after a parameter, for example `/blog/:slug(\\d)` will match `/blog/123` but not `/blog/abc`:

```js filename="next.config.js"
module.exports = {
  headers() {
    return [
      {
        source: '/blog/:post(\\d)',
        headers: [
          ,
        ],
      },
    ]
  },
}
```

The following characters `(`, `)`, ``, `:`, `*`, `+`, `?` are used for regex path matching, so when used in the `source` as non-special values they must be escaped by adding `\\` before them:

```js filename="next.config.js"
module.exports = {
  headers() {
    return [
      {
        // this will match `/english(default)/something` being requested
        source: '/english\\(default\\)/:slug',
        headers: [
          ,
        ],
      },
    ]
  },
}
```

## Header, Cookie, and Query Matching

To only apply a header when header, cookie, or query values also match the `has` field or don't match the `missing` field can be used. Both the `source` and all `has` items must match and all `missing` items must not match for the header to be applied.

`has` and `missing` items can have the following fields:

- `type`: `String` - must be either `header`, `cookie`, `host`, or `query`.
- `key`: `String` - the key from the selected type to match against.
- `value`: `String` or `undefined` - the value to check for, if undefined any value will match. A regex like string can be used to capture a specific part of the value, e.g. if the value `first-(?.*)` is used for `first-second` then `second` will be usable in the destination with `:paramName`.

```js filename="next.config.js"
module.exports = {
  headers() {
    return [
      // if the header `x-add-header` is present,
      // the `x-another-header` header will be applied
      {
        source: '/:path*',
        has: [
          ,
        ],
        headers: [
          ,
        ],
      },
      // if the header `x-no-header` is not present,
      // the `x-another-header` header will be applied
      {
        source: '/:path*',
        missing: [
          ,
        ],
        headers: [
          ,
        ],
      },
      // if the source, query, and cookie are matched,
      // the `x-authorized` header will be applied
      {
        source: '/specific/:path*',
        has: [
          ,
          ,
        ],
        headers: [
          ,
        ],
      },
      // if the header `x-authorized` is present and
      // contains a matching value, the `x-another-header` will be applied
      {
        source: '/:path*',
        has: [
          ,
        ],
        headers: [
          ,
        ],
      },
      // if the host is `example.com`,
      // this header will be applied
      {
        source: '/:path*',
        has: [
          ,
        ],
        headers: [
          ,
        ],
      },
    ]
  },
}
```

## Headers with basePath support

When leveraging [`basePath` support](/docs/app/api-reference/config/next-config-js/basePath) with headers each `source` is automatically prefixed with the `basePath` unless you add `basePath: false` to the header:

```js filename="next.config.js"
module.exports = {
  basePath: '/docs',

  headers() {
    return [
      {
        source: '/with-basePath', // becomes /docs/with-basePath
        headers: [
          ,
        ],
      },
      {
        source: '/without-basePath', // is not modified since basePath: false is set
        headers: [
          ,
        ],
        basePath: false,
      },
    ]
  },
}
```

## Headers with i18n support

```js filename="next.config.js"
module.exports = {
  i18n: ,

  headers() {
    return [
      {
        source: '/with-locale', // automatically handles all locales
        headers: [
          ,
        ],
      },
      {
        // does not handle locales automatically since locale: false is set
        source: '/nl/with-locale-manual',
        locale: false,
        headers: [
          ,
        ],
      },
      {
        // this matches '/' since `en` is the defaultLocale
        source: '/en',
        locale: false,
        headers: [
          ,
        ],
      },
      {
        // this gets converted to /(en|fr|de)/(.*) so will not match the top-level
        // `/` or `/fr` routes like /:path* would
        source: '/(.*)',
        headers: [
          ,
        ],
      },
    ]
  },
}
```

## Cache-Control

Next.js sets the `Cache-Control` header of `public, max-age=31536000, immutable` for truly immutable assets. It cannot be overridden. These immutable files contain a SHA-hash in the file name, so they can be safely cached indefinitely. For example, [Static Image Imports](/docs/app/getting-started/images#local-images). You cannot set `Cache-Control` headers in `next.config.js` for these assets.

However, you can set `Cache-Control` headers for other responses or data.

## Options

### CORS

[Cross-Origin Resource Sharing (CORS)](https://developer.mozilla.org/docs/Web/HTTP/CORS) is a security feature that allows you to control which sites can access your resources. You can set the `Access-Control-Allow-Origin` header to allow a specific origin to access your .

```js
headers() {
    return [
      {
        source: "/api/:path*",
        headers: [
          ,
          ,
          ,
        ],
      },
    ];
  },
```

### X-DNS-Prefetch-Control

[This header](https://developer.mozilla.org/docs/Web/HTTP/Headers/X-DNS-Prefetch-Control) controls DNS prefetching, allowing browsers to proactively perform domain name resolution on external links, images, CSS, JavaScript, and more. This prefetching is performed in the background, so the [DNS](https://developer.mozilla.org/docs/Glossary/DNS) is more likely to be resolved by the time the referenced items are needed. This reduces latency when the user clicks a link.

```js

```

### Strict-Transport-Security

[This header](https://developer.mozilla.org/docs/Web/HTTP/Headers/Strict-Transport-Security) informs browsers it should only be accessed using HTTPS, instead of using HTTP. Using the configuration below, all present and future subdomains will use HTTPS for a `max-age` of 2 years. This blocks access to pages or subdomains that can only be served over HTTP.

```js

```

### X-Frame-Options

[This header](https://developer.mozilla.org/docs/Web/HTTP/Headers/X-Frame-Options) indicates whether the site should be allowed to be displayed within an `iframe`. This can prevent against clickjacking attacks.

**This header has been superseded by CSP's `frame-ancestors` option**, which has better support in modern browsers (see [Content Security Policy](/docs/app/guides/content-security-policy) for configuration details).

```js

```

### Permissions-Policy

[This header](https://developer.mozilla.org/docs/Web/HTTP/Headers/Permissions-Policy) allows you to control which features and APIs can be used in the browser. It was previously named `Feature-Policy`.

```js

```

### X-Content-Type-Options

[This header](https://developer.mozilla.org/docs/Web/HTTP/Headers/X-Content-Type-Options) prevents the browser from attempting to guess the type of content if the `Content-Type` header is not explicitly set. This can prevent XSS exploits for websites that allow users to upload and share files.

For example, a user trying to download an image, but having it treated as a different `Content-Type` like an executable, which could be malicious. This header also applies to downloading browser extensions. The only valid value for this header is `nosniff`.

```js

```

### Referrer-Policy

[This header](https://developer.mozilla.org/docs/Web/HTTP/Headers/Referrer-Policy) controls how much information the browser includes when navigating from the current website (origin) to another.

```js

```

### Content-Security-Policy

Learn more about adding a [Content Security Policy](/docs/app/guides/content-security-policy) to your application.

## Version History

| Version   | Changes          |
| --------- | ---------------- |
| `v13.3.0` | `missing` added. |
| `v10.2.0` | `has` added.     |
| `v9.5.0`  | Headers added.   |
