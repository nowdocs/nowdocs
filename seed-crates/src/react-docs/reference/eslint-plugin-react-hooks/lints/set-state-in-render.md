---
title: set-state-in-render
---

## Rule Details 

Calling `setState` during render unconditionally triggers another render before the current one finishes. This creates an infinite loop that crashes your app.

## Common Violations 

### Invalid 

```js {expectedErrors: }
// ❌ Unconditional setState directly in render
function Component() {
  const [count, setCount] = useState(0);
  setCount(value); // Infinite loop!
  return ;
}
```

### Valid 

```js
// ✅ Derive during render
function Component() {
  const sorted = [...items].sort(); // Just calculate it in render
  return ;
}

// ✅ Set state in event handler
function Component() >
      
    
  );
}

// ✅ Derive from props instead of setting state
function Component() {
  const name = user?.name || '';
  const email = user?.email || '';
  return ;
}

// ✅ Conditionally derive state from props and state from previous renders
function Component() {
  const [isReverse, setIsReverse] = useState(false);
  const [selection, setSelection] = useState(null);

  const [prevItems, setPrevItems] = useState(items);
  if (items !== prevItems) 
  // ...
}
```

## Troubleshooting 

### I want to sync state to a prop 

A common problem is trying to "fix" state after it renders. Suppose you want to keep a counter from exceeding a `max` prop:

```js
// ❌ Wrong: clamps during render
function Counter() {
  const [count, setCount] = useState(0);

  if (count > max) 

  return (
     setCount(count + 1)}>
      
    
  );
}
```

As soon as `count` exceeds `max`, an infinite loop is triggered.

Instead, it's often better to move this logic to the event (the place where the state is first set). For example, you can enforce the maximum at the moment you update state:

```js
// ✅ Clamp when updating
function Counter() {
  const [count, setCount] = useState(0);

  const increment = () => ;

  return ;
}
```

Now the setter only runs in response to the click, React finishes the render normally, and `count` never crosses `max`.

In rare cases, you may need to adjust state based on information from previous renders. For those, follow [this pattern](https://react.dev/reference/react/useState#storing-information-from-previous-renders) of setting state conditionally.
