# \ 

`` is a compile-time syntactic sugar for using Composition API inside Single-File Components (SFCs). It is the recommended syntax if you are using both SFCs and Composition API. It provides a number of advantages over the normal `` syntax:

- More succinct code with less boilerplate
- Ability to declare props and emitted events using pure TypeScript
- Better runtime performance (the template is compiled into a render function in the same scope, without an intermediate proxy)
- Better IDE type-inference performance (less work for the language server to extract types from code)

## Basic Syntax 

To opt-in to the syntax, add the `setup` attribute to the `` block:

```vue

console.log('hello script setup')

```

The code inside is compiled as the content of the component's `setup()` function. This means that unlike normal ``, which only executes once when the component is first imported, code inside `` will **execute every time an instance of the component is created**.

### Top-level bindings are exposed to template 

When using ``, any top-level bindings (including variables, function declarations, and imports) declared inside `` are directly usable in the template:

```vue

// variable
const msg = 'Hello!'

// functions
function log() 

  {}

```

Imports are exposed in the same fashion. This means you can directly use an imported helper function in template expressions without having to expose it via the `methods` option:

```vue

import  from './helpers'

  {}

```

## Reactivity 

Reactive state needs to be explicitly created using [Reactivity APIs](./reactivity-core). Similar to values returned from a `setup()` function, refs are automatically unwrapped when referenced in templates:

```vue

import  from 'vue'

const count = ref(0)

  {}

```

## Using Components 

Values in the scope of `` can also be used directly as custom component tag names:

```vue

import MyComponent from './MyComponent.vue'

  

```

:::

### Modifiers and Transformers 

To access modifiers used with the `v-model` directive, we can destructure the return value of `defineModel()` like this:

```js
const [modelValue, modelModifiers] = defineModel()

// corresponds to v-model.trim
if (modelModifiers.trim) 
```

When a modifier is present, we likely need to transform the value when reading or syncing it back to the parent. We can achieve this by using the `get` and `set` transformer options:

```js
const [modelValue, modelModifiers] = defineModel({
  // get() omitted as it is not needed here
  set(value) {
    // if the .trim modifier is used, return trimmed value
    if (modelModifiers.trim) 
    // otherwise, return the value as-is
    return value
  }
})
```

### Usage with TypeScript  

Like `defineProps` and `defineEmits`, `defineModel` can also receive type arguments to specify the types of the model value and the modifiers:

```ts
const modelValue = defineModel()
//    ^? Ref

// default model with options, required removes possible undefined values
const modelValue = defineModel()
//    ^? Ref

const [modelValue, modifiers] = defineModel()
//                 ^? Record<'trim' | 'uppercase', true | undefined>
```

## defineExpose() 

Components using `` are **closed by default** - i.e. the public instance of the component, which is retrieved via template refs or `$parent` chains, will **not** expose any of the bindings declared inside ``.

To explicitly expose properties in a `` component, use the `defineExpose` compiler macro:

```vue

import  from 'vue'

const a = 1
const b = ref(2)

defineExpose()

```

When a parent gets an instance of this component via template refs, the retrieved instance will be of the shape `` (refs are automatically unwrapped just like on normal instances).

## defineOptions() 

- Only supported in 3.3+

This macro can be used to declare component options directly inside `` without having to use a separate `` block:

```vue

defineOptions({
  inheritAttrs: false,
  customOptions: 
})

```

- This is a macro. The options will be hoisted to module scope and cannot access local variables in `` that are not literal constants.

## defineSlots() 

- Only supported in 3.3+

This macro can be used to provide type hints to IDEs for slot name and props type checking.

`defineSlots()` only accepts a type parameter and no runtime arguments. The type parameter should be a type literal where the property key is the slot name, and the value type is the slot function. The first argument of the function is the props the slot expects to receive, and its type will be used for slot props in the template. The return type is currently ignored and can be `any`, but we may leverage it for slot content checking in the future.

