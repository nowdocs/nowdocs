---
title: images
description: Custom configuration for the next/image loader
---

If you want to use a cloud provider to optimize images instead of using the Next.js built-in Image Optimization API, you can configure `next.config.js` with the following:

```js filename="next.config.js"
module.exports = {
  images: ,
}
```

This `loaderFile` must point to a file relative to the root of your Next.js application. The file must export a default function that returns a string, for example:

## Example Loader Configuration

- [Akamai](#akamai)
- [AWS CloudFront](#aws-cloudfront)
- [Cloudinary](#cloudinary)
- [Cloudflare](#cloudflare)
- [Contentful](#contentful)
- [Fastly](#fastly)
- [Gumlet](#gumlet)
- [ImageEngine](#imageengine)
- [Imgix](#imgix)
- [PixelBin](#pixelbin)
- [Sanity](#sanity)
- [Sirv](#sirv)
- [Supabase](#supabase)
- [Thumbor](#thumbor)
- [Imagekit](#imagekitio)
- [Nitrogen AIO](#nitrogen-aio)

### Akamai

```js
// Docs: https://techdocs.akamai.com/ivm/reference/test-images-on-demand
export default function akamaiLoader() {
  return `https://example.com/$?imwidth=$`
}
```

### AWS CloudFront

```js
// Docs: https://aws.amazon.com/developer/application-security-performance/articles/image-optimization
export default function cloudfrontLoader() {
  const url = new URL(`https://example.com$`)
  url.searchParams.set('format', 'auto')
  url.searchParams.set('width', width.toString())
  url.searchParams.set('quality', (quality || 75).toString())
  return url.href
}
```

### Cloudinary

```js
// Demo: https://res.cloudinary.com/demo/image/upload/w_300,c_limit,q_auto/turtles.jpg
export default function cloudinaryLoader() {
  const params = ['f_auto', 'c_limit', `w_$`, `q_$`]
  return `https://example.com/$$`
}
```

### Cloudflare

```js
// Docs: https://developers.cloudflare.com/images/transform-images
export default function cloudflareLoader() {
  const params = [`width=$`, `quality=$`, 'format=auto']
  return `https://example.com/cdn-cgi/image/$/$`
}
```

### Contentful

```js
// Docs: https://www.contentful.com/developers/docs/references/images-api/
export default function contentfulLoader() {
  const url = new URL(`https://example.com$`)
  url.searchParams.set('fm', 'webp')
  url.searchParams.set('w', width.toString())
  url.searchParams.set('q', (quality || 75).toString())
  return url.href
}
```

### Fastly

```js
// Docs: https://developer.fastly.com/reference/io/
export default function fastlyLoader() {
  const url = new URL(`https://example.com$`)
  url.searchParams.set('auto', 'webp')
  url.searchParams.set('width', width.toString())
  url.searchParams.set('quality', (quality || 75).toString())
  return url.href
}
```

### Gumlet

```js
// Docs: https://docs.gumlet.com/reference/image-transform-size
export default function gumletLoader() {
  const url = new URL(`https://example.com$`)
  url.searchParams.set('format', 'auto')
  url.searchParams.set('w', width.toString())
  url.searchParams.set('q', (quality || 75).toString())
  return url.href
}
```

### ImageEngine

```js
// Docs: https://support.imageengine.io/hc/en-us/articles/360058880672-Directives
export default function imageengineLoader() {
  const compression = 100 - (quality || 50)
  const params = [`w_$`, `cmpr_$`)]
  return `https://example.com$?imgeng=/$
```

### Imgix

```js
// Demo: https://static.imgix.net/daisy.png?format=auto&fit=max&w=300
export default function imgixLoader() {
  const url = new URL(`https://example.com$`)
  const params = url.searchParams
  params.set('auto', params.getAll('auto').join(',') || 'format')
  params.set('fit', params.get('fit') || 'max')
  params.set('w', params.get('w') || width.toString())
  params.set('q', (quality || 50).toString())
  return url.href
}
```

### PixelBin

```js
// Doc (Resize): https://www.pixelbin.io/docs/transformations/basic/resize/#width-w
// Doc (Optimise): https://www.pixelbin.io/docs/optimizations/quality/#image-quality-when-delivering
// Doc (Auto Format Delivery): https://www.pixelbin.io/docs/optimizations/format/#automatic-format-selection-with-f_auto-url-parameter
export default function pixelBinLoader() {
  const name = ''
  const opt = `t.resize(w:$)~t.compress(q:$)`
  return `https://cdn.pixelbin.io/v2/$/$/$?f_auto=true`
}
```

### Sanity

```js
// Docs: https://www.sanity.io/docs/image-urls
export default function sanityLoader() {
  const prj = 'zp7mbokg'
  const dataset = 'production'
  const url = new URL(`https://cdn.sanity.io/images/$/$$`)
  url.searchParams.set('auto', 'format')
  url.searchParams.set('fit', 'max')
  url.searchParams.set('w', width.toString())
  if (quality) 
  return url.href
}
```

### Sirv

```js
// Docs: https://sirv.com/help/articles/dynamic-imaging/
export default function sirvLoader() {
  const url = new URL(`https://example.com$`)
  const params = url.searchParams
  params.set('format', params.getAll('format').join(',') || 'optimal')
  params.set('w', params.get('w') || width.toString())
  params.set('q', (quality || 85).toString())
  return url.href
}
```

### Supabase

```js
// Docs: https://supabase.com/docs/guides/storage/image-transformations#nextjs-loader
export default function supabaseLoader() {
  const url = new URL(`https://example.com$`)
  url.searchParams.set('width', width.toString())
  url.searchParams.set('quality', (quality || 75).toString())
  return url.href
}
```

### Thumbor

```js
// Docs: https://thumbor.readthedocs.io/en/latest/
export default function thumborLoader() {
  const params = [`$x0`, `filters:quality($)`]
  return `https://example.com$$`
}
```

### ImageKit.io

```js
// Docs: https://imagekit.io/docs/image-transformation
export default function imageKitLoader() {
  const params = [`w-$`, `q-$`]
  return `https://ik.imagekit.io/your_imagekit_id/$?tr=$`
}
```

### Nitrogen AIO

```js
// Docs: https://docs.n7.io/aio/intergrations/
export default function aioLoader() {
  const url = new URL(src, window.location.href)
  const params = url.searchParams
  const aioParams = params.getAll('aio')
  aioParams.push(`w-$`)
  if (quality) {
    aioParams.push(`q-$`)
  }
  params.set('aio', aioParams.join(';'))
  return url.href
}
```
