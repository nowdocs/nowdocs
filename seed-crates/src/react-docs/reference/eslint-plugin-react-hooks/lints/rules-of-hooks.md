---
title: rules-of-hooks
---

## Rule Details 

React relies on the order in which hooks are called to correctly preserve state between renders. Each time your component renders, React expects the exact same hooks to be called in the exact same order. When hooks are called conditionally or in loops, React loses track of which state corresponds to which hook call, leading to bugs like state mismatches and "Rendered fewer/more hooks than expected" errors.

## Common Violations 

These patterns violate the Rules of Hooks:

- **Hooks in conditions** (`if`/`else`, ternary, `&&`/`||`)
- **Hooks in loops** (`for`, `while`, `do-while`)
- **Hooks after early returns**
- **Hooks in callbacks/event handlers**
- **Hooks in async functions**
- **Hooks in class methods**
- **Hooks at module level**

### Invalid 

Examples of incorrect code for this rule:

```js
// ❌ Hook in condition
if (isLoggedIn) 

// ❌ Hook after early return
if (!data) return 

### I need different state for different scenarios 

You're trying to conditionally initialize state:

```js
// ❌ Conditional state
if (userType === 'admin')  else 
```

Always call useState, conditionally set the initial value:

```js
// ✅ Conditional initial value
const [permissions, setPermissions] = useState(
  userType === 'admin' ? adminPerms : userPerms
);
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

- `additionalEffectHooks`: Regex pattern matching custom hooks that should be treated as effects. This allows `useEffectEvent` and similar event functions to be called from your custom effect hooks.

This shared configuration is used by both `rules-of-hooks` and `exhaustive-deps` rules, ensuring consistent behavior across all hook-related linting.
