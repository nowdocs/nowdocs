---
title: 
```

```

React will display your  until all the code and data needed by  has been loaded.

In the example below, the `Albums` component *suspends* while fetching the list of albums. Until it's ready to render, React switches the closest Suspense boundary above to show the fallback--your `Loading` component. Then, when the data loads, React hides the `Loading` fallback and renders the `Albums` component with data.

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
  if (!cache.has(url)) 
  return cache.get(url);
}

async function getData(url) {
  if (url === '/the-beatles/albums')  else 
}

async function getAlbums() {
  // Add a fake delay to make waiting noticeable.
  await new Promise(resolve => );

  return [, , , , , , , , , , , , ];
}
```

---

### Revealing content together at once 

By default, the whole tree inside Suspense is treated as a single unit. For example, even if *only one* of these components suspends waiting for some data, *all* of them together will be replaced by the loading indicator:

```js 

```

Then, after all of them are ready to be displayed, they will all appear together at once.

In the example below, both `Biography` and `Albums` fetch some data. However, because they are grouped under a single Suspense boundary, these components always "pop in" together at the same time.

      
    </>
  );
}

function Loading() 
```

```js src/Panel.js
export default function Panel() {
  return (
    
      
    
  );
}
```

```js src/Biography.js
import  from 'react';
import  from './data.js';

export default function Biography() {
  const bio = use(fetchData(`/$/bio`));
  return (
    
      
    
  );
}
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
  if (!cache.has(url)) 
  return cache.get(url);
}

async function getData(url) {
  if (url === '/the-beatles/albums')  else if (url === '/the-beatles/bio')  else 
}

async function getBio() {
  // Add a fake delay to make waiting noticeable.
  await new Promise(resolve => );

  return `The Beatles were an English rock band,
    formed in Liverpool in 1960, that comprised
    John Lennon, Paul McCartney, George Harrison
    and Ringo Starr.`;
}

async function getAlbums() {
  // Add a fake delay to make waiting noticeable.
  await new Promise(resolve => );

  return [, , , , , , , , , , , , ];
}
```

```css
.bio 

.panel 
```

Components that load data don't have to be direct children of the Suspense boundary. For example, you can move `Biography` and `Albums` into a new `Details` component. This doesn't change the behavior. `Biography` and `Albums` share the same closest parent Suspense boundary, so their reveal is coordinated together.

```js 

function Details() 
```

---

### Revealing nested content as it loads 

When a component suspends, the closest parent Suspense component shows the fallback. This lets you nest multiple Suspense components to create a loading sequence. Each Suspense boundary's fallback will be filled in as the next level of content becomes available. For example, you can give the album list its own fallback:

```js 

  

```

With this change, displaying the `Biography` doesn't need to "wait" for the `Albums` to load.

The sequence will be:

1. If `Biography` hasn't loaded yet, `BigSpinner` is shown in place of the entire content area.
2. Once `Biography` finishes loading, `BigSpinner` is replaced by the content.
3. If `Albums` hasn't loaded yet, `AlbumsGlimmer` is shown in place of `Albums` and its parent `Panel`.
4. Finally, once `Albums` finishes loading, it replaces `AlbumsGlimmer`.

        
      
    </>
  );
}

function BigSpinner() 

function AlbumsGlimmer() 
```

```js src/Panel.js
export default function Panel() {
  return (
    
      
    
  );
}
```

```js src/Biography.js
import  from 'react';
import  from './data.js';

export default function Biography() {
  const bio = use(fetchData(`/$/bio`));
  return (
    
      
    
  );
}
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
  if (!cache.has(url)) 
  return cache.get(url);
}

async function getData(url) {
  if (url === '/the-beatles/albums')  else if (url === '/the-beatles/bio')  else 
}

async function getBio() {
  // Add a fake delay to make waiting noticeable.
  await new Promise(resolve => );

  return `The Beatles were an English rock band,
    formed in Liverpool in 1960, that comprised
    John Lennon, Paul McCartney, George Harrison
    and Ringo Starr.`;
}

async function getAlbums() {
  // Add a fake delay to make waiting noticeable.
  await new Promise(resolve => );

  return [, , , , , , , , , , , , ];
}
```

```css
.bio 

.panel 

.glimmer-panel 

.glimmer-line 
```

Suspense boundaries let you coordinate which parts of your UI should always "pop in" together at the same time, and which parts should progressively reveal more content in a sequence of loading states. You can add, move, or delete Suspense boundaries in any place in the tree without affecting the rest of your app's behavior.

Don't put a Suspense boundary around every component. Suspense boundaries should not be more granular than the loading sequence that you want the user to experience. If you work with a designer, ask them where the loading states should be placed--it's likely that they've already included them in their design wireframes.

---

### Showing stale content while fresh content is loading 

In this example, the `SearchResults` component suspends while fetching the search results. Type `"a"`, wait for the results, and then edit it to `"ab"`. The results for `"a"` will get replaced by the loading fallback.

    </>
  );
}
```

