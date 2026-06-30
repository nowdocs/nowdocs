---
title: renderToReadableStream
---

---

## Reference 

### `renderToReadableStream(reactNode, options?)` 

Call `renderToReadableStream` to render your React tree as HTML into a [Readable Web Stream.](https://developer.mozilla.org/en-US/docs/Web/API/ReadableStream)

```js
import  from 'react-dom/server';

async function handler(request) {
  const stream = await renderToReadableStream(, you need to provide a list of . Your root component should return **the entire document including the root `` tag.**

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

### Streaming more content as it loads 

Streaming allows the user to start seeing the content even before all the data has loaded on the server. For example, consider a profile page that shows a cover, a sidebar with friends and photos, and a list of posts:

```js
function ProfilePage() 
```

Imagine that loading data for `
      
    
  );
}
```

This tells React to start streaming the HTML before `Posts` loads its data. React will send the HTML for the loading fallback (`PostsGlimmer`) first, and then, when `Posts` finishes loading its data, React will send the remaining HTML along with an inline `` tag that replaces the loading fallback with that HTML. From the user's perspective, the page will first appear with the `PostsGlimmer`, later replaced by the `Posts`.

You can further [nest `
        
      
    
  );
}
```

In this example, React can start streaming the page even earlier. Only `ProfileLayout` and `ProfileCover` must finish rendering first because they are not wrapped in any `

---

### Specifying what goes into the shell 

The part of your app outside of any `
        
      
    
  );
}
```

It determines the earliest loading state that the user may see:

```js 
```

If an error occurs while rendering those components, React won't have any meaningful HTML to send to the client. Wrap your `renderToReadableStream` call in a `try...catch` to send a fallback HTML that doesn't rely on server rendering as the last resort:

```js 
async function handler(request) {
  try 
```

If an error happens in the `Posts` component or somewhere inside it, React will [try to recover from it:](/reference/react/Suspense#providing-a-fallback-for-server-errors-and-client-only-content)

1. It will emit the loading fallback for the closest `` boundary (`PostsGlimmer`) into the HTML.
2. It will "give up" on trying to render the `Posts` content on the server anymore.
3. When the JavaScript code loads on the client, React will *retry* rendering `Posts` on the client.

If retrying rendering `Posts` on the client *also* fails, React will throw the error on the client. As with all the errors thrown during rendering, the [closest parent error boundary](/reference/react/Component#static-getderivedstatefromerror) determines how to present the error to the user. In practice, this means that the user will see a loading indicator until it is certain that the error is not recoverable.

If retrying rendering `Posts` on the client succeeds, the loading fallback from the server will be replaced with the client rendering output. The user will not know that there was a server error. However, the server `onError` callback and the client [`onRecoverableError`](/reference/react-dom/client/hydrateRoot#hydrateroot) callbacks will fire so that you can get notified about the error.

---

### Setting the status code 

Streaming introduces a tradeoff. You want to start streaming the page as early as possible so that the user can see the content sooner. However, once you start streaming, you can no longer set the response status code.

By [dividing your app](#specifying-what-goes-into-the-shell) into the shell (above all `` boundaries) and the rest of the content, you've already solved a part of this problem. If the shell errors, your `catch` block will run which lets you set the error status code. Otherwise, you know that the app may recover on the client, so you can send "OK".

```js 
async function handler(request) {
  try {
    const stream = await renderToReadableStream(, {
      bootstrapScripts: ['/main.js'],
      onError(error) 
    });
    return new Response(stream, {
      status: 200,
      headers: ,
    });
  } catch (error) {
    return new Response('Something went wrong', {
      status: 500,
      headers: ,
    });
  }
}
```

If a component *outside* the shell (i.e. inside a `` boundary) throws an error, React will not stop rendering. This means that the `onError` callback will fire, but your code will continue running without getting into the `catch` block. This is because React will try to recover from that error on the client, [as described above.](#recovering-from-errors-outside-the-shell)

However, if you'd like, you can use the fact that something has errored to set the status code:

```js 
async function handler(request) {
  try {
    let didError = false;
    const stream = await renderToReadableStream(, {
      bootstrapScripts: ['/main.js'],
      onError(error) 
    });
    return new Response(stream, {
      status: didError ? 500 : 200,
      headers: ,
    });
  } catch (error) {
    return new Response('Something went wrong', {
      status: 500,
      headers: ,
    });
  }
}
```

This will only catch errors outside the shell that happened while generating the initial shell content, so it's not exhaustive. If knowing whether an error occurred for some content is critical, you can move it up into the shell.

---

### Handling different errors in different ways 

You can [create your own `Error` subclasses](https://javascript.info/custom-errors) and use the [`instanceof`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/instanceof) operator to check which error is thrown. For example, you can define a custom `NotFoundError` and throw it from your component. Then you can save the error in `onError` and do something different before returning the response depending on the error type:

```js 
async function handler(request) {
  let didError = false;
  let caughtError = null;

  function getStatusCode() {
    if (didError) {
      if (caughtError instanceof NotFoundError)  else 
    } else 
  }

  try {
    const stream = await renderToReadableStream(, {
      bootstrapScripts: ['/main.js'],
      onError(error) 
    });
    return new Response(stream, {
      status: getStatusCode(),
      headers: ,
    });
  } catch (error) {
    return new Response('Something went wrong', {
      status: getStatusCode(),
      headers: ,
    });
  }
}
```

Keep in mind that once you emit the shell and start streaming, you can't change the status code.

---

### Waiting for all content to load for crawlers and static generation 

Streaming offers a better user experience because the user can see the content as it becomes available.

However, when a crawler visits your page, or if you're generating the pages at the build time, you might want to let all of the content load first and then produce the final HTML output instead of revealing it progressively.

You can wait for all the content to load by awaiting the `stream.allReady` Promise:

```js 
async function handler(request) {
  try {
    let didError = false;
    const stream = await renderToReadableStream(, {
      bootstrapScripts: ['/main.js'],
      onError(error) 
    });
    let isCrawler = // ... depends on your bot detection strategy ...
    if (isCrawler) 
    return new Response(stream, {
      status: didError ? 500 : 200,
      headers: ,
    });
  } catch (error) {
    return new Response('Something went wrong', {
      status: 500,
      headers: ,
    });
  }
}
```

A regular visitor will get a stream of progressively loaded content. A crawler will receive the final HTML output after all the data loads. However, this also means that the crawler will have to wait for *all* data, some of which might be slow to load or error. Depending on your app, you could choose to send the shell to the crawlers too.

---

### Aborting server rendering 

You can force the server rendering to "give up" after a timeout:

```js 
async function handler(request) {
  try {
    const controller = new AbortController();
    setTimeout(() => , 10000);

    const stream = await renderToReadableStream(, {
      signal: controller.signal,
      bootstrapScripts: ['/main.js'],
      onError(error) 
    });
    // ...
```

React will flush the remaining loading fallbacks as HTML, and will attempt to render the rest on the client.
