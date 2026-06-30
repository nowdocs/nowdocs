---
title: lazy
---

```

In this example, the code for `MarkdownPreview` won't be loaded until you attempt to render it. If `MarkdownPreview` hasn't loaded yet, `Loading` will be shown in its place. Try ticking the checkbox:

      )}
    </>
  );
}

// Add a fixed delay so you can see the loading state
function delayForDemo(promise) {
  return new Promise(resolve => ).then(() => promise);
}
```

```js src/Loading.js
export default function Loading() 
```

```js src/MarkdownPreview.js
import  from 'remarkable';

const md = new Remarkable();

export default function MarkdownPreview() 
```

```json package.json hidden
{
  "dependencies": ,
  "scripts": 
}
```

```css
label 

input, textarea 

body 
```

This demo loads with an artificial delay. The next time you untick and tick the checkbox, `Preview` will be cached, so there will be no loading state. To see the loading state again, click "Reset" on the sandbox.

[Learn more about managing loading states with Suspense.](/reference/react/Suspense)

---

## Troubleshooting 

### My `lazy` component's state gets reset unexpectedly 

Do not declare `lazy` components *inside* other components:

```js 
import  from 'react';

function Editor() 
```

Instead, always declare them at the top level of your module:

```js 
import  from 'react';

// ✅ Good: Declare lazy components outside of your components
const MarkdownPreview = lazy(() => import('./MarkdownPreview.js'));

function Editor() 
```
