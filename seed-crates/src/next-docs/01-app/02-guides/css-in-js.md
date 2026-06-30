---
title: How to use CSS-in-JS libraries
nav_title: CSS-in-JS
description: Use CSS-in-JS libraries with Next.js
---

}
```

```jsx filename="app/registry.js" switcher
'use client'

import React,  from 'react'
import  from 'next/navigation'
import  from 'styled-jsx'

export default function StyledJsxRegistry() {
  // Only create stylesheet once with lazy initial state
  // x-ref: https://reactjs.org/docs/hooks-reference.html#lazy-initial-state
  const [jsxStyleRegistry] = useState(() => createStyleRegistry())

  useServerInsertedHTML(() => {
    const styles = jsxStyleRegistry.styles()
    jsxStyleRegistry.flush()
    return <></>
  })

  return 
}
```

Then, wrap your [root layout](/docs/app/api-reference/file-conventions/layout#root-layout) with the registry:

```tsx filename="app/layout.tsx" switcher
import StyledJsxRegistry from './registry'

export default function RootLayout(: ) 
```

```jsx filename="app/layout.js" switcher
import StyledJsxRegistry from './registry'

export default function RootLayout() 
```

[View an example here](https://github.com/vercel/next.js/tree/canary/examples/with-styled-jsx).

### Styled Components

Below is an example of how to configure `styled-components@6` or newer:

First, enable styled-components in `next.config.js`.

```js filename="next.config.js"
module.exports = {
  compiler: ,
}
```

Then, use the `styled-components` API to create a global registry component to collect all CSS style rules generated during a render, and a function to return those rules. Then use the `useServerInsertedHTML` hook to inject the styles collected in the registry into the `` HTML tag in the root layout.

```tsx filename="lib/registry.tsx" switcher
'use client'

import React,  from 'react'
import  from 'next/navigation'
import  from 'styled-components'

export default function StyledComponentsRegistry(: ) {
  // Only create stylesheet once with lazy initial state
  // x-ref: https://reactjs.org/docs/hooks-reference.html#lazy-initial-state
  const [styledComponentsStyleSheet] = useState(() => new ServerStyleSheet())

  useServerInsertedHTML(() => {
    const styles = styledComponentsStyleSheet.getStyleElement()
    styledComponentsStyleSheet.instance.clearTag()
    return <></>
  })

  if (typeof window !== 'undefined') return <></>

  return (
    
  )
}
```

```jsx filename="lib/registry.js" switcher
'use client'

import React,  from 'react'
import  from 'next/navigation'
import  from 'styled-components'

export default function StyledComponentsRegistry() {
  // Only create stylesheet once with lazy initial state
  // x-ref: https://reactjs.org/docs/hooks-reference.html#lazy-initial-state
  const [styledComponentsStyleSheet] = useState(() => new ServerStyleSheet())

  useServerInsertedHTML(() => {
    const styles = styledComponentsStyleSheet.getStyleElement()
    styledComponentsStyleSheet.instance.clearTag()
    return <></>
  })

  if (typeof window !== 'undefined') return <></>

  return (
    
  )
}
```

Wrap the `children` of the root layout with the style registry component:

```tsx filename="app/layout.tsx" switcher
import StyledComponentsRegistry from './lib/registry'

export default function RootLayout(: ) 
```

```jsx filename="app/layout.js" switcher
import StyledComponentsRegistry from './lib/registry'

export default function RootLayout() 
```

[View an example here](https://github.com/vercel/next.js/tree/canary/examples/with-styled-components).

> **Good to know**:
>
> - During server rendering, styles will be extracted to a global registry and flushed to the `` of your HTML. This ensures the style rules are placed before any content that might use them. In the future, we may use an upcoming React feature to determine where to inject the styles.
> - During streaming, styles from each chunk will be collected and appended to existing styles. After client-side hydration is complete, `styled-components` will take over as usual and inject any further dynamic styles.
> - We specifically use a Client Component at the top level of the tree for the style registry because it's more efficient to extract CSS rules this way. It avoids re-generating styles on subsequent server renders, and prevents them from being sent in the Server Component payload.
> - For advanced use cases where you need to configure individual properties of styled-components compilation, you can read our [Next.js styled-components API reference](/docs/architecture/nextjs-compiler#styled-components) to learn more.

