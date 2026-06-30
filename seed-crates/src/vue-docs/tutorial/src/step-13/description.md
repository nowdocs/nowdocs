# Emits 

In addition to receiving props, a child component can also emit events to the parent:

```vue

// declare emitted events
const emit = defineEmits(['response'])

// emit with argument
emit('response', 'hello from child')

```

```js
export default {
  // declare emitted events
  emits: ['response'],
  setup(props, ) 
}
```

```js
export default {
  // declare emitted events
  emits: ['response'],
  created() 
}
```

The first argument to `this.$emit()``emit()` is the event name. Any additional arguments are passed on to the event listener.

The parent can listen to child-emitted events using `v-on` - here the handler receives the extra argument from the child emit call and assigns it to local state:

```vue-html
 childMsg = msg" />
```

```vue-html
 childMsg = msg">
```

Now try it yourself in the editor.
