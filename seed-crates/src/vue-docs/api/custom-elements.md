# Custom Elements API 

## defineCustomElement() 

This method accepts the same argument as [`defineComponent`](#definecomponent), but instead returns a native [Custom Element](https://developer.mozilla.org/en-US/docs/Web/Web_Components/Using_custom_elements) class constructor.

- **Type**

  ```ts
  function defineCustomElement(
    component:
      | (ComponentOptions & CustomElementsOptions)
      | ComponentOptions['setup'],
    options?: CustomElementsOptions
  ): 

  interface CustomElementsOptions 
  ```

  > Type is simplified for readability.

- **Details**

  In addition to normal component options, `defineCustomElement()` also supports a number of options that are custom-elements-specific:

  - **`styles`**: an array of inlined CSS strings for providing CSS that should be injected into the element's shadow root.

  - **`configureApp`** : a function that can be used to configure the Vue app instance for the custom element.

  - **`shadowRoot`** : `boolean`, defaults to `true`. Set to `false` to render the custom element without a shadow root. This means `` in custom element SFCs will no longer be encapsulated.

  - **`nonce`** : `string`, if provided, will be set as the `nonce` attribute on style tags injected to the shadow root.

  Note that instead of being passed as part of the component itself, these options can also be passed via a second argument:

  ```js
  import Element from './MyElement.ce.vue'

  defineCustomElement(Element, {
    configureApp(app) 
  })
  ```

  The return value is a custom element constructor that can be registered using [`customElements.define()`](https://developer.mozilla.org/en-US/docs/Web/API/CustomElementRegistry/define).

- **Example**

  ```js
  import  from 'vue'

  const MyVueElement = defineCustomElement()

  // Register the custom element.
  customElements.define('my-vue-element', MyVueElement)
  ```

- **See also**

  - [Guide - Building Custom Elements with Vue](/guide/extras/web-components#building-custom-elements-with-vue)

  - Also note that `defineCustomElement()` requires [special config](/guide/extras/web-components#sfc-as-custom-element) when used with Single-File Components.

## useHost()  

A Composition API helper that returns the host element of the current Vue custom element.

## useShadowRoot()  

A Composition API helper that returns the shadow root of the current Vue custom element.

## this.$host  

An Options API property that exposes the host element of the current Vue custom element.
