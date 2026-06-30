---
title: use
---

 for the  you passed. To determine the context value, React searches the component tree and finds **the closest context provider above** for that particular context.

To pass context to a `Button`, wrap it or one of its parent components into the corresponding context provider.

```js [[1, 3, "ThemeContext"], [2, 3, "\\"dark\\""], [1, 5, "ThemeContext"]]
function MyPage() 

function Form() 
```

It doesn't matter how many layers of components there are between the provider and the `Button`. When a `Button` *anywhere* inside of `Form` calls `use(ThemeContext)`, it will receive `"dark"` as the value.

Unlike [`useContext`](/reference/react/useContext),  can be called in conditionals and loops like .

```js [[1, 2, "if"], [2, 3, "use"]]
function HorizontalRule() {
  if (show) 
  return false;
}
```

 is called from inside a  statement, allowing you to conditionally read values from a Context.

  )
}

function Form() 

function Panel() {
  const theme = use(ThemeContext);
  const className = 'panel-' + theme;
  return (
    
      
      
    
  )
}

function Button() {
  if (show) {
    const theme = use(ThemeContext);
    const className = 'button-' + theme;
    return (
      
        
      
    );
  }
  return false
}
```

```css
.panel-light,
.panel-dark 
.panel-light 

.panel-dark 

.button-light,
.button-dark 

.button-dark 

.button-light 
```

### Reading a Promise from context 

To share asynchronous data without prop drilling, set a Promise as a context value, then read it with `use(context)` and resolve it with `use(promise)`:

```js
import  from 'react';
import  from './UserContext';

function Profile() {
  const userPromise = use(UserContext);
  const user = use(userPromise);
  return ;
}
```

Reading the value requires two `use` calls because the context value itself isn't awaited. See [Before you use context](/learn/passing-data-deeply-with-context#before-you-use-context) for alternatives to consider before reaching for context.

Wrap the components that read the Promise in a [Suspense](/reference/react/Suspense) boundary so only that subtree suspends while the Promise is pending. See [Usage (Promises)](#usage-promises) below for more on reading Promises with `use`.

---

## Usage (Promises) 

### Reading a Promise with `use` 

Call `use` with a Promise to read its resolved value. The component will [suspend](/reference/react/Suspense) while the Promise is pending.

```js [[1, 4, "use(albumsPromise)"]]
import  from 'react';

function Albums() {
  const albums = use(albumsPromise);
  return (
    
      {albums.map(album => (
        
           ()
        
      ))}
    
  );
}
```

Wrap the component that calls  in a [Suspense](/reference/react/Suspense) boundary so React can show a fallback while the Promise is pending. The closest Suspense boundary above the suspending component shows its fallback. Once the Promise resolves, React reads the value with `use` and replaces the fallback with the rendered component.

    
  );
}

function Albums() {
  const albums = use(fetchData('/albums'));
  return (
    
      {albums.map(album => (
        
           ()
        
      ))}
    
  );
}

function Loading() 
```

```js src/data.js hidden
// Note: the way you would do data fetching depends on
// the framework that you use together with Suspense.
// Normally, the caching logic would be inside a framework.

let cache = new Map();

export function fetchData(url) {
  if (!cache.has(url)) 
  return cache.get(url);
}

async function getData(url) {
  if (url === '/albums')  else 
}

async function getAlbums() {
  // Add a fake delay to make waiting noticeable.
  await new Promise(resolve => );

  return [, , , ];
}
```

```json package.json hidden
{
  "dependencies": ,
  "main": "/index.js"
}
```

---

### Caching Promises for Client Components 

Promises passed to `use` in Client Components must be cached so the same Promise instance is reused across re-renders. If a new Promise is created directly in render, React will display the Suspense fallback on every re-render.

```js
// ✅ Cache the Promise so the same one is reused across renders
let cache = new Map();

export function fetchData(url) {
  if (!cache.has(url)) 
  return cache.get(url);
}
```

