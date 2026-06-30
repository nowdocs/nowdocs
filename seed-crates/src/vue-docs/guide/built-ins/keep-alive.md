
import SwitchComponent from './keep-alive-demos/SwitchComponent.vue'

# KeepAlive 

`
```

Now, the state will be persisted across component switches:

```

The match is checked against the component's [`name`](/api/options-misc#name) option, so components that need to be conditionally cached by `KeepAlive` must explicitly declare a `name` option.

:::tip
Since version 3.2.34, a single-file component using `` will automatically infer its `name` option based on the filename, removing the need to manually declare the name.
:::

## Max Cached Instances 

We can limit the maximum number of component instances that can be cached via the `max` prop. When `max` is specified, `
```

## Lifecycle of Cached Instance 

When a component instance is removed from the DOM but is part of a component tree cached by ``, it goes into a **deactivated** state instead of being unmounted. When a component instance is inserted into the DOM as part of a cached tree, it is **activated**.

A kept-alive component can register lifecycle hooks for these two states using [`onActivated()`](/api/composition-api-lifecycle#onactivated) and [`onDeactivated()`](/api/composition-api-lifecycle#ondeactivated):

```vue

import  from 'vue'

onActivated(() => )

onDeactivated(() => )

```

A kept-alive component can register lifecycle hooks for these two states using [`activated`](/api/options-lifecycle#activated) and [`deactivated`](/api/options-lifecycle#deactivated) hooks:

```js
export default {
  activated() ,
  deactivated() 
}
```

Note that:

- `onActivated``activated` is also called on mount, and `onDeactivated``deactivated` on unmount.

- Both hooks work for not only the root component cached by ``, but also the descendant components in the cached tree.
---

**Related**

- [`` API reference](/api/built-in-components#keepalive)
