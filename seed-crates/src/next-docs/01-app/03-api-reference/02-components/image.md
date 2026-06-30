---
title: Image Component
description: Optimize Images in your Next.js Application using the built-in `next/image` Component.
---

The Next.js Image component extends the HTML `` element for automatic image optimization.

```jsx filename="app/page.js"
import Image from 'next/image'

export default function Page() {
  return (
    

## Reference

### Props

The following props are available:

| Prop                                      | Example                                  | Type            | Status     |
| ----------------------------------------- | ---------------------------------------- | --------------- | ---------- |
| [`src`](#src)                             | `src="/profile.png"`                     | String          | Required   |
| [`alt`](#alt)                             | `alt="Picture of the author"`            | String          | Required   |
| [`width`](#width-and-height)                         | `width=`                            | Integer (px)    | -   |
| [`height`](#width-and-height)                       | `height=`                           | Integer (px)    | -   |
| [`fill`](#fill)                           | `fill=`                            | Boolean         | -          |
| [`loader`](#loader)                       | `loader=`                   | Function        | -          |
| [`sizes`](#sizes)                         | `sizes="(max-width: 768px) 100vw, 33vw"` | String          | -          |
| [`quality`](#quality)                     | `quality=`                           | Integer (1-100) | -          |
| [`preload`](#preload)                     | `preload=`                         | Boolean         | -          |
| [`placeholder`](#placeholder)             | `placeholder="blur"`                     | String          | -          |
| [`style`](#style)                         | `style={}`         | Object          | -          |
| [`onLoadingComplete`](#onloadingcomplete) | `onLoadingComplete=`     | Function        | Deprecated |
| [`onLoad`](#onload)                       | `onLoad=`              | Function        | -          |
| [`onError`](#onerror)                     | `onError(event => fail()}`               | Function        | -          |
| [`loading`](#loading)                     | `loading="lazy"`                         | String          | -          |
| [`blurDataURL`](#blurdataurl)             | `blurDataURL="data:image/jpeg..."`       | String          | -          |
| [`unoptimized`](#unoptimized)             | `unoptimized=`                     | Boolean         | -          |
| [`overrideSrc`](#overridesrc)             | `overrideSrc="/seo.png"`                 | String          | -          |
| [`decoding`](#decoding)                   | `decoding="async"`                       | String          | -          |

#### `src`

The source of the image. Can be one of the following:

An internal path string.

```jsx

