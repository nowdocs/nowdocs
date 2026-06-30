---
title: 
```

```

#### Props 

* `id`: A string identifying the part of the UI you are measuring.
* `onRender`: An [`onRender` callback](#onrender-callback) that React calls every time components within the profiled tree update. It receives information about what was rendered and how much time it took.

#### Caveats 

* Profiling adds some additional overhead, so **it is disabled in the production build by default.** To opt into production profiling, you need to enable a [special production build with profiling enabled.](/reference/dev-tools/react-performance-tracks#using-profiling-builds)

---

### `onRender` callback 

React will call your `onRender` callback with information about what was rendered.

```js
function onRender(id, phase, actualDuration, baseDuration, startTime, commitTime) 
```

#### Parameters 

* `id`: The string `id` prop of the `
  
```

It requires two props: an `id` (string) and an `onRender` callback (function) which React calls any time a component within the tree "commits" an update.

---

### Measuring different parts of the application 

You can use multiple `
  

```

You can also nest `
  
      
  

```

Although `` is a lightweight component, it should be used only when necessary. Each use adds some CPU and memory overhead to an application.

---