```js src/SearchResults.js
import  from 'react';
import  from './data.js';

export default function SearchResults() {
  if (query === '') 
  const albums = use(fetchData(`/search?q=$`));
  if (albums.length === 0) {
    return No matches for "";
  }
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
  if (url.startsWith('/search?q='))  else 
}

async function getSearchResults(query) {
  // Add a fake delay to make waiting noticeable.
  await new Promise(resolve => );

  const allAlbums = [, , , , , , , , , , , , ];

  const lowerQuery = query.trim().toLowerCase();
  return allAlbums.filter(album => );
}
```

```css
input 
```

A common alternative UI pattern is to *defer* updating the list and to keep showing the previous results until the new results are ready. The [`useDeferredValue`](/reference/react/useDeferredValue) Hook lets you pass a deferred version of the query down:

```js 
export default function App()  />
      
      
    </>
  );
}
```

The `query` will update immediately, so the input will display the new value. However, the `deferredQuery` will keep its previous value until the data has loaded, so `SearchResults` will show the stale results for a bit.

To make it more obvious to the user, you can add a visual indication when the stale result list is displayed:

```js 

  
    </>
  );
}
```

```js src/SearchResults.js hidden
import  from 'react';
import  from './data.js';

export default function SearchResults() {
  if (query === '') 
  const albums = use(fetchData(`/search?q=$`));
  if (albums.length === 0) {
    return No matches for "";
  }
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
  if (url.startsWith('/search?q='))  else 
}

async function getSearchResults(query) {
  // Add a fake delay to make waiting noticeable.
  await new Promise(resolve => );

  const allAlbums = [, , , , , , , , , , , , ];

  const lowerQuery = query.trim().toLowerCase();
  return allAlbums.filter(album => );
}
```

```css
input 
```

---

### Preventing already revealed content from hiding 

When a component suspends, the closest parent Suspense boundary switches to showing the fallback. This can lead to a jarring user experience if it was already displaying some content. Try pressing this button:

  );
}

function Router() {
  const [page, setPage] = useState('/');

  function navigate(url) 

  let content;
  if (page === '/') 

function BigSpinner() 
```

```js src/Layout.js
export default function Layout() {
  return (
    
      
        Music Browser
      
      
        
      
    
  );
}
```

```js src/IndexPage.js
export default function IndexPage() >
      Open The Beatles artist page
    
  );
}
```

```js src/ArtistPage.js
import  from 'react';
import Albums from './Albums.js';
import Biography from './Biography.js';
import Panel from './Panel.js';

export default function ArtistPage() {
  return (
    <>
      
      
      
    </>
  );
}

function AlbumsGlimmer() 
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

```js src/Biography.js
import  from 'react';
import  from './data.js';

export default function Biography() {
  const bio = use(fetchData(`/$/bio`));
  return (
    
      
    
  );
}
```

```js src/Panel.js
export default function Panel() {
  return (
    
      
    
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
  if (url === '/the-beatles/albums')  else if (url === '/the-beatles/bio')  else 
}

async function getBio() {
  // Add a fake delay to make waiting noticeable.
  await new Promise(resolve => );

  return `The Beatles were an English rock band,
    formed in Liverpool in 1960, that comprised
    John Lennon, Paul McCartney, George Harrison
    and Ringo Starr.`;
}

async function getAlbums() {
  // Add a fake delay to make waiting noticeable.
  await new Promise(resolve => );

  return [, , , , , , , , , , , , ];
}
```

```css
main 

.layout 

.header 

.bio 

.panel 

.glimmer-panel 

.glimmer-line 
```

When you pressed the button, the `Router` component rendered `ArtistPage` instead of `IndexPage`. A component inside `ArtistPage` suspended, so the closest Suspense boundary started showing the fallback. The closest Suspense boundary was near the root, so the whole site layout got replaced by `BigSpinner`.

To prevent this, you can mark the navigation state update as a *Transition* with [`startTransition`:](/reference/react/startTransition)

```js 
function Router() {
  const [page, setPage] = useState('/');

  function navigate(url) {
    startTransition(() => );
  }
  // ...
```

This tells React that the state transition is not urgent, and it's better to keep showing the previous page instead of hiding any already revealed content. Now clicking the button "waits" for the `Biography` to load:

  );
}

function Router() {
  const [page, setPage] = useState('/');

  function navigate(url) {
    startTransition(() => );
  }

  let content;
  if (page === '/') 

function BigSpinner() 
```

```js src/Layout.js
export default function Layout() {
  return (
    
      
        Music Browser
      
      
        
      
    
  );
}
```

```js src/IndexPage.js
export default function IndexPage() >
      Open The Beatles artist page
    
  );
}
```

```js src/ArtistPage.js
import  from 'react';
import Albums from './Albums.js';
import Biography from './Biography.js';
import Panel from './Panel.js';

