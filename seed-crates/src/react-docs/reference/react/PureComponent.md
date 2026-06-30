---
title: PureComponent
---

---

## Alternatives 

### Migrating from a `PureComponent` class component to a function 

We recommend using function components instead of [class components](/reference/react/Component) in new code. If you have some existing class components using `PureComponent`, here is how you can convert them. This is the original code:

When you [convert this component from a class to a function,](/reference/react/Component#alternatives) wrap it in [`memo`:](/reference/react/memo)

