---
title: hydrateRoot
---

 with the  for your app. Usually, you will do it once at startup. If you use a framework, it might do this behind the scenes for you.

To hydrate your app, React will "attach" your components' logic to the initial generated HTML from the server. Hydration turns the initial HTML snapshot from the server into a fully interactive app that runs in the browser.

You shouldn't need to call `hydrateRoot` again or to call it in more places. From this point on, React will be managing the DOM of your application. To update the UI, your components will [use state](/reference/react/useState) instead.

---

### Hydrating an entire document 

Apps fully built with React can render the entire document as JSX, including the [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/html) tag:

```js 
function App() {
  return (
    
      
        
        
        
        My app
      
      
        

This only works one level deep, and is intended to be an escape hatch. Don’t overuse it. React will **not** attempt to patch mismatched text content.

---

### Handling different client and server content 

If you intentionally need to render something different on the server and the client, you can do a two-pass rendering. Components that render something different on the client can read a [state variable](/reference/react/useState) like `isClient`, which you can set to `true` in an [Effect](/reference/react/useEffect):

This way the initial render pass will render the same content as the server, avoiding mismatches, but an additional pass will happen synchronously right after hydration.

---

### Updating a hydrated root component 

After the root has finished hydrating, you can call [`root.render`](#root-render) to update the root React component. **Unlike with [`createRoot`](/reference/react-dom/client/createRoot), you don't usually need to do this because the initial content was already rendered as HTML.**

If you call `root.render` at some point after hydration, and the component tree structure matches up with what was previously rendered, React will [preserve the state.](/learn/preserving-and-resetting-state) Notice how you can type in the input, which means that the updates from repeated `render` calls every second in this example are not destructive:

It is uncommon to call [`root.render`](#root-render) on a hydrated root. Usually, you'll [update state](/reference/react/useState) inside one of the components instead.

### Error logging in production 

By default, React will log all errors to the console. To implement your own error reporting, you can provide the optional error handler root options `onUncaughtError`, `onCaughtError` and `onRecoverableError`:

```js [[1, 7, "onCaughtError"], [2, 7, "error", 1], [3, 7, "errorInfo"], [4, 11, "componentStack", 15]]
import  from "react-dom/client";
import App from "./App.js";
import  from "./reportError";

const container = document.getElementById("root");
const root = hydrateRoot(container,  option is a function called with two arguments:

1. The  that was thrown.
2. An  object that contains the  of the error.

Together with `onUncaughtError` and `onRecoverableError`, you can implement your own error reporting system:

      )}
    </>
  );
}
```

```html public/index.html hidden
<!DOCTYPE html>

  My app

Server content before hydration.

```

## Troubleshooting 

### I'm getting an error: "You passed a second argument to root.render" 

A common mistake is to pass the options for `hydrateRoot` to `root.render(...)`:

To fix, pass the root options to `hydrateRoot(...)`, not `root.render(...)`:
```js 
// 🚩 Wrong: root.render only takes one argument.
root.render(App, );

// ✅ Correct: pass options to createRoot.
const root = hydrateRoot(container, , );
```
