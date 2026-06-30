---
title: prerenderToNodeStream
---

---

## Reference 

### `prerenderToNodeStream(reactNode, options?)` 

Call `prerenderToNodeStream` to render your app to static HTML.

```js
import  from 'react-dom/static';

// The route handler syntax depends on your backend framework
app.use('/', async (request, response) => {
  const  = await prerenderToNodeStream(

---

## Usage 

### Rendering a React tree to a stream of static HTML 

Call `prerenderToNodeStream` to render your React tree to static HTML into a [Node.js Stream](https://nodejs.org/api/stream.html):

```js [[1, 5, ", you need to provide a list of . Your root component should return **the entire document including the root `` tag.**

For example, it might look like this:

```js [[1, 1, "App"]]
export default function App() {
  return (
    
      
        
        
        
        My app
      
      
         into the resulting HTML stream:

```html [[2, 5, "/main.js"]]
<!DOCTYPE html>

  

```

On the client, your bootstrap script should [hydrate the entire `document` with a call to `hydrateRoot`:](/reference/react-dom/client/hydrateRoot#hydrating-an-entire-document)

```js [[1, 4, "

---

### Rendering a React tree to a string of static HTML 

Call `prerenderToNodeStream` to render your app to a static HTML string:

```js
import  from 'react-dom/static';

async function renderToString() {
  const  = await prerenderToNodeStream(
      
    
  );
}
```

Imagine that `

---

### Aborting prerendering 

You can force the prerender to "give up" after a timeout:

```js 
async function renderToString() {
  const controller = new AbortController();
  setTimeout(() => , 10000);

  try {
    // the prelude will contain all the HTML that was prerendered
    // before the controller aborted.
    const  = await prerenderToNodeStream(, );
    //...
```

Any Suspense boundaries with incomplete children will be included in the prelude in the fallback state.

This can be used for partial prerendering together with [`resumeToPipeableStream`](/reference/react-dom/server/resumeToPipeableStream) or [`resumeAndPrerenderToNodeStream`](/reference/react-dom/static/resumeAndPrerenderToNodeStream).

## Troubleshooting 

### My stream doesn't start until the entire app is rendered 

The `prerenderToNodeStream` response waits for the entire app to finish rendering, including waiting for all Suspense boundaries to resolve, before resolving. It is designed for static site generation (SSG) ahead of time and does not support streaming more content as it loads.

To stream content as it loads, use a streaming server render API like [renderToPipeableStream](/reference/react-dom/server/renderToPipeableStream).
