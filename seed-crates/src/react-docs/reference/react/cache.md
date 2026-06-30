---
title: cache
---

#### Caveats 

- React will invalidate the cache for all memoized functions for each server request.
- Each call to `cache` creates a new function. This means that calling `cache` with the same function multiple times will return different memoized functions that do not share the same cache.
- `cachedFn` will also cache errors. If `fn` throws an error for certain arguments, it will be cached, and the same error is re-thrown when `cachedFn` is called with those same arguments.
- `cache` is for use in [Server Components](/reference/rsc/server-components) only.

---

## Usage 

### Cache an expensive computation 

Use `cache` to skip duplicate work.

```js [[1, 7, "getUserMetrics(user)"],[2, 13, "getUserMetrics(user)"]]
import  from 'react';
import calculateUserMetrics from 'lib/user';

const getUserMetrics = cache(calculateUserMetrics);

function Profile() 

function TeamReport() {
  for (let user in users) 
  // ...
}
```

If the same `user` object is rendered in both `Profile` and `TeamReport`, the two components can share work and only call `calculateUserMetrics` once for that `user`.

Assume `Profile` is rendered first. It will call , and check if there is a cached result. Since it is the first time `getUserMetrics` is called with that `user`, there will be a cache miss. `getUserMetrics` will then call `calculateUserMetrics` with that `user` and write the result to cache.

When `TeamReport` renders its list of `users` and reaches the same `user` object, it will call  and read the result from cache.

