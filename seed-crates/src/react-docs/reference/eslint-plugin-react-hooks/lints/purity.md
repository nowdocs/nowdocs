---
title: purity
---

## Rule Details 

React components must be pure functions - given the same props, they should always return the same JSX. When components use functions like `Math.random()` or `Date.now()` during render, they produce different output each time, breaking React's assumptions and causing bugs like hydration mismatches, incorrect memoization, and unpredictable behavior.

## Common Violations 

In general, any API that returns a different value for the same inputs violates this rule. Usual examples include:

- `Math.random()`
- `Date.now()` / `new Date()`
- `crypto.randomUUID()`
- `performance.now()`

### Invalid 

Examples of incorrect code for this rule:

```js
// ❌ Math.random() in render
function Component() 

// ❌ Date.now() for values
function Component() {
  const timestamp = Date.now(); // Changes every render
  return Created at: ;
}
```

### Valid 

Examples of correct code for this rule:

```js
// ✅ Stable IDs from initial state
function Component() 
```

## Troubleshooting 

### I need to show the current time 

Calling `Date.now()` during render makes your component impure:

```js {expectedErrors: }
// ❌ Wrong: Time changes every render
function Clock() {
  return Current time: ;
}
```

Instead, [move the impure function outside of render](/reference/rules/components-and-hooks-must-be-pure#components-and-hooks-must-be-idempotent):

```js
function Clock() {
  const [time, setTime] = useState(() => Date.now());

  useEffect(() => {
    const interval = setInterval(() => , 1000);

    return () => clearInterval(interval);
  }, []);

  return Current time: ;
}
```