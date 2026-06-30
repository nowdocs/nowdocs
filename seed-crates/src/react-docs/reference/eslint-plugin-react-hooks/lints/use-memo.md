---
title: use-memo
---

## Rule Details 

`useMemo` is for computing and caching expensive values, not for side effects. Without a return value, `useMemo` returns `undefined`, which defeats its purpose and likely indicates you're using the wrong hook.

### Invalid 

Examples of incorrect code for this rule:

```js {expectedErrors: }
// ❌ No return value
function Component() {
  const processed = useMemo(() => , [data]);

  return ; // Always undefined
}
```

### Valid 

Examples of correct code for this rule:

```js
// ✅ Returns computed value
function Component() {
  const processed = useMemo(() => , [data]);

  return ;
}
```

## Troubleshooting 

### I need to run side effects when dependencies change 

You might try to use `useMemo` for side effects:

```js {expectedErrors: }
// ❌ Wrong: Side effects in useMemo
function Component() {
  // No return value, just side effect
  useMemo(() => {
    analytics.track('UserViewed', );
  }, [user.id]);

  // Not assigned to a variable
  useMemo(() => {
    return analytics.track('UserViewed', );
  }, [user.id]);
}
```

If the side effect needs to happen in response to user interaction, it's best to colocate the side effect with the event:

```js
// ✅ Good: Side effects in event handlers
function Component() {
  const handleClick = () => {
    analytics.track('ButtonClicked', );
    // Other click logic...
  };

  return Click me;
}
```

If the side effect sychronizes React state with some external state (or vice versa), use `useEffect`:

```js
// ✅ Good: Synchronization in useEffect
function Component() {
  useEffect(() => , [theme]);

  return Current theme: ;
}
```
