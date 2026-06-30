---
title: compilationMode
---

```js

```

---

## Reference 

### `compilationMode` 

Controls the strategy for determining which functions the React Compiler will optimize.

#### Type 

```
'infer' | 'syntax' | 'annotation' | 'all'
```

#### Default value 

`'infer'`

#### Options 

- **`'infer'`** (default): The compiler uses intelligent heuristics to identify React components and hooks:
  - Functions explicitly annotated with `"use memo"` directive
  - Functions that are named like components (PascalCase) or hooks (`use` prefix) AND create JSX and/or call other hooks

- **`'annotation'`**: Only compile functions explicitly marked with the `"use memo"` directive. Ideal for incremental adoption.

- **`'syntax'`**: Only compile components and hooks that use Flow's [component](https://flow.org/en/docs/react/component-syntax/) and [hook](https://flow.org/en/docs/react/hook-syntax/) syntax.

- **`'all'`**: Compile all top-level functions. Not recommended as it may compile non-React functions.

#### Caveats 

- The `'infer'` mode requires functions to follow React naming conventions to be detected
- Using `'all'` mode may negatively impact performance by compiling utility functions
- The `'syntax'` mode requires Flow and won't work with TypeScript
- Regardless of mode, functions with `"use no memo"` directive are always skipped

---

## Usage 

### Default inference mode 

The default `'infer'` mode works well for most codebases that follow React conventions:

```js

```

With this mode, these functions will be compiled:

```js
// ✅ Compiled: Named like a component + returns JSX
function Button(props) {
  return ;
}

// ✅ Compiled: Named like a hook + calls hooks
function useCounter() 

// ✅ Compiled: Explicit directive
function expensiveCalculation(data) 

// ❌ Not compiled: Not a component/hook pattern
function calculateTotal(items) 
```

### Incremental adoption with annotation mode 

For gradual migration, use `'annotation'` mode to only compile marked functions:

```js

```

Then explicitly mark functions to compile:

```js
// Only this function will be compiled
function ExpensiveList(props) {
  "use memo";
  return (
    
      {props.items.map(item => (
        
      ))}
    
  );
}

// This won't be compiled without the directive
function NormalComponent(props) {
  return ;
}
```

### Using Flow syntax mode 

If your codebase uses Flow instead of TypeScript:

```js

```

Then use Flow's component syntax:

```js
// Compiled: Flow component syntax
component Button(label: string) {
  return ;
}

// Compiled: Flow hook syntax
hook useCounter(initial: number) 

// Not compiled: Regular function syntax
function helper(data) 
```

### Opting out specific functions 

Regardless of compilation mode, use `"use no memo"` to skip compilation:

```js
function ComponentWithSideEffects() 
```

---

## Troubleshooting 

### Component not being compiled in infer mode 

In `'infer'` mode, ensure your component follows React conventions:

```js
// ❌ Won't be compiled: lowercase name
function button(props) {
  return ;
}

// ✅ Will be compiled: PascalCase name
function Button(props) {
  return ;
}

// ❌ Won't be compiled: doesn't create JSX or call hooks
function useData() 

// ✅ Will be compiled: calls a hook
function useData() 
```
