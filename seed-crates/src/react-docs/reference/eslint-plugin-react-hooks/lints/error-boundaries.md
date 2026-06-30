---
title: error-boundaries
---

## Rule Details 

Try/catch blocks can't catch errors that happen during React's rendering process. Errors thrown in rendering methods or hooks bubble up through the component tree. Only [Error Boundaries](/reference/react/Component#catching-rendering-errors-with-an-error-boundary) can catch these errors.

### Invalid 

Examples of incorrect code for this rule:

```js {expectedErrors: }
// ❌ Try/catch won't catch render errors
function Parent() {
  try 
```

## Troubleshooting 

### Why is the linter telling me not to wrap `use` in `try`/`catch`? 

The `use` hook doesn't throw errors in the traditional sense, it suspends component execution. When `use` encounters a pending promise, it suspends the component and lets React show a fallback. Only Suspense and Error Boundaries can handle these cases. The linter warns against `try`/`catch` around `use` to prevent confusion as the `catch` block would never run.

```js {expectedErrors: }
// ❌ Try/catch around `use` hook
function Component() {
  try {
    const data = use(promise); // Won't catch - `use` suspends, not throws
    return ;
  } catch (error) 
}

// ✅ Error boundary catches `use` errors
function App() 
```