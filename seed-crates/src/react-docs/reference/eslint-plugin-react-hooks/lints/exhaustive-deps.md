---
title: exhaustive-deps
---

## Rule Details 

React hooks like `useEffect`, `useMemo`, and `useCallback` accept dependency arrays. When a value referenced inside these hooks isn't included in the dependency array, React won't re-run the effect or recalculate the value when that dependency changes. This causes stale closures where the hook uses outdated values.

## Common Violations 

This error often happens when you try to "trick" React about dependencies to control when an effect runs. Effects should synchronize your component with external systems. The dependency array tells React which values the effect uses, so React knows when to re-synchronize.

If you find yourself fighting with the linter, you likely need to restructure your code. See [Removing Effect Dependencies](/learn/removing-effect-dependencies) to learn how.

### Invalid 

Examples of incorrect code for this rule:

```js
// ❌ Missing dependency
useEffect(() => , []); // Missing 'count'

// ❌ Missing prop
useEffect(() => , []); // Missing 'userId'

// ❌ Incomplete dependencies
useMemo(() => , [items]); // Missing 'sortOrder'
```

### Valid 

Examples of correct code for this rule:

```js
// ✅ All dependencies included
useEffect(() => , [count]);

// ✅ All dependencies included
useEffect(() => , [userId]);
```

## Troubleshooting 

### Adding a function dependency causes infinite loops 

You have an effect, but you're creating a new function on every render:

```js
// ❌ Causes infinite loop
const logItems = () => ;

useEffect(() => , [logItems]); // Infinite loop!
```

In most cases, you don't need the effect. Call the function where the action happens instead:

```js
// ✅ Call it from the event handler
const logItems = () => ;

return Log;

// ✅ Or derive during render if there's no side effect
items.forEach(item => );
```

If you genuinely need the effect (for example, to subscribe to something external), make the dependency stable:

```js
// ✅ useCallback keeps the function reference stable
const logItems = useCallback(() => , [items]);

useEffect(() => , [logItems]);

// ✅ Or move the logic straight into the effect
useEffect(() => , [items]);
```

### Running an effect only once 

You want to run an effect once on mount, but the linter complains about missing dependencies:

```js
// ❌ Missing dependency
useEffect(() => , []); // Missing 'userId'
```

Either include the dependency (recommended) or use a ref if you truly need to run once:

```js
// ✅ Include dependency
useEffect(() => , [userId]);

// ✅ Or use a ref guard inside an effect
const sent = useRef(false);

useEffect(() => {
  if (sent.current) 

  sent.current = true;
  sendAnalytics(userId);
}, [userId]);
```

## Options 

You can configure custom effect hooks using shared ESLint settings (available in `eslint-plugin-react-hooks` 6.1.1 and later):

```js
{
  "settings": {
    "react-hooks": 
  }
}
```

- `additionalEffectHooks`: Regex pattern matching custom hooks that should be checked for exhaustive dependencies. This configuration is shared across all `react-hooks` rules.

For backward compatibility, this rule also accepts a rule-level option:

```js
{
  "rules": {
    "react-hooks/exhaustive-deps": ["warn", ]
  }
}
```

- `additionalHooks`: Regex for hooks that should be checked for exhaustive dependencies. **Note:** If this rule-level option is specified, it takes precedence over the shared `settings` configuration.
