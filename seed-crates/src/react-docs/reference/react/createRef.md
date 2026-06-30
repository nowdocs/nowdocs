---
title: createRef
---

---

## Alternatives 

### Migrating from a class with `createRef` to a function with `useRef` 

We recommend using function components instead of [class components](/reference/react/Component) in new code. If you have some existing class components using `createRef`, here is how you can convert them. This is the original code:

When you [convert this component from a class to a function,](/reference/react/Component#alternatives) replace calls to `createRef` with calls to [`useRef`:](/reference/react/useRef)