It also returns the `slots` object, which is equivalent to the `slots` object exposed on the setup context or returned by `useSlots()`.

```vue

const slots = defineSlots<{
  default(props: ): any
}>()

```

## `useSlots()` & `useAttrs()` 

Usage of `slots` and `attrs` inside `` should be relatively rare, since you can access them directly as `$slots` and `$attrs` in the template. In the rare case where you do need them, use the `useSlots` and `useAttrs` helpers respectively:

```vue

import  from 'vue'

const slots = useSlots()
const attrs = useAttrs()

```

`useSlots` and `useAttrs` are actual runtime functions that return the equivalent of `setupContext.slots` and `setupContext.attrs`. They can be used in normal composition API functions as well.

## Usage alongside normal `` 

`` can be used alongside normal ``. A normal `` may be needed in cases where we need to:

- Declare options that cannot be expressed in ``, for example `inheritAttrs` or custom options enabled via plugins (Can be replaced by [`defineOptions`](/api/sfc-script-setup#defineoptions) in 3.3+).
- Declaring named exports.
- Run side effects or create objects that should only execute once.

```vue

// normal , executed in module scope (only once)
runSideEffectOnce()

// declare additional options
export default {
  inheritAttrs: false,
  customOptions: 
}

// executed in setup() scope (for each instance)

```

Support for combining `` and `` in the same component is limited to the scenarios described above. Specifically:

- Do **NOT** use a separate `` section for options that can already be defined using ``, such as `props` and `emits`.
- Variables created inside `` are not added as properties to the component instance, making them inaccessible from the Options API. Mixing APIs in this way is strongly discouraged.

If you find yourself in one of the scenarios that is not supported then you should consider switching to an explicit [`setup()`](/api/composition-api-setup) function, instead of using ``.

## Top-level `await` 

Top-level `await` can be used inside ``. The resulting code will be compiled as `async setup()`:

```vue

const post = await fetch(`/api/post/1`).then((r) => r.json())

```

In addition, the awaited expression will be automatically compiled in a format that preserves the current component instance context after the `await`.

:::warning Note
`async setup()` must be used in combination with [`Suspense`](/guide/built-ins/suspense.html), which is currently still an experimental feature. We plan to finalize and document it in a future release - but if you are curious now, you can refer to its [tests](https://github.com/vuejs/core/blob/main/packages/runtime-core/__tests__/components/Suspense.spec.ts) to see how it works.
:::

## Import Statements 

Import statements in vue follow [ECMAScript module specification](https://nodejs.org/api/esm.html).
In addition, you can use aliases defined in your build tool configuration:

```vue

import  from 'vue'
import  from './Components'
import  from '@/Components'
import  from '~/Components'

```

## Generics  

Generic type parameters can be declared using the `generic` attribute on the `` tag:

```vue

defineProps<>()

```

The value of `generic` works exactly the same as the parameter list between `<...>` in TypeScript. For example, you can use multiple parameters, `extends` constraints, default types, and reference imported types:

```vue

import type  from './types'
defineProps<>()

```

You can use `@vue-generic` the directive to pass in explicit types, for when the type cannot be inferred:

```vue

  
  

  
  

```

In order to use a reference to a generic component in a `ref` you need to use the [`vue-component-type-helpers`](https://www.npmjs.com/package/vue-component-type-helpers) library as `InstanceType` won't work.

```vue

import componentWithoutGenerics from '../component-without-generics.vue';
import genericComponent from '../generic-component.vue';

import type  from 'vue-component-type-helpers';

// Works for a component without generics
ref>();

ref>();
```

## Restrictions 

- Due to the difference in module execution semantics, code inside `` relies on the context of an SFC. When moved into external `.js` or `.ts` files, it may lead to confusion for both developers and tools. Therefore, **``** cannot be used with the `src` attribute.
- `` does not support In-DOM Root Component Template.([Related Discussion](https://github.com/vuejs/core/issues/8391))
