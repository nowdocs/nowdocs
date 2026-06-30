# Built-in Special Elements 

:::info Not Components
``, `` and `` are component-like features and part of the template syntax. They are not true components and are compiled away during template compilation. As such, they are conventionally written with lowercase in templates.
:::

## `` 

A "meta component" for rendering dynamic components or elements.

- **Props**

  ```ts
  interface DynamicComponentProps 
  ```

- **Details**

  The actual component to render is determined by the `is` prop.

  - When `is` is a string, it could be either an HTML tag name or a component's registered name.

  - Alternatively, `is` can also be directly bound to the definition of a component.

- **Example**

  Rendering components by registered name (Options API):

  ```vue
  
  import Foo from './Foo.vue'
  import Bar from './Bar.vue'

  export default {
    components: ,
    data() {
      return 
    }
  }
  

  
    
  
  ```

  Rendering components by definition (Composition API with ``):

  ```vue
  
  import Foo from './Foo.vue'
  import Bar from './Bar.vue'
  

  
     0.5 ? Foo : Bar" />
  
  ```

  Rendering HTML elements:

  ```vue-html
  
  ```

  The [built-in components](./built-in-components) can all be passed to `is`, but you must register them if you want to pass them by name. For example:

  ```vue
  
  import  from 'vue'

  export default {
    components: 
  }
  

  
    
      ...
    
  
  ```

  Registration is not required if you pass the component itself to `is` rather than its name, e.g. in ``.

  If `v-model` is used on a `` tag, the template compiler will expand it to a `modelValue` prop and `update:modelValue` event listener, much like it would for any other component. However, this won't be compatible with native HTML elements, such as `` or ``. As a result, using `v-model` with a dynamically created native element won't work:

  ```vue
  
  import  from 'vue'

  const tag = ref('input')
  const username = ref('')
  

  
    
    
  
  ```

  In practice, this edge case isn't common as native form fields are typically wrapped in components in real applications. If you do need to use a native element directly then you can split the `v-model` into an attribute and event manually.

- **See also** [Dynamic Components](/guide/essentials/component-basics#dynamic-components)

## `` 

Denotes slot content outlets in templates.

- **Props**

  ```ts
  interface SlotProps 
  ```

- **Details**

  The `` element can use the `name` attribute to specify a slot name. When no `name` is specified, it will render the default slot. Additional attributes passed to the slot element will be passed as slot props to the scoped slot defined in the parent.

  The element itself will be replaced by its matched slot content.

  `` elements in Vue templates are compiled into JavaScript, so they are not to be confused with [native `` elements](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/slot).

- **See also** [Component - Slots](/guide/components/slots)

## `` 

The `` tag is used as a placeholder when we want to use a built-in directive without rendering an element in the DOM.

- **Details**

  The special handling for `` is only triggered if it is used with one of these directives:

  - `v-if`, `v-else-if`, or `v-else`
  - `v-for`
  - `v-slot`

  If none of those directives are present then it will be rendered as a [native `` element](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/template) instead.

  A `` with a `v-for` can also have a [`key` attribute](/api/built-in-special-attributes#key). All other attributes and directives will be discarded, as they aren't meaningful without a corresponding element.

  Single-file components use a [top-level `` tag](/api/sfc-spec#language-blocks) to wrap the entire template. That usage is separate from the use of `` described above. That top-level tag is not part of the template itself and doesn't support template syntax, such as directives.

- **See also**
  - [Guide - `v-if` on ``](/guide/essentials/conditional#v-if-on-template)
  - [Guide - `v-for` on ``](/guide/essentials/list#v-for-on-template)
  - [Guide - Named slots](/guide/components/slots#named-slots)
