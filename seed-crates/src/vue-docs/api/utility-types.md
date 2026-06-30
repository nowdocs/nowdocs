# Utility Types 

:::info
This page only lists a few commonly used utility types that may need explanation for their usage. For a full list of exported types, consult the [source code](https://github.com/vuejs/core/blob/main/packages/runtime-core/src/index.ts#L131).
:::

## PropType\ 

Used to annotate a prop with more advanced types when using runtime props declarations.

- **Example**

  ```ts
  import type  from 'vue'

  interface Book 

  export default {
    props: {
      book: 
    }
  }
  ```

- **See also** [Guide - Typing Component Props](/guide/typescript/options-api#typing-component-props)

## MaybeRef\ 

- Only supported in 3.3+

Alias for `T | Ref`. Useful for annotating arguments of [Composables](/guide/reusability/composables.html).

## MaybeRefOrGetter\ 

- Only supported in 3.3+

Alias for `T | Ref | (() => T)`. Useful for annotating arguments of [Composables](/guide/reusability/composables.html).

## ExtractPropTypes\ 

Extract prop types from a runtime props options object. The extracted types are internal facing - i.e. the resolved props received by the component. This means boolean props and props with default values are always defined, even if they are not required.

To extract public facing props, i.e. props that the parent is allowed to pass, use [`ExtractPublicPropTypes`](#extractpublicproptypes).

- **Example**

  ```ts
  const propsOptions = {
    foo: String,
    bar: Boolean,
    baz: ,
    qux: 
  } as const

  type Props = ExtractPropTypes
  // 
  ```

## ExtractPublicPropTypes\ 

- Only supported in 3.3+

Extract prop types from a runtime props options object. The extracted types are public facing - i.e. the props that the parent is allowed to pass.

- **Example**

  ```ts
  const propsOptions = {
    foo: String,
    bar: Boolean,
    baz: ,
    qux: 
  } as const

  type Props = ExtractPublicPropTypes
  // 
  ```

## ComponentCustomProperties 

Used to augment the component instance type to support custom global properties.

- **Example**

  ```ts
  import axios from 'axios'

  declare module 'vue' {
    interface ComponentCustomProperties 
  }
  ```

  :::tip
  Augmentations must be placed in a module `.ts` or `.d.ts` file. See [Type Augmentation Placement](/guide/typescript/options-api#augmenting-global-properties) for more details.
  :::

- **See also** [Guide - Augmenting Global Properties](/guide/typescript/options-api#augmenting-global-properties)

## ComponentCustomOptions 

Used to augment the component options type to support custom options.

- **Example**

  ```ts
  import  from 'vue-router'

  declare module 'vue' {
    interface ComponentCustomOptions 
  }
  ```

  :::tip
  Augmentations must be placed in a module `.ts` or `.d.ts` file. See [Type Augmentation Placement](/guide/typescript/options-api#augmenting-global-properties) for more details.
  :::

- **See also** [Guide - Augmenting Custom Options](/guide/typescript/options-api#augmenting-custom-options)

## ComponentCustomProps 

Used to augment allowed TSX props in order to use non-declared props on TSX elements.

- **Example**

  ```ts
  declare module 'vue' {
    interface ComponentCustomProps 
  }

  export 
  ```

  ```tsx
  // now works even if hello is not a declared prop
  
  ```

  :::tip
  Augmentations must be placed in a module `.ts` or `.d.ts` file. See [Type Augmentation Placement](/guide/typescript/options-api#augmenting-global-properties) for more details.
  :::

## CSSProperties 

Used to augment allowed values in style property bindings.

- **Example**

  Allow any custom CSS property

  ```ts
  declare module 'vue' {
    interface CSSProperties {
      [key: `--$`]: string
    }
  }
  ```

  ```tsx
  
  ```

  ```html
  
  ```

:::tip
Augmentations must be placed in a module `.ts` or `.d.ts` file. See [Type Augmentation Placement](/guide/typescript/options-api#augmenting-global-properties) for more details.
:::

:::info See also
SFC `` tags support linking CSS values to dynamic component state using the `v-bind` CSS function. This allows for custom properties without type augmentation.

- [v-bind() in CSS](/api/sfc-css-features#v-bind-in-css)
  :::
