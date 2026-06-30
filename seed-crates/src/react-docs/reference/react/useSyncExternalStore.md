---
title: useSyncExternalStore
---

 of the data in the store. You need to pass two functions as arguments:

1. The  should subscribe to the store and return a function that unsubscribes.
2. The  should read a snapshot of the data from the store.

React will use these functions to keep your component subscribed to the store and re-render it on changes.

For example, in the sandbox below, `todosStore` is implemented as an external store that stores data outside of React. The `TodosApp` component connects to that external store with the `useSyncExternalStore` Hook.

---

### Subscribing to a browser API 

Another reason to add `useSyncExternalStore` is when you want to subscribe to some value exposed by the browser that changes over time. For example, suppose that you want your component to display whether the network connection is active. The browser exposes this information via a property called [`navigator.onLine`.](https://developer.mozilla.org/en-US/docs/Web/API/Navigator/onLine)

This value can change without React's knowledge, so you should read it with `useSyncExternalStore`.

```js
import  from 'react';

function ChatIndicator() 
```

To implement the `getSnapshot` function, read the current value from the browser API:

```js
function getSnapshot() 
```

Next, you need to implement the `subscribe` function. For example, when `navigator.onLine` changes, the browser fires the [`online`](https://developer.mozilla.org/en-US/docs/Web/API/Window/online_event) and [`offline`](https://developer.mozilla.org/en-US/docs/Web/API/Window/offline_event) events on the `window` object. You need to subscribe the `callback` argument to the corresponding events, and then return a function that cleans up the subscriptions:

```js
function subscribe(callback) {
  window.addEventListener('online', callback);
  window.addEventListener('offline', callback);
  return () => ;
}
```

Now React knows how to read the value from the external `navigator.onLine` API and how to subscribe to its changes. Disconnect your device from the network and notice that the component re-renders in response:

---

### Extracting the logic to a custom Hook 

Usually you won't write `useSyncExternalStore` directly in your components. Instead, you'll typically call it from your own custom Hook. This lets you use the same external store from different components.

For example, this custom `useOnlineStatus` Hook tracks whether the network is online:

```js 
import  from 'react';

export function useOnlineStatus() 

function getSnapshot() 

function subscribe(callback) 
```

Now different components can call `useOnlineStatus` without repeating the underlying implementation:

---

### Adding support for server rendering 

If your React app uses [server rendering,](/reference/react-dom/server) your React components will also run outside the browser environment to generate the initial HTML. This creates a few challenges when connecting to an external store:

- If you're connecting to a browser-only API, it won't work because it does not exist on the server.
- If you're connecting to a third-party data store, you'll need its data to match between the server and client.

To solve these issues, pass a `getServerSnapshot` function as the third argument to `useSyncExternalStore`:

```js 
import  from 'react';

export function useOnlineStatus() 

function getSnapshot() 

function getServerSnapshot() 

function subscribe(callback) 
```

The `getServerSnapshot` function is similar to `getSnapshot`, but it runs only in two situations:

- It runs on the server when generating the HTML.
- It runs on the client during [hydration](/reference/react-dom/client/hydrateRoot), i.e. when React takes the server HTML and makes it interactive.

This lets you provide the initial snapshot value which will be used before the app becomes interactive. If there is no meaningful initial value for the server rendering, omit this argument to [force rendering on the client.](/reference/react/Suspense#providing-a-fallback-for-server-errors-and-client-only-content)

---

## Troubleshooting 

### I'm getting an error: "The result of `getSnapshot` should be cached" 

This error means your `getSnapshot` function returns a new object every time it's called, for example:

```js 
function getSnapshot() {
  // 🔴 Do not return always different objects from getSnapshot
  return ;
}
```

React will re-render the component if `getSnapshot` return value is different from the last time. This is why, if you always return a different value, you will enter an infinite loop and get this error.

Your `getSnapshot` object should only return a different object if something has actually changed. If your store contains immutable data, you can return that data directly:

```js 
function getSnapshot() 
```

If your store data is mutable, your `getSnapshot` function should return an immutable snapshot of it. This means it *does* need to create new objects, but it shouldn't do this for every single call. Instead, it should store the last calculated snapshot, and return the same snapshot as the last time if the data in the store has not changed. How you determine whether mutable data has changed depends on your mutable store.

---

### My `subscribe` function gets called after every re-render 

This `subscribe` function is defined *inside* a component so it is different on every re-render:

```js 
function ChatIndicator() {
  // 🚩 Always a different function, so React will resubscribe on every re-render
  function subscribe() 

  const isOnline = useSyncExternalStore(subscribe, getSnapshot);

  // ...
}
```

React will resubscribe to your store if you pass a different `subscribe` function between re-renders. If this causes performance issues and you'd like to avoid resubscribing, move the `subscribe` function outside:

```js 
// ✅ Always the same function, so React won't need to resubscribe
function subscribe() 

function ChatIndicator() 
```

Alternatively, wrap `subscribe` into [`useCallback`](/reference/react/useCallback) to only resubscribe when some argument changes:

```js 
function ChatIndicator() {
  // ✅ Same function as long as userId doesn't change
  const subscribe = useCallback(() => , [userId]);

  const isOnline = useSyncExternalStore(subscribe, getSnapshot);

  // ...
}
```
