---
title: component-hook-factories
---

## Rule Details 

Defining components or hooks inside other functions creates new instances on every call. React treats each as a completely different component, destroying and recreating the entire component tree, losing all state, and causing performance problems.

### Invalid 

Examples of incorrect code for this rule:

```js {expectedErrors: }
// ❌ Factory function creating components
function createComponent(defaultValue) {
  return function Component() ;
}

// ❌ Component defined inside component
function Parent() {
  function Child() 

  return 
      
    </>
  );
}
```
