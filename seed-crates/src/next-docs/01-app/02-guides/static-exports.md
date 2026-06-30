---
title: How to create a static export of your Next.js application
nav_title: Static Exports
description: Next.js enables starting as a static site or Single-Page Application (SPA), then later optionally upgrading to use features that require a server.
---

Next.js enables starting as a static site or Single-Page Application (SPA), then later optionally upgrading to use features that require a server.

When running `next build`, Next.js generates an HTML file per route. By breaking a strict SPA into individual HTML files, Next.js can avoid loading unnecessary JavaScript code on the client-side, reducing the bundle size and enabling faster page loads.

Since Next.js supports this static export, it can be deployed and hosted on any web server that can serve HTML/CSS/JS static assets.

## Configuration

To enable a static export, change the output mode inside `next.config.js`:

```js filename="next.config.js" highlight=
/**
 * @type 
 */
const nextConfig = 

module.exports = nextConfig
```

After running `next build`, Next.js will create an `out` folder with the HTML/CSS/JS assets for your application.

        
        
          
        
      
    </>
  )
}
```

```jsx filename="app/page.js" switcher
import Link from 'next/link'

export default function Page() 
```

### Image Optimization

[Image Optimization](/docs/app/api-reference/components/image) through `next/image` can be used with a static export by defining a custom image loader in `next.config.js`. For example, you can optimize images with a service like Cloudinary:

```js filename="next.config.js"
/** @type  */
const nextConfig = {
  output: 'export',
  images: ,
}

module.exports = nextConfig
```

This custom loader will define how to fetch images from a remote source. For example, the following loader will construct the URL for Cloudinary:

```ts filename="my-loader.ts" switcher
export default function cloudinaryLoader(: ) {
  const params = ['f_auto', 'c_limit', `w_$`, `q_$`]
  return `https://res.cloudinary.com/demo/image/upload/$$`
}
```

```js filename="my-loader.js" switcher
export default function cloudinaryLoader() {
  const params = ['f_auto', 'c_limit', `w_$`, `q_$`]
  return `https://res.cloudinary.com/demo/image/upload/$$`
}
```

You can then use `next/image` in your application, defining relative paths to the image in Cloudinary:

```tsx filename="app/page.tsx" switcher
import Image from 'next/image'

export default function Page() {
  return 

## Unsupported Features

Features that require a Node.js server, or dynamic logic that cannot be computed during the build process, are **not** supported:

## Deploying

With a static export, Next.js can be deployed and hosted on any web server that can serve HTML/CSS/JS static assets.

When running `next build`, Next.js generates the static export into the `out` folder. For example, let's say you have the following routes:

- `/`
- `/blog/[id]`

After running `next build`, Next.js will generate the following files:

- `/out/index.html`
- `/out/404.html`
- `/out/blog/post-1.html`
- `/out/blog/post-2.html`

If you are using a static host like Nginx, you can configure rewrites from incoming requests to the correct files:

```nginx filename="nginx.conf"
server {
  listen 80;
  server_name acme.com;

  root /var/www/out;

  location / 

  # This is necessary when `trailingSlash: false`.
  # You can omit this when `trailingSlash: true`.
  location /blog/ 

  error_page 404 /404.html;
  location = /404.html 
}
```

## Version History

| Version   | Changes                                                                                                              |
| --------- | -------------------------------------------------------------------------------------------------------------------- |
| `v14.0.0` | `next export` has been removed in favor of `"output": "export"`                                                      |
| `v13.4.0` | App Router (Stable) adds enhanced static export support, including using React Server Components and Route Handlers. |
| `v13.3.0` | `next export` is deprecated and replaced with `"output": "export"`                                                   |
