---
title: Image (Legacy)
description: Backwards compatible Image Optimization with the Legacy Image component.
version: legacy
---

Starting with Next.js 13, the `next/image` component was rewritten to improve both the performance and developer experience. In order to provide a backwards compatible upgrade solution, the old `next/image` was renamed to `next/legacy/image`.

> **Warning**: `next/legacy/image` is deprecated and will be removed in a future version of Next.js. Please use [`next/image`](/docs/app/api-reference/components/image) instead.

## Comparison

Compared to `next/legacy/image`, the new `next/image` component has the following changes:

- Removes `` wrapper around `` in favor of [native computed aspect ratio](https://caniuse.com/mdn-html_elements_img_aspect_ratio_computed_from_attributes)
- Adds support for canonical `style` prop
  - Removes `layout` prop in favor of `style` or `className`
  - Removes `objectFit` prop in favor of `style` or `className`
  - Removes `objectPosition` prop in favor of `style` or `className`
- Removes `IntersectionObserver` implementation in favor of [native lazy loading](https://caniuse.com/loading-lazy-attr)
  - Removes `lazyBoundary` prop since there is no native equivalent
  - Removes `lazyRoot` prop since there is no native equivalent
- Removes `loader` config in favor of [`loader`](#loader) prop
- Changed `alt` prop from optional to required
- Changed `onLoadingComplete` callback to receive reference to `` element

## Required Props

The `
  )
}
```

[Learn more](https://developer.mozilla.org/docs/Web/API/IntersectionObserver/root)

### unoptimized

When true, the source image will be served as-is from the `src` instead of changing quality, size, or format. Defaults to `false`.

This is useful for images that do not benefit from optimization such as small images (less than 1KB), vector images (SVG), or animated images (GIF).

```js
import Image from 'next/image'

const UnoptimizedImage = (props) => 
```

Since Next.js 12.3.0, this prop can be assigned to all images by updating `next.config.js` with the following configuration:

```js filename="next.config.js"
module.exports = {
  images: ,
}
```

## Other Props

Other properties on the `` component will be passed to the underlying
`img` element with the exception of the following:

- `srcSet`. Use
  [Device Sizes](#device-sizes)
  instead.
- `ref`. Use [`onLoadingComplete`](#onloadingcomplete) instead.
- `decoding`. It is always `"async"`.

## Configuration Options

### Remote Patterns

To protect your application from malicious users, configuration is required in order to use external images. This ensures that only external images from your account can be served from the Next.js Image Optimization API. These external images can be configured with the `remotePatterns` property in your `next.config.js` file, as shown below:

```js filename="next.config.js"
module.exports = {
  images: {
    remotePatterns: [
      ,
    ],
  },
}
```

> **Good to know**: The example above will ensure the `src` property of `next/legacy/image` must start with `https://example.com/account123/` and must not have a query string. Any other protocol, hostname, port, or unmatched path will respond with 400 Bad Request.

Below is an example of the `remotePatterns` property in the `next.config.js` file using a wildcard pattern in the `hostname`:

```js filename="next.config.js"
module.exports = {
  images: {
    remotePatterns: [
      ,
    ],
  },
}
```

> **Good to know**: The example above will ensure the `src` property of `next/legacy/image` must start with `https://img1.example.com` or `https://me.avatar.example.com` or any number of subdomains. It cannot have a port or query string. Any other protocol or unmatched hostname will respond with 400 Bad Request.

Wildcard patterns can be used for both `pathname` and `hostname` and have the following syntax:

- `*` match a single path segment or subdomain
- `**` match any number of path segments at the end or subdomains at the beginning

The `**` syntax does not work in the middle of the pattern.

> **Good to know**: When omitting `protocol`, `port`, `pathname`, or `search` then the wildcard `**` is implied. This is not recommended because it may allow malicious actors to optimize urls you did not intend.

Below is an example of the `remotePatterns` property in the `next.config.js` file using `search`:

```js filename="next.config.js"
module.exports = {
  images: {
    remotePatterns: [
      ,
    ],
  },
}
```

> **Good to know**: The example above will ensure the `src` property of `next/legacy/image` must start with `https://assets.example.com` and must have the exact query string `?v=1727111025337`. Any other protocol or query string will respond with 400 Bad Request.

### Domains

> **Warning**: Deprecated since Next.js 14 in favor of strict [`remotePatterns`](#remote-patterns) in order to protect your application from malicious users. Only use `domains` if you own all the content served from the domain.

Similar to [`remotePatterns`](#remote-patterns), the `domains` configuration can be used to provide a list of allowed hostnames for external images.

However, the `domains` configuration does not support wildcard pattern matching and it cannot restrict protocol, port, or pathname.

Below is an example of the `domains` property in the `next.config.js` file:

```js filename="next.config.js"
module.exports = {
  images: ,
}
```

### Loader Configuration

If you want to use a cloud provider to optimize images instead of using the Next.js built-in Image Optimization API, you can configure the `loader` and `path` prefix in your `next.config.js` file. This allows you to use relative URLs for the Image [`src`](#src) and automatically generate the correct absolute URL for your provider.

```js filename="next.config.js"
module.exports = {
  images: ,
}
```

#### Customizing the Built-in Image Path

If you want to change or prefix the default path for the built-in Next.js image optimization, you can do so with the `path` property. The default value for `path` is `/_next/image`.

```js filename="next.config.js"
module.exports = {
  images: ,
}
```

### Built-in Loaders

The following Image Optimization cloud providers are included:

- Default: Works automatically with `next dev`, `next start`, or a custom server
- [Vercel](https://vercel.com): Works automatically when you deploy on Vercel, no configuration necessary. [Learn more](https://vercel.com/docs/concepts/image-optimization?utm_source=next-site&utm_medium=docs&utm_campaign=next-website)
- [Imgix](https://www.imgix.com): `loader: 'imgix'`
- [Cloudinary](https://cloudinary.com): `loader: 'cloudinary'`
- [Akamai](https://www.akamai.com): `loader: 'akamai'`
- Custom: `loader: 'custom'` use a custom cloud provider by implementing the [`loader`](#loader) prop on the `next/legacy/image` component

If you need a different provider, you can use the [`loader`](#loader) prop with `next/legacy/image`.

> Images cannot be optimized at build time using [`output: 'export'`](/docs/pages/guides/static-exports), only on-demand. To use `next/legacy/image` with `output: 'export'`, you will need to use a different loader than the default. [Read more in the discussion.](https://github.com/vercel/next.js/discussions/19065)

## Advanced

The following configuration is for advanced use cases and is usually not necessary. If you choose to configure the properties below, you will override any changes to the Next.js defaults in future updates.

### Device Sizes

If you know the expected device widths of your users, you can specify a list of device width breakpoints using the `deviceSizes` property in `next.config.js`. These widths are used when the `next/legacy/image` component uses `layout="responsive"` or `layout="fill"` to ensure the correct image is served for user's device.

If no configuration is provided, the default below is used.

```js filename="next.config.js"
module.exports = {
  images: ,
}
```

### Image Sizes

You can specify a list of image widths using the `images.imageSizes` property in your `next.config.js` file. These widths are concatenated with the array of [device sizes](#device-sizes) to form the full array of sizes used to generate image [srcset](https://developer.mozilla.org/docs/Web/API/HTMLImageElement/srcset)s.

The reason there are two separate lists is that imageSizes is only used for images which provide a [`sizes`](#sizes) prop, which indicates that the image is less than the full width of the screen. **Therefore, the sizes in imageSizes should all be smaller than the smallest size in deviceSizes.**

If no configuration is provided, the default below is used.

```js filename="next.config.js"
module.exports = {
  images: ,
}
```

### Acceptable Formats

The default [Image Optimization API](#loader-configuration) will automatically detect the browser's supported image formats via the request's `Accept` header in order to determine the best output format.

If the `Accept` header matches more than one of the configured formats, the first match in the array is used. Therefore, the array order matters. If there is no match (or the source image is [animated](#animated-images)), the Image Optimization API will fallback to the original image's format.

If no configuration is provided, the default below is used.

```js filename="next.config.js"
module.exports = {
  images: ,
}
```

You can enable AVIF support, which will fallback to the original format of the src image if the browser [does not support AVIF](https://caniuse.com/avif):

```js filename="next.config.js"
module.exports = {
  images: ,
}
```

You can also enable both AVIF and WebP formats together. AVIF will be preferred for browsers that support it, with WebP as a fallback:

```js filename="next.config.js"
module.exports = {
  images: ,
}
```

> **Good to know**:
>
> - We still recommend using WebP for most use cases.
> - AVIF generally takes 50% longer to encode but it compresses 20% smaller compared to WebP. This means that the first time an image is requested, it will typically be slower and then subsequent requests that are cached will be faster.
> - When using multiple formats, Next.js will cache each format separately. This means increased storage requirements compared to using a single format, as both AVIF and WebP versions of images will be stored for different browser support.
> - If you self-host with a Proxy/CDN in front of Next.js, you must configure the Proxy to forward the `Accept` header.

## Caching Behavior

The following describes the caching algorithm for the default [loader](#loader). For all other loaders, please refer to your cloud provider's documentation.

Images are optimized dynamically upon request and stored in the `/cache/images` directory. The optimized image file will be served for subsequent requests until the expiration is reached. When a request is made that matches a cached but expired file, the expired image is served stale immediately. Then the image is optimized again in the background (also called revalidation) and saved to the cache with the new expiration date.

The cache status of an image can be determined by reading the value of the `x-nextjs-cache` (`x-vercel-cache` when deployed on Vercel) response header. The possible values are the following:

- `MISS` - the path is not in the cache (occurs at most once, on the first visit)
- `STALE` - the path is in the cache but exceeded the revalidate time so it will be updated in the background
- `HIT` - the path is in the cache and has not exceeded the revalidate time

The expiration (or rather Max Age) is defined by either the [`minimumCacheTTL`](#minimum-cache-ttl) configuration or the upstream image `Cache-Control` header, whichever is larger. Specifically, the `max-age` value of the `Cache-Control` header is used. If both `s-maxage` and `max-age` are found, then `s-maxage` is preferred. The `max-age` is also passed-through to any downstream clients including CDNs and browsers.

- You can configure [`minimumCacheTTL`](#minimum-cache-ttl) to increase the cache duration when the upstream image does not include `Cache-Control` header or the value is very low.
- You can configure [`deviceSizes`](#device-sizes) and [`imageSizes`](#image-sizes) to reduce the total number of possible generated images.
- You can configure [formats](#acceptable-formats) to disable multiple formats in favor of a single image format.

### Minimum Cache TTL

You can configure the Time to Live (TTL) in seconds for cached optimized images. In many cases, it's better to use a [Static Image Import](/docs/pages/api-reference/components/image#src) which will automatically hash the file contents and cache the image forever with a `Cache-Control` header of `immutable`.

If no configuration is provided, the default below is used.

```js filename="next.config.js"
module.exports = {
  images: ,
}
```

You can increase the TTL to reduce the number of revalidations and potentially lower cost:

```js filename="next.config.js"
module.exports = {
  images: ,
}
```

The expiration (or rather Max Age) of the optimized image is defined by either the `minimumCacheTTL` or the upstream image `Cache-Control` header, whichever is larger.

If you need to change the caching behavior per image, you can configure [`headers`](/docs/pages/api-reference/config/next-config-js/headers) to set the `Cache-Control` header on the upstream image (e.g. `/some-asset.jpg`, not `/_next/image` itself).

There is no mechanism to invalidate the cache at this time, so its best to keep `minimumCacheTTL` low. Otherwise you may need to manually change the [`src`](#src) prop or delete `/cache/images`.

### Disable Static Imports

The default behavior allows you to import static files such as `import icon from './icon.png'` and then pass that to the `src` property.

In some cases, you may wish to disable this feature if it conflicts with other plugins that expect the import to behave differently.

You can disable static image imports inside your `next.config.js`:

```js filename="next.config.js"
module.exports = {
  images: ,
}
```

### Dangerously Allow SVG

The default [loader](#loader) does not optimize SVG images for a few reasons. First, SVG is a vector format meaning it can be resized losslessly. Second, SVG has many of the same features as HTML/CSS, which can lead to vulnerabilities without proper [Content Security Policy (CSP) headers](/docs/app/api-reference/config/next-config-js/headers#content-security-policy).

Therefore, we recommended using the [`unoptimized`](#unoptimized) prop when the [`src`](#src) prop is known to be SVG. This happens automatically when `src` ends with `".svg"`.

However, if you need to serve SVG images with the default Image Optimization API, you can set `dangerouslyAllowSVG` inside your `next.config.js`:

```js filename="next.config.js"
module.exports = {
  images: ,
}
```

In addition, it is strongly recommended to also set `contentDispositionType` to force the browser to download the image, as well as `contentSecurityPolicy` to prevent scripts embedded in the image from executing.

### `contentDispositionType`

The default [loader](#loader) sets the [`Content-Disposition`](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Disposition#as_a_response_header_for_the_main_body) header to `attachment` for added protection since the API can serve arbitrary remote images.

The default value is `attachment` which forces the browser to download the image when visiting directly. This is particularly important when [`dangerouslyAllowSVG`](#dangerously-allow-svg) is true.

You can optionally configure `inline` to allow the browser to render the image when visiting directly, without downloading it.

```js filename="next.config.js"
module.exports = {
  images: ,
}
```

### Animated Images

The default [loader](#loader) will automatically bypass Image Optimization for animated images and serve the image as-is.

Auto-detection for animated files is best-effort and supports GIF, APNG, and WebP. If you want to explicitly bypass Image Optimization for a given animated image, use the [unoptimized](#unoptimized) prop.

## Version History

| Version   | Changes                                                                                                             |
| --------- | ------------------------------------------------------------------------------------------------------------------- |
| `v16.0.0` | `next/legacy/image` deprecated and will be removed in a future version of Next.js. Please use `next/image` instead. |
| `v13.0.0` | `next/image` renamed to `next/legacy/image`                                                                         |