The `fetchData` function returns the same Promise each time it's called with the same URL. When `use` receives the same Promise on a re-render, it reads the already-resolved value synchronously without suspending.

In the example below, clicking "Re-render" updates state in `App` and triggers a re-render. Because `fetchData` returns the same cached Promise, `Albums` reads the value synchronously instead of showing the Suspense fallback again.

    </>
  );
}

function Albums() {
  const albums = use(fetchData('/albums'));
  return (
    
      {albums.map(album => (
        
           ()
        
      ))}
    
  );
}
```

```js src/data.js hidden
// Note: the way you would do data fetching depends on
// the framework that you use together with Suspense.
// Normally, the caching logic would be inside a framework.

let cache = new Map();

export function fetchData(url) {
  if (!cache.has(url)) 
  return cache.get(url);
}

async function getData(url) {
  if (url === '/albums')  else 
}

async function getAlbums() {
  // Add a fake delay to make waiting noticeable.
  await new Promise(resolve => );

  return [, , ];
}
```

---

### Re-fetching data in Client Components 

To refresh data at the same URL (for example, with a "Refresh" button), invalidate the cache entry and start a new fetch inside a [`startTransition`](/reference/react/startTransition). Store the resulting Promise in state to trigger a re-render. While the new Promise is pending, React keeps showing the existing content because the update is inside a Transition.

```js
function App() {
  const [albumsPromise, setAlbumsPromise] = useState(fetchData('/albums'));
  const [isPending, startTransition] = useTransition();

  function handleRefresh() {
    startTransition(() => );
  }
  // ...
}
```

`refetchData` clears the old cache entry and starts a new fetch at the same URL. Storing the resulting Promise in state triggers a re-render inside the Transition. On re-render, `Albums` receives the new Promise and `use` suspends on it while React keeps showing the old content.

      
    </>
  );
}

function Albums() {
  const albums = use(albumsPromise);
  return (
    
      {albums.map(album => (
        
           ()
        
      ))}
    
  );
}

function Loading() 
```

```js src/data.js hidden
// Note: the way you would do data fetching depends on
// the framework that you use together with Suspense.
// Normally, the caching logic would be inside a framework.

let cache = new Map();

export function fetchData(url) {
  if (!cache.has(url)) 
  return cache.get(url);
}

export function refetchData(url) 

async function getData(url) {
  if (url.startsWith('/the-beatles/albums'))  else 
}

async function getAlbums() {
  // Add a fake delay to make waiting noticeable.
  await new Promise(resolve => );

  return [, , , , ];
}
```

```css
button 
```

---

### Preloading data on hover 

You can start loading data before it's needed by calling `fetchData` during a hover event. Since `fetchData` caches the Promise, the data may already be available by the time the user clicks. If the Promise has resolved by the time `use` reads it, React renders the component immediately without showing a Suspense fallback.

```js
 fetchData(`/$/albums`)}
  onClick={() => {
    startTransition(() => );
  }}
>
```

In this example, hovering over an artist button starts fetching their albums in the background. Without hovering first, clicking shows a loading fallback. Try hovering over a button for a moment before clicking to see the difference.

    </>
  );
}

function Loading() 
```

```js src/Albums.js
import  from 'react';
import  from './data.js';

export default function Albums() {
  const albums = use(fetchData(`/$/albums`));
  return (
    
      {albums.map(album => (
        
           ()
        
      ))}
    
  );
}
```

```js src/data.js hidden
// Note: the way you would do data fetching depends on
// the framework that you use together with Suspense.
// Normally, the caching logic would be inside a framework.

let cache = new Map();

export function fetchData(url) {
  if (!cache.has(url)) {
    const promise = getData(url);
    // Set status fields so React can read the value
    // synchronously if the Promise resolves before
    // `use` is called (e.g. when preloading on hover).
    promise.status = 'pending';
    promise.then(
      value => ,
      reason => ,
    );
    cache.set(url, promise);
  }
  return cache.get(url);
}

async function getData(url) {
  if (url.startsWith('/the-beatles/albums'))  else if (url.startsWith('/led-zeppelin/albums'))  else if (url.startsWith('/pink-floyd/albums'))  else 
}

