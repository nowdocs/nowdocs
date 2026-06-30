---
title: Rules of Hooks
---

---

## Only call Hooks from React functions 

Don’t call Hooks from regular JavaScript functions. Instead, you can:

✅ Call Hooks from React function components.
✅ Call Hooks from [custom Hooks](/learn/reusing-logic-with-custom-hooks#extracting-your-own-custom-hook-from-a-component).

By following this rule, you ensure that all stateful logic in a component is clearly visible from its source code.

```js 
function FriendList() 

function setOnlineStatus() 
```
