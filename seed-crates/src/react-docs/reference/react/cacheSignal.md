---
title: cacheSignal
---

 to abort in-flight requests.

```js [[1, 4, "cacheSignal()"]]
import  from 'react';
const dedupedFetch = cache(fetch);
async function Component() {
  await dedupedFetch(url, );
}
```

### Ignore errors after React has finished rendering 

If a function throws, it may be due to cancellation (e.g.  has been closed). You can use the  to check if the error was due to cancellation or a real error. You may want to  that were due to cancellation.

```js [[1, 2, "./database"], [2, 8, "cacheSignal()?.aborted"], [3, 12, "return null"]]
import  from "react";
import  from "./database";

async function getData(id) {
  try  catch (x) {
     if (!cacheSignal()?.aborted) 
     return null;
  }
}

async function Component() {
  const data = await getData(id);
  if (data === null) 
  return ;
}
```
