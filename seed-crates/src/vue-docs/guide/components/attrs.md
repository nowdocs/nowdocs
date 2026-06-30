---
outline: deep
---

# Fallthrough Attributes 

> This page assumes you've already read the [Components Basics](/guide/essentials/component-basics). Read that first if you are new to components.

## Attribute Inheritance 

A "fallthrough attribute" is an attribute or `v-on` event listener that is passed to a component, but is not explicitly declared in the receiving component's [props](./props) or [emits](./events#declaring-emitted-events). Common examples of this include `class`, `style`, and `id` attributes.

When a component renders a single root element, fallthrough attributes will be automatically added to the root element's attributes. For example, given a `` component with the following template:

```vue-html

Click Me
```

And a parent using this component with:

```vue-html

```

The final rendered DOM would be:

```html
Click Me
```

Here, `` did not declare `class` as an accepted prop. Therefore, `class` is treated as a fallthrough attribute and automatically added to ``'s root element.

### `class` and `style` Merging 

If the child component's root element already has existing `class` or `style` attributes, it will be merged with the `class` and `style` values that are inherited from the parent. Suppose we change the template of `` in the previous example to:

```vue-html

Click Me
```

Then the final rendered DOM would now become:

```html
Click Me
```

### `v-on` Listener Inheritance 

The same rule applies to `v-on` event listeners:

```vue-html

```

The `click` listener will be added to the root element of ``, i.e. the native `` element. When the native `` is clicked, it will trigger the `onClick` method of the parent component. If the native `` already has a `click` listener bound with `v-on`, then both listeners will trigger.

### Nested Component Inheritance 

If a component renders another component as its root node, for example, we refactored `` to render a `` as its root:

```vue-html

```

Then the fallthrough attributes received by `` will be automatically forwarded to ``.

Note that:

1. Forwarded attributes do not include any attributes that are declared as props, or `v-on` listeners of declared events by `` - in other words, the declared props and listeners have been "consumed" by ``.

2. Forwarded attributes may be accepted as props by ``, if declared by it.

## Disabling Attribute Inheritance 

If you do **not** want a component to automatically inherit attributes, you can set `inheritAttrs: false` in the component's options.

 Since 3.3 you can also use [`defineOptions`](/api/sfc-script-setup#defineoptions) directly in ``:

```vue

defineOptions()
// ...setup logic

```

The common scenario for disabling attribute inheritance is when attributes need to be applied to other elements besides the root node. By setting the `inheritAttrs` option to `false`, you can take full control over where the fallthrough attributes should be applied.

These fallthrough attributes can be accessed directly in template expressions as `$attrs`:

```vue-html
Fallthrough attributes: {}
```

The `$attrs` object includes all attributes that are not declared by the component's `props` or `emits` options (e.g., `class`, `style`, `v-on` listeners, etc.).

Some notes:

- Unlike props, fallthrough attributes preserve their original casing in JavaScript, so an attribute like `foo-bar` needs to be accessed as `$attrs['foo-bar']`.

- A `v-on` event listener like `@click` will be exposed on the object as a function under `$attrs.onClick`.

Using our `` component example from the [previous section](#attribute-inheritance) - sometimes we may need to wrap the actual `` element with an extra `` for styling purposes:

```vue-html

  Click Me

```

We want all fallthrough attributes like `class` and `v-on` listeners to be applied to the inner ``, not the outer ``. We can achieve this with `inheritAttrs: false` and `v-bind="$attrs"`:

```vue-html

  Click Me

```

Remember that [`v-bind` without an argument](/guide/essentials/template-syntax#dynamically-binding-multiple-attributes) binds all the properties of an object as attributes of the target element.

## Attribute Inheritance on Multiple Root Nodes 

Unlike components with a single root node, components with multiple root nodes do not have an automatic attribute fallthrough behavior. If `$attrs` are not bound explicitly, a runtime warning will be issued.

```vue-html

```

If `` has the following multi-root template, there will be a warning because Vue cannot be sure where to apply the fallthrough attributes:

```vue-html
...
...
...
```

The warning will be suppressed if `$attrs` is explicitly bound:

```vue-html
...
...
...
```

## Accessing Fallthrough Attributes in JavaScript 

If needed, you can access a component's fallthrough attributes in `` using the `useAttrs()` API:

```vue

import  from 'vue'

const attrs = useAttrs()

```

If not using ``, `attrs` will be exposed as a property of the `setup()` context:

```js
export default {
  setup(props, ctx) 
}
```

Note that although the `attrs` object here always reflects the latest fallthrough attributes, it isn't reactive (for performance reasons). You cannot use watchers to observe its changes. If you need reactivity, use a prop. Alternatively, you can use `onUpdated()` to perform side effects with the latest `attrs` on each update.

If needed, you can access a component's fallthrough attributes via the `$attrs` instance property:

```js
export default {
  created() 
}
```

