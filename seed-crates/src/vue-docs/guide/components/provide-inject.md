# Provide / Inject 

> This page assumes you've already read the [Components Basics](/guide/essentials/component-basics). Read that first if you are new to components.

## Prop Drilling 

Usually, when we need to pass data from the parent to a child component, we use [props](/guide/components/props). However, imagine the case where we have a large component tree, and a deeply nested component needs something from a distant ancestor component. With only props, we would have to pass the same prop across the entire parent chain:

Notice although the `` component may not care about these props at all, it still needs to declare and pass them along just so `` can access them. If there is a longer parent chain, more components would be affected along the way. This is called "props drilling" and definitely isn't fun to deal with.

We can solve props drilling with `provide` and `inject`. A parent component can serve as a **dependency provider** for all its descendants. Any component in the descendant tree, regardless of how deep it is, can **inject** dependencies provided by components up in its parent chain.

## Provide 

To provide data to a component's descendants, use the [`provide()`](/api/composition-api-dependency-injection#provide) function:

```vue

import  from 'vue'

provide(/* key */ 'message', /* value */ 'hello!')

```

If not using ``, make sure `provide()` is called synchronously inside `setup()`:

```js
import  from 'vue'

export default {
  setup() 
}
```

The `provide()` function accepts two arguments. The first argument is called the **injection key**, which can be a string or a `Symbol`. The injection key is used by descendant components to lookup the desired value to inject. A single component can call `provide()` multiple times with different injection keys to provide different values.

The second argument is the provided value. The value can be of any type, including reactive state such as refs:

```js
import  from 'vue'

const count = ref(0)
provide('key', count)
```

Providing reactive values allows the descendant components using the provided value to establish a reactive connection to the provider component.

To provide data to a component's descendants, use the [`provide`](/api/options-composition#provide) option:

```js
export default {
  provide: 
}
```

For each property in the `provide` object, the key is used by child components to locate the correct value to inject, while the value is what ends up being injected.

If we need to provide per-instance state, for example data declared via the `data()`, then `provide` must use a function value:

```js
export default {
  data() {
    return 
  },
  provide() {
    // use function syntax so that we can access `this`
    return 
  }
}
```

