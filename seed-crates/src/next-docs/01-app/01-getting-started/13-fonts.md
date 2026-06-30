---
title: Font Optimization
description: Learn how to optimize fonts in Next.js
related:
  title: API Reference
  description: See the API Reference for the full feature set of Next.js Font
  links:
    - app/api-reference/components/font
---

The [`next/font`](/docs/app/api-reference/components/font) module automatically optimizes your fonts and removes external network requests for improved privacy and performance.

It includes **built-in self-hosting** for any font file. This means you can optimally load web fonts with no layout shift.

## Google fonts

You can automatically self-host any Google Font. Fonts are included as static assets and served from the same domain as your deployment, meaning no requests are sent to Google by the browser when the user visits your site.

To start using a Google Font, import your chosen font from `next/font/google`:

We recommend using [variable fonts](https://fonts.google.com/variablefonts) for the best performance and flexibility. But if you can't use a variable font, you will need to specify a weight:

## Local fonts

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
