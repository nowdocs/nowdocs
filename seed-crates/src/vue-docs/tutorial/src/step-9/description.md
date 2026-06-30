# Lifecycle and Template Refs 

So far, Vue has been handling all the DOM updates for us, thanks to reactivity and declarative rendering. However, inevitably there will be cases where we need to manually work with the DOM.

We can request a **template ref** - i.e. a reference to an element in the template - using the special `ref` attribute:

```vue-html
hello
```

To access the ref, we need to declare and expose a ref with matching name:

```js
const pElementRef = ref(null)
```

```js
setup() {
  const pElementRef = ref(null)

  return 
}
```

Notice the ref is initialized with `null` value. This is because the element doesn't exist yet when ```setup()` is executed. The template ref is only accessible after the component is **mounted**.

To run code after mount, we can use the `onMounted()` function:

```js
import  from 'vue'

onMounted(() => )
```

```js
import  from 'vue'

createApp({
  setup() {
    onMounted(() => )
  }
})
```

The element will be exposed on `this.$refs` as `this.$refs.pElementRef`. However, you can only access it after the component is **mounted**.

To run code after mount, we can use the `mounted` option:

```js
export default {
  mounted() 
}
```

```js
createApp({
  mounted() 
})
```

This is called a **lifecycle hook** - it allows us to register a callback to be called at certain times of the component's lifecycle. There are other hooks such as `created` and `updated``onUpdated` and `onUnmounted`. Check out the Lifecycle Diagram for more details.

Now, try to add a `mounted`an `onMounted` hook, access the `` via `this.$refs.pElementRef``pElementRef.value`, and perform some direct DOM operations on it (e.g. changing its `textContent`).
