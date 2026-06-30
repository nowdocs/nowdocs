---
title: useDeferredValue
---

 will be the same as the  you provided.

During updates, the  will "lag behind" the latest . In particular, React will first re-render *without* updating the deferred value, and then try to re-render with the newly received value in the background.

**Let's walk through an example to see when this is useful.**

In this example, the `SearchResults` component [suspends](/reference/react/Suspense#displaying-a-fallback-while-content-is-loading) while fetching the search results. Try typing `"a"`, waiting for the results, and then editing it to `"ab"`. The results for `"a"` get replaced by the loading fallback.

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

A common alternative UI pattern is to *defer* updating the list of results and to keep showing the previous results until the new results are ready. Call `useDeferredValue` to pass a deferred version of the query down:

```js 
export default function App()  />
      
      
    </>
  );
}
```

The `query` will update immediately, so the input will display the new value. However, the `deferredQuery` will keep its previous value until the data has loaded, so `SearchResults` will show the stale results for a bit.

Enter `"a"` in the example below, wait for the results to load, and then edit the input to `"ab"`. Notice how instead of the Suspense fallback, you now see the stale result list until the new results have loaded:

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

---

### Indicating that the content is stale 

In the example above, there is no indication that the result list for the latest query is still loading. This can be confusing to the user if the new results take a while to load. To make it more obvious to the user that the result list does not match the latest query, you can add a visual indication when the stale result list is displayed:

```js 

  
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

---

### Deferring re-rendering for a part of the UI 

You can also apply `useDeferredValue` as a performance optimization. It is useful when a part of your UI is slow to re-render, there's no easy way to optimize it, and you want to prevent it from blocking the rest of the UI.

Imagine you have a text field and a component (like a chart or a long list) that re-renders on every keystroke:

```js
function App()  />
      