async function getAlbums(artistId) {
  // Add a fake delay to make waiting noticeable.
  await new Promise(resolve => );

  if (artistId === 'the-beatles') {
    return [, , ];
  } else if (artistId === 'led-zeppelin') {
    return [, , ];
  } else {
    return [, , ];
  }
}
```

```css
button 
```

---

### Streaming data from server to client 

Data can be streamed from the server to the client by passing a Promise as a prop from a Server Component to a Client Component.

```js
import  from './lib.js';
import  from './message.js';

export default function App() 
```

The Client Component then takes the Promise it received as a prop and passes it to the `use` API. This allows the Client Component to read the value from the Promise that was initially created by the Server Component.

```js
// message.js
'use client';

import  from 'react';

export function Message() {
  const messageContent = use(messagePromise);
  return Here is the message: ;
}
```
Because `Message` is wrapped in a [Suspense](/reference/react/Suspense) boundary, the fallback will be displayed until the Promise is resolved. When the Promise is resolved, the value will be read by the `use` API and the `Message` component will replace the Suspense fallback.

  );
}
```

```js src/App.js hidden
import  from "react";
import  from "./message.js";

function fetchMessage() 

export default function App() {
  const [messagePromise, setMessagePromise] = useState(null);
  const [show, setShow] = useState(false);
  function download() 

  if (show) {
    return 
);
```

---

### Displaying an error with an Error Boundary 

If the Promise passed to `use` is rejected, the error propagates to the nearest [Error Boundary](/reference/react/Component#catching-rendering-errors-with-an-error-boundary). Wrap the component that calls `use` in an Error Boundary to display a fallback when the Promise is rejected.

In the example below, `fetchData` rejects on the first attempt and succeeds on retry. The Error Boundary catches the rejection and shows a fallback with a "Try again" button.

    
  );
}

function Albums() {
  const albums = use(albumsPromise);
  return (
    
      {albums.map(album => (
        
           ()
        
      ))}
    
  );
}
```

```js src/data.js hidden
// Note: the way you would do data fetching depends on
// the framework that you use together with Suspense.
// Normally, the caching logic would be inside a framework.

let cache = new Map();
let retried = false;

export function fetchData(url) {
  if (!cache.has(url)) 
  return cache.get(url);
}

export function refetchData(url) 

async function getData(url) {
  // Add a fake delay to make the loading state visible.
  await new Promise(resolve => setTimeout(resolve, 1000));
  if (url === '/the-beatles/albums') {
    // Fail the first attempt to demonstrate the Error Boundary,
    // then succeed on retry.
    if (!retried) 
    return [, , , ];
  }
  throw new Error('Not implemented');
}
```

```json package.json hidden
{
  "dependencies": ,
  "main": "/index.js"
}
```

---

## Troubleshooting 

### I'm getting an error: "Suspense Exception: This is not a real error!" 

You are calling `use` inside a try-catch block. `use` throws internally to integrate with Suspense, so it cannot be wrapped in try-catch. Instead, wrap the component that calls `use` in an [Error Boundary](#displaying-an-error-with-an-error-boundary) to handle errors.

```jsx
function Albums() {
  try  catch (e) 
  // ...
```

Instead, wrap the component in an Error Boundary:

```jsx
function Albums() {
  // ✅ Call `use` without try-catch
  const albums = use(albumsPromise);
  // ...
```

```jsx
// ✅ Use an Error Boundary to handle errors

```

---

### I'm getting a warning: "A component was suspended by an uncached promise" 

The Promise passed to `use` is not cached, so React cannot reuse it across re-renders.

This commonly happens when calling `fetch` or an `async` function directly in render:

```js
function Albums() 
```

To fix this, cache the Promise so the same instance is reused:

```js
// ✅ fetchData returns the same Promise for the same URL
const albums = use(fetchData('/albums'));
```

See [caching Promises for Client Components](#caching-promises-for-client-components) for more details.
