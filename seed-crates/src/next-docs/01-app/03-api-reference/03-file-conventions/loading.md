---
title: loading.js
description: API reference for the loading.js file.
---

The special file `loading.js` helps you create meaningful Loading UI with [React Suspense](https://react.dev/reference/react/Suspense). With this convention, you can show an [instant loading state](#instant-loading-states) from the server while the content of a route segment streams in. The new content is automatically swapped in once complete.

      
    
  )
}
```

```jsx filename="app/dashboard/page.js" switcher
import  from 'react'
import  from './Components'

export default function Posts() 
```

By using Suspense, you get the benefits of:

1. **Streaming Server Rendering** - Progressively rendering HTML from the server to the client.
2. **Selective Hydration** - React prioritizes what components to make interactive first based on user interaction.

For more Suspense examples and use cases, please see the [React Documentation](https://react.dev/reference/react/Suspense).

## Version History

| Version   | Changes               |
| --------- | --------------------- |
| `v13.0.0` | `loading` introduced. |
