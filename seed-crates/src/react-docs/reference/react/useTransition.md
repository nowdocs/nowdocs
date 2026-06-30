---
title: useTransition
---

#### Parameters 

* `action`: A function that updates some state by calling one or more [`set` functions](/reference/react/useState#setstate). React calls `action` immediately with no parameters and marks all state updates scheduled synchronously during the `action` function call as Transitions. Any async calls that are awaited in the `action` will be included in the Transition, but currently require wrapping any `set` functions after the `await` in an additional `startTransition` (see [Troubleshooting](#react-doesnt-treat-my-state-update-after-await-as-a-transition)). State updates marked as Transitions will be [non-blocking](#perform-non-blocking-updates-with-actions) and [will not display unwanted loading indicators](#preventing-unwanted-loading-indicators).

#### Returns 

`startTransition` does not return anything.

#### Caveats 

* `useTransition` is a Hook, so it can only be called inside components or custom Hooks. If you need to start a Transition somewhere else (for example, from a data library), call the standalone [`startTransition`](/reference/react/startTransition) instead.

* You can wrap an update into a Transition only if you have access to the `set` function of that state. If you want to start a Transition in response to some prop or a custom Hook value, try [`useDeferredValue`](/reference/react/useDeferredValue) instead.

* The function you pass to `startTransition` is called immediately, marking all state updates that happen while it executes as Transitions. If you try to perform state updates in a `setTimeout`, for example, they won't be marked as Transitions.

* You must wrap any state updates after any async requests in another `startTransition` to mark them as Transitions. This is a known limitation that we will fix in the future (see [Troubleshooting](#react-doesnt-treat-my-state-update-after-await-as-a-transition)).

* The `startTransition` function has a stable identity, so you will often see it omitted from Effect dependencies, but including it will not cause the Effect to fire. If the linter lets you omit a dependency without errors, it is safe to do. [Learn more about removing Effect dependencies.](/learn/removing-effect-dependencies#move-dynamic-objects-and-functions-inside-your-effect)

* A state update marked as a Transition will be interrupted by other state updates. For example, if you update a chart component inside a Transition, but then start typing into an input while the chart is in the middle of a re-render, React will restart the rendering work on the chart component after handling the input update.

* Transition updates can't be used to control text inputs.

* If there are multiple ongoing Transitions, React currently batches them together. This is a limitation that may be removed in a future release.

## Usage 

### Perform non-blocking updates with Actions 

Call `useTransition` at the top of your component to create Actions, and access the pending state:

```js [[1, 4, "isPending"], [2, 4, "startTransition"]]
import  from 'react';

function CheckoutForm() 
```

`useTransition` returns an array with exactly two items:

1. The  that tells you whether there is a pending Transition.
2. The  that lets you create an Action.

To start a Transition, pass a function to `startTransition` like this:

```js
import  from 'react';
import  from './api';

function CheckoutForm() {
  const [isPending, startTransition] = useTransition();
  const [quantity, setQuantity] = useState(1);

  function onSubmit(newQuantity) {
    startTransition(async function () {
      const savedQuantity = await updateQuantity(newQuantity);
      startTransition(() => );
    });
  }
  // ...
}
```

The function passed to `startTransition` is called the "Action". You can update state and (optionally) perform side effects within an Action, and the work will be done in the background without blocking user interactions on the page. A Transition can include multiple Actions, and while a Transition is in progress, your UI stays responsive. For example, if the user clicks a tab but then changes their mind and clicks another tab, the second click will be immediately handled without waiting for the first update to finish.

To give the user feedback about in-progress Transitions, the `isPending` state switches to `true` at the first call to `startTransition`, and stays `true` until all Actions complete and the final state is shown to the user. Transitions ensure side effects in Actions to complete in order to [prevent unwanted loading indicators](#preventing-unwanted-loading-indicators), and you can provide immediate feedback while the Transition is in progress with `useOptimistic`.

This is a basic example to demonstrate how Actions work, but this example does not handle requests completing out of order. When updating the quantity multiple times, it's possible for the previous requests to finish after later requests causing the quantity to update out of order. This is a known limitation that we will fix in the future (see [Troubleshooting](#my-state-updates-in-transitions-are-out-of-order) below).

For common use cases, React provides built-in abstractions such as:
- [`useActionState`](/reference/react/useActionState)
- [`` actions](/reference/react-dom/components/form)
- [Server Functions](/reference/rsc/server-functions)

These solutions handle request ordering for you. When using Transitions to build your own custom hooks or libraries that manage async state transitions, you have greater control over the request ordering, but you must handle it yourself.

A common solution to this problem is to prevent the user from making changes while the quantity is updating:

This solution makes the app feel slow, because the user must wait each time they update the quantity. It's possible to add more complex handling manually to allow the user to interact with the UI while the quantity is updating, but Actions handle this case with a straight-forward built-in API.

---

### Exposing `action` prop from components 

You can expose an `action` prop from a component to allow a parent to call an Action.

For example, this `TabButton` component wraps its `onClick` logic in an `action` prop:

```js 
export default function TabButton() {
  const [isPending, startTransition] = useTransition();
  if (isActive) {
    return 
  }
  return (
     {
      startTransition(async () => );
    }}>
      
    
  );
}
```

Because the parent component updates its state inside the `action`, that state update gets marked as a Transition. This means you can click on "Posts" and then immediately click "Contact" and it does not block user interactions:

      
      
      
      {tab === 'about' && 

---

### Displaying a pending visual state 

You can use the `isPending` boolean value returned by `useTransition` to indicate to the user that a Transition is in progress. For example, the tab button can have a special "pending" visual state:

```js 
function TabButton() {
  const [isPending, startTransition] = useTransition();
  // ...
  if (isPending) {
    return ;
  }
  // ...
```

Notice how clicking "Posts" now feels more responsive because the tab button itself updates right away:

      
      
      
      {tab === 'about' && 

---

### Preventing unwanted loading indicators 

In this example, the `PostsTab` component fetches some data using [use](/reference/react/use). When you click the "Posts" tab, the `PostsTab` component *suspends*, causing the closest loading fallback to appear:

      
      
      
      
```

```js src/TabButton.js
export default function TabButton() {
  if (isActive) {
    return 
  }
  return (
     }>
      
    
  );
}
```

```js src/AboutTab.js hidden
export default function AboutTab() 
```

```js src/PostsTab.js hidden
import  from 'react';
import  from './data.js';

function PostsTab() {
  const posts = use(fetchData('/posts'));
  return (
    
      {posts.map(post =>
        

Hiding the entire tab container to show a loading indicator leads to a jarring user experience. If you add `useTransition` to `TabButton`, you can instead display the pending state in the tab button instead.

Notice that clicking "Posts" no longer replaces the entire tab container with a spinner:

      
      
      
      
```

```js src/TabButton.js active
import  from 'react';

export default function TabButton() {
  const [isPending, startTransition] = useTransition();
  if (isActive) {
    return 
  }
  if (isPending) {
    return ;
  }
  return (
     {
      startTransition(async () => );
    }}>
      
    
  );
}
```

```js src/AboutTab.js hidden
export default function AboutTab() 
```

```js src/PostsTab.js hidden
import  from 'react';
import  from './data.js';

function PostsTab() {
  const posts = use(fetchData('/posts'));
  return (
    
      {posts.map(post =>
        

[Read more about using Transitions with Suspense.](/reference/react/Suspense#preventing-already-revealed-content-from-hiding)

---

### Building a Suspense-enabled router 

If you're building a React framework or a router, we recommend marking page navigations as Transitions.

```js 
function Router() {
  const [page, setPage] = useState('/');
  const [isPending, startTransition] = useTransition();

  function navigate(url) {
    startTransition(() => );
  }
  // ...
```

This is recommended for three reasons:

- [Transitions are interruptible,](#perform-non-blocking-updates-with-actions) which lets the user click away without waiting for the re-render to complete.
- [Transitions prevent unwanted loading indicators,](#preventing-unwanted-loading-indicators) which lets the user avoid jarring jumps on navigation.
- [Transitions wait for all pending actions](#perform-non-blocking-updates-with-actions) which lets the user wait for side effects to complete before the new page is shown.

Here is a simplified router example using Transitions for navigations.

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

### Displaying an error to users with an error boundary 

If a function passed to `startTransition` throws an error, you can display an error to your user with an [error boundary](/reference/react/Component#catching-rendering-errors-with-an-error-boundary). To use an error boundary, wrap the component where you are calling the `useTransition` in an error boundary. Once the function passed to `startTransition` errors, the fallback for the error boundary will be displayed.

  );
}

function addComment(comment) {
  // For demonstration purposes to show Error Boundary
  if (comment == null) 
}

function AddCommentButton() {
  const [pending, startTransition] = useTransition();

  return (
     {
        startTransition(() => );
      }}
    >
      Add comment
    
  );
}
```

```js src/App.js hidden
import  from "./AddCommentContainer.js";

export default function App() {
  return 
);
```

```json package.json hidden
{
  "dependencies": ,
  "main": "/index.js"
}
```

---

## Troubleshooting 

### Updating an input in a Transition doesn't work 

You can't use a Transition for a state variable that controls an input:

```js 
const [text, setText] = useState('');
// ...
function handleChange(e) {
  // ❌ Can't use Transitions for controlled input state
  startTransition(() => );
}
// ...
return ;
```

This is because Transitions are non-blocking, but updating an input in response to the change event should happen synchronously. If you want to run a Transition in response to typing, you have two options:

1. You can declare two separate state variables: one for the input state (which always updates synchronously), and one that you will update in a Transition. This lets you control the input using the synchronous state, and pass the Transition state variable (which will "lag behind" the input) to the rest of your rendering logic.
2. Alternatively, you can have one state variable, and add [`useDeferredValue`](/reference/react/useDeferredValue) which will "lag behind" the real value. It will trigger non-blocking re-renders to "catch up" with the new value automatically.

---

### React doesn't treat my state update as a Transition 

When you wrap a state update in a Transition, make sure that it happens *during* the `startTransition` call:

```js
startTransition(() => );
```

The function you pass to `startTransition` must be synchronous. You can't mark an update as a Transition like this:

```js
startTransition(() => {
  // ❌ Setting state *after* startTransition call
  setTimeout(() => , 1000);
});
```

Instead, you could do this:

```js
setTimeout(() => {
  startTransition(() => );
}, 1000);
```

---

### React doesn't treat my state update after `await` as a Transition 

When you use `await` inside a `startTransition` function, the state updates that happen after the `await` are not marked as Transitions. You must wrap state updates after each `await` in a `startTransition` call:

```js
startTransition(async () => );
```

However, this works instead:

```js
startTransition(async () => {
  await someAsyncFunction();
  // ✅ Using startTransition *after* await
  startTransition(() => );
});
```

This is a JavaScript limitation due to React losing the scope of the async context. In the future, when [AsyncContext](https://github.com/tc39/proposal-async-context) is available, this limitation will be removed.

---

### I want to call `useTransition` from outside a component 

You can't call `useTransition` outside a component because it's a Hook. In this case, use the standalone [`startTransition`](/reference/react/startTransition) method instead. It works the same way, but it doesn't provide the `isPending` indicator.

---

### The function I pass to `startTransition` executes immediately 

If you run this code, it will print 1, 2, 3:

```js 
console.log(1);
startTransition(() => );
console.log(3);
```

**It is expected to print 1, 2, 3.** The function you pass to `startTransition` does not get delayed. Unlike with the browser `setTimeout`, it does not run the callback later. React executes your function immediately, but any state updates scheduled *while it is running* are marked as Transitions. You can imagine that it works like this:

```js
// A simplified version of how React works

let isInsideTransition = false;

function startTransition(scope) 

function setState() {
  if (isInsideTransition)  else 
}
```

### My state updates in Transitions are out of order 

If you `await` inside `startTransition`, you might see the updates happen out of order.

In this example, the `updateQuantity` function simulates a request to the server to update the item's quantity in the cart. This function *artificially returns every other request after the previous* to simulate race conditions for network requests.

Try updating the quantity once, then update it quickly multiple times. You might see the incorrect total:

When clicking multiple times, it's possible for previous requests to finish after later requests. When this happens, React currently has no way to know the intended order. This is because the updates are scheduled asynchronously, and React loses context of the order across the async boundary.

This is expected, because Actions within a Transition do not guarantee execution order. For common use cases, React provides higher-level abstractions like [`useActionState`](/reference/react/useActionState) and [`` actions](/reference/react-dom/components/form) that handle ordering for you. For advanced use cases, you'll need to implement your own queuing and abort logic to handle this.

Example of `useActionState` handling execution order:

