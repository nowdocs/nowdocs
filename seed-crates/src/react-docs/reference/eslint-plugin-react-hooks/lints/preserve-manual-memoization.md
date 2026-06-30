---
title: preserve-manual-memoization
---

## Rule Details 

React Compiler preserves your existing `useMemo`, `useCallback`, and `React.memo` calls. If you've manually memoized something, the compiler assumes you had a good reason and won't remove it. However, incomplete dependencies prevent the compiler from understanding your code's data flow and applying further optimizations.

### Invalid 

Examples of incorrect code for this rule:

```js
// ❌ Missing dependencies in useMemo
function Component() 

// ❌ Missing dependencies in useCallback
function Component() {
  const handleClick = useCallback(() => , [onUpdate]); // Missing 'value'

  return Update;
}
```

### Valid 

Examples of correct code for this rule:

```js
// ✅ Complete dependencies
function Component() 

// ✅ Or let the compiler handle it
function Component() 
```

## Troubleshooting 

### Should I remove my manual memoization? 

You might wonder if React Compiler makes manual memoization unnecessary:

```js
// Do I still need this?
function Component() {
  const sorted = useMemo(() => {
    return [...items].sort((a, b) => );
  }, [items, sortBy]);

  return ;
}
```

You can safely remove it if using React Compiler:

```js
// ✅ Better: Let the compiler optimize
function Component() {
  const sorted = [...items].sort((a, b) => );

  return ;
}
```