However, do note this does **not** make the injection reactive. We will discuss [making injections reactive](#working-with-reactivity) below.

## App-level Provide 

In addition to providing data in a component, we can also provide at the app level:

```js
import  from 'vue'

const app = createApp()

app.provide(/* key */ 'message', /* value */ 'hello!')
```

App-level provides are available to all components rendered in the app. This is especially useful when writing [plugins](/guide/reusability/plugins), as plugins typically wouldn't be able to provide values using components.

## Inject 

To inject data provided by an ancestor component, use the [`inject()`](/api/composition-api-dependency-injection#inject) function:

```vue

import  from 'vue'

const message = inject('message')

```

If multiple parents provide data with the same key, inject will resolve to the value from the closest parent in component's parent chain.

If the provided value is a ref, it will be injected as-is and will **not** be automatically unwrapped. This allows the injector component to retain the reactivity connection to the provider component.

[Full provide + inject Example with Reactivity](https://play.vuejs.org/#eNqFUUFugzAQ/MrKF1IpxfeIVKp66Kk/8MWFDXYFtmUbpArx967BhURRU9/WOzO7MzuxV+fKcUB2YlWovXYRAsbBvQije2d9hAk8Xo7gvB11gzDDxdseCuIUG+ZN6a7JjZIvVRIlgDCcw+d3pmvTglz1okJ499I0C3qB1dJQT9YRooVaSdNiACWdQ5OICj2WwtTWhAg9hiBbhHNSOxQKu84WT8LkNQ9FBhTHXyg1K75aJHNUROxdJyNSBVBp44YI43NvG+zOgmWWYGt7dcipqPhGZEe2ef07wN3lltD+lWN6tNkV/37+rdKjK2rzhRTt7f3u41xhe37/xJZGAL2PLECXa9NKdD/a6QTTtGnP88LgiXJtYv4BaLHhvg==)

Again, if not using ``, `inject()` should only be called synchronously inside `setup()`:

```js
import  from 'vue'

export default {
  setup() {
    const message = inject('message')
    return 
  }
}
```

To inject data provided by an ancestor component, use the [`inject`](/api/options-composition#inject) option:

```js
export default {
  inject: ['message'],
  created() 
}
```

Injections are resolved **before** the component's own state, so you can access injected properties in `data()`:

```js
export default {
  inject: ['message'],
  data() {
    return 
  }
}
```

If multiple parents provide data with the same key, inject will resolve to the value from the closest parent in component's parent chain.

[Full provide + inject example](https://play.vuejs.org/#eNqNkcFqwzAQRH9l0EUthOhuRKH00FO/oO7B2JtERZaEvA4F43+vZCdOTAIJCImRdpi32kG8h7A99iQKobs6msBvpTNt8JHxcTC2wS76FnKrJpVLZelKR39TSUO7qreMoXRA7ZPPkeOuwHByj5v8EqI/moZeXudCIBL30Z0V0FLXVXsqIA9krU8R+XbMR9rS0mqhS4KpDbZiSgrQc5JKQqvlRWzEQnyvuc9YuWbd4eXq+TZn0IvzOeKr8FvsNcaK/R6Ocb9Uc4FvefpE+fMwP0wH8DU7wB77nIo6x6a2hvNEME5D0CpbrjnHf+8excI=)

### Injection Aliasing \* 

When using the array syntax for `inject`, the injected properties are exposed on the component instance using the same key. In the example above, the property was provided under the key `"message"`, and injected as `this.message`. The local key is the same as the injection key.

If we want to inject the property using a different local key, we need to use the object syntax for the `inject` option:

```js
export default {
  inject: {
    /* local key */ localMessage: 
  }
}
```

Here, the component will locate a property provided with the key `"message"`, and then expose it as `this.localMessage`.

### Injection Default Values 

By default, `inject` assumes that the injected key is provided somewhere in the parent chain. In the case where the key is not provided, there will be a runtime warning.

If we want to make an injected property work with optional providers, we need to declare a default value, similar to props:

```js
// `value` will be "default value"
// if no data matching "message" was provided
const value = inject('message', 'default value')
```

In some cases, the default value may need to be created by calling a function or instantiating a new class. To avoid unnecessary computation or side effects in case the optional value is not used, we can use a factory function for creating the default value:

```js
const value = inject('key', () => new ExpensiveClass(), true)
```

The third parameter indicates the default value should be treated as a factory function.

```js
export default {
  // object syntax is required
  // when declaring default values for injections
  inject: {
    message: ,
    user: {
      // use a factory function for non-primitive values that are expensive
      // to create, or ones that should be unique per component instance.
      default: () => ()
    }
  }
}
```

## Working with Reactivity 

When using reactive provide / inject values, **it is recommended to keep any mutations to reactive state inside of the _provider_ whenever possible**. This ensures that the provided state and its possible mutations are co-located in the same component, making it easier to maintain in the future.

There may be times when we need to update the data from an injector component. In such cases, we recommend providing a function that is responsible for mutating the state:

```vue

import  from 'vue'

const location = ref('North Pole')

function updateLocation() 

provide('location', )

```

```vue

import  from 'vue'

const  = inject('location')

  {}

```

Finally, you can wrap the provided value with [`readonly()`](/api/reactivity-core#readonly) if you want to ensure that the data passed through `provide` cannot be mutated by the injector component.

```vue

import  from 'vue'

const count = ref(0)
provide('read-only-count', readonly(count))

```

In order to make injections reactively linked to the provider, we need to provide a computed property using the [computed()](/api/reactivity-core#computed) function:

```js
import  from 'vue'

export default {
  data() {
    return 
  },
  provide() {
    return 
  }
}
```

[Full provide + inject Example with Reactivity](https://play.vuejs.org/#eNqNUctqwzAQ/JVFFyeQxnfjBEoPPfULqh6EtYlV9EKWTcH43ytZtmPTQA0CsdqZ2dlRT16tPXctkoKUTeWE9VeqhbLGeXirheRwc0ZBds7HKkKzBdBDZZRtPXIYJlzqU40/I4LjjbUyIKmGEWw0at8UgZrUh1PscObZ4ZhQAA596/RcAShsGnbHArIapTRBP74O8Up060wnOO5QmP0eAvZyBV+L5jw1j2tZqsMp8yWRUHhUVjKPoQIohQ460L0ow1FeKJlEKEnttFweijJfiORElhCf5f3umObb0B9PU/I7kk17PJj7FloN/2t7a2Pj/Zkdob+x8gV8ZlMs2de/8+14AXwkBngD9zgVqjg2rNXPvwjD+EdlHilrn8MvtvD1+Q==)

The `computed()` function is typically used in Composition API components, but can also be used to complement certain use cases in Options API. You can learn more about its usage by reading the [Reactivity Fundamentals](/guide/essentials/reactivity-fundamentals) and [Computed Properties](/guide/essentials/computed) with the API Preference set to Composition API.

## Working with Symbol Keys 

So far, we have been using string injection keys in the examples. If you are working in a large application with many dependency providers, or you are authoring components that are going to be used by other developers, it is best to use [Symbol](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Symbol) injection keys to avoid potential collisions.

It's recommended to export the Symbols in a dedicated file:

```js [keys.js]
export const myInjectionKey = Symbol()
```

```js
// in provider component
import  from 'vue'
import  from './keys.js'

provide(myInjectionKey, )
```

```js
// in injector component
import  from 'vue'
import  from './keys.js'

const injected = inject(myInjectionKey)
```

See also: [Typing Provide / Inject](/guide/typescript/composition-api#typing-provide-inject) 

```js
// in provider component
import  from './keys.js'

export default {
  provide() {
    return {
      [myInjectionKey]: 
    }
  }
}
```

```js
// in injector component
import  from './keys.js'

export default {
  inject: {
    injected: 
  }
}
```

