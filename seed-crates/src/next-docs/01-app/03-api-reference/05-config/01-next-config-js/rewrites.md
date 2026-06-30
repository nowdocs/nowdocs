---
title: rewrites
description: Add rewrites to your Next.js app.
---

Rewrites allow you to map an incoming request path to a different destination path.

To use rewrites you can use the `rewrites` key in `next.config.js`:

```js filename="next.config.js"
module.exports = {
  rewrites() {
    return [
      ,
    ]
  },
}
```

Rewrites are applied to client-side routing. In the example above, navigating to `

## Rewrite parameters

When using parameters in a rewrite the parameters will be passed in the query by default when none of the parameters are used in the `destination`.

```js filename="next.config.js"
module.exports = {
  rewrites() {
    return [
      ,
    ]
  },
}
```

If a parameter is used in the destination none of the parameters will be automatically passed in the query.

```js filename="next.config.js"
module.exports = {
  rewrites() {
    return [
      ,
    ]
  },
}
```

You can still pass the parameters manually in the query if one is already used in the destination by specifying the query in the `destination`.

```js filename="next.config.js"
module.exports = {
  rewrites() {
    return [
      ,
    ]
  },
}
```

> **Good to know**: Static pages from [Automatic Static Optimization](/docs/pages/building-your-application/rendering/automatic-static-optimization) or [prerendering](/docs/pages/building-your-application/data-fetching/get-static-props) params from rewrites will be parsed on the client after hydration and provided in the query.

## Path Matching

Path matches are allowed, for example `/blog/:slug` will match `/blog/first-post` (no nested paths):

```js filename="next.config.js"
module.exports = {
  rewrites() {
    return [
      ,
    ]
  },
}
```

The pattern `/blog/:slug` matches `/blog/first-post` and `/blog/post-1` but not `/blog/a/b` (no nested paths). Patterns are anchored to the start: `/blog/:slug` will not match `/archive/blog/first-post`.

You can use modifiers on parameters: `*` (zero or more), `+` (one or more), `?` (zero or one). For example, `/blog/:slug*` matches `/blog`, `/blog/a`, and `/blog/a/b/c`.

Read more details on [path-to-regexp](https://github.com/pillarjs/path-to-regexp) documentation.

### Wildcard Path Matching

To match a wildcard path you can use `*` after a parameter, for example `/blog/:slug*` will match `/blog/a/b/c/d/hello-world`:

```js filename="next.config.js"
module.exports = {
  rewrites() {
    return [
      ,
    ]
  },
}
```

### Regex Path Matching

To match a regex path you can wrap the regex in parenthesis after a parameter, for example `/blog/:slug(\\d)` will match `/blog/123` but not `/blog/abc`:

```js filename="next.config.js"
module.exports = {
  rewrites() {
    return [
      {
        source: '/old-blog/:post(\\d)',
        destination: '/blog/:post', // Matched parameters can be used in the destination
      },
    ]
  },
}
```

The following characters `(`, `)`, ``, `[`, `]`, `|`, `\`, `^`, `.`, `:`, `*`, `+`, `-`, `?`, `$` are used for regex path matching, so when used in the `source` as non-special values they must be escaped by adding `\\` before them:

```js filename="next.config.js"
module.exports = {
  rewrites() {
    return [
      ,
    ]
  },
}
```

## Header, Cookie, and Query Matching

To only match a rewrite when header, cookie, or query values also match the `has` field or don't match the `missing` field can be used. Both the `source` and all `has` items must match and all `missing` items must not match for the rewrite to be applied.

`has` and `missing` items can have the following fields:

- `type`: `String` - must be either `header`, `cookie`, `host`, or `query`.
- `key`: `String` - the key from the selected type to match against.
- `value`: `String` or `undefined` - the value to check for, if undefined any value will match. A regex like string can be used to capture a specific part of the value, e.g. if the value `first-(?.*)` is used for `first-second` then `second` will be usable in the destination with `:paramName`.

```js filename="next.config.js"
module.exports = {
  rewrites() {
    return [
      // if the header `x-rewrite-me` is present,
      // this rewrite will be applied
      {
        source: '/:path*',
        has: [
          ,
        ],
        destination: '/another-page',
      },
      // if the header `x-rewrite-me` is not present,
      // this rewrite will be applied
      {
        source: '/:path*',
        missing: [
          ,
        ],
        destination: '/another-page',
      },
      // if the source, query, and cookie are matched,
      // this rewrite will be applied
      {
        source: '/specific/:path*',
        has: [
          ,
          ,
        ],
        destination: '/:path*/home',
      },
      // if the header `x-authorized` is present and
      // contains a matching value, this rewrite will be applied
      {
        source: '/:path*',
        has: [
          ,
        ],
        destination: '/home?authorized=:authorized',
      },
      // if the host is `example.com`,
      // this rewrite will be applied
      {
        source: '/:path*',
        has: [
          ,
        ],
        destination: '/another-page',
      },
    ]
  },
}
```

## Rewriting to an external URL

  Examples

- [Using Multiple Zones](https://github.com/vercel/next.js/tree/canary/examples/with-zones)

Rewrites allow you to rewrite to an external URL. This is especially useful for incrementally adopting Next.js. The following is an example rewrite for redirecting the `/blog` route of your main app to an external site.

```js filename="next.config.js"
module.exports = {
  rewrites() {
    return [
      ,
      ,
    ]
  },
}
```

If you're using `trailingSlash: true`, you also need to insert a trailing slash in the `source` parameter. If the destination server is also expecting a trailing slash it should be included in the `destination` parameter as well.

```js filename="next.config.js"
module.exports = {
  trailingSlash: true,
  rewrites() {
    return [
      ,
      ,
    ]
  },
}
```

### Incremental adoption of Next.js

You can also have Next.js fall back to proxying to an existing website after checking all Next.js routes.

This way you don't have to change the rewrites configuration when migrating more pages to Next.js

```js filename="next.config.js"
module.exports = {
  rewrites() {
    return {
      fallback: [
        ,
      ],
    }
  },
}
```

### Rewrites with basePath support

When leveraging [`basePath` support](/docs/app/api-reference/config/next-config-js/basePath) with rewrites each `source` and `destination` is automatically prefixed with the `basePath` unless you add `basePath: false` to the rewrite:

```js filename="next.config.js"
module.exports = {
  basePath: '/docs',

  rewrites() {
    return [
      ,
      ,
    ]
  },
}
```

## Version History

| Version   | Changes          |
| --------- | ---------------- |
| `v13.3.0` | `missing` added. |
| `v10.2.0` | `has` added.     |
| `v9.5.0`  | Headers added.   |
