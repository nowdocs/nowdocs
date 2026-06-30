---
pageClass: api
---

# Built-in Components 

:::info Registration and Usage
Built-in components can be used directly in templates without needing to be registered. They are also tree-shakeable: they are only included in the build when they are used.

When using them in [render functions](/guide/extras/render-function), they need to be imported explicitly. For example:

```js
import  from 'vue'

h(Transition, )
```

:::

## `
  ```

  Forcing a transition by changing the `key` attribute:

  ```vue-html
  
  ```

  Dynamic component, with transition mode + animate on appear:

  ```vue-html
  
  ```

  Listening to transition events:

  ```vue-html
  
  ```

- **See also** [Guide - Transition](/guide/built-ins/transition)

## `
  ```

- **See also** [Guide - TransitionGroup](/guide/built-ins/transition-group)

## `
  ```

  When used with `v-if` / `v-else` branches, there must be only one component rendered at a time:

  ```vue-html
  
  ```

  Used together with `
  
  ```

  Using `include` / `exclude`:

  ```vue-html
  
  

  
  

  
  
  ```

  Usage with `max`:

  ```vue-html
  
  ```

- **See also** [Guide - KeepAlive](/guide/built-ins/keep-alive)

## `
  ```

  Defer target resolution :

  ```vue-html
  

  
  
  ```

- **See also** [Guide - Teleport](/guide/built-ins/teleport)

## ``  

Used for orchestrating nested async dependencies in a component tree.

- **Props**

  ```ts
  interface SuspenseProps 
  ```

- **Events**

  - `@resolve`
  - `@pending`
  - `@fallback`

- **Details**

  `` accepts two slots: the `#default` slot and the `#fallback` slot. It will display the content of the fallback slot while rendering the default slot in memory.

  If it encounters async dependencies ([Async Components](/guide/components/async) and components with [`async setup()`](/guide/built-ins/suspense#async-setup)) while rendering the default slot, it will wait until all of them are resolved before displaying the default slot.

  By setting the Suspense as `suspensible`, all the async dependency handling will be handled by the parent Suspense. See [implementation details](https://github.com/vuejs/core/pull/6736)

- **See also** [Guide - Suspense](/guide/built-ins/suspense)
