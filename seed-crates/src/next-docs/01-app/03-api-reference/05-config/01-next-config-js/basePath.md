---
title: basePath
description: Use `basePath` to deploy a Next.js application under a sub-path of a domain.
---

To deploy a Next.js application under a sub-path of a domain you can use the `basePath` config option.

`basePath` allows you to set a path prefix for the application. For example, to use `/docs` instead of `''` (an empty string, the default), open `next.config.js` and add the `basePath` config:

```js filename="next.config.js"
module.exports = 
```

> **Good to know**: This value must be set at build time and cannot be changed without re-building as the value is inlined in the client-side bundles.

### Links

When linking to other pages using `next/link` and `next/router` the `basePath` will be automatically applied.

For example, using `/about` will automatically become `/docs/about` when `basePath` is set to `/docs`.

```js
export default function HomePage() 
```

Output html:

```html
About Page
```

This makes sure that you don't have to change all links in your application when changing the `basePath` value.

### Images

For example, using `/docs/me.png` will properly serve your image when `basePath` is set to `/docs`.

```jsx
import Image from 'next/image'

function Home() 

export default Home
```
