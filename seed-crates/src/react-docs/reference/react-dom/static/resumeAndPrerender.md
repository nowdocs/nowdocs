---
title: resumeAndPrerender
---

---

## Reference 

### `resumeAndPrerender(reactNode, postponedState, options?)` 

Call `resumeAndPrerender` to continue a prerendered React tree to a static HTML string.

```js
import  from 'react-dom/static';
import  from 'storage';

async function handler(request, response) {
  const postponedState = getPostponedState(request);
  const  = await resumeAndPrerender(

---

## Usage 

### Further reading 

`resumeAndPrerender` behaves similarly to [`prerender`](/reference/react-dom/static/prerender) but can be used to continue a previously started prerendering process that was aborted.
For more information about resuming a prerendered tree, see the [resume documentation](/reference/react-dom/server/resume#resuming-a-prerender).
