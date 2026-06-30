---
outline: deep
---

# Suspense 

:::warning Experimental Feature
`
```

On initial render, `
      
    
  

```

Vue Router has built-in support for [lazily loading components](https://router.vuejs.org/guide/advanced/lazy-loading.html) using dynamic imports. These are distinct from async components and currently they will not trigger `
```

`
  

```

If you don't set the `suspensible` prop, the inner `` will be treated like a sync component by the parent ``. That means that it has its own fallback slot and if both `Dynamic` components change at the same time, there might be empty nodes and multiple patching cycles while the child `` is loading its own dependency tree, which might not be desirable. When it's set, all the async dependency handling is given to the parent `` (including the events emitted) and the inner `` serves solely as another boundary for the dependency resolution and patching.

---

**Related**

- [`` API reference](/api/built-in-components#suspense)