If `calculateUserMetrics` can be aborted by passing an [`AbortSignal`](https://developer.mozilla.org/en-US/docs/Web/API/AbortSignal), you can use [`cacheSignal()`](/reference/react/cacheSignal) to cancel the expensive computation if React has finished rendering. `calculateUserMetrics` may already handle cancellation internally by using `cacheSignal` directly.

 and  each call `cache` to create a new memoized function with their own cache look-up. If both components render for the same `cityData`, they will do duplicate work to call `calculateWeekReport`.

In addition, `Temperature` creates a  each time the component is rendered which doesn't allow for any cache sharing.

To maximize cache hits and reduce work, the two components should call the same memoized function to access the same cache. Instead, define the memoized function in a dedicated module that can be [`import`-ed](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/import) across components.

```js [[3, 5, "export default cache(calculateWeekReport)"]]
// getWeekReport.js
import  from 'react';
import  from './report';

export default cache(calculateWeekReport);
```

```js [[3, 2, "getWeekReport", 0], [3, 5, "getWeekReport"]]
// Temperature.js
import getWeekReport from './getWeekReport';

export default function Temperature() 
```

```js [[3, 2, "getWeekReport", 0], [3, 5, "getWeekReport"]]
// Precipitation.js
import getWeekReport from './getWeekReport';

export default function Precipitation() 
```
Here, both components call the  exported from `./getWeekReport.js` to read and write to the same cache.

### Share a snapshot of data 

To share a snapshot of data between components, call `cache` with a data-fetching function like `fetch`. When multiple components make the same data fetch, only one request is made and the data returned is cached and shared across components. All components refer to the same snapshot of data across the server render.

```js [[1, 4, "city"], [1, 5, "fetchTemperature(city)"], [2, 4, "getTemperature"], [2, 9, "getTemperature"], [1, 9, "city"], [2, 14, "getTemperature"], [1, 14, "city"]]
import  from 'react';
import  from './api.js';

const getTemperature = cache(async (city) => );

async function AnimatedWeatherCard() 

async function MinimalWeatherCard() 
```

If `AnimatedWeatherCard` and `MinimalWeatherCard` both render for the same , they will receive the same snapshot of data from the .

If `AnimatedWeatherCard` and `MinimalWeatherCard` supply different  arguments to , then `fetchTemperature` will be called twice and each call site will receive different data.

The  acts as a cache key.

 is only supported for Server Components.

```js [[3, 1, "async"], [3, 2, "await"]]
async function AnimatedWeatherCard() 
```

To render components that use asynchronous data in Client Components, see [`use()` documentation](/reference/react/use).

### Preload data 

By caching a long-running data fetch, you can kick off asynchronous work prior to rendering the component.

```jsx [[2, 6, "await getUser(id)"], [1, 17, "getUser(id)"]]
const getUser = cache(async (id) => );

async function Profile() {
  const user = await getUser(id);
  return (
    
      
      
    
  );
}

function Page() {
  // ✅ Good: start fetching the user data
  getUser(id);
  // ... some computational work
  return (
    <>
       but note that it doesn't use the returned data. This early  call kicks off the asynchronous database query that occurs while `Page` is doing other computational work and rendering children.

When rendering `Profile`, we call  again. If the initial  call has already returned and cached the user data, when `Profile` , it can simply read from the cache without requiring another remote procedure call. If the  hasn't been completed, preloading data in this pattern reduces delay in data-fetching.

 returns a promise that is awaiting the `fetch`.

```js [[1, 1, "fetchData()"], [2, 8, "getData()"], [3, 10, "getData()"]]
async function fetchData() 

const getData = cache(fetchData);

async function MyComponent() 
```

In calling  the first time, the promise returned from  is cached. Subsequent look-ups will then return the same promise.

Notice that the first  call does not `await` whereas the  does. [`await`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/await) is a JavaScript operator that will wait and return the settled result of the promise. The first  call simply initiates the `fetch` to cache the promise for the second  to look-up.

If by the  the promise is still _pending_, then `await` will pause for the result. The optimization is that while we wait on the `fetch`, React can continue with computational work, thus reducing the wait time for the .

If the promise is already settled, either to an error or the _fulfilled_ result, `await` will return that value immediately. In both outcomes, there is a performance benefit.

 outside of a component, it will still evaluate the function but not read or update the cache.

This is because cache access is provided through a [context](/learn/passing-data-deeply-with-context) which is only accessible from a component.

 will be able to skip duplicate work and read from the same cache as the . Another difference from the previous example is that `cache` is also recommended for , unlike `useMemo` which should only be used for computations.

At this time, `cache` should only be used in Server Components and the cache will be invalidated across server requests.

#### `memo` 

You should use [`memo`](reference/react/memo) to prevent a component re-rendering if its props are unchanged.

```js
'use client';

function WeatherReport() 

const MemoWeatherReport = memo(WeatherReport);

function App() {
  const record = getRecord();
  return (
    <>
      

---

## Troubleshooting 

### My memoized function still runs even though I've called it with the same arguments 

See prior mentioned pitfalls
* [Calling different memoized functions will read from different caches.](#pitfall-different-memoized-functions)
* [Calling a memoized function outside of a component will not use the cache.](#pitfall-memoized-call-outside-component)

If none of the above apply, it may be a problem with how React checks if something exists in cache.

If your arguments are not [primitives](https://developer.mozilla.org/en-US/docs/Glossary/Primitive) (ex. objects, functions, arrays), ensure you're passing the same object reference.

When calling a memoized function, React will look up the input arguments to see if a result is already cached. React will use shallow equality of the arguments to determine if there is a cache hit.

```js
import  from 'react';

const calculateNorm = cache((vector) => );

function MapMarker(props) 

function App() 
```

In this case the two `MapMarker`s look like they're doing the same work and calling `calculateNorm` with the same value of ``. Even though the objects contain the same values, they are not the same object reference as each component creates its own `props` object.

React will call [`Object.is`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/is) on the input to verify if there is a cache hit.

```js 
import  from 'react';

const calculateNorm = cache((x, y, z) => );

function MapMarker(props) 

function App() 
```

One way to address this could be to pass the vector dimensions to `calculateNorm`. This works because the dimensions themselves are primitives.

Another solution may be to pass the vector object itself as a prop to the component. We'll need to pass the same object to both component instances.

```js 
import  from 'react';

const calculateNorm = cache((vector) => );

function MapMarker(props) 

function App() 
```
