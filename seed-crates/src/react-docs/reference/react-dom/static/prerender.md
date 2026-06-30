---
title: prerender
---

---

## Reference 

### `prerender(reactNode, options?)` 

Call `prerender` to render your app to static HTML.

```js
import  from 'react-dom/static';

async function handler(request, response) {
  const  = await prerender(

---

## Usage 

### Rendering a React tree to a stream of static HTML 

Call `prerender` to render your React tree to static HTML into a [Readable Web Stream:](https://developer.mozilla.org/en-US/docs/Web/API/ReadableStream):

```js [[1, 4, ", you need to provide a list of . Your root component should return **the entire document including the root `` tag.**

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

Call `prerender` to render your app to a static HTML string:

```js
import  from 'react-dom/static';

async function renderToString() {
  const  = await prerender(
      
    
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
    const  = await prerender(, );
    //...
```

Any Suspense boundaries with incomplete children will be included in the prelude in the fallback state.

This can be used for partial prerendering together with [`resume`](/reference/react-dom/server/resume) or [`resumeAndPrerender`](/reference/react-dom/static/resumeAndPrerender).

## Troubleshooting 

### My stream doesn't start until the entire app is rendered 

The `prerender` response waits for the entire app to finish rendering, including waiting for all Suspense boundaries to resolve, before resolving. It is designed for static site generation (SSG) ahead of time and does not support streaming more content as it loads.

To stream content as it loads, use a streaming server render API like [renderToReadableStream](/reference/react-dom/server/renderToReadableStream).
