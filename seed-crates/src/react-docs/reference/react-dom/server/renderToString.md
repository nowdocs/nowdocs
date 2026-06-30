---
title: renderToString
---

---

## Alternatives 

### Migrating from `renderToString` to a streaming render on the server 

`renderToString` returns a string immediately, so it does not support streaming content as it loads.

When possible, we recommend using these fully-featured alternatives:

* If you use Node.js, use [`renderToPipeableStream`.](/reference/react-dom/server/renderToPipeableStream)
* If you use Deno or a modern edge runtime with [Web Streams](https://developer.mozilla.org/en-US/docs/Web/API/Streams_API), use [`renderToReadableStream`.](/reference/react-dom/server/renderToReadableStream)

You can continue using `renderToString` if your server environment does not support streams.

---

### Migrating from `renderToString` to a static prerender on the server 

`renderToString` returns a string immediately, so it does not support waiting for data to load for static HTML generation.

We recommend using these fully-featured alternatives:

* If you use Node.js, use [`prerenderToNodeStream`.](/reference/react-dom/static/prerenderToNodeStream)
* If you use Deno or a modern edge runtime with [Web Streams](https://developer.mozilla.org/en-US/docs/Web/API/Streams_API), use [`prerender`.](/reference/react-dom/static/prerender)

You can continue using `renderToString` if your static site generation environment does not support streams.

---

### Removing `renderToString` from the client code 

Sometimes, `renderToString` is used on the client to convert some component to HTML.

```js 
// 🚩 Unnecessary: using renderToString on the client
import  from 'react-dom/server';

const html = renderToString();
console.log(html); // For example, "..."
```

Importing `react-dom/server` **on the client** unnecessarily increases your bundle size and should be avoided. If you need to render some component to HTML in the browser, use [`createRoot`](/reference/react-dom/client/createRoot) and read HTML from the DOM:

```js
import  from 'react-dom/client';
import  from 'react-dom';

const div = document.createElement('div');
const root = createRoot(div);
flushSync(() => );
console.log(div.innerHTML); // For example, "..."
```

The [`flushSync`](/reference/react-dom/flushSync) call is necessary so that the DOM is updated before reading its [`innerHTML`](https://developer.mozilla.org/en-US/docs/Web/API/Element/innerHTML) property.

---

## Troubleshooting 

### When a component suspends, the HTML always contains a fallback 

`renderToString` does not fully support Suspense.

If some component suspends (for example, because it's defined with [`lazy`](/reference/react/lazy) or fetches data), `renderToString` will not wait for its content to resolve. Instead, `renderToString` will find the closest [``](/reference/react/Suspense) boundary above it and render its `fallback` prop in the HTML. The content will not appear until the client code loads.

To solve this, use one of the [recommended streaming solutions.](#alternatives) For server side rendering, they can stream content in chunks as it resolves on the server so that the user sees the page being progressively filled in before the client code loads. For static site generation, they can wait for all the content to resolve before generating the static HTML.