export default function ArtistPage() {
  return (
    <>
      
      
      
    </>
  );
}

function AlbumsGlimmer() 
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

```js src/Biography.js
import  from 'react';
import  from './data.js';

export default function Biography() {
  const bio = use(fetchData(`/$/bio`));
  return (
    
      
    
  );
}
```

```js src/Panel.js
export default function Panel() {
  return (
    
      
    
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
  if (url === '/the-beatles/albums')  else if (url === '/the-beatles/bio')  else 
}

async function getBio() {
  // Add a fake delay to make waiting noticeable.
  await new Promise(resolve => );

  return `The Beatles were an English rock band,
    formed in Liverpool in 1960, that comprised
    John Lennon, Paul McCartney, George Harrison
    and Ringo Starr.`;
}

async function getAlbums() {
  // Add a fake delay to make waiting noticeable.
  await new Promise(resolve => );

  return [, , , , , , , , , , , , ];
}
```

```css
main 

.layout 

.header 

.bio 

.panel 

.glimmer-panel 

.glimmer-line 
```

A Transition doesn't wait for *all* content to load. It only waits long enough to avoid hiding already revealed content. For example, the website `Layout` was already revealed, so it would be bad to hide it behind a loading spinner. However, the nested `Suspense` boundary around `Albums` is new, so the Transition doesn't wait for it.

---

### Indicating that a Transition is happening 

In the above example, once you click the button, there is no visual indication that a navigation is in progress. To add an indicator, you can replace [`startTransition`](/reference/react/startTransition) with [`useTransition`](/reference/react/useTransition) which gives you a boolean `isPending` value. In the example below, it's used to change the website header styling while a Transition is happening:

  );
}

function Router() {
  const [page, setPage] = useState('/');
  const [isPending, startTransition] = useTransition();

  function navigate(url) {
    startTransition(() => );
  }

  let content;
  if (page === '/') 

function BigSpinner() 
```

```js src/Layout.js
export default function Layout() {
  return (
    
      
        Music Browser
      
      
        
      
    
  );
}
```

```js src/IndexPage.js
export default function IndexPage() >
      Open The Beatles artist page
    
  );
}
```

```js src/ArtistPage.js
import  from 'react';
import Albums from './Albums.js';
import Biography from './Biography.js';
import Panel from './Panel.js';

export default function ArtistPage() {
  return (
    <>
      
      
      
    </>
  );
}

function AlbumsGlimmer() 
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

```js src/Biography.js
import  from 'react';
import  from './data.js';

export default function Biography() {
  const bio = use(fetchData(`/$/bio`));
  return (
    
      
    
  );
}
```

```js src/Panel.js
export default function Panel() {
  return (
    
      
    
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
  if (url === '/the-beatles/albums')  else if (url === '/the-beatles/bio')  else 
}

async function getBio() {
  // Add a fake delay to make waiting noticeable.
  await new Promise(resolve => );

  return `The Beatles were an English rock band,
    formed in Liverpool in 1960, that comprised
    John Lennon, Paul McCartney, George Harrison
    and Ringo Starr.`;
}

async function getAlbums() {
  // Add a fake delay to make waiting noticeable.
  await new Promise(resolve => );

  return [, , , , , , , , , , , , ];
}
```

```css
main 

.layout 

.header 

.bio 

.panel 

.glimmer-panel 

.glimmer-line 
```

---

### Resetting Suspense boundaries on navigation 

During a Transition, React will avoid hiding already revealed content. However, if you navigate to a route with different parameters, you might want to tell React it is *different* content. You can express this with a `key`:

```js

function Chat() {
  if (typeof window === 'undefined') 
  // ...
}
```

The server HTML will include the loading indicator. It will be replaced by the `Chat` component on the client.

---

## Troubleshooting 

### How do I prevent the UI from being replaced by a fallback during an update? 

Replacing visible UI with a fallback creates a jarring user experience. This can happen when an update causes a component to suspend, and the nearest Suspense boundary is already showing content to the user.

To prevent this from happening, [mark the update as non-urgent using `startTransition`](#preventing-already-revealed-content-from-hiding). During a Transition, React will wait until enough data has loaded to prevent an unwanted fallback from appearing:

```js 
function handleNextPageClick() {
  // If this update suspends, don't hide the already displayed content
  startTransition(() => );
}
```

This will avoid hiding existing content. However, any newly rendered `Suspense` boundaries will still immediately display fallbacks to avoid blocking the UI and let the user see the content as it becomes available.

**React will only prevent unwanted fallbacks during non-urgent updates**. It will not delay a render if it's the result of an urgent update. You must opt in with an API like [`startTransition`](/reference/react/startTransition) or [`useDeferredValue`](/reference/react/useDeferredValue).

If your router is integrated with Suspense, it should wrap its updates into [`startTransition`](/reference/react/startTransition) automatically.
