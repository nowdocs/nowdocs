# Declarative Rendering 

What you see in the editor is a Vue Single-File Component (SFC). An SFC is a reusable self-contained block of code that encapsulates HTML, CSS and JavaScript that belong together, written inside a `.vue` file.

The core feature of Vue is **declarative rendering**: using a template syntax that extends HTML, we can describe how the HTML should look based on JavaScript state. When the state changes, the HTML updates automatically.

State that can trigger updates when changed is considered **reactive**. We can declare reactive state using Vue's `reactive()` API. Objects created from `reactive()` are JavaScript [Proxies](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Proxy) that work just like normal objects:

```js
import  from 'vue'

const counter = reactive()

console.log(counter.count) // 0
counter.count++
```

`reactive()` only works on objects (including arrays and built-in types like `Map` and `Set`). `ref()`, on the other hand, can take any value type and create an object that exposes the inner value under a `.value` property:

```js
import  from 'vue'

const message = ref('Hello World!')

console.log(message.value) // "Hello World!"
message.value = 'Changed'
```

Details on `reactive()` and `ref()` are discussed in Guide - Reactivity Fundamentals.

Reactive state declared in the component's `` block can be used directly in the template. This is how we can render dynamic text based on the value of the `counter` object and `message` ref, using mustaches syntax:

The object being passed to `createApp()` is a Vue component. A component's state should be declared inside its `setup()` function, and returned using an object:

```js
setup() {
  const counter = reactive()
  const message = ref('Hello World!')
  return 
}
```

Properties in the returned object will be made available in the template. This is how we can render dynamic text based on the value of `message`, using mustaches syntax:

```vue-html
{}
Count is: {}
```

Notice how we did not need to use `.value` when accessing the `message` ref in templates: it is automatically unwrapped for more succinct usage.

State that can trigger updates when changed are considered **reactive**. In Vue, reactive state is held in components. In the example code, the object being passed to `createApp()` is a component.

We can declare reactive state using the `data` component option, which should be a function that returns an object:

```js
export default {
  data() {
    return 
  }
}
```

```js
createApp({
  data() {
    return 
  }
})
```

The `message` property will be made available in the template. This is how we can render dynamic text based on the value of `message`, using mustaches syntax:

```vue-html
{}
```

The content inside the mustaches is not limited to just identifiers or paths - we can use any valid JavaScript expression:

```vue-html
{}
```

Now, try to create some reactive state yourself, and use it to render dynamic text content for the `` in the template.

Now, try to create a data property yourself, and use it as the text content for the `` in the template.

