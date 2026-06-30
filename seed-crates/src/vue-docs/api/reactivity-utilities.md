# Reactivity API: Utilities 

## isRef() 

Checks if a value is a ref object.

- **Type**

  ```ts
  function isRef(r: Ref | unknown): r is Ref
  ```

  Note the return type is a [type predicate](https://www.typescriptlang.org/docs/handbook/2/narrowing.html#using-type-predicates), which means `isRef` can be used as a type guard:

  ```ts
  let foo: unknown
  if (isRef(foo)) 
  ```

## unref() 

Returns the inner value if the argument is a ref, otherwise return the argument itself. This is a sugar function for `val = isRef(val) ? val.value : val`.

- **Type**

  ```ts
  function unref(ref: T | Ref): T
  ```

- **Example**

  ```ts
  function useFoo(x: number | Ref) 
  ```

## toRef() 

Can be used to normalize values / refs / getters into refs (3.3+).

Can also be used to create a ref for a property on a source reactive object. The created ref is synced with its source property: mutating the source property will update the ref, and vice-versa.

- **Type**

  ```ts
  // normalization signature (3.3+)
  function toRef(
    value: T
  ): T extends () => infer R
    ? Readonly>
    : T extends Ref
    ? T
    : Ref>

  // object property signature
  function toRef(
    object: T,
    key: K,
    defaultValue?: T[K]
  ): ToRef

  type ToRef = T extends Ref ? T : Ref
  ```

- **Example**

  Normalization signature (3.3+):

  ```js
  // returns existing refs as-is
  toRef(existingRef)

  // creates a readonly ref that calls the getter on .value access
  toRef(() => props.foo)

  // creates normal refs from non-function values
  // equivalent to ref(1)
  toRef(1)
  ```

  Object property signature:

  ```js
  const state = reactive()

  // a two-way ref that syncs with the original property
  const fooRef = toRef(state, 'foo')

  // mutating the ref updates the original
  fooRef.value++
  console.log(state.foo) // 2

  // mutating the original also updates the ref
  state.foo++
  console.log(fooRef.value) // 3
  ```

  Note this is different from:

  ```js
  const fooRef = ref(state.foo)
  ```

  The above ref is **not** synced with `state.foo`, because the `ref()` receives a plain number value.

  `toRef()` is useful when you want to pass the ref of a prop to a composable function:

  ```vue
  
  import  from 'vue'

  const props = defineProps(/* ... */)

  // convert `props.foo` into a ref, then pass into
  // a composable
  useSomeFeature(toRef(props, 'foo'))

  // getter syntax - recommended in 3.3+
  useSomeFeature(toRef(() => props.foo))
  
  ```

  When `toRef` is used with component props, the usual restrictions around mutating the props still apply. Attempting to assign a new value to the ref is equivalent to trying to modify the prop directly and is not allowed. In that scenario you may want to consider using [`computed`](./reactivity-core#computed) with `get` and `set` instead. See the guide to [using `v-model` with components](/guide/components/v-model) for more information.

  When using the object property signature, `toRef()` will return a usable ref even if the source property doesn't currently exist. This makes it possible to work with optional properties, which wouldn't be picked up by [`toRefs`](#torefs).

## toValue() 

- Only supported in 3.3+

Normalizes values / refs / getters to values. This is similar to [unref()](#unref), except that it also normalizes getters. If the argument is a getter, it will be invoked and its return value will be returned.

This can be used in [Composables](/guide/reusability/composables.html) to normalize an argument that can be either a value, a ref, or a getter.

- **Type**

  ```ts
  function toValue(source: T | Ref | (() => T)): T
  ```

- **Example**

  ```js
  toValue(1) //       --> 1
  toValue(ref(1)) //  --> 1
  toValue(() => 1) // --> 1
  ```

  Normalizing arguments in composables:

  ```ts
  import type  from 'vue'

  function useFeature(id: MaybeRefOrGetter) {
    watch(() => toValue(id), id => )
  }

  // this composable supports any of the following:
  useFeature(1)
  useFeature(ref(1))
  useFeature(() => 1)
  ```

## toRefs() 

Converts a reactive object to a plain object where each property of the resulting object is a ref pointing to the corresponding property of the original object. Each individual ref is created using [`toRef()`](#toref).

- **Type**

  ```ts
  function toRefs(
    object: T
  ): 

  type ToRef = T extends Ref ? T : Ref
  ```

- **Example**

  ```js
  const state = reactive()

  const stateAsRefs = toRefs(state)
  /*
  Type of stateAsRefs: 
  */

  // The ref and the original property is "linked"
  state.foo++
  console.log(stateAsRefs.foo.value) // 2

  stateAsRefs.foo.value++
  console.log(state.foo) // 3
  ```

  `toRefs` is useful when returning a reactive object from a composable function so that the consuming component can destructure/spread the returned object without losing reactivity:

  ```js
  function useFeatureX() {
    const state = reactive()

    // ...logic operating on state

    // convert to refs when returning
    return toRefs(state)
  }

  // can destructure without losing reactivity
  const  = useFeatureX()
  ```

  `toRefs` will only generate refs for properties that are enumerable on the source object at call time. To create a ref for a property that may not exist yet, use [`toRef`](#toref) instead.

## isProxy() 

Checks if an object is a proxy created by [`reactive()`](./reactivity-core#reactive), [`readonly()`](./reactivity-core#readonly), [`shallowReactive()`](./reactivity-advanced#shallowreactive) or [`shallowReadonly()`](./reactivity-advanced#shallowreadonly).

- **Type**

  ```ts
  function isProxy(value: any): boolean
  ```

## isReactive() 

Checks if an object is a proxy created by [`reactive()`](./reactivity-core#reactive) or [`shallowReactive()`](./reactivity-advanced#shallowreactive).

- **Type**

  ```ts
  function isReactive(value: unknown): boolean
  ```

## isReadonly() 

Checks whether the passed value is a readonly object. The properties of a readonly object can change, but they can't be assigned directly via the passed object.

The proxies created by [`readonly()`](./reactivity-core#readonly) and [`shallowReadonly()`](./reactivity-advanced#shallowreadonly) are both considered readonly, as is a [`computed()`](./reactivity-core#computed) ref without a `set` function.

- **Type**

  ```ts
  function isReadonly(value: unknown): boolean
  ```
