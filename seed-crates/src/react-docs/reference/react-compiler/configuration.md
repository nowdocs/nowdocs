---
title: Configuration
---

```js
// babel.config.js
module.exports = {
  plugins: [
    [
      'babel-plugin-react-compiler', 
    ]
  ]
};
```

---

## Compilation Control 

These options control *what* the compiler optimizes and *how* it selects components and hooks to compile.

* [`compilationMode`](/reference/react-compiler/compilationMode) controls the strategy for selecting functions to compile (e.g., all functions, only annotated ones, or intelligent detection).

```js

```

---

## Version Compatibility 

React version configuration ensures the compiler generates code compatible with your React version.

[`target`](/reference/react-compiler/target) specifies which React version you're using (17, 18, or 19).

```js
// For React 18 projects

```

---

## Error Handling 

These options control how the compiler responds to code that doesn't follow the [Rules of React](/reference/rules).

[`panicThreshold`](/reference/react-compiler/panicThreshold) determines whether to fail the build or skip problematic components.

```js
// Recommended for production

```

---

## Debugging 

Logging and analysis options help you understand what the compiler is doing.

[`logger`](/reference/react-compiler/logger) provides custom logging for compilation events.

```js
{
  logger: {
    logEvent(filename, event) {
      if (event.kind === 'CompileSuccess') 
    }
  }
}
```

---

## Feature Flags 

Conditional compilation lets you control when optimized code is used.

[`gating`](/reference/react-compiler/gating) enables runtime feature flags for A/B testing or gradual rollouts.

```js
{
  gating: 
}
```

---

## Common Configuration Patterns 

### Default configuration 

For most React 19 applications, the compiler works without configuration:

```js
// babel.config.js
module.exports = ;
```

### React 17/18 projects 

Older React versions need the runtime package and target configuration:

```bash
npm install react-compiler-runtime@latest
```

```js

```

### Incremental adoption 

Start with specific directories and expand gradually:

```js

```

