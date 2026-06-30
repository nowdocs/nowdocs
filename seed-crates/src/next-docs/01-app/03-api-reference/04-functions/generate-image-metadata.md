---
title: generateImageMetadata
description: Learn how to generate multiple images in a single Metadata API special file.
related:
  title: Next Steps
  description: View all the Metadata API options.
  links:
    - app/api-reference/file-conventions/metadata
---

You can use `generateImageMetadata` to generate different versions of one image or return multiple images for one route segment. This is useful for when you want to avoid hard-coding metadata values, such as for icons.

## Parameters

`generateImageMetadata` function accepts the following parameters:

#### `params` (optional)

An object containing the [dynamic route parameters](/docs/app/api-reference/file-conventions/dynamic-routes) object from the root segment down to the segment `generateImageMetadata` is called from.

```tsx filename="icon.tsx" switcher
export function generateImageMetadata(: {
  params: 
}) 
```

```jsx filename="icon.js" switcher
export function generateImageMetadata() 
```

| Route                           | URL         | `params`                  |
| ------------------------------- | ----------- | ------------------------- |
| `app/shop/icon.js`              | `/shop`     | `undefined`               |
| `app/shop/[slug]/icon.js`       | `/shop/1`   | ``           |
| `app/shop/[tag]/[item]/icon.js` | `/shop/1/2` | `` |

## Returns

The `generateImageMetadata` function should return an `array` of objects containing the image's metadata such as `alt` and `size`. In addition, each item **must** include an `id` value which will be passed as a promise to the props of the image generating function.

| Image Metadata Object | Type                                |
| --------------------- | ----------------------------------- |
| `id`                  | `string` (required)                 |
| `alt`                 | `string`                            |
| `size`                | `` |
| `contentType`         | `string`                            |

```tsx filename="icon.tsx" switcher
import  from 'next/og'

export function generateImageMetadata() {
  return [
    {
      contentType: 'image/png',
      size: ,
      id: 'small',
    },
    {
      contentType: 'image/png',
      size: ,
      id: 'medium',
    },
  ]
}

export default async function Icon(: ) {
  const iconId = await id
  return new ImageResponse(
    (
      
        Icon 
      
    )
  )
}
```

```jsx filename="icon.js" switcher
import  from 'next/og'

export function generateImageMetadata() {
  return [
    {
      contentType: 'image/png',
      size: ,
      id: 'small',
    },
    {
      contentType: 'image/png',
      size: ,
      id: 'medium',
    },
  ]
}

export default async function Icon() {
  const iconId = await id
  return new ImageResponse(
    (
      
        Icon 
      
    )
  )
}
```

## Image generation function props

When using `generateImageMetadata`, the default export image generation function receives the following props:

#### `id`

A promise that resolves to the `id` value from one of the items returned by `generateImageMetadata`. The `id` will be a `string` or `number` depending on what was returned from `generateImageMetadata`.

```tsx filename="icon.tsx" switcher
export default async function Icon(: ) 
```

```jsx filename="icon.js" switcher
export default async function Icon() 
```

#### `params` (optional)

A promise that resolves to an object containing the [dynamic route parameters](/docs/app/api-reference/file-conventions/dynamic-routes) from the root segment down to the segment the image is colocated in.

```tsx filename="icon.tsx" switcher
export default async function Icon(: {
  params: Promise<>
}) {
  const  = await params
  // Use slug to generate the image
}
```

```jsx filename="icon.js" switcher
export default async function Icon() {
  const  = await params
  // Use slug to generate the image
}
```

### Examples

#### Using external data

This example uses the `params` object and external data to generate multiple [Open Graph images](/docs/app/api-reference/file-conventions/metadata/opengraph-image) for a route segment.

```tsx filename="app/products/[id]/opengraph-image.tsx" switcher
import  from 'next/og'
import  from '@/app/utils/images'

export async function generateImageMetadata(: {
  params: 
}) {
  const images = await getOGImages(params.id)

  return images.map((image, idx) => ({
    id: idx,
    size: ,
    alt: image.text,
    contentType: 'image/png',
  }))
}

export default async function Image(: {
  params: Promise<>
  id: Promise
}) {
  const productId = (await params).id
  const imageId = await id
  const text = await getCaptionForImage(productId, imageId)

  return new ImageResponse(
    (
      
        
      
    )
  )
}
```

```jsx filename="app/products/[id]/opengraph-image.js" switcher
import  from 'next/og'
import  from '@/app/utils/images'

export async function generateImageMetadata() {
  const images = await getOGImages(params.id)

  return images.map((image, idx) => ({
    id: idx,
    size: ,
    alt: image.text,
    contentType: 'image/png',
  }))
}

export default async function Image() {
  const productId = (await params).id
  const imageId = await id
  const text = await getCaptionForImage(productId, imageId)

  return new ImageResponse(
    (
      
        
      
    )
  )
}
```

## Version History

| Version   | Changes                                                                                             |
| --------- | --------------------------------------------------------------------------------------------------- |
| `v16.0.0` | `id` passed to the Image generation function is now a promise that resolves to `string` or `number` |
| `v16.0.0` | `params` passed to the Image generation function is now a promise that resolves to an object        |
| `v13.3.0` | `generateImageMetadata` introduced.                                                                 |
