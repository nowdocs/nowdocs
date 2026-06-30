---
title: onDemandEntries
description: Configure how Next.js will dispose and keep in memory pages created in development.
---

Next.js exposes some options that give you some control over how the server will dispose or keep in memory built pages in development.

To change the defaults, open `next.config.js` and add the `onDemandEntries` config:

```js filename="next.config.js"
module.exports = {
  onDemandEntries: ,
}
```
