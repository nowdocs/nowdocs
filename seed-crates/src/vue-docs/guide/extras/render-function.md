---
outline: deep
---

# Render Functions & JSX 

Vue recommends using templates to build applications in the vast majority of cases. However, there are situations where we need the full programmatic power of JavaScript. That's where we can use the **render function**.

> If you are new to the concept of virtual DOM and render functions, make sure to read the [Rendering Mechanism](/guide/extras/rendering-mechanism) chapter first.

## Basic Usage 

### Creating Vnodes 

Vue provides an `h()` function for creating vnodes:

```js
import  from 'vue'

const vnode = h(
  'div', // type
  , // props
  [
    /* children */
  ]
)
```

`h()` is short for **hyperscript** - which means "JavaScript that produces HTML (hypertext markup language)". This name is inherited from conventions shared by many virtual DOM implementations. A more descriptive name could be `createVNode()`, but a shorter name helps when you have to call this function many times in a render function.

The `h()` function is designed to be very flexible:

```js
// all arguments except the type are optional
h('div')
h('div', )

// both attributes and properties can be used in props
// Vue automatically picks the right way to assign it
h('div', )

// props modifiers such as `.prop` and `.attr` can be added
// with `.` and `^` prefixes respectively
h('div', )

// class and style have the same object / array
// value support that they have in templates
h('div', { class: [foo, ], style:  })

// event listeners should be passed as onXxx
h('div', { onClick: () =>  })

// children can be a string
h('div', , 'hello')

// props can be omitted when there are no props
h('div', 'hello')
h('div', [h('span', 'hello')])

// children array can contain mixed vnodes and strings
h('div', ['hello', h('span', 'hello')])
```

The resulting vnode has the following shape:

```js
const vnode = h('div', , [])

vnode.type // 'div'
vnode.props // 
vnode.children // []
vnode.key // null
```

:::warning Note
The full `VNode` interface contains many other internal properties, but it is strongly recommended to avoid relying on any properties other than the ones listed here. This avoids unintended breakage in case the internal properties are changed.
:::

### Declaring Render Functions 

When using templates with Composition API, the return value of the `setup()` hook is used to expose data to the template. When using render functions, however, we can directly return the render function instead:

```js
import  from 'vue'

export default {
  props: ,
  setup(props) 
}
```

The render function is declared inside `setup()` so it naturally has access to the props and any reactive state declared in the same scope.

In addition to returning a single vnode, you can also return strings or arrays:

```js
export default {
  setup() 
}
```

```js
import  from 'vue'

export default {
  setup() 
}
```

:::tip
Make sure to return a function instead of directly returning values! The `setup()` function is called only once per component, while the returned render function will be called multiple times.
:::

We can declare render functions using the `render` option:

```js
import  from 'vue'

export default {
  data() {
    return 
  },
  render() 
}
```

The `render()` function has access to the component instance via `this`.

In addition to returning a single vnode, you can also return strings or arrays:

```js
export default {
  render() 
}
```

```js
import  from 'vue'

export default {
  render() 
}
```

If a render function component doesn't need any instance state, they can also be declared directly as a function for brevity:

```js
function Hello() 
```

That's right, this is a valid Vue component! See [Functional Components](#functional-components) for more details on this syntax.

### Vnodes Must Be Unique 

All vnodes in the component tree must be unique. That means the following render function is invalid:

```js
function render() 
```

If you really want to duplicate the same element/component many times, you can do so with a factory function. For example, the following render function is a perfectly valid way of rendering 20 identical paragraphs:

```js
function render() {
  return h(
    'div',
    Array.from().map(() => )
  )
}
```

### Using Vnodes in `` 

```vue

import  from 'vue'

const vnode = h('button', ['Hello'])

  
  Hi

  
  
  Hi

```

A vnode object has been declared in `setup()`, you can use it like a normal component for rendering.

:::warning
A vnode represents an already created render output, not a component definition. Using a vnode in `` does not create a new component instance, and the vnode will be rendered as-is.

This pattern should be used with care and is not a replacement for normal components.
:::

## JSX / TSX 

