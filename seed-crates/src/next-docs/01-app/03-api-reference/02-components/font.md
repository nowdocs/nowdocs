---
title: Font Module
nav_title: Font
description: Optimizing loading web fonts with the built-in `next/font` loaders.
---

[`next/font`](/docs/app/api-reference/components/font) automatically optimizes your fonts (including custom fonts) and removes external network requests for improved privacy and performance.

It includes **built-in automatic self-hosting** for any font file. This means you can optimally load web fonts with no [layout shift](https://web.dev/articles/cls).

You can also conveniently use all [Google Fonts](https://fonts.google.com/). CSS and font files are downloaded at build time and self-hosted with the rest of your static assets. **No requests are sent to Google by the browser.**

> **🎥 Watch:** Learn more about using `next/font` → [YouTube (6 minutes)](https://www.youtube.com/watch?v=L8_98i_bMMA).

## Reference

| Key                                         | `font/google`       | `font/local`        | Type                       | Required          |
| ------------------------------------------- | ------------------- | ------------------- | -------------------------- | ----------------- |
| [`src`](#src)                               | 

You can specify multiple weights and/or styles by using an array:

```jsx filename="app/layout.js"
const roboto = Roboto()
```

> **Good to know**: Use an underscore (\_) for font names with multiple words. E.g. `Roboto Mono` should be imported as `Roboto_Mono`.

### Specifying a subset

Google Fonts are automatically [subset](https://fonts.google.com/knowledge/glossary/subsetting). This reduces the size of the font file and improves performance. You'll need to define which of these subsets you want to preload. Failing to specify any subsets while [`preload`](/docs/app/api-reference/components/font#preload) is `true` will result in a warning.

This can be done by adding it to the function call:

View the [Font API Reference](/docs/app/api-reference/components/font) for more information.

## Using Multiple Fonts

You can import and use multiple fonts in your application. There are two approaches you can take.

The first approach is to create a utility function that exports a font, imports it, and applies its `className` where needed. This ensures the font is preloaded only when it's rendered:

```ts filename="app/fonts.ts" switcher
import  from 'next/font/google'

export const inter = Inter()

export const roboto_mono = Roboto_Mono()
```

```js filename="app/fonts.js" switcher
import  from 'next/font/google'

export const inter = Inter()

export const roboto_mono = Roboto_Mono()
```

In the example above, `Inter` will be applied globally, and `Roboto Mono` can be imported and applied as needed.

Alternatively, you can create a [CSS variable](/docs/app/api-reference/components/font#variable) and use it with your preferred CSS solution:

```css filename="app/global.css"
html 

h1 
```

In the example above, `Inter` will be applied globally, and any `` tags will be styled with `Roboto Mono`.

> **Recommendation**: Use multiple fonts conservatively since each new font is an additional resource the client has to download.

### Local Fonts

Import `next/font/local` and specify the `src` of your local font file. We recommend using [variable fonts](https://fonts.google.com/variablefonts) for the best performance and flexibility.

If you want to use multiple files for a single font family, `src` can be an array:

```js
const roboto = localFont({
  src: [
    ,
    ,
    ,
    ,
  ],
})
```

View the [Font API Reference](/docs/app/api-reference/components/font) for more information.

### With Tailwind CSS

`next/font` integrates seamlessly with [Tailwind CSS](https://tailwindcss.com/) using [CSS variables](/docs/app/api-reference/components/font#css-variables).

In the example below, we use the `Inter` and `Roboto_Mono` fonts from `next/font/google` (you can use any Google Font or Local Font). Use the `variable` option to define a CSS variable name, such as `inter` and `roboto_mono` for these fonts, respectively. Then, apply `inter.variable` and `roboto_mono.variable` to include the CSS variables in your HTML document.

> **Good to know**: You can add these variables to the `` or `` tag, depending on your preference, styling needs or project requirements.

Finally, add the CSS variable to your [Tailwind CSS config](/docs/app/getting-started/css#tailwind-css):

```css filename="global.css"
@import 'tailwindcss';

@theme inline 
```

### Tailwind CSS v3

```js filename="tailwind.config.js"
/** @type  */
module.exports = {
  content: [
    './pages/**/*.',
    './components/**/*.',
    './app/**/*.',
  ],
  theme: {
    extend: {
      fontFamily: ,
    },
  },
  plugins: [],
}
```

You can now use the `font-sans` and `font-mono` utility classes to apply the font to your elements.

```html
The quick brown fox ...
The quick brown fox ...
```

### Applying Styles

You can apply the font styles in three ways:

- [`className`](#classname)
- [`style`](#style-1)
- [CSS Variables](#css-variables)

#### `className`

Returns a read-only CSS `className` for the loaded font to be passed to an HTML element.

```tsx
Hello, Next.js!
```

#### `style`

Returns a read-only CSS `style` object for the loaded font to be passed to an HTML element, including `style.fontFamily` to access the font family name and fallback fonts.

```tsx
Hello World
```

#### CSS Variables

If you would like to set your styles in an external style sheet and specify additional options there, use the CSS variable method.

In addition to importing the font, also import the CSS file where the CSS variable is defined and set the variable option of the font loader object as follows:

```tsx filename="app/page.tsx" switcher
import  from 'next/font/google'
import styles from '../styles/component.module.css'

const inter = Inter()
```

```jsx filename="app/page.js" switcher
import  from 'next/font/google'
import styles from '../styles/component.module.css'

const inter = Inter()
```

To use the font, set the `className` of the parent container of the text you would like to style to the font loader's `variable` value and the `className` of the text to the `styles` property from the external CSS file.

```tsx filename="app/page.tsx" switcher

  Hello World

```

```jsx filename="app/page.js" switcher

  Hello World

```

Define the `text` selector class in the `component.module.css` CSS file as follows:

```css filename="styles/component.module.css"
.text 
```

In the example above, the text `Hello World` is styled using the `Inter` font and the generated font fallback with `font-weight: 200` and `font-style: italic`.

### Using a font definitions file

Every time you call the `localFont` or Google font function, that font will be hosted as one instance in your application. Therefore, if you need to use the same font in multiple places, you should load it in one place and import the related font object where you need it. This is done using a font definitions file.

For example, create a `fonts.ts` file in a `styles` folder at the root of your app directory.

Then, specify your font definitions as follows:

```ts filename="styles/fonts.ts" switcher
import  from 'next/font/google'
import localFont from 'next/font/local'

// define your variable fonts
const inter = Inter()
const lora = Lora()
// define 2 weights of a non-variable font
const sourceCodePro400 = Source_Sans_3()
const sourceCodePro700 = Source_Sans_3()
// define a custom local font where GreatVibes-Regular.ttf is stored in the styles folder
const greatVibes = localFont()

export 
```

```js filename="styles/fonts.js" switcher
import  from 'next/font/google'
import localFont from 'next/font/local'

// define your variable fonts
const inter = Inter()
const lora = Lora()
// define 2 weights of a non-variable font
const sourceCodePro400 = Source_Sans_3()
const sourceCodePro700 = Source_Sans_3()
// define a custom local font where GreatVibes-Regular.ttf is stored in the styles folder
const greatVibes = localFont()

export 
```

You can now use these definitions in your code as follows:

```tsx filename="app/page.tsx" switcher
import  from '../styles/fonts'

export default function Page() 
```

```jsx filename="app/page.js" switcher
import  from '../styles/fonts'

export default function Page() 
```

To make it easier to access the font definitions in your code, you can define a path alias in your `tsconfig.json` or `jsconfig.json` files as follows:

```json filename="tsconfig.json"
{
  "compilerOptions": {
    "paths": 
  }
}
```

You can now import any font definition as follows:

```tsx filename="app/about/page.tsx" switcher
import  from '@/fonts'
```

```jsx filename="app/about/page.js" switcher
import  from '@/fonts'
```

### Preloading

## Version Changes

| Version   | Changes                                                               |
| --------- | --------------------------------------------------------------------- |
| `v13.2.0` | `@next/font` renamed to `next/font`. Installation no longer required. |
| `v13.0.0` | `@next/font` was added.                                               |
