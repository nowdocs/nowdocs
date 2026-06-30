---
title: static-components
---

## Rule Details 

Components defined inside other components are recreated on every render. React sees each as a brand new component type, unmounting the old one and mounting the new one, destroying all state and DOM nodes in the process.

### Invalid 

Examples of incorrect code for this rule:

```js
// ❌ Component defined inside component
function Parent() {
  const ChildComponent = () => >;
  };

  return 