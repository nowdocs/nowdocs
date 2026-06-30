---
title: captureOwnerStack
---

  );
}
```

```js src/index.js
import  from 'react';
import  from 'react-dom/client';
import App,  from './App.js';
import './styles.css';

createRoot(document.createElement('div'), {
  onUncaughtError: (error, errorInfo) => ,
}).render(
  
);
```

```html public/index.html hidden
<!DOCTYPE html>

  
    
    
    Document
  
  
    Check the console output.
  

```

`SubComponent` would throw an error.
The Component Stack of that error would be

```
at SubComponent
at fieldset
at Component
at main
at React.Suspense
at App
```

However, the Owner Stack would only read

```
at Component
```

Neither `App` nor the DOM components (e.g. `fieldset`) are considered Owners in this Stack since they didn't contribute to "creating" the node containing `SubComponent`. `App` and DOM components only forwarded the node. `App` just rendered the `children` node as opposed to `Component` which created a node containing `SubComponent` via `

## Usage 

### Enhance a custom error overlay 

```js [[1, 5, "console.error"], [4, 7, "captureOwnerStack"]]
import  from "react";
import  from "./errorOverlay";

const originalConsoleError = console.error;
console.error = function patchedConsoleError(...args) {
  originalConsoleError.apply(console, args);
  const ownerStack = captureOwnerStack();
  onConsoleError();
};
```

If you intercept  calls to highlight them in an error overlay, you can call  to include the Owner Stack.

## Troubleshooting 

### The Owner Stack is `null` 

The call of `captureOwnerStack` happened outside of a React controlled function e.g. in a `setTimeout` callback, after a `fetch` call or in a custom DOM event handler. During render, Effects, React event handlers, and React error handlers (e.g. `hydrateRoot#options.onCaughtError`) Owner Stacks should be available.

In the example below, clicking the button will log an empty Owner Stack because `captureOwnerStack` was called during a custom DOM event handler. The Owner Stack must be captured earlier e.g. by moving the call of `captureOwnerStack` into the Effect body.

### `captureOwnerStack` is not available 

`captureOwnerStack` is only exported in development builds. It will be `undefined` in production builds. If `captureOwnerStack` is used in files that are bundled for production and development, you should conditionally access it from a namespace import.

```js
// Don't use named imports of `captureOwnerStack` in files that are bundled for development and production.
import  from 'react';
// Use a namespace import instead and access `captureOwnerStack` conditionally.
import * as React from 'react';

if (process.env.NODE_ENV !== 'production') 
```
