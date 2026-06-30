# TypeScript with Composition API 

> This page assumes you've already read the overview on [Using Vue with TypeScript](./overview).

## Typing Component Props 

### Using `` 

When using ``, the `defineProps()` macro supports inferring the props types based on its argument:

```vue

const props = defineProps({
  foo: ,
  bar: Number
})

props.foo // string
props.bar // number | undefined

```

This is called "runtime declaration", because the argument passed to `defineProps()` will be used as the runtime `props` option.

However, it is usually more straightforward to define props with pure types via a generic type argument:

```vue

const props = defineProps<>()

```

This is called "type-based declaration". The compiler will try to do its best to infer the equivalent runtime options based on the type argument. In this case, our second example compiles into the exact same runtime options as the first example.

You can use either type-based declaration OR runtime declaration, but you cannot use both at the same time.

We can also move the props types into a separate interface:

```vue

interface Props 

const props = defineProps()

```

This also works if `Props` is imported from another file such as a relative import, a path alias (e.g., `@/types`), or an external dependency (e.g., `node_modules`). This feature requires TypeScript to be a peer dependency of Vue.

```vue

import type  from './foo'

const props = defineProps()

```

#### Syntax Limitations 

In version 3.2 and below, the generic type parameter for `defineProps()` were limited to a type literal or a reference to a local interface.

This limitation was resolved in 3.3. The latest version of Vue supports referencing imported and a limited set of complex types in the type parameter position. However, because the type to runtime conversion is still AST-based, some complex types that require actual type analysis, e.g. conditional types, are not supported. You can use conditional types for the type of a single prop, but not the entire props object.

### Props Default Values 

