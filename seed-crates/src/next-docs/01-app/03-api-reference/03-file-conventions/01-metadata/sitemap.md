---
title: sitemap.xml
description: API Reference for the sitemap.xml file.
related:
  title: Next Steps
  description: Learn how to use the generateSitemaps function.
  links:
    - app/api-reference/functions/generate-sitemaps
---

`sitemap.(xml|js|ts)` is a special file that matches the [Sitemaps XML format](https://www.sitemaps.org/protocol.html) to help search engine crawlers index your site more efficiently.

### Sitemap files (.xml)

For smaller applications, you can create a `sitemap.xml` file and place it in the root of your `app` directory.

```xml filename="app/sitemap.xml"

  
    https://acme.com
    2023-04-06T15:02:24.021Z
    yearly
    1
  
  
    https://acme.com/about
    2023-04-06T15:02:24.021Z
    monthly
    0.8
  
  
    https://acme.com/blog
    2023-04-06T15:02:24.021Z
    weekly
    0.5
  

```

### Generating a sitemap using code (.js, .ts)

You can use the `sitemap.(js|ts)` file convention to programmatically **generate** a sitemap by exporting a default function that returns an array of URLs. If using TypeScript, a [`Sitemap`](#returns) type is available.

> **Good to know**: `sitemap.js` is a special Route Handler that is cached by default unless it uses a [Request-time API](/docs/app/glossary#request-time-apis) or [dynamic config](/docs/app/guides/caching-without-cache-components#dynamic) option.

```ts filename="app/sitemap.ts" switcher
import type  from 'next'

export default function sitemap(): MetadataRoute.Sitemap {
  return [
    ,
    ,
    ,
  ]
}
```

```js filename="app/sitemap.js" switcher
export default function sitemap() {
  return [
    ,
    ,
    ,
  ]
}
```

Output:

```xml filename="acme.com/sitemap.xml"

  
    https://acme.com
    2023-04-06T15:02:24.021Z
    yearly
    1
  
  
    https://acme.com/about
    2023-04-06T15:02:24.021Z
    monthly
    0.8
  
  
    https://acme.com/blog
    2023-04-06T15:02:24.021Z
    weekly
    0.5
  

```

### Image Sitemaps

You can use `images` property to create image sitemaps. Learn more details in the [Google Developer Docs](https://developers.google.com/search/docs/crawling-indexing/sitemaps/image-sitemaps).

```ts filename="app/sitemap.ts" switcher
import type  from 'next'

export default function sitemap(): MetadataRoute.Sitemap {
  return [
    ,
  ]
}
```

Output:

```xml filename="acme.com/sitemap.xml"
<?xml version="1.0" encoding="UTF-8"?>

  
    https://example.com
    
      https://example.com/image.jpg
    
    2021-01-01
    weekly
    0.5
  

```

### Video Sitemaps

You can use `videos` property to create video sitemaps. Learn more details in the [Google Developer Docs](https://developers.google.com/search/docs/crawling-indexing/sitemaps/video-sitemaps).

```ts filename="app/sitemap.ts" switcher
import type  from 'next'

export default function sitemap(): MetadataRoute.Sitemap {
  return [
    {
      url: 'https://example.com',
      lastModified: '2021-01-01',
      changeFrequency: 'weekly',
      priority: 0.5,
      videos: [
        ,
      ],
    },
  ]
}
```

Output:

```xml filename="acme.com/sitemap.xml"
<?xml version="1.0" encoding="UTF-8"?>

  
    https://example.com
    
      example
      https://example.com/image.jpg
      this is the description
    
    2021-01-01
    weekly
    0.5
  

```

### Generate a localized Sitemap

```ts filename="app/sitemap.ts" switcher
import type  from 'next'

export default function sitemap(): MetadataRoute.Sitemap {
  return [
    {
      url: 'https://acme.com',
      lastModified: new Date(),
      alternates: {
        languages: ,
      },
    },
    {
      url: 'https://acme.com/about',
      lastModified: new Date(),
      alternates: {
        languages: ,
      },
    },
    {
      url: 'https://acme.com/blog',
      lastModified: new Date(),
      alternates: {
        languages: ,
      },
    },
  ]
}
```

```js filename="app/sitemap.js" switcher
export default function sitemap() {
  return [
    {
      url: 'https://acme.com',
      lastModified: new Date(),
      alternates: {
        languages: ,
      },
    },
    {
      url: 'https://acme.com/about',
      lastModified: new Date(),
      alternates: {
        languages: ,
      },
    },
    {
      url: 'https://acme.com/blog',
      lastModified: new Date(),
      alternates: {
        languages: ,
      },
    },
  ]
}
```

Output:

```xml filename="acme.com/sitemap.xml"

  
    https://acme.com
    
    
    2023-04-06T15:02:24.021Z
  
  
    https://acme.com/about
    
    
    2023-04-06T15:02:24.021Z
  
  
    https://acme.com/blog
    
    
    2023-04-06T15:02:24.021Z
  

```

### Generating multiple sitemaps

While a single sitemap will work for most applications. For large web applications, you may need to split a sitemap into multiple files.

There are two ways you can create multiple sitemaps:

- By nesting `sitemap.(xml|js|ts)` inside multiple route segments e.g. `app/sitemap.xml` and `app/products/sitemap.xml`.
- By using the [`generateSitemaps`](/docs/app/api-reference/functions/generate-sitemaps) function.

For example, to split a sitemap using `generateSitemaps`, return an array of objects with the sitemap `id`. Then, use the `id` to generate the unique sitemaps.

```ts filename="app/product/sitemap.ts" switcher
import type  from 'next'
import  from '@/app/lib/constants'

export async function generateSitemaps() {
  // Fetch the total number of products and calculate the number of sitemaps needed
  return [, , , ]
}

export default async function sitemap(props: ): Promise {
  const id = await props.id
  // Google's limit is 50,000 URLs per sitemap
  const start = id * 50000
  const end = start + 50000
  const products = await getProducts(
    `SELECT id, date FROM products WHERE id BETWEEN $ AND $`
  )
  return products.map((product) => ({
    url: `$/product/$`,
    lastModified: product.date,
  }))
}
```

```js filename="app/product/sitemap.js" switcher
import  from '@/app/lib/constants'

export async function generateSitemaps() {
  // Fetch the total number of products and calculate the number of sitemaps needed
  return [, , , ]
}

export default async function sitemap(props) {
  const id = await props.id
  // Google's limit is 50,000 URLs per sitemap
  const start = id * 50000
  const end = start + 50000
  const products = await getProducts(
    `SELECT id, date FROM products WHERE id BETWEEN $ AND $`
  )
  return products.map((product) => ({
    url: `$/product/$`,
    lastModified: product.date,
  }))
}
```

Your generated sitemaps will be available at `/.../sitemap/[id]`. For example, `/product/sitemap/1.xml`.

See the [`generateSitemaps` API reference](/docs/app/api-reference/functions/generate-sitemaps) for more information.

## Returns

The default function exported from `sitemap.(xml|ts|js)` should return an array of objects with the following properties:

```tsx
type Sitemap = Array<{
  url: string
  lastModified?: string | Date
  changeFrequency?:
    | 'always'
    | 'hourly'
    | 'daily'
    | 'weekly'
    | 'monthly'
    | 'yearly'
    | 'never'
  priority?: number
  alternates?: 
}>
```

## Version History

| Version    | Changes                                                      |
| ---------- | ------------------------------------------------------------ |
| `v16.0.0`  | `id` is now a promise that resolves to a `string`.           |
| `v14.2.0`  | Add localizations support.                                   |
| `v13.4.14` | Add `changeFrequency` and `priority` attributes to sitemaps. |
| `v13.3.0`  | `sitemap` introduced.                                        |
