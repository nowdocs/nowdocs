---
title: resume
---

---

## Reference 

### `resume(node, postponedState, options?)` 

Call `resume` to resume rendering a pre-rendered React tree as HTML into a [Readable Web Stream.](https://developer.mozilla.org/en-US/docs/Web/API/ReadableStream)

```js
import  from 'react-dom/server';
import  from './storage';

async function handler(request, writable) 

async function main(frame) {
  // Layer 1
  const controller = new AbortController();
  const prerenderedApp = prerender(

### Further reading 

Resuming behaves like `renderToReadableStream`. For more examples, check out the [usage section of `renderToReadableStream`](/reference/react-dom/server/renderToReadableStream#usage).
The [usage section of `prerender`](/reference/react-dom/static/prerender#usage) includes examples of how to use `prerender` specifically.