[JSX](https://facebook.github.io/jsx/) is an XML-like extension to JavaScript that allows us to write code like this:

```jsx
const vnode = hello
```

Inside JSX expressions, use curly braces to embed dynamic values:

```jsx
const vnode = hello, 
```

`create-vue` and Vue CLI both have options for scaffolding projects with pre-configured JSX support. If you are configuring JSX manually, please refer to the documentation of [`@vue/babel-plugin-jsx`](https://github.com/vuejs/jsx-next) for details.

Although first introduced by React, JSX actually has no defined runtime semantics and can be compiled into various different outputs. If you have worked with JSX before, do note that **Vue JSX transform is different from React's JSX transform**, so you can't use React's JSX transform in Vue applications. Some notable differences from React JSX include:

- You can use HTML attributes such as `class` and `for` as props - no need to use `className` or `htmlFor`.
- Passing children to components (i.e. slots) [works differently](#passing-slots).

Vue's type definition also provides type inference for TSX usage. When using TSX, make sure to specify `"jsx": "preserve"` in `tsconfig.json` so that TypeScript leaves the JSX syntax intact for Vue JSX transform to process.

### JSX Type Inference 

Similar to the transform, Vue's JSX also needs different type definitions.

Starting in Vue 3.4, Vue no longer implicitly registers the global `JSX` namespace. To instruct TypeScript to use Vue's JSX type definitions, make sure to include the following in your `tsconfig.json`:

```json
{
  "compilerOptions": 
}
```

You can also opt-in per file by adding a `/* @jsxImportSource vue */` comment at the top of the file.

If there is code that depends on the presence of the global `JSX` namespace,  you can retain the exact pre-3.4 global behavior by explicitly importing or referencing `vue/jsx` in your project, which registers the global `JSX` namespace.

## Render Function Recipes 

Below we will provide some common recipes for implementing template features as their equivalent render functions / JSX.

### `v-if` 

Template:

```vue-html

  yes
  no

```

Equivalent render function / JSX:

```js
h('div', [ok.value ? h('div', 'yes') : h('span', 'no')])
```

```jsx

```

```js
h('div', [this.ok ? h('div', 'yes') : h('span', 'no')])
```

```jsx

```

### `v-for` 

Template:

```vue-html

  
    {}
  

```

Equivalent render function / JSX:

```js
h(
  'ul',
  // assuming `items` is a ref with array value
  items.value.map(() => {
    return h('li', , text)
  })
)
```

```jsx

  {items.value.map(() => {
    return 
  })}

```

```js
h(
  'ul',
  this.items.map(() => {
    return h('li', , text)
  })
)
```

```jsx

  {this.items.map(() => {
    return 
  })}

```

### `v-on` 

Props with names that start with `on` followed by an uppercase letter are treated as event listeners. For example, `onClick` is the equivalent of `@click` in templates.

```js
h(
  'button',
  {
    onClick(event) 
  },
  'Click Me'
)
```

```jsx
 }
>
  Click Me

```

#### Event Modifiers 

For the `.passive`, `.capture`, and `.once` event modifiers, they can be concatenated after the event name using camelCase.

For example:

```js
h('input', {
  onClickCapture() ,
  onKeyupOnce() ,
  onMouseoverOnceCapture() 
})
```

```jsx
 }
  onKeyupOnce={() => }
  onMouseoverOnceCapture={() => }
/>
```

For other event and key modifiers, the [`withModifiers`](/api/render-function#withmodifiers) helper can be used:

```js
import  from 'vue'

h('div', {
  onClick: withModifiers(() => , ['self'])
})
```

```jsx
 , ['self'])} />
```

### Components 

To create a vnode for a component, the first argument passed to `h()` should be the component definition. This means when using render functions, it is unnecessary to register components - you can just use the imported components directly:

```js
import Foo from './Foo.vue'
import Bar from './Bar.jsx'

function render() 
```

```jsx
function render() {
  return (
    
      

// named

```

Passing slots as functions allows them to be invoked lazily by the child component. This leads to the slot's dependencies being tracked by the child instead of the parent, which results in more accurate and efficient updates.

### Scoped Slots 

To render a scoped slot in the parent component, a slot is passed to the child. Notice how the slot now has a parameter `text`. The slot will be called in the child component and the data from the child component will be passed up to the parent component.

```js
// parent component
export default {
  setup() {
    return () => h(MyComp, null, {
      default: () => h('p', text)
    })
  }
}
```

Remember to pass `null` so the slots will not be treated as props.

```js
// child component
export default {
  setup(props, ) {
    const text = ref('hi')
    return () => h('div', null, slots.default())
  }
}
```

JSX equivalent:

```jsx

```

### Built-in Components 

[Built-in components](/api/built-in-components) such as ``, ``, ``, `` and `` must be imported for use in render functions:

```js
import  from 'vue'

export default {
  setup () {
    return () => h(Transition, , /* ... */)
  }
}
```

```js
import  from 'vue'

export default {
  render () {
    return h(Transition, , /* ... */)
  }
}
```

### `v-model` 

The `v-model` directive is expanded to `modelValue` and `onUpdate:modelValue` props during template compilation—we will have to provide these props ourselves:

```js
export default {
  props: ['modelValue'],
  emits: ['update:modelValue'],
  setup(props, ) {
    return () =>
      h(SomeComponent, )
  }
}
```

```js
export default {
  props: ['modelValue'],
  emits: ['update:modelValue'],
  render() {
    return h(SomeComponent, )
  }
}
```

### Custom Directives 

Custom directives can be applied to a vnode using [`withDirectives`](/api/render-function#withdirectives):

```js
import  from 'vue'

// a custom directive
const pin = {
  mounted() ,
  updated() 
}

// 
const vnode = withDirectives(h('div'), [
  [pin, 200, 'top', ]
])
```

If the directive is registered by name and cannot be imported directly, it can be resolved using the [`resolveDirective`](/api/render-function#resolvedirective) helper.

### Template Refs 

With the Composition API, when using [`useTemplateRef()`](/api/composition-api-helpers#usetemplateref)   template refs are created by passing the string value as prop to the vnode:

```js
import  from 'vue'

export default {
  setup() {
    const divEl = useTemplateRef('my-div')

    // 
    return () => h('div', )
  }
}
```

Usage before 3.5

In versions before 3.5 where useTemplateRef() was not introduced, template refs are created by passing the ref() itself as a prop to the vnode:

```js
import  from 'vue'

export default {
  setup() {
    const divEl = ref()

    // 
    return () => h('div', )
  }
}
```

With the Options API, template refs are created by passing the ref name as a string in the vnode props:

```js
export default {
  render() {
    // 
    return h('div', )
  }
}
```

## Functional Components 

Functional components are an alternative form of component that don't have any state of their own. They act like pure functions: props in, vnodes out. They are rendered without creating a component instance (i.e. no `this`), and without the usual component lifecycle hooks.

To create a functional component we use a plain function, rather than an options object. The function is effectively the `render` function for the component.

The signature of a functional component is the same as the `setup()` hook:

```js
function MyComponent(props, ) 
```

As there is no `this` reference for a functional component, Vue will pass in the `props` as the first argument:

```js
function MyComponent(props, context) 
```

The second argument, `context`, contains three properties: `attrs`, `emit`, and `slots`. These are equivalent to the instance properties [`$attrs`](/api/component-instance#attrs), [`$emit`](/api/component-instance#emit), and [`$slots`](/api/component-instance#slots) respectively.

Most of the usual configuration options for components are not available for functional components. However, it is possible to define [`props`](/api/options-state#props) and [`emits`](/api/options-state#emits) by adding them as properties:

```js
MyComponent.props = ['value']
MyComponent.emits = ['click']
```

If the `props` option is not specified, then the `props` object passed to the function will contain all attributes, the same as `attrs`. The prop names will not be normalized to camelCase unless the `props` option is specified.

For functional components with explicit `props`, [attribute fallthrough](/guide/components/attrs) works much the same as with normal components. However, for functional components that don't explicitly specify their `props`, only the `class`, `style`, and `onXxx` event listeners will be inherited from the `attrs` by default. In either case, `inheritAttrs` can be set to `false` to disable attribute inheritance:

```js
MyComponent.inheritAttrs = false
```

Functional components can be registered and consumed just like normal components. If you pass a function as the first argument to `h()`, it will be treated as a functional component.

### Typing Functional Components 

Functional Components can be typed based on whether they are named or anonymous. [Vue - Official extension](https://github.com/vuejs/language-tools) also supports type checking properly typed functional components when consuming them in SFC templates.

**Named Functional Component**

```tsx
import type  from 'vue'
type FComponentProps = 

type Events = 

function FComponent(
  props: FComponentProps,
  context: SetupContext
) >
         
    
  )
}

FComponent.props = {
  message: 
}

FComponent.emits = 
```

**Anonymous Functional Component**

```tsx
import type  from 'vue'

type FComponentProps = 

type Events = 

const FComponent: FunctionalComponent = (
  props,
  context
) => >
         
    
  )
}

FComponent.props = {
  message: 
}

FComponent.emits = 
```
