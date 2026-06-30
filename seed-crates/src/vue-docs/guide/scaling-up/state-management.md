# State Management 

## What is State Management? 

Technically, every Vue component instance already "manages" its own reactive state. Take a simple counter component as an example:

```vue

import  from 'vue'

// state
const count = ref(0)

// actions
function increment() 

{}
```

```vue

export default {
  // state
  data() {
    return 
  },
  // actions
  methods: {
    increment() 
  }
}

{}
```

It is a self-contained unit with the following parts:

- The **state**, the source of truth that drives our app;
- The **view**, a declarative mapping of the **state**;
- The **actions**, the possible ways the state could change in reaction to user inputs from the **view**.

This is a simple representation of the concept of "one-way data flow":

  

However, the simplicity starts to break down when we have **multiple components that share a common state**:

1. Multiple views may depend on the same piece of state.
2. Actions from different views may need to mutate the same piece of state.

For case one, a possible workaround is by "lifting" the shared state up to a common ancestor component, and then pass it down as props. However, this quickly gets tedious in component trees with deep hierarchies, leading to another problem known as [Prop Drilling](/guide/components/provide-inject#prop-drilling).

For case two, we often find ourselves resorting to solutions such as reaching for direct parent / child instances via template refs, or trying to mutate and synchronize multiple copies of the state via emitted events. Both of these patterns are brittle and quickly lead to unmaintainable code.

A simpler and more straightforward solution is to extract the shared state out of the components, and manage it in a global singleton. With this, our component tree becomes a big "view", and any component can access the state or trigger actions, no matter where they are in the tree!

## Simple State Management with Reactivity API 

In Options API, reactive data is declared using the `data()` option. Internally, the object returned by `data()` is made reactive via the [`reactive()`](/api/reactivity-core#reactive) function, which is also available as a public API.

If you have a piece of state that should be shared by multiple instances, you can use [`reactive()`](/api/reactivity-core#reactive) to create a reactive object, and then import it into multiple components:

```js [store.js]
import  from 'vue'

export const store = reactive()
```

```vue [ComponentA.vue]

import  from './store.js'

From A: {}
```

```vue [ComponentB.vue]

import  from './store.js'

From B: {}
```

```vue [ComponentA.vue]

import  from './store.js'

export default {
  data() {
    return 
  }
}

From A: {}
```

```vue [ComponentB.vue]

import  from './store.js'

export default {
  data() {
    return 
  }
}

From B: {}
```

Now whenever the `store` object is mutated, both `` and `` will update their views automatically - we have a single source of truth now.

However, this also means any component importing `store` can mutate it however they want:

```vue-html

  
    From B: {}
  

```

While this works in simple cases, global state that can be arbitrarily mutated by any component is not going to be very maintainable in the long run. To ensure the state-mutating logic is centralized like the state itself, it is recommended to define methods on the store with names that express the intention of the actions:

```js [store.js]
import  from 'vue'

export const store = reactive({
  count: 0,
  increment() 
})
```

```vue-html

  
    From B: {}
  

```

[Try it in the Playground](https://play.vuejs.org/#eNrNkk1uwyAQha8yYpNEiUzXllPVrtRTeJNSqtLGgGBsVbK4ewdwnT9FWWSTFczwmPc+xMhqa4uhl6xklRdOWQQvsbfPrVadNQ7h1dCqpcYaPp3pYFHwQyteXVxKm0tpM0krnm3IgAqUnd3vUFIFUB1Z8bNOkzoVny+wDTuNcZ1gBI/GSQhzqlQX3/5Gng81pA1t33tEo+FF7JX42bYsT1BaONlRguWqZZMU4C261CWMk3EhTK8RQphm8Twse/BscoUsvdqDkTX3kP3nI6aZwcmdQDUcMPJPabX8TQphtCf0RLqd1csxuqQAJTxtYnEUGtIpAH4pn1Ou17FDScOKhT+QNAVM)

[Try it in the Playground](https://play.vuejs.org/#eNrdU8FqhDAU/JVHLruyi+lZ3FIt9Cu82JilaTWR5CkF8d8bE5O1u1so9FYQzAyTvJnRTKTo+3QcOMlIbpgWPT5WUnS90gjPyr4ll1jAWasOdim9UMum3a20vJWWqxSgkvzTyRt+rocWYVpYFoQm8wRsJh+viHLBcyXtk9No2ALkXd/WyC0CyDfW6RVTOiancQM5ku+x7nUxgUGlOcwxn8Ppu7HJ7udqaqz3SYikOQ5aBgT+OA9slt9kasToFnb5OiAqCU+sFezjVBHvRUimeWdT7JOKrFKAl8VvYatdI6RMDRJhdlPtWdQf5mdQP+SHdtyX/IftlH9pJyS1vcQ2NK8ZivFSiL8BsQmmpMG1s1NU79frYA1k8OD+/I3pUA6+CeNdHg6hmoTMX9pPSnk=)

:::tip
Note the click handler uses `store.increment()` with parentheses - this is necessary to call the method with the proper `this` context since it's not a component method.
:::

Although here we are using a single reactive object as a store, you can also share reactive state created using other [Reactivity APIs](/api/reactivity-core) such as `ref()` or `computed()`, or even return global state from a [Composable](/guide/reusability/composables):

```js
import  from 'vue'

// global state, created in module scope
const globalCount = ref(1)

export function useCount() {
  // local state, created per-component
  const localCount = ref(1)

  return 
}
```

The fact that Vue's reactivity system is decoupled from the component model makes it extremely flexible.

## SSR Considerations 

If you are building an application that leverages [Server-Side Rendering (SSR)](./ssr), the above pattern can lead to issues due to the store being a singleton shared across multiple requests. This is discussed in [more details](./ssr#cross-request-state-pollution) in the SSR guide.

## Pinia 

While our hand-rolled state management solution will suffice in simple scenarios, there are many more things to consider in large-scale production applications:

- Stronger conventions for team collaboration
- Integrating with the Vue DevTools, including timeline, in-component inspection, and time-travel debugging
- Hot Module Replacement
- Server-Side Rendering support

[Pinia](https://pinia.vuejs.org) is a state management library that implements all of the above. It is maintained by the Vue core team, and works with both Vue 2 and Vue 3.

Existing users may be familiar with [Vuex](https://vuex.vuejs.org/), the previous official state management library for Vue. With Pinia serving the same role in the ecosystem, Vuex is now in maintenance mode. It still works, but will no longer receive new features. It is recommended to use Pinia for new applications.

Pinia started out as an exploration of what the next iteration of Vuex could look like, incorporating many ideas from core team discussions for Vuex 5. Eventually, we realized that Pinia already implements most of what we wanted in Vuex 5, and decided to make it the new recommendation instead.

Compared to Vuex, Pinia provides a simpler API with less ceremony, offers Composition-API-style APIs, and most importantly, has solid type inference support when used with TypeScript.
