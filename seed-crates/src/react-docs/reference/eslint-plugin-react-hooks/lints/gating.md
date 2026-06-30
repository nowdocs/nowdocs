---
title: gating
---

## Rule Details 

Gating mode lets you gradually adopt React Compiler by marking specific components for optimization. This rule ensures your gating configuration is valid so the compiler knows which components to process.

### Invalid 

Examples of incorrect code for this rule:

```js
// ❌ Missing required fields
module.exports = {
  plugins: [
    ['babel-plugin-react-compiler', {
      gating: 
    }]
  ]
};

// ❌ Invalid gating type
module.exports = {
  plugins: [
    ['babel-plugin-react-compiler', ]
  ]
};
```

### Valid 

Examples of correct code for this rule:

```js
// ✅ Complete gating configuration
module.exports = {
  plugins: [
    ['babel-plugin-react-compiler', {
      gating: 
    }]
  ]
};

// featureFlags.js
export function isCompilerEnabled() 

// ✅ No gating (compile everything)
module.exports = {
  plugins: [
    ['babel-plugin-react-compiler', ]
  ]
};
```