Alternatively, you can use the [loaderFile](#loaderfile) configuration in `next.config.js` to configure every instance of `next/image` in your application, without passing a prop.

#### `sizes`

Define the sizes of the image at different breakpoints. Used by the browser to choose the most appropriate size from the generated `srcset`.

```jsx
import Image from 'next/image'

export default function Page() {
  return (
    
      

#### `onError`

A callback function that is invoked if the image fails to load.

```jsx

#### `unoptimized`

A boolean that indicates if the image should be optimized. This is useful for images that do not benefit from optimization such as small images (less than 1KB), vector images (SVG), or animated images (GIF).

```js
import Image from 'next/image'

const UnoptimizedImage = (props) => {
  // Default is false
  return 

### Configuration options

You can configure the Image Component in `next.config.js`. The following options are available:

#### `localPatterns`

Use `localPatterns` in your `next.config.js` file to allow images from specific local paths to be optimized and block all others.

```js filename="next.config.js"
module.exports = {
  images: {
    localPatterns: [
      ,
    ],
  },
}
```

The example above will ensure the `src` property of `next/image` must start with `/assets/images/` and must not have a query string. Attempting to optimize any other path will respond with `400` Bad Request error.

> **Good to know**: Omitting the `search` property allows all search parameters which could allow malicious actors to optimize URLs you did not intend. Try using a specific value like `search: '?v=2'` to ensure an exact match.

#### `remotePatterns`

Use `remotePatterns` in your `next.config.js` file to allow images from specific external paths and block all others. This ensures that only external images from your account can be served.

```js filename="next.config.js"
module.exports = {
  images: ,
}
```

You can also configure `remotePatterns` using the object:

```js filename="next.config.js"
module.exports = {
  images: {
    remotePatterns: [
      ,
    ],
  },
}
```

The example above will ensure the `src` property of `next/image` must start with `https://example.com/account123/` and must not have a query string. Any other protocol, hostname, port, or unmatched path will respond with `400` Bad Request.

**Wildcard Patterns:**

Wildcard patterns can be used for both `pathname` and `hostname` and have the following syntax:

- `*` match a single path segment or subdomain
- `**` match any number of path segments at the end or subdomains at the beginning. This syntax does not work in the middle of the pattern.

```js filename="next.config.js"
module.exports = {
  images: {
    remotePatterns: [
      ,
    ],
  },
}
```

This allows subdomains like `image.example.com`. Query strings and custom ports are still blocked.

> **Good to know**: When omitting `protocol`, `port`, `pathname`, or `search` then the wildcard `**` is implied. This is not recommended because it may allow malicious actors to optimize urls you did not intend.

**Query Strings**:

You can also restrict query strings using the `search` property:

```js filename="next.config.js"
module.exports = {
  images: {
    remotePatterns: [
      ,
    ],
  },
}
```

The example above will ensure the `src` property of `next/image` must start with `https://assets.example.com` and must have the exact query string `?v=1727111025337`. Any other protocol or query string will respond with `400` Bad Request.

Note that any allowed `remotePatterns` that respond with a redirect will follow the redirect from the remote image server without validating `remotePatterns` again on the redirect location. You can reduce or disable redirects by configuring [maximumRedirects](#maximumredirects).

#### `loaderFile`

`loaderFiles` allows you to use a custom image optimization service instead of Next.js.

```js filename="next.config.js"
module.exports = {
  images: ,
}
```

The path must be relative to the project root. The file must export a default function that returns a URL string:

**Example:**

- [Custom Image Loader Configuration](/docs/app/api-reference/config/next-config-js/images#example-loader-configuration)

> Alternatively, you can use the [`loader` prop](#loader) to configure each instance of `next/image`.

#### `path`

If you want to change or prefix the default path for the Image Optimization API, you can do so with the `path` property. The default value for `path` is `/_next/image`.

```js filename="next.config.js"
module.exports = {
  images: ,
}
```

#### `deviceSizes`

`deviceSizes` allows you to specify a list of device width breakpoints. These widths are used when the `next/image` component uses [`sizes`](#sizes) prop to ensure the correct image is served for the user's device.

If no configuration is provided, the default below is used:

```js filename="next.config.js"
module.exports = {
  images: ,
}
```

#### `imageSizes`

`imageSizes` allows you to specify a list of image widths. These widths are concatenated with the array of [device sizes](#devicesizes) to form the full array of sizes used to generate image [srcset](https://developer.mozilla.org/docs/Web/API/HTMLImageElement/srcset).

If no configuration is provided, the default below is used:

```js filename="next.config.js"
module.exports = {
  images: ,
}
```

`imageSizes` is only used for images which provide a [`sizes`](#sizes) prop, which indicates that the image is less than the full width of the screen. Therefore, the sizes in `imageSizes` should all be smaller than the smallest size in `deviceSizes`.

#### `qualities`

`qualities` allows you to specify a list of image quality values.

If not configuration is provided, the default below is used:

```js filename="next.config.js"
module.exports = {
  images: ,
}
```

> **Good to know**: This field is required starting with Next.js 16 because unrestricted access could allow malicious actors to optimize more qualities than you intended.

You can add more image qualities to the allowlist, such as the following:

```js filename="next.config.js"
module.exports = {
  images: ,
}
```

In the example above, only four qualities are allowed: 25, 50, 75, and 100.

If the [`quality`](#quality) prop does not match a value in this array, the closest allowed value will be used.

If the REST API is visited directly with a quality that does not match a value in this array, the server will return a 400 Bad Request response.

#### `formats`

`formats` allows you to specify a list of image formats to be used.

```js filename="next.config.js"
module.exports = {
  images: ,
}
```

Next.js automatically detects the browser's supported image formats via the request's `Accept` header in order to determine the best output format.

If the `Accept` header matches more than one of the configured formats, the first match in the array is used. Therefore, the array order matters. If there is no match (or the source image is animated), it will use the original image's format.

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
> - AVIF generally takes 50% longer to encode but it compresses 20% smaller compared to WebP. This means that the first time an image is requested, it will typically be slower, but subsequent requests that are cached will be faster.
> - When using multiple formats, Next.js will cache each format separately. This means increased storage requirements compared to using a single format, as both AVIF and WebP versions of images will be stored for different browser support.
> - If you self-host with a Proxy/CDN in front of Next.js, you must configure the Proxy to forward the `Accept` header.

#### `minimumCacheTTL`

`minimumCacheTTL` allows you to configure the Time to Live (TTL) in seconds for cached optimized images. In many cases, it's better to use a [Static Image Import](/docs/app/getting-started/images#local-images) which will automatically hash the file contents and cache the image forever with a `Cache-Control` header of `immutable`.

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

If you need to change the caching behavior per image, you can configure [`headers`](/docs/app/api-reference/config/next-config-js/headers) to set the `Cache-Control` header on the upstream image (e.g. `/some-asset.jpg`, not `/_next/image` itself).

There is no mechanism to invalidate the cache at this time, so its best to keep `minimumCacheTTL` low. Otherwise you may need to manually change the [`src`](#src) prop or delete the cached file `/cache/images`.

#### `disableStaticImages`

`disableStaticImages` allows you to disable static image imports.

The default behavior allows you to import static files such as `import icon from './icon.png'` and then pass that to the `src` property. In some cases, you may wish to disable this feature if it conflicts with other plugins that expect the import to behave differently.

You can disable static image imports inside your `next.config.js`:

```js filename="next.config.js"
module.exports = {
  images: ,
}
```

#### `maximumRedirects`

The default image optimization loader will follow HTTP redirects when fetching remote images up to 3 times.

```js filename="next.config.js"
module.exports = {
  images: ,
}
```

For your convenience, these redirects do not need to satisfy [remotePatterns](#remotepatterns).

You can configure the number of redirects to follow when fetching remote images. Setting the value to `0` will disable following redirects.

```js filename="next.config.js"
module.exports = {
  images: ,
}
```

#### `maximumDiskCacheSize`

The default image optimization loader will write optimized images to disk so subsequent requests can be served faster from the disk cache.

You can configure the maximum disk cache size in bytes, for example 500 MB:

```js filename="next.config.js"
module.exports = {
  images: ,
}
```

You can also disable the disk cache entirely by setting the value to `0`.

```js filename="next.config.js"
module.exports = {
  images: ,
}
```

If no value is configured, the default behavior is to check the current available disk space once during startup and use 50%.

When the disk cache exceeds the configured size, the least recently used optimized images will be deleted until the cache is under the limit again.

Alternatively, you can implement your own cache handler using [`cacheHandler`](/docs/app/api-reference/config/next-config-js/incrementalCacheHandlerPath) which will ignore the `maximumDiskCacheSize` configuration.

#### `maximumResponseBody`

The default image optimization loader will fetch source images up to 50 MB in size.

```js filename="next.config.js"
module.exports = {
  images: ,
}
```

If you know all your source images are small, you can protect memory constrained servers by reducing this to a smaller value such as 5 MB.

```js filename="next.config.js"
module.exports = {
  images: ,
}
```

#### `dangerouslyAllowLocalIP`

In rare cases when self-hosting Next.js on a private network, you may want to allow optimizing images from local IP addresses on the same network. This is not recommended for most users because it could allow malicious users to access content on your internal network.

By default, the value is false.

```js filename="next.config.js"
module.exports = {
  images: ,
}
```

If you need to optimize remote images hosted elsewhere in your local network, you can set the value to true.

```js filename="next.config.js"
module.exports = {
  images: ,
}
```

This might be necessary when hosting Next.js in a VPC with split-horizon DNS and you receive status 400 Bad Request. Only enable once you understand the SSRF risk.

#### `dangerouslyAllowSVG`

`dangerouslyAllowSVG` allows you to serve SVG images.

```js filename="next.config.js"
module.exports = {
  images: ,
}
```

By default, Next.js does not optimize SVG images for a few reasons:

- SVG is a vector format meaning it can be resized losslessly.
- SVG has many of the same features as HTML/CSS, which can lead to vulnerabilities without proper [Content Security Policy (CSP) headers](/docs/app/api-reference/config/next-config-js/headers#content-security-policy).

We recommend using the [`unoptimized`](#unoptimized) prop when the [`src`](#src) prop is known to be SVG. This happens automatically when `src` ends with `".svg"`.

```jsx

```

In addition, it is strongly recommended to also set `contentDispositionType` to force the browser to download the image, as well as `contentSecurityPolicy` to prevent scripts embedded in the image from executing.

```js filename="next.config.js"
module.exports = {
  images: ,
}
```

#### `contentDispositionType`

`contentDispositionType` allows you to configure the [`Content-Disposition`](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Disposition#as_a_response_header_for_the_main_body) header.

```js filename="next.config.js"
module.exports = {
  images: ,
}
```

#### `contentSecurityPolicy`

`contentSecurityPolicy` allows you to configure the [`Content-Security-Policy`](https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/CSP) header for images. This is particularly important when using [`dangerouslyAllowSVG`](#dangerouslyallowsvg) to prevent scripts embedded in the image from executing.

```js filename="next.config.js"
module.exports = {
  images: ,
}
```

By default, the [loader](#loader) sets the [`Content-Disposition`](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Disposition#as_a_response_header_for_the_main_body) header to `attachment` for added protection since the API can serve arbitrary remote images.

The default value is `attachment` which forces the browser to download the image when visiting directly. This is particularly important when [`dangerouslyAllowSVG`](#dangerouslyallowsvg) is true.

You can optionally configure `inline` to allow the browser to render the image when visiting directly, without downloading it.

### Deprecated configuration options

#### `domains`

> **Warning**: Deprecated since Next.js 14 in favor of strict [`remotePatterns`](#remotepatterns) in order to protect your application from malicious users.

Similar to [`remotePatterns`](#remotepatterns), the `domains` configuration can be used to provide a list of allowed hostnames for external images. However, the `domains` configuration does not support wildcard pattern matching and it cannot restrict protocol, port, or pathname.

Since most remote image servers are shared between multiple tenants, it's safer to use `remotePatterns` to ensure only the intended images are optimized.

Below is an example of the `domains` property in the `next.config.js` file:

```js filename="next.config.js"
module.exports = {
  images: ,
}
```

## Functions

### `getImageProps`

The `getImageProps` function can be used to get the props that would be passed to the underlying `` element, and instead pass them to another component, style, canvas, etc.

```jsx
import  from 'next/image'

const  = getImageProps()

function ImageWithCaption() 
```

This also avoid calling React `useState()` so it can lead to better performance, but it cannot be used with the [`placeholder`](#placeholder) prop because the placeholder will never be removed.

## Known browser bugs

This `next/image` component uses browser native [lazy loading](https://caniuse.com/loading-lazy-attr), which may fallback to eager loading for older browsers before Safari 15.4. When using the blur-up placeholder, older browsers before Safari 12 will fallback to empty placeholder. When using styles with `width`/`height` of `auto`, it is possible to cause [Layout Shift](https://web.dev/cls/) on older browsers before Safari 15 that don't [preserve the aspect ratio](https://caniuse.com/mdn-html_elements_img_aspect_ratio_computed_from_attributes). For more details, see [this MDN video](https://www.youtube.com/watch?v=4-d_SoCHeWE).

- [Safari 15 - 16.3](https://bugs.webkit.org/show_bug.cgi?id=243601) display a gray border while loading. Safari 16.4 [fixed this issue](https://webkit.org/blog/13966/webkit-features-in-safari-16-4/#:~:text=Now%20in%20Safari%2016.4%2C%20a%20gray%20line%20no%20longer%20appears%20to%20mark%20the%20space%20where%20a%20lazy%2Dloaded%20image%20will%20appear%20once%20it%E2%80%99s%20been%20loaded.). Possible solutions:
  - Use CSS `@supports (font: -apple-system-body) and (-webkit-appearance: none) { img[loading="lazy"]  }`
  - Use [`loading="eager"`](#loading) if the image is above the fold
- [Firefox 67+](https://bugzilla.mozilla.org/show_bug.cgi?id=1556156) displays a white background while loading. Possible solutions:
  - Enable [AVIF `formats`](#formats)
  - Use [`placeholder`](#placeholder)

## Examples

### Styling images

Styling the Image component is similar to styling a normal `` element, but there are a few guidelines to keep in mind:

Use `className` or `style`, not `styled-jsx`. In most cases, we recommend using the `className` prop. This can be an imported [CSS Module](/docs/app/getting-started/css), a [global stylesheet](/docs/app/getting-started/css#global-css), etc.

```jsx
import styles from './styles.module.css'

export default function MyImage() 
```

You can also use the `style` prop to assign inline styles.

```jsx
export default function MyImage() 
```

When using `fill`, the parent element must have `position: relative` or `display: block`. This is necessary for the proper rendering of the image element in that layout mode.

```jsx

  

```

You cannot use [styled-jsx](/docs/app/guides/css-in-js) because it's scoped to the current component (unless you mark the style as `global`).

### Responsive images with a static export

When you import a static image, Next.js automatically sets its width and height based on the file. You can make the image responsive by setting the style:

```jsx
import Image from 'next/image'
import mountains from '../public/mountains.jpg'

export default function Responsive() 
```

### Responsive images with a remote URL

If the source image is a dynamic or a remote URL, you must provide the width and height props so Next.js can calculate the aspect ratio:

```jsx filename="components/page.js"
import Image from 'next/image'

export default function Page() 
```

Try it out:

- [Demo the image responsive to viewport](https://image-component.nextjs.gallery/responsive)

### Responsive image with `fill`

If you don't know the aspect ratio of the image, you can add the [`fill` prop](#fill) with the `objectFit` prop set to `cover`. This will make the image fill the full width of its parent container.

```jsx
import Image from 'next/image'
import mountains from '../public/mountains.jpg'

export default function Fill() {
  return (
    
      
        
      
      
    
  )
}
```

### Background Image

Use the `fill` prop to make the image cover the entire screen area:

```jsx
import Image from 'next/image'
import mountains from '../public/mountains.jpg'

export default function Background() 
```

For examples of the Image component used with the various styles, see the [Image Component Demo](https://image-component.nextjs.gallery).

### Remote images

To use a remote image, the `src` property should be a URL string.

```jsx filename="app/page.js"
import Image from 'next/image'

export default function Page() 
```

Since Next.js does not have access to remote files during the build process, you'll need to provide the [`width`](/docs/app/api-reference/components/image#width-and-height), [`height`](/docs/app/api-reference/components/image#width-and-height) and optional [`blurDataURL`](/docs/app/api-reference/components/image#blurdataurl) props manually.

The `width` and `height` attributes are used to infer the correct aspect ratio of image and avoid layout shift from the image loading in. The `width` and `height` do _not_ determine the rendered size of the image file.

To safely allow optimizing images, define a list of supported URL patterns in `next.config.js`. Be as specific as possible to prevent malicious usage. For example, the following configuration will only allow images from a specific AWS S3 bucket:

```js filename="next.config.js"
module.exports = {
  images: {
    remotePatterns: [
      ,
    ],
  },
}
```

### Theme detection

If you want to display a different image for light and dark mode, you can create a new component that wraps two `` components and reveals the correct one based on a CSS media query.

```css filename="components/theme-image.module.css"
.imgDark 

@media (prefers-color-scheme: dark) {
  .imgLight 
  .imgDark 
}
```

```tsx filename="components/theme-image.tsx" switcher
import styles from './theme-image.module.css'
import Image,  from 'next/image'

type Props = Omit & 

const ThemeImage = (props: Props) => {
  const  = props

  return (
    <>
      
      
    </>
  )
}
```

```jsx filename="components/theme-image.js" switcher
import styles from './theme-image.module.css'
import Image from 'next/image'

const ThemeImage = (props) => {
  const  = props

  return (
    <>
      
      
    </>
  )
}
```

> **Good to know**: The default behavior of `loading="lazy"` ensures that only the correct image is loaded. You cannot use `preload` or `loading="eager"` because that would cause both images to load. Instead, you can use [`fetchPriority="high"`](https://developer.mozilla.org/docs/Web/API/HTMLImageElement/fetchPriority).

Try it out:

- [Demo light/dark mode theme detection](https://image-component.nextjs.gallery/theme)

### Art direction

If you want to display a different image for mobile and desktop, sometimes called [Art Direction](https://developer.mozilla.org/en-US/docs/Learn/HTML/Multimedia_and_embedding/Responsive_images#art_direction), you can provide different `src`, `width`, `height`, and `quality` props to `getImageProps()`.

```jsx filename="app/page.js"
import  from 'next/image'

export default function Home() {
  const common = 
  const {
    props: ,
  } = getImageProps()
  const {
    props: ,
  } = getImageProps()

  return (
    
      
      
      
    
  )
}
```

### Background CSS

You can even convert the `srcSet` string to the [`image-set()`](https://developer.mozilla.org/en-US/docs/Web/CSS/image/image-set) CSS function to optimize a background image.

```jsx filename="app/page.js"
import  from 'next/image'

function getBackgroundImage(srcSet = '') {
  const imageSet = srcSet
    .split(', ')
    .map((str) => {
      const [url, dpi] = str.split(' ')
      return `url("$") $`
    })
    .join(', ')
  return `image-set($)`
}

export default function Home() {
  const {
    props: ,
  } = getImageProps()
  const backgroundImage = getBackgroundImage(srcSet)
  const style = 

  return (
    
      Hello World
    
  )
}
```

## Version History

| Version    | Changes                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `v16.1.7`  | `maximumDiskCacheSize` configuration added.                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| `v16.1.2`  | `maximumResponseBody` configuration added.                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| `v16.0.0`  | `qualities` default configuration changed to `[75]`, `preload` prop added, `priority` prop deprecated, `dangerouslyAllowLocalIP` config added, `maximumRedirects` config added.                                                                                                                                                                                                                                                                                                            |
| `v15.3.0`  | `remotePatterns` added support for array of `URL` objects.                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| `v15.0.0`  | `contentDispositionType` configuration default changed to `attachment`.                                                                                                                                                                                                                                                                                                                                                                                                                    |
| `v14.2.23` | `qualities` configuration added.                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| `v14.2.15` | `decoding` prop added and `localPatterns` configuration added.                                                                                                                                                                                                                                                                                                                                                                                                                             |
| `v14.2.14` | `remotePatterns.search` prop added.                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| `v14.2.0`  | `overrideSrc` prop added.                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| `v14.1.0`  | `getImageProps()` is stable.                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| `v14.0.0`  | `onLoadingComplete` prop and `domains` config deprecated.                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| `v13.4.14` | `placeholder` prop support for `data:/image...`                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| `v13.2.0`  | `contentDispositionType` configuration added.                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| `v13.0.6`  | `ref` prop added.                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| `v13.0.0`  | The `next/image` import was renamed to `next/legacy/image`. The `next/future/image` import was renamed to `next/image`. A [codemod is available](/docs/app/guides/upgrading/codemods#next-image-to-legacy-image) to safely and automatically rename your imports. `` wrapper removed. `layout`, `objectFit`, `objectPosition`, `lazyBoundary`, `lazyRoot` props removed. `alt` is required. `onLoadingComplete` receives reference to `img` element. Built-in loader config removed. |
| `v12.3.0`  | `remotePatterns` and `unoptimized` configuration is stable.                                                                                                                                                                                                                                                                                                                                                                                                                                |
| `v12.2.0`  | Experimental `remotePatterns` and experimental `unoptimized` configuration added. `layout="raw"` removed.                                                                                                                                                                                                                                                                                                                                                                                  |
| `v12.1.1`  | `style` prop added. Experimental support for `layout="raw"` added.                                                                                                                                                                                                                                                                                                                                                                                                                         |
| `v12.1.0`  | `dangerouslyAllowSVG` and `contentSecurityPolicy` configuration added.                                                                                                                                                                                                                                                                                                                                                                                                                     |
| `v12.0.9`  | `lazyRoot` prop added.                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| `v12.0.0`  | `formats` configuration added.AVIF support added.Wrapper `` changed to ``.                                                                                                                                                                                                                                                                                                                                                                                            |
| `v11.1.0`  | `onLoadingComplete` and `lazyBoundary` props added.                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| `v11.0.0`  | `src` prop support for static import.`placeholder` prop added.`blurDataURL` prop added.                                                                                                                                                                                                                                                                                                                                                                                          |
| `v10.0.5`  | `loader` prop added.                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| `v10.0.1`  | `layout` prop added.                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| `v10.0.0`  | `next/image` introduced.                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
