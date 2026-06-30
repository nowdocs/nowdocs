---
title: useDebugValue
---

 for [React DevTools.](/learn/react-developer-tools)

```js [[1, 5, "isOnline ? 'Online' : 'Offline'"]]
import  from 'react';

function useOnlineStatus() 
```

This gives components calling `useOnlineStatus` a label like `OnlineStatus: "Online"` when you inspect them:

Without the `useDebugValue` call, only the underlying data (in this example, `true`) would be displayed.

---

### Deferring formatting of a debug value 

You can also pass a formatting function as the second argument to `useDebugValue`:

```js [[1, 1, "date", 18], [2, 1, "date.toDateString()"]]
useDebugValue(date, date => date.toDateString());
```

Your formatting function will receive the  as a parameter and should return a . When your component is inspected, React DevTools will call this function and display its result.

This lets you avoid running potentially expensive formatting logic unless the component is actually inspected. For example, if `date` is a Date value, this avoids calling `toDateString()` on it for every render.
