# Props 

A child component can accept input from the parent via **props**. First, it needs to declare the props it accepts:

```vue [ChildComp.vue]

const props = defineProps()

```

Note `defineProps()` is a compile-time macro and doesn't need to be imported. Once declared, the `msg` prop can be used in the child component's template. It can also be accessed in JavaScript via the returned object of `defineProps()`.

```js
// in child component
export default {
  props: ,
  setup(props) 
}
```

Once declared, the `msg` prop is exposed on `this` and can be used in the child component's template. The received props are passed to `setup()` as the first argument.

```js
// in child component
export default {
  props: 
}
```

Once declared, the `msg` prop is exposed on `this` and can be used in the child component's template.

The parent can pass the prop to the child just like attributes. To pass a dynamic value, we can also use the `v-bind` syntax:

```vue-html

```

```vue-html

```

Now try it yourself in the editor.
