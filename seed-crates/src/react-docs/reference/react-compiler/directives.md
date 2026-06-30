---
title: Directives
---

```js
function MyComponent() {
  "use memo"; // Opt this component into compilation
  return ;
}
```

---

## Overview 

React Compiler directives provide fine-grained control over which functions are optimized by the compiler. They are string literals placed at the beginning of a function body or at the top of a module.

### Available directives 

* **[`"use memo"`](/reference/react-compiler/directives/use-memo)** - Opts a function into compilation
* **[`"use no memo"`](/reference/react-compiler/directives/use-no-memo)** - Opts a function out of compilation

### Quick comparison 

| Directive | Purpose | When to use |
|-----------|---------|-------------|
| [`"use memo"`](/reference/react-compiler/directives/use-memo) | Force compilation | When using `annotation` mode or to override `infer` mode heuristics |
| [`"use no memo"`](/reference/react-compiler/directives/use-no-memo) | Prevent compilation | Debugging issues or working with incompatible code |

---

## Usage 

### Function-level directives 

Place directives at the beginning of a function to control its compilation:

```js
// Opt into compilation
function OptimizedComponent() 

// Opt out of compilation
function UnoptimizedComponent() 
```

### Module-level directives 

Place directives at the top of a file to affect all functions in that module:

```js
// At the very top of the file
"use memo";

// All functions in this file will be compiled
function Component1() 

function Component2() 

// Can be overridden at function level
function Component3() 
```

### Compilation modes interaction 

Directives behave differently depending on your [`compilationMode`](/reference/react-compiler/compilationMode):

* **`annotation` mode**: Only functions with `"use memo"` are compiled
* **`infer` mode**: Compiler decides what to compile, directives override decisions
* **`all` mode**: Everything is compiled, `"use no memo"` can exclude specific functions

---

## Best practices 

### Use directives sparingly 

Directives are escape hatches. Prefer configuring the compiler at the project level:

```js
// ✅ Good - project-wide configuration
{
  plugins: [
    ['babel-plugin-react-compiler', ]
  ]
}

// ⚠️ Use directives only when needed
function SpecialCase() 
```

### Document directive usage 

Always explain why a directive is used:

```js
// ✅ Good - clear explanation
function DataGrid() 

// ❌ Bad - no explanation
function Mystery() 
```

### Plan for removal 

Opt-out directives should be temporary:

1. Add the directive with a TODO comment
2. Create a tracking issue
3. Fix the underlying problem
4. Remove the directive

```js
function TemporaryWorkaround() 
```

---

## Common patterns 

### Gradual adoption 

When adopting the React Compiler in a large codebase:

```js
// Start with annotation mode

// Opt in stable components
function StableComponent() 

// Later, switch to infer mode and opt out problematic ones
function ProblematicComponent() 
```

---

## Troubleshooting 

For specific issues with directives, see the troubleshooting sections in:

* [`"use memo"` troubleshooting](/reference/react-compiler/directives/use-memo#troubleshooting)
* [`"use no memo"` troubleshooting](/reference/react-compiler/directives/use-no-memo#troubleshooting)

### Common issues 

1. **Directive ignored**: Check placement (must be first) and spelling
2. **Compilation still happens**: Check `ignoreUseNoForget` setting
3. **Module directive not working**: Ensure it's before all imports

---

## See also 

* [`compilationMode`](/reference/react-compiler/compilationMode) - Configure how the compiler chooses what to optimize
* [`Configuration`](/reference/react-compiler/configuration) - Full compiler configuration options
* [React Compiler documentation](https://react.dev/learn/react-compiler) - Getting started guide