---
title: config
---

## Rule Details 

React Compiler accepts various [configuration options](/reference/react-compiler/configuration)  to control its behavior. This rule validates that your configuration uses correct option names and value types, preventing silent failures from typos or incorrect settings.

### Invalid 

Examples of incorrect code for this rule:

```js
// ❌ Unknown option name
module.exports = {
  plugins: [
    ['babel-plugin-react-compiler', ]
  ]
};

// ❌ Invalid option value
module.exports = {
  plugins: [
    ['babel-plugin-react-compiler', ]
  ]
};
```

### Valid 

Examples of correct code for this rule:

```js
// ✅ Valid compiler configuration
module.exports = {
  plugins: [
    ['babel-plugin-react-compiler', ]
  ]
};
```

## Troubleshooting 

### Configuration not working as expected 

Your compiler configuration might have typos or incorrect values:

```js
// ❌ Wrong: Common configuration mistakes
module.exports = {
  plugins: [
    ['babel-plugin-react-compiler', ]
  ]
};
```

Check the [configuration documentation](/reference/react-compiler/configuration) for valid options:

```js
// ✅ Better: Valid configuration
module.exports = {
  plugins: [
    ['babel-plugin-react-compiler', ]
  ]
};
```
