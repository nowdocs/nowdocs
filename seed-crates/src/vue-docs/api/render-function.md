# Render Function APIs 

## h() 

Creates virtual DOM nodes (vnodes).

- **Type**

  ```ts
  // full signature
  function h(
    type: string | Component,
    props?: object | null,
    children?: Children | Slot | Slots
  ): VNode

  // omitting props
  function h(type: string | Component, children?: Children | Slot): VNode

  type Children = string | number | boolean | VNode | null | Children[]

  type Slot = () => Children

  type Slots = 
  ```

  > Types are simplified for readability.

- **Details**

  The first argument can either be a string (for native elements) or a Vue component definition. The second argument is the props to be passed, and the third argument is the children.

  When creating a component vnode, the children must be passed as slot functions. A single slot function can be passed if the component expects only the default slot. Otherwise, the slots must be passed as an object of slot functions.

  For convenience, the props argument can be omitted when the children is not a slots object.

- **Example**

  Creating native elements:

  ```js
  import  from 'vue'

  // all arguments except the type are optional
  h('div')
  h('div', )

  // both attributes and properties can be used in props
  // Vue automatically picks the right way to assign it
  h('div', )

  // class and style have the same object / array
  // value support like in templates
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

  Creating components:

  ```js
  import Foo from './Foo.vue'

  // passing props
  h(Foo, {
    // equivalent of some-prop="hello"
    someProp: 'hello',
    // equivalent of @update="() => "
    onUpdate: () => 
  })

  // passing single default slot
  h(Foo, () => 'default slot')

  // passing named slots
  // notice the `null` is required to avoid
  // slots object being treated as props
  h(MyComponent, null, )
  ```

- **See also** [Guide - Render Functions - Creating VNodes](/guide/extras/render-function#creating-vnodes)

## mergeProps() 

Merge multiple props objects with special handling for certain props.

- **Type**

  ```ts
  function mergeProps(...args: object[]): object
  ```

- **Details**

  `mergeProps()` supports merging multiple props objects with special handling for the following props:

  - `class`
  - `style`
  - `onXxx` event listeners - multiple listeners with the same name will be merged into an array.

  If you do not need the merge behavior and want simple overwrites, native object spread can be used instead.

- **Example**

  ```js
  import  from 'vue'

  const one = 

  const two = {
    class: ,
    onClick: handlerB
  }

  const merged = mergeProps(one, two)
  /**
   
   */
  ```

## cloneVNode() 

Clones a vnode.

- **Type**

  ```ts
  function cloneVNode(vnode: VNode, extraProps?: object): VNode
  ```

- **Details**

  Returns a cloned vnode, optionally with extra props to merge with the original.

  Vnodes should be considered immutable once created, and you should not mutate the props of an existing vnode. Instead, clone it with different / extra props.

  Vnodes have special internal properties, so cloning them is not as simple as an object spread. `cloneVNode()` handles most of the internal logic.

- **Example**

  ```js
  import  from 'vue'

  const original = h('div')
  const cloned = cloneVNode(original, )
  ```

## isVNode() 

Checks if a value is a vnode.

- **Type**

  ```ts
  function isVNode(value: unknown): boolean
  ```

## resolveComponent() 

For manually resolving a registered component by name.

- **Type**

  ```ts
  function resolveComponent(name: string): Component | string
  ```

- **Details**

  **Note: you do not need this if you can import the component directly.**

  `resolveComponent()` must be called inside either `setup()` or the render function in order to resolve from the correct component context.

  If the component is not found, a runtime warning will be emitted, and the name string is returned.

- **Example**

  

  ```js
  import  from 'vue'

  export default {
    setup() {
      const ButtonCounter = resolveComponent('ButtonCounter')

      return () => 
    }
  }
  ```

  
  

  ```js
  import  from 'vue'

  export default {
    render() 
  }
  ```

  

- **See also** [Guide - Render Functions - Components](/guide/extras/render-function#components)

## resolveDirective() 

For manually resolving a registered directive by name.

- **Type**

  ```ts
  function resolveDirective(name: string): Directive | undefined
  ```

- **Details**

  **Note: you do not need this if you can import the directive directly.**

  `resolveDirective()` must be called inside either `setup()` or the render function in order to resolve from the correct component context.

  If the directive is not found, a runtime warning will be emitted, and the function returns `undefined`.

- **See also** [Guide - Render Functions - Custom Directives](/guide/extras/render-function#custom-directives)

## withDirectives() 

For adding custom directives to vnodes.

- **Type**

  ```ts
  function withDirectives(
    vnode: VNode,
    directives: DirectiveArguments
  ): VNode

  // [Directive, value, argument, modifiers]
  type DirectiveArguments = Array<
    | [Directive]
    | [Directive, any]
    | [Directive, any, string]
    | [Directive, any, string, DirectiveModifiers]
  >
  ```

- **Details**

  Wraps an existing vnode with custom directives. The second argument is an array of custom directives. Each custom directive is also represented as an array in the form of `[Directive, value, argument, modifiers]`. Tailing elements of the array can be omitted if not needed.

- **Example**

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

- **See also** [Guide - Render Functions - Custom Directives](/guide/extras/render-function#custom-directives)

## withModifiers() 

For adding built-in [`v-on` modifiers](/guide/essentials/event-handling#event-modifiers) to an event handler function.

- **Type**

  ```ts
  function withModifiers(fn: Function, modifiers: ModifierGuardsKeys[]): Function
  ```

- **Example**

  ```js
  import  from 'vue'

  const vnode = h('button', {
    // equivalent of v-on:click.stop.prevent
    onClick: withModifiers(() => , ['stop', 'prevent'])
  })
  ```

- **See also** [Guide - Render Functions - Event Modifiers](/guide/extras/render-function#event-modifiers)
