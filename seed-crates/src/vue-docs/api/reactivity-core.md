# Reactivity API: Core 

:::info See also
To better understand the Reactivity APIs, it is recommended to read the following chapters in the guide:

- [Reactivity Fundamentals](/guide/essentials/reactivity-fundamentals) (with the API preference set to Composition API)
- [Reactivity in Depth](/guide/extras/reactivity-in-depth)
  :::

## ref() 

Takes an inner value and returns a reactive and mutable ref object, which has a single property `.value` that points to the inner value.

- **Type**

  ```ts
  function ref(value: T): Ref>

  interface Ref 
  ```

- **Details**

  The ref object is mutable - i.e. you can assign new values to `.value`. It is also reactive - i.e. any read operations to `.value` are tracked, and write operations will trigger associated effects.

  If an object is assigned as a ref's value, the object is made deeply reactive with [reactive()](#reactive). This also means if the object contains nested refs, they will be deeply unwrapped.

  To avoid the deep conversion, use [`shallowRef()`](./reactivity-advanced#shallowref) instead.

- **Example**

  ```js
  const count = ref(0)
  console.log(count.value) // 0

  count.value = 1
  console.log(count.value) // 1
  ```

- **See also**
  - [Guide - Reactivity Fundamentals with `ref()`](/guide/essentials/reactivity-fundamentals#ref)
  - [Guide - Typing `ref()`](/guide/typescript/composition-api#typing-ref) 

## computed() 

Takes a [getter function](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions/get#description) and returns a readonly reactive [ref](#ref) object for the returned value from the getter. It can also take an object with `get` and `set` functions to create a writable ref object.

- **Type**

  ```ts
  // read-only
  function computed(
    getter: (oldValue: T | undefined) => T,
    // see "Computed Debugging" link below
    debuggerOptions?: DebuggerOptions
  ): Readonly>>

  // writable
  function computed(
    options: ,
    debuggerOptions?: DebuggerOptions
  ): Ref
  ```

- **Example**

  Creating a readonly computed ref:

  ```js
  const count = ref(1)
  const plusOne = computed(() => count.value + 1)

  console.log(plusOne.value) // 2

  plusOne.value++ // error
  ```

  Creating a writable computed ref:

  ```js
  const count = ref(1)
  const plusOne = computed({
    get: () => count.value + 1,
    set: (val) => 
  })

  plusOne.value = 1
  console.log(count.value) // 0
  ```

  Debugging:

  ```js
  const plusOne = computed(() => count.value + 1, {
    onTrack(e) ,
    onTrigger(e) 
  })
  ```

- **See also**
  - [Guide - Computed Properties](/guide/essentials/computed)
  - [Guide - Computed Debugging](/guide/extras/reactivity-in-depth#computed-debugging)
  - [Guide - Typing `computed()`](/guide/typescript/composition-api#typing-computed) 
  - [Guide - Performance - Computed Stability](/guide/best-practices/performance#computed-stability)

## reactive() 

Returns a reactive proxy of the object.

- **Type**

  ```ts
  function reactive(target: T): UnwrapNestedRefs
  ```

- **Details**

  The reactive conversion is "deep": it affects all nested properties. A reactive object also deeply unwraps any properties that are [refs](#ref) while maintaining reactivity.

  It should also be noted that there is no ref unwrapping performed when the ref is accessed as an element of a reactive array or a native collection type like `Map`.

  To avoid the deep conversion and only retain reactivity at the root level, use [shallowReactive()](./reactivity-advanced#shallowreactive) instead.

  The returned object and its nested objects are wrapped with [ES Proxy](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Proxy) and **not** equal to the original objects. It is recommended to work exclusively with the reactive proxy and avoid relying on the original object.

- **Example**

  Creating a reactive object:

  ```js
  const obj = reactive()
  obj.count++
  ```

  Ref unwrapping:

  ```ts
  const count = ref(1)
  const obj = reactive()

  // ref will be unwrapped
  console.log(obj.count === count.value) // true

  // it will update `obj.count`
  count.value++
  console.log(count.value) // 2
  console.log(obj.count) // 2

  // it will also update `count` ref
  obj.count++
  console.log(obj.count) // 3
  console.log(count.value) // 3
  ```

  Note that refs are **not** unwrapped when accessed as array or collection elements:

  ```js
  const books = reactive([ref('Vue 3 Guide')])
  // need .value here
  console.log(books[0].value)

  const map = reactive(new Map([['count', ref(0)]]))
  // need .value here
  console.log(map.get('count').value)
  ```

  When assigning a [ref](#ref) to a `reactive` property, that ref will also be automatically unwrapped:

  ```ts
  const count = ref(1)
  const obj = reactive()

  obj.count = count

  console.log(obj.count) // 1
  console.log(obj.count === count.value) // true
  ```

- **See also**
  - [Guide - Reactivity Fundamentals](/guide/essentials/reactivity-fundamentals)
  - [Guide - Typing `reactive()`](/guide/typescript/composition-api#typing-reactive) 

## readonly() 

Takes an object (reactive or plain) or a [ref](#ref) and returns a readonly proxy to the original.

- **Type**

  ```ts
  function readonly(
    target: T
  ): DeepReadonly>
  ```

- **Details**

  A readonly proxy is deep: any nested property accessed will be readonly as well. It also has the same ref-unwrapping behavior as `reactive()`, except the unwrapped values will also be made readonly.

  To avoid the deep conversion, use [shallowReadonly()](./reactivity-advanced#shallowreadonly) instead.

- **Example**

  ```js
  const original = reactive()

  const copy = readonly(original)

  watchEffect(() => )

  // mutating original will trigger watchers relying on the copy
  original.count++

  // mutating the copy will fail and result in a warning
  copy.count++ // warning!
  ```

## watchEffect() 

Runs a function immediately while reactively tracking its dependencies and re-runs it whenever the dependencies are changed.

- **Type**

  ```ts
  function watchEffect(
    effect: (onCleanup: OnCleanup) => void,
    options?: WatchEffectOptions
  ): WatchHandle

  type OnCleanup = (cleanupFn: () => void) => void

  interface WatchEffectOptions 

  interface WatchHandle 
  ```

- **Details**

  The first argument is the effect function to be run. The effect function receives a function that can be used to register a cleanup callback. The cleanup callback will be called right before the next time the effect is re-run, and can be used to clean up invalidated side effects, e.g. a pending async request (see example below).

  The second argument is an optional options object that can be used to adjust the effect's flush timing or to debug the effect's dependencies.

  By default, watchers will run just prior to component rendering. Setting `flush: 'post'` will defer the watcher until after component rendering. See [Callback Flush Timing](/guide/essentials/watchers#callback-flush-timing) for more information. In rare cases, it might be necessary to trigger a watcher immediately when a reactive dependency changes, e.g. to invalidate a cache. This can be achieved using `flush: 'sync'`. However, this setting should be used with caution, as it can lead to problems with performance and data consistency if multiple properties are being updated at the same time.

  The return value is a handle function that can be called to stop the effect from running again.

- **Example**

  ```js
  const count = ref(0)

  watchEffect(() => console.log(count.value))
  // -> logs 0

  count.value++
  // -> logs 1
  ```

  Stopping the watcher:

  ```js
  const stop = watchEffect(() => )

  // when the watcher is no longer needed:
  stop()
  ```

  Pausing / resuming the watcher: 

  ```js
  const  = watchEffect(() => )

  // temporarily pause the watcher
  pause()

  // resume later
  resume()

  // stop
  stop()
  ```

  Side effect cleanup:

  ```js
  watchEffect(async (onCleanup) => {
    const  = doAsyncWork(newId)
    // `cancel` will be called if `id` changes, cancelling
    // the previous request if it hasn't completed yet
    onCleanup(cancel)
    data.value = await response
  })
  ```

  Side effect cleanup in 3.5+:

  ```js
  import  from 'vue'

  watchEffect(async () => {
    const  = doAsyncWork(newId)
    // `cancel` will be called if `id` changes, cancelling
    // the previous request if it hasn't completed yet
    onWatcherCleanup(cancel)
    data.value = await response
  })
  ```

  Options:

  ```js
  watchEffect(() => , {
    flush: 'post',
    onTrack(e) ,
    onTrigger(e) 
  })
  ```

- **See also**
  - [Guide - Watchers](/guide/essentials/watchers#watcheffect)
  - [Guide - Watcher Debugging](/guide/extras/reactivity-in-depth#watcher-debugging)

## watchPostEffect() 

Alias of [`watchEffect()`](#watcheffect) with `flush: 'post'` option.

## watchSyncEffect() 

Alias of [`watchEffect()`](#watcheffect) with `flush: 'sync'` option.

## watch() 

Watches one or more reactive data sources and invokes a callback function when the sources change.

- **Type**

  ```ts
  // watching single source
  function watch(
    source: WatchSource,
    callback: WatchCallback,
    options?: WatchOptions
  ): WatchHandle

  // watching multiple sources
  function watch(
    sources: WatchSource[],
    callback: WatchCallback,
    options?: WatchOptions
  ): WatchHandle

  type WatchCallback = (
    value: T,
    oldValue: T,
    onCleanup: (cleanupFn: () => void) => void
  ) => void

  type WatchSource =
    | Ref // ref
    | (() => T) // getter
    | (T extends object ? T : never) // reactive object

  interface WatchOptions extends WatchEffectOptions 

  interface WatchHandle 
  ```

  > Types are simplified for readability.

- **Details**

  `watch()` is lazy by default - i.e. the callback is only called when the watched source has changed.

  The first argument is the watcher's **source**. The source can be one of the following:

  - A getter function that returns a value
  - A ref
  - A reactive object
  - ...or an array of the above.

  The second argument is the callback that will be called when the source changes. The callback receives three arguments: the new value, the old value, and a function for registering a side effect cleanup callback. The cleanup callback will be called right before the next time the effect is re-run, and can be used to clean up invalidated side effects, e.g. a pending async request.

  When watching multiple sources, the callback receives two arrays containing new / old values corresponding to the source array.

  The third optional argument is an options object that supports the following options:

  - **`immediate`**: trigger the callback immediately on watcher creation. Old value will be `undefined` on the first call.
  - **`deep`**: force deep traversal of the source if it is an object, so that the callback fires on deep mutations. In 3.5+, this can also be a number indicating the max traversal depth. See [Deep Watchers](/guide/essentials/watchers#deep-watchers).
  - **`flush`**: adjust the callback's flush timing. See [Callback Flush Timing](/guide/essentials/watchers#callback-flush-timing) and [`watchEffect()`](/api/reactivity-core#watcheffect).
  - **`onTrack / onTrigger`**: debug the watcher's dependencies. See [Watcher Debugging](/guide/extras/reactivity-in-depth#watcher-debugging).
  - **`once`**: (3.4+) run the callback only once. The watcher is automatically stopped after the first callback run.

  Compared to [`watchEffect()`](#watcheffect), `watch()` allows us to:

  - Perform the side effect lazily;
  - Be more specific about what state should trigger the watcher to re-run;
  - Access both the previous and current value of the watched state.

- **Example**

  Watching a getter:

  ```js
  const state = reactive()
  watch(
    () => state.count,
    (count, prevCount) => 
  )
  ```

  Watching a ref:

  ```js
  const count = ref(0)
  watch(count, (count, prevCount) => )
  ```

  When watching multiple sources, the callback receives arrays containing new / old values corresponding to the source array:

  ```js
  watch([fooRef, barRef], ([foo, bar], [prevFoo, prevBar]) => )
  ```

  When using a getter source, the watcher only fires if the getter's return value has changed. If you want the callback to fire even on deep mutations, you need to explicitly force the watcher into deep mode with ``. Note in deep mode, the new value and the old will be the same object if the callback was triggered by a deep mutation:

  ```js
  const state = reactive()
  watch(
    () => state,
    (newValue, oldValue) => ,
    
  )
  ```

  When directly watching a reactive object, the watcher is automatically in deep mode:

  ```js
  const state = reactive()
  watch(state, () => )
  ```

  `watch()` shares the same flush timing and debugging options with [`watchEffect()`](#watcheffect):

  ```js
  watch(source, callback, {
    flush: 'post',
    onTrack(e) ,
    onTrigger(e) 
  })
  ```

  Stopping the watcher:

  ```js
  const stop = watch(source, callback)

  // when the watcher is no longer needed:
  stop()
  ```

  Pausing / resuming the watcher: 

  ```js
  const  = watch(() => )

  // temporarily pause the watcher
  pause()

  // resume later
  resume()

  // stop
  stop()
  ```

  Side effect cleanup:

  ```js
  watch(id, async (newId, oldId, onCleanup) => {
    const  = doAsyncWork(newId)
    // `cancel` will be called if `id` changes, cancelling
    // the previous request if it hasn't completed yet
    onCleanup(cancel)
    data.value = await response
  })
  ```

  Side effect cleanup in 3.5+:

  ```js
  import  from 'vue'

  watch(id, async (newId) => {
    const  = doAsyncWork(newId)
    onWatcherCleanup(cancel)
    data.value = await response
  })
  ```

- **See also**

  - [Guide - Watchers](/guide/essentials/watchers)
  - [Guide - Watcher Debugging](/guide/extras/reactivity-in-depth#watcher-debugging)

## onWatcherCleanup()  

Register a cleanup function to be executed when the current watcher is about to re-run. Can only be called during the synchronous execution of a `watchEffect` effect function or `watch` callback function (i.e. it cannot be called after an `await` statement in an async function.)

- **Type**

  ```ts
  function onWatcherCleanup(
    cleanupFn: () => void,
    failSilently?: boolean
  ): void
  ```

- **Example**

  ```ts
  import  from 'vue'

  watch(id, (newId) => {
    const  = doAsyncWork(newId)
    // `cancel` will be called if `id` changes, cancelling
    // the previous request if it hasn't completed yet
    onWatcherCleanup(cancel)
  })
  ```