When using type-based declaration, we lose the ability to declare default values for the props. This can be resolved by using [Reactive Props Destructure](/guide/components/props#reactive-props-destructure) :

```ts
interface Props 

const  = defineProps()
```

In 3.4 and below, Reactive Props Destructure is not enabled by default. An alternative is to use the `withDefaults` compiler macro:

```ts
interface Props 

const props = withDefaults(defineProps(), )
```

This will be compiled to equivalent runtime props `default` options. In addition, the `withDefaults` helper provides type checks for the default values, and ensures the returned `props` type has the optional flags removed for properties that do have default values declared.

:::info
Note that default values for mutable reference types (like arrays or objects) should be wrapped in functions when using `withDefaults` to avoid accidental modification and external side effects. This ensures each component instance gets its own copy of the default value. This is **not** necessary when using default values with destructure.
:::

### Without `` 

If not using ``, it is necessary to use `defineComponent()` to enable props type inference. The type of the props object passed to `setup()` is inferred from the `props` option.

```ts
import  from 'vue'

export default defineComponent({
  props: ,
  setup(props) 
})
```

### Complex prop types 

With type-based declaration, a prop can use a complex type much like any other type:

```vue

interface Book 

const props = defineProps<>()

```

For runtime declaration, we can use the `PropType` utility type:

```ts
import type  from 'vue'

const props = defineProps()
```

This works in much the same way if we're specifying the `props` option directly:

```ts
import  from 'vue'
import type  from 'vue'

export default defineComponent({
  props: 
})
```

The `props` option is more commonly used with the Options API, so you'll find more detailed examples in the guide to [TypeScript with Options API](/guide/typescript/options-api#typing-component-props). The techniques shown in those examples also apply to runtime declarations using `defineProps()`.

## Typing Component Emits 

In ``, the `emit` function can also be typed using either runtime declaration OR type declaration:

```vue

// runtime
const emit = defineEmits(['change', 'update'])

// options based
const emit = defineEmits({
  change: (id: number) => ,
  update: (value: string) => 
})

// type-based
const emit = defineEmits<>()

// 3.3+: alternative, more succinct syntax
const emit = defineEmits<>()

```

The type argument can be one of the following:

1. A callable function type, but written as a type literal with [Call Signatures](https://www.typescriptlang.org/docs/handbook/2/functions.html#call-signatures). It will be used as the type of the returned `emit` function.
2. A type literal where the keys are the event names, and values are array / tuple types representing the additional accepted parameters for the event. The example above is using named tuples so each argument can have an explicit name.

As we can see, the type declaration gives us much finer-grained control over the type constraints of emitted events.

When not using ``, `defineComponent()` is able to infer the allowed events for the `emit` function exposed on the setup context:

```ts
import  from 'vue'

export default defineComponent({
  emits: ['change'],
  setup(props, ) 
})
```

## Typing `ref()` 

Refs infer the type from the initial value:

```ts
import  from 'vue'

// inferred type: Ref
const year = ref(2020)

// => TS Error: Type 'string' is not assignable to type 'number'.
year.value = '2020'
```

Sometimes we may need to specify complex types for a ref's inner value. We can do that by using the `Ref` type:

```ts
import  from 'vue'
import type  from 'vue'

const year: Ref = ref('2020')

year.value = 2020 // ok!
```

Or, by passing a generic argument when calling `ref()` to override the default inference:

```ts
// resulting type: Ref
const year = ref('2020')

year.value = 2020 // ok!
```

If you specify a generic type argument but omit the initial value, the resulting type will be a union type that includes `undefined`:

```ts
// inferred type: Ref
const n = ref()
```

## Typing `reactive()` 

`reactive()` also implicitly infers the type from its argument:

```ts
import  from 'vue'

// inferred type: 
const book = reactive()
```

To explicitly type a `reactive` property, we can use interfaces:

```ts
import  from 'vue'

interface Book 

const book: Book = reactive()
```

:::tip
It's not recommended to use the generic argument of `reactive()` because the returned type, which handles nested ref unwrapping, is different from the generic argument type.
:::

## Typing `computed()` 

`computed()` infers its type based on the getter's return value:

```ts
import  from 'vue'

const count = ref(0)

// inferred type: ComputedRef
const double = computed(() => count.value * 2)

// => TS Error: Property 'split' does not exist on type 'number'
const result = double.value.split('')
```

You can also specify an explicit type via a generic argument:

```ts
const double = computed(() => )
```

## Typing Event Handlers 

When dealing with native DOM events, it might be useful to type the argument we pass to the handler correctly. Let's take a look at this example:

```vue

function handleChange(event) 

  

```

Without type annotation, the `event` argument will implicitly have a type of `any`. This will also result in a TS error if `"strict": true` or `"noImplicitAny": true` are used in `tsconfig.json`. It is therefore recommended to explicitly annotate the argument of event handlers. In addition, you may need to use type assertions when accessing the properties of `event`:

```ts
function handleChange(event: Event) 
```

## Typing Provide / Inject 

Provide and inject are usually performed in separate components. To properly type injected values, Vue provides an `InjectionKey` interface, which is a generic type that extends `Symbol`. It can be used to sync the type of the injected value between the provider and the consumer:

```ts
import  from 'vue'
import type  from 'vue'

const key = Symbol() as InjectionKey

provide(key, 'foo') // providing non-string value will result in error

const foo = inject(key) // type of foo: string | undefined
```

It's recommended to place the injection key in a separate file so that it can be imported in multiple components.

When using string injection keys, the type of the injected value will be `unknown`, and needs to be explicitly declared via a generic type argument:

```ts
const foo = inject('foo') // type: string | undefined
```

Notice the injected value can still be `undefined`, because there is no guarantee that a provider will provide this value at runtime.

The `undefined` type can be removed by providing a default value:

```ts
const foo = inject('foo', 'bar') // type: string
```

If you are sure that the value is always provided, you can also force cast the value:

```ts
const foo = inject('foo') as string
```

## Typing Template Refs 

With Vue 3.5 and `@vue/language-tools` 2.1 (powering both the IDE language service and `vue-tsc`), the type of refs created by `useTemplateRef()` in SFCs can be **automatically inferred** for static refs based on what element the matching `ref` attribute is used on.

In cases where auto-inference is not possible, you can still cast the template ref to an explicit type via the generic argument:

```ts
const el = useTemplateRef('el')
```

Usage before 3.5

Template refs should be created with an explicit generic type argument and an initial value of `null`:

```vue

import  from 'vue'

const el = ref(null)

onMounted(() => )

  

```

To get the right DOM interface you can check pages like [MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input#technical_summary).

Note that for strict type safety, it is necessary to use optional chaining or type guards when accessing `el.value`. This is because the initial ref value is `null` until the component is mounted, and it can also be set to `null` if the referenced element is unmounted by `v-if`.

## Typing Component Template Refs 

With Vue 3.5 and `@vue/language-tools` 2.1 (powering both the IDE language service and `vue-tsc`), the type of refs created by `useTemplateRef()` in SFCs can be **automatically inferred** for static refs based on what element or component the matching `ref` attribute is used on.

In cases where auto-inference is not possible (e.g. non-SFC usage or dynamic components), you can still cast the template ref to an explicit type via the generic argument.

In order to get the instance type of an imported component, we need to first get its type via `typeof`, then use TypeScript's built-in `InstanceType` utility to extract its instance type:

```vue [App.vue]

import  from 'vue'
import Foo from './Foo.vue'
import Bar from './Bar.vue'

type FooType = InstanceType
type BarType = InstanceType

const compRef = useTemplateRef('comp')

   0.5 ? Foo : Bar" ref="comp" />

```

In cases where the exact type of the component isn't available or isn't important, `ComponentPublicInstance` can be used instead. This will only include properties that are shared by all components, such as `$el`:

```ts
import  from 'vue'
import type  from 'vue'

const child = useTemplateRef('child')
```

In cases where the component referenced is a [generic component](/guide/typescript/overview.html#generic-components), for instance `MyGenericModal`:

```vue [MyGenericModal.vue]

import  from 'vue'

const content = ref(null)

const open = (newContent: ContentType) => (content.value = newContent)

defineExpose()

```

It needs to be referenced using `ComponentExposed` from the [`vue-component-type-helpers`](https://www.npmjs.com/package/vue-component-type-helpers) library as `InstanceType` won't work.

```vue [App.vue]

import  from 'vue'
import MyGenericModal from './MyGenericModal.vue'
import type  from 'vue-component-type-helpers'

const modal =
  useTemplateRef>('modal')

const openModal = () => 

```

Note that with `@vue/language-tools` 2.1+, static template refs' types can be automatically inferred and the above is only needed in edge cases.

## Typing Global Custom Directives 

In order to get type hints and type checking for global custom directives declared with `app.directive()`, you can extend `GlobalDirectives`

```ts [src/directives/highlight.ts]
import type  from 'vue'

export type HighlightDirective = Directive

declare module 'vue' {
  export interface GlobalDirectives 
}

export default {
  mounted: (el, binding) => 
} satisfies HighlightDirective
```

```ts [main.ts]
import highlight from './directives/highlight'
// ...other code
const app = createApp(App)
app.directive('highlight', highlight)
```

Usage in component

```vue [App.vue]

  This sentence is important!

```
