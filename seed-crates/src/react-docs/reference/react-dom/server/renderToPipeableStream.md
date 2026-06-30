---
title: renderToPipeableStream
---

---

## Reference 

### `renderToPipeableStream(reactNode, options?)` 

Call `renderToPipeableStream` to render your React tree as HTML into a [Node.js Stream.](https://nodejs.org/api/stream.html#writable-streams)

```js
import  from 'react-dom/server';

const  = renderToPipeableStream(, you need to provide a list of . Your root component should return **the entire document including the root `` tag.**

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

If an error occurs while rendering those components, React won't have any meaningful HTML to send to the client. Override `onShellError` to send a fallback HTML that doesn't rely on server rendering as the last resort:

```js 
const  = renderToPipeableStream(
    
  );
}
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

By [dividing your app](#specifying-what-goes-into-the-shell) into the shell (above all `` boundaries) and the rest of the content, you've already solved a part of this problem. If the shell errors, you'll get the `onShellError` callback which lets you set the error status code. Otherwise, you know that the app may recover on the client, so you can send "OK".

```js 
const  = renderToPipeableStream(, {
  bootstrapScripts: ['/main.js'],
  onShellReady() ,
  onShellError(error) ,
  onError(error) 
});
```

If a component *outside* the shell (i.e. inside a `` boundary) throws an error, React will not stop rendering. This means that the `onError` callback will fire, but you will still get `onShellReady` instead of `onShellError`. This is because React will try to recover from that error on the client, [as described above.](#recovering-from-errors-outside-the-shell)

However, if you'd like, you can use the fact that something has errored to set the status code:

```js 
let didError = false;

const  = renderToPipeableStream(, {
  bootstrapScripts: ['/main.js'],
  onShellReady() ,
  onShellError(error) ,
  onError(error) 
});
```

This will only catch errors outside the shell that happened while generating the initial shell content, so it's not exhaustive. If knowing whether an error occurred for some content is critical, you can move it up into the shell.

---

### Handling different errors in different ways 

You can [create your own `Error` subclasses](https://javascript.info/custom-errors) and use the [`instanceof`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/instanceof) operator to check which error is thrown. For example, you can define a custom `NotFoundError` and throw it from your component. Then your `onError`, `onShellReady`, and `onShellError` callbacks can do something different depending on the error type:

```js 
let didError = false;
let caughtError = null;

function getStatusCode() {
  if (didError) {
    if (caughtError instanceof NotFoundError)  else 
  } else 
}

const  = renderToPipeableStream(, {
  bootstrapScripts: ['/main.js'],
  onShellReady() ,
  onShellError(error) ,
  onError(error) 
});
```

Keep in mind that once you emit the shell and start streaming, you can't change the status code.

---

### Waiting for all content to load for crawlers and static generation 

Streaming offers a better user experience because the user can see the content as it becomes available.

However, when a crawler visits your page, or if you're generating the pages at the build time, you might want to let all of the content load first and then produce the final HTML output instead of revealing it progressively.

You can wait for all the content to load using the `onAllReady` callback:

```js 
let didError = false;
let isCrawler = // ... depends on your bot detection strategy ...

const  = renderToPipeableStream(, {
  bootstrapScripts: ['/main.js'],
  onShellReady() {
    if (!isCrawler) 
  },
  onShellError(error) ,
  onAllReady() {
    if (isCrawler) 
  },
  onError(error) 
});
```

A regular visitor will get a stream of progressively loaded content. A crawler will receive the final HTML output after all the data loads. However, this also means that the crawler will have to wait for *all* data, some of which might be slow to load or error. Depending on your app, you could choose to send the shell to the crawlers too.

---

### Aborting server rendering 

You can force the server rendering to "give up" after a timeout:

```js 
const  = renderToPipeableStream(, );

setTimeout(() => , 10000);
```

React will flush the remaining loading fallbacks as HTML, and will attempt to render the rest on the client.
