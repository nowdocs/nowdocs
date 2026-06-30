
import  from 'vue'

if (typeof window !== 'undefined') {
  const hash = window.location.hash

  // The docs for v-model used to be part of this page. Attempt to redirect outdated links.
  if ([
    '#usage-with-v-model',
    '#v-model-arguments',
    '#multiple-v-model-bindings',
    '#handling-v-model-modifiers'
  ].includes(hash)) {
    onMounted(() => )
  }
}

# Component Events 

> This page assumes you've already read the [Components Basics](/guide/essentials/component-basics). Read that first if you are new to components.

  

## Emitting and Listening to Events 

A component can emit custom events directly in template expressions (e.g. in a `v-on` handler) using the built-in `$emit` method:

```vue-html

Click Me
```

The `$emit()` method is also available on the component instance as `this.$emit()`:

```js
export default {
  methods: {
    submit() 
  }
}
```

The parent can then listen to it using `v-on`:

```vue-html

```

The `.once` modifier is also supported on component event listeners:

```vue-html

```

Like components and props, event names provide an automatic case transformation. Notice we emitted a camelCase event, but can listen for it using a kebab-cased listener in the parent. As with [props casing](/guide/components/props#prop-name-casing), we recommend using kebab-cased event listeners in templates.

:::tip
Unlike native DOM events, component emitted events do **not** bubble. You can only listen to the events emitted by a direct child component. If there is a need to communicate between sibling or deeply nested components, use an external event bus or a [global state management solution](/guide/scaling-up/state-management).
:::

## Event Arguments 

It's sometimes useful to emit a specific value with an event. For example, we may want the `` component to be in charge of how much to enlarge the text by. In those cases, we can pass extra arguments to `$emit` to provide this value:

```vue-html

  Increase by 1

```

Then, when we listen to the event in the parent, we can use an inline arrow function as the listener, which allows us to access the event argument:

```vue-html
 count += n" />
```

Or, if the event handler is a method:

```vue-html

```

Then the value will be passed as the first parameter of that method:

```js
methods: {
  increaseCount(n) 
}
```

```js
function increaseCount(n) 
```

:::tip
All extra arguments passed to `$emit()` after the event name will be forwarded to the listener. For example, with `$emit('foo', 1, 2, 3)` the listener function will receive three arguments.
:::

## Declaring Emitted Events 

A component can explicitly declare the events it will emit using the [`defineEmits()`](/api/sfc-script-setup#defineprops-defineemits) macro[`emits`](/api/options-state#emits) option:

```vue

defineEmits(['inFocus', 'submit'])

```

The `$emit` method that we used in the `` isn't accessible within the `` section of a component, but `defineEmits()` returns an equivalent function that we can use instead:

```vue

const emit = defineEmits(['inFocus', 'submit'])

function buttonClick() 

```

The `defineEmits()` macro **cannot** be used inside a function, it must be placed directly within ``, as in the example above.

If you're using an explicit `setup` function instead of ``, events should be declared using the [`emits`](/api/options-state#emits) option, and the `emit` function is exposed on the `setup()` context:

```js
export default {
  emits: ['inFocus', 'submit'],
  setup(props, ctx) 
}
```

As with other properties of the `setup()` context, `emit` can safely be destructured:

```js
export default {
  emits: ['inFocus', 'submit'],
  setup(props, ) 
}
```

```js
export default 
```

The `emits` option and `defineEmits()` macro also support an object syntax. If using TypeScript you can type arguments, which allows us to perform runtime validation of the payload of the emitted events:

```vue

const emit = defineEmits({
  submit(payload: ) 
})

```

If you are using TypeScript with ``, it's also possible to declare emitted events using pure type annotations:

```vue

const emit = defineEmits<>()

```

More details: [Typing Component Emits](/guide/typescript/composition-api#typing-component-emits) 

```js
export default {
  emits: {
    submit(payload: ) 
  }
}
```

See also: [Typing Component Emits](/guide/typescript/options-api#typing-component-emits) 

Although optional, it is recommended to define all emitted events in order to better document how a component should work. It also allows Vue to exclude known listeners from [fallthrough attributes](/guide/components/attrs#v-on-listener-inheritance), avoiding edge cases caused by DOM events manually dispatched by 3rd party code.

:::tip
If a native event (e.g., `click`) is defined in the `emits` option, the listener will now only listen to component-emitted `click` events and no longer respond to native `click` events.
:::

## Events Validation 

Similar to prop type validation, an emitted event can be validated if it is defined with the object syntax instead of the array syntax.

To add validation, the event is assigned a function that receives the arguments passed to the `this.$emit``emit` call and returns a boolean to indicate whether the event is valid or not.

```vue

const emit = defineEmits({
  // No validation
  click: null,

  // Validate submit event
  submit: () => {
    if (email && password)  else 
  }
})

function submitForm(email, password) {
  emit('submit', )
}

```

```js
export default {
  emits: {
    // No validation
    click: null,

    // Validate submit event
    submit: () => {
      if (email && password)  else 
    }
  },
  methods: {
    submitForm(email, password) {
      this.$emit('submit', )
    }
  }
}
```

