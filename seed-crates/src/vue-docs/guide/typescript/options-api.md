# TypeScript with Options API 

> This page assumes you've already read the overview on [Using Vue with TypeScript](./overview).

:::tip
While Vue does support TypeScript usage with Options API, it is recommended to use Vue with TypeScript via Composition API as it offers simpler, more efficient and more robust type inference.
:::

## Typing Component Props 

Type inference for props in Options API requires wrapping the component with `defineComponent()`. With it, Vue is able to infer the types for the props based on the `props` option, taking additional options such as `required: true` and `default` into account:

```ts
import  from 'vue'

export default defineComponent({
  // type inference enabled
  props: {
    name: String,
    id: [Number, String],
    msg: ,
    metadata: null
  },
  mounted() 
})
```

However, the runtime `props` options only support using constructor functions as a prop's type - there is no way to specify complex types such as objects with nested properties or function call signatures.

To annotate complex props types, we can use the `PropType` utility type:

```ts
import  from 'vue'
import type  from 'vue'

interface Book 

export default defineComponent({
  props: {
    book: ,
    // can also annotate functions
    callback: Function as PropType<(id: number) => void>
  },
  mounted() 
})
```

### Caveats 

If your TypeScript version is less than `4.7`, you have to be careful when using function values for `validator` and `default` prop options - make sure to use arrow functions:

```ts
import  from 'vue'
import type  from 'vue'

interface Book 

export default defineComponent({
  props: {
    bookA: {
      type: Object as PropType,
      // Make sure to use arrow functions if your TypeScript version is less than 4.7
      default: () => (),
      validator: (book: Book) => !!book.title
    }
  }
})
```

This prevents TypeScript from having to infer the type of `this` inside these functions, which, unfortunately, can cause the type inference to fail. It was a previous [design limitation](https://github.com/microsoft/TypeScript/issues/38845), and now has been improved in [TypeScript 4.7](https://www.typescriptlang.org/docs/handbook/release-notes/typescript-4-7.html#improved-function-inference-in-objects-and-methods).

## Typing Component Emits 

We can declare the expected payload type for an emitted event using the object syntax of the `emits` option. Also, all non-declared emitted events will throw a type error when called:

```ts
import  from 'vue'

export default defineComponent({
  emits: {
    addBook(payload: ) 
  },
  methods: {
    onSubmit() {
      this.$emit('addBook', )

      this.$emit('non-declared-event') // Type error!
    }
  }
})
```

## Typing Computed Properties 

A computed property infers its type based on its return value:

```ts
import  from 'vue'

export default defineComponent({
  data() {
    return 
  },
  computed: {
    greeting() 
  },
  mounted() 
})
```

In some cases, you may want to explicitly annotate the type of a computed property to ensure its implementation is correct:

```ts
import  from 'vue'

export default defineComponent({
  data() {
    return 
  },
  computed: {
    // explicitly annotate return type
    greeting(): string ,

    // annotating a writable computed property
    greetingUppercased: {
      get(): string ,
      set(newValue: string) 
    }
  }
})
```

Explicit annotations may also be required in some edge cases where TypeScript fails to infer the type of a computed property due to circular inference loops.

## Typing Event Handlers 

When dealing with native DOM events, it might be useful to type the argument we pass to the handler correctly. Let's take a look at this example:

```vue

import  from 'vue'

export default defineComponent({
  methods: {
    handleChange(event) 
  }
})

  

```

Without type annotation, the `event` argument will implicitly have a type of `any`. This will also result in a TS error if `"strict": true` or `"noImplicitAny": true` are used in `tsconfig.json`. It is therefore recommended to explicitly annotate the argument of event handlers. In addition, you may need to use type assertions when accessing the properties of `event`:

```ts
import  from 'vue'

export default defineComponent({
  methods: {
    handleChange(event: Event) 
  }
})
```

## Augmenting Global Properties 

Some plugins install globally available properties to all component instances via [`app.config.globalProperties`](/api/application#app-config-globalproperties). For example, we may install `this.$http` for data-fetching or `this.$translate` for internationalization. To make this play well with TypeScript, Vue exposes a `ComponentCustomProperties` interface designed to be augmented via [TypeScript module augmentation](https://www.typescriptlang.org/docs/handbook/declaration-merging.html#module-augmentation):

```ts
import axios from 'axios'

declare module 'vue' {
  interface ComponentCustomProperties 
}
```

See also:

- [TypeScript unit tests for component type extensions](https://github.com/vuejs/core/blob/main/packages-private/dts-test/componentTypeExtensions.test-d.tsx)

### Type Augmentation Placement 

We can put this type augmentation in a `.ts` file, or in a project-wide `*.d.ts` file. Either way, make sure it is included in `tsconfig.json`. For library / plugin authors, this file should be specified in the `types` property in `package.json`.

In order to take advantage of module augmentation, you will need to ensure the augmentation is placed in a [TypeScript module](https://www.typescriptlang.org/docs/handbook/modules.html). That is to say, the file needs to contain at least one top-level `import` or `export`, even if it is just `export `. If the augmentation is placed outside of a module, it will overwrite the original types rather than augmenting them!

```ts
// Does not work, overwrites the original types.
declare module 'vue' {
  interface ComponentCustomProperties 
}
```

```ts
// Works correctly
export 

declare module 'vue' {
  interface ComponentCustomProperties 
}
```

## Augmenting Custom Options 

Some plugins, for example `vue-router`, provide support for custom component options such as `beforeRouteEnter`:

```ts
import  from 'vue'

export default defineComponent({
  beforeRouteEnter(to, from, next) 
})
```

Without proper type augmentation, the arguments of this hook will implicitly have `any` type. We can augment the `ComponentCustomOptions` interface to support these custom options:

```ts
import  from 'vue-router'

declare module 'vue' {
  interface ComponentCustomOptions 
}
```

Now the `beforeRouteEnter` option will be properly typed. Note this is just an example - well-typed libraries like `vue-router` should automatically perform these augmentations in their own type definitions.

The placement of this augmentation is subject to the [same restrictions](#type-augmentation-placement) as global property augmentations.

See also:

- [TypeScript unit tests for component type extensions](https://github.com/vuejs/core/blob/main/packages-private/dts-test/componentTypeExtensions.test-d.tsx)

## Typing Global Custom Directives 

See: [Typing Custom Global Directives](/guide/typescript/composition-api#typing-global-custom-directives) 
