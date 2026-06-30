---
title: "use no memo"
titleForTitleTag: "'use no memo' directive"
---

---

## Reference 

### `"use no memo"` 

Add `"use no memo"` at the beginning of a function to prevent React Compiler optimization.

```js 
function MyComponent() 
```

When a function contains `"use no memo"`, the React Compiler will skip it entirely during optimization. This is useful as a temporary escape hatch when debugging or when dealing with code that doesn't work correctly with the compiler.

#### Caveats 

* `"use no memo"` must be at the very beginning of a function body, before any imports or other code (comments are OK).
* The directive must be written with double or single quotes, not backticks.
* The directive must exactly match `"use no memo"` or its alias `"use no forget"`.
* This directive takes precedence over all compilation modes and other directives.
* It's intended as a temporary debugging tool, not a permanent solution.

### How `"use no memo"` opts-out of optimization 

React Compiler analyzes your code at build time to apply optimizations. `"use no memo"` creates an explicit boundary that tells the compiler to skip a function entirely.

This directive takes precedence over all other settings:
* In `all` mode: The function is skipped despite the global setting
* In `infer` mode: The function is skipped even if heuristics would optimize it

The compiler treats these functions as if the React Compiler wasn't enabled, leaving them exactly as written.

### When to use `"use no memo"` 

`"use no memo"` should be used sparingly and temporarily. Common scenarios include:

#### Debugging compiler issues 
When you suspect the compiler is causing issues, temporarily disable optimization to isolate the problem:

```js
function ProblematicComponent() 
```

#### Third-party library integration 
When integrating with libraries that might not be compatible with the compiler:

```js
function ThirdPartyWrapper() 
```

---

## Usage 

The `"use no memo"` directive is placed at the beginning of a function body to prevent React Compiler from optimizing that function:

```js
function MyComponent() 
```

The directive can also be placed at the top of a file to affect all functions in that module:

```js
"use no memo";

// All functions in this file will be skipped by the compiler
```

`"use no memo"` at the function level overrides the module level directive.

---

## Troubleshooting 

### Directive not preventing compilation 

If `"use no memo"` isn't working:

```js
// ❌ Wrong - directive after code
function Component() 

// ✅ Correct - directive first
function Component() 
```

Also check:
* Spelling - must be exactly `"use no memo"`
* Quotes - must use single or double quotes, not backticks

### Best practices 

**Always document why** you're disabling optimization:

```js
// ✅ Good - clear explanation and tracking
function DataProcessor() 

// ❌ Bad - no explanation
function Mystery() 
```

### See also 

* [`"use memo"`](/reference/react-compiler/directives/use-memo) - Opt into compilation
* [React Compiler](/learn/react-compiler) - Getting started guide