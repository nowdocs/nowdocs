---
title: flushSync
---

 inside the callback synchronously:

```js [[1, 2, "setSomething(123)"]]
flushSync(() => );
// By this line, the DOM is updated.
```

This ensures that, by the time the next line of code runs, React has already updated the DOM.

**Using `flushSync` is uncommon, and using it often can significantly hurt the performance of your app.** If your app only uses React APIs, and does not integrate with third-party libraries, `flushSync` should be unnecessary.

However, it can be helpful for integrating with third-party code like browser APIs.

Some browser APIs expect results inside of callbacks to be written to the DOM synchronously, by the end of the callback, so the browser can do something with the rendered DOM. In most cases, React handles this for you automatically. But in some cases it may be necessary to force a synchronous update.

For example, the browser `onbeforeprint` API allows you to change the page immediately before the print dialog opens. This is useful for applying custom print styles that allow the document to display better for printing. In the example below, you use `flushSync` inside of the `onbeforeprint` callback to immediately "flush" the React state to the DOM. Then, by the time the print dialog opens, `isPrinting` displays "yes":

Without `flushSync`, the print dialog will display `isPrinting` as "no". This is because React batches the updates asynchronously and the print dialog is displayed before the state is updated.

---

## Troubleshooting 

### I'm getting an error: "flushSync was called from inside a lifecycle method" 

React cannot `flushSync` in the middle of a render. If you do, it will noop and warn:

This includes calling `flushSync` inside:

- rendering a component.
- `useLayoutEffect` or `useEffect` hooks.
- Class component lifecycle methods.

For example, calling `flushSync` in an Effect will noop and warn:

```js
import  from 'react';
import  from 'react-dom';

function MyComponent() {
  useEffect(() => {
    // 🚩 Wrong: calling flushSync inside an effect
    flushSync(() => );
  }, []);

  return ;
}
```

To fix this, you usually want to move the `flushSync` call to an event:

```js
function handleClick() {
  // ✅ Correct: flushSync in event handlers is safe
  flushSync(() => );
}
```

If it's difficult to move to an event, you can defer `flushSync` in a microtask:

```js 
useEffect(() => {
  // ✅ Correct: defer flushSync to a microtask
  queueMicrotask(() => {
    flushSync(() => );
  });
}, []);
```

This will allow the current render to finish and schedule another syncronous render to flush the updates.

