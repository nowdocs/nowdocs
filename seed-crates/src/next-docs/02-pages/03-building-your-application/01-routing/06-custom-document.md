---
title: Custom Document
description: Extend the default document markup added by Next.js.
---

A custom `Document` can update the `` and `` tags used to render a [Page](/docs/pages/building-your-application/routing/pages-and-layouts).

To override the default `Document`, create the file `pages/_document` as shown below:

```tsx filename="pages/_document.tsx" switcher
import  from 'next/document'

export default function Document() 
```

```jsx filename="pages/_document.jsx" switcher
import  from 'next/document'

export default function Document() 
```

> **Good to know**:
>
> - `_document` is only rendered on the server, so event handlers like `onClick` cannot be used in this file.
> - `
    )
  }
}

export default MyDocument
```

```jsx filename="pages/_document.jsx" switcher
import Document,  from 'next/document'

class MyDocument extends Document {
  static async getInitialProps(ctx) {
    const originalRenderPage = ctx.renderPage

    // Run the React rendering logic synchronously
    ctx.renderPage = () =>
      originalRenderPage()

    // Run the parent `getInitialProps`, it now includes the custom `renderPage`
    const initialProps = await Document.getInitialProps(ctx)

    return initialProps
  }

  render() 
}

export default MyDocument
```

> **Good to know**:
>
> - `getInitialProps` in `_document` is not called during client-side transitions.
> - The `ctx` object for `_document` is equivalent to the one received in [`getInitialProps`](/docs/pages/api-reference/functions/get-initial-props#context-object), with the addition of `renderPage`.
