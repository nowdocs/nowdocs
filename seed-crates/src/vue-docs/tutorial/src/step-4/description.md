# Event Listeners 

We can listen to DOM events using the `v-on` directive:

```vue-html
{}
```

Due to its frequent use, `v-on` also has a shorthand syntax:

```vue-html
{}
```

Here, `increment` references a function declared using the `methods` option:

```js
export default {
  data() {
    return 
  },
  methods: {
    increment() 
  }
}
```

```js
createApp({
  data() {
    return 
  },
  methods: {
    increment() 
  }
})
```

Inside a method, we can access the component instance using `this`. The component instance exposes the data properties declared by `data`. We can update the component state by mutating these properties.

Here, `increment` is referencing a function declared in ``:

```vue

import  from 'vue'

const count = ref(0)

function increment() 

```

Here, `increment` is referencing a method in the object returned from `setup()`:

```js
setup() {
  const count = ref(0)

  function increment(e) 

  return 
}
```

Inside the function, we can update the component state by mutating refs.

Event handlers can also use inline expressions, and can simplify common tasks with modifiers. These details are covered in Guide - Event Handling.

Now, try to implement the `increment` methodfunction yourself and bind it to the button using `v-on`.
