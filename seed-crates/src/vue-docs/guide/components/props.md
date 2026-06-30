# Props 

> This page assumes you've already read the [Components Basics](/guide/essentials/component-basics). Read that first if you are new to components.

  

## Props Declaration 

Vue components require explicit props declaration so that Vue knows what external props passed to the component should be treated as fallthrough attributes (which will be discussed in [its dedicated section](/guide/components/attrs)).

In SFCs using ``, props can be declared using the `defineProps()` macro:

```vue

const props = defineProps(['foo'])

console.log(props.foo)

```

In non-`` components, props are declared using the [`props`](/api/options-state#props) option:

```js
export default {
  props: ['foo'],
  setup(props) 
}
```

Notice the argument passed to `defineProps()` is the same as the value provided to the `props` options: the same props options API is shared between the two declaration styles.

Props are declared using the [`props`](/api/options-state#props) option:

```js
export default {
  props: ['foo'],
  created() 
}
```

In addition to declaring props using an array of strings, we can also use the object syntax:

```js
export default {
  props: 
}
```

```js
// in 
defineProps()
```

```js
// in non-
export default {
  props: 
}
```

For each property in the object declaration syntax, the key is the name of the prop, while the value should be the constructor function of the expected type.

This not only documents your component, but will also warn other developers using your component in the browser console if they pass the wrong type. We will discuss more details about [prop validation](#prop-validation) further down this page.

See also: [Typing Component Props](/guide/typescript/options-api#typing-component-props) 

If you are using TypeScript with ``, it's also possible to declare props using pure type annotations:

```vue

defineProps<>()

```

More details: [Typing Component Props](/guide/typescript/composition-api#typing-component-props) 

## Reactive Props Destructure  \*\* 

Vue's reactivity system tracks state usage based on property access. E.g. when you access `props.foo` in a computed getter or a watcher, the `foo` prop gets tracked as a dependency.

So, given the following code:

```js
const  = defineProps(['foo'])

watchEffect(() => )
```

In version 3.4 and below, `foo` is an actual constant and will never change. In version 3.5 and above, Vue's compiler automatically prepends `props.` when code in the same `` block accesses variables destructured from `defineProps`. Therefore the code above becomes equivalent to the following:

```js 
const props = defineProps(['foo'])

watchEffect(() => )
```

In addition, you can use JavaScript's native default value syntax to declare default values for the props. This is particularly useful when using the type-based props declaration:

```ts
const  = defineProps<>()
```

If you prefer to have more visual distinction between destructured props and normal variables in your IDE, Vue's VSCode extension provides a setting to enable inlay-hints for destructured props.

### Passing Destructured Props into Functions 

When we pass a destructured prop into a function, e.g.:

```js
const  = defineProps(['foo'])

watch(foo, /* ... */)
```

This will not work as expected because it is equivalent to `watch(props.foo, ...)` - we are passing a value instead of a reactive data source to `watch`. In fact, Vue's compiler will catch such cases and throw a warning.

Similar to how we can watch a normal prop with `watch(() => props.foo, ...)`, we can watch a destructured prop also by wrapping it in a getter:

```js
watch(() => foo, /* ... */)
```

In addition, this is the recommended approach when we need to pass a destructured prop into an external function while retaining reactivity:

```js
useComposable(() => foo)
```

The external function can call the getter (or normalize it with [toValue](/api/reactivity-utilities.html#tovalue)) when it needs to track changes of the provided prop, e.g. in a computed or watcher getter.

## Prop Passing Details 

### Prop Name Casing 

We declare long prop names using camelCase because this avoids having to use quotes when using them as property keys, and allows us to reference them directly in template expressions because they are valid JavaScript identifiers:

```js
defineProps()
```

```js
export default {
  props: 
}
```

```vue-html
{}
```

Technically, you can also use camelCase when passing props to a child component (except in [in-DOM templates](/guide/essentials/component-basics#in-dom-template-parsing-caveats)). However, the convention is using kebab-case in all cases to align with HTML attributes:

```vue-html

```

We use [PascalCase for component tags](/guide/components/registration#component-name-casing) when possible because it improves template readability by differentiating Vue components from native elements. However, there isn't as much practical benefit in using camelCase when passing props, so we choose to follow each language's conventions.

### Static vs. Dynamic Props 

So far, you've seen props passed as static values, like in:

```vue-html

```

You've also seen props assigned dynamically with `v-bind` or its `:` shortcut, such as in:

```vue-html

```

### Passing Different Value Types 

In the two examples above, we happen to pass string values, but _any_ type of value can be passed to a prop.

#### Number 

```vue-html

```

#### Boolean 

```vue-html

```

#### Array 

```vue-html

```

#### Object 

```vue-html

```

### Binding Multiple Properties Using an Object 

If you want to pass all the properties of an object as props, you can use [`v-bind` without an argument](/guide/essentials/template-syntax#dynamically-binding-multiple-attributes) (`v-bind` instead of `:prop-name`). For example, given a `post` object:

```js
export default {
  data() {
    return {
      post: 
    }
  }
}
```

```js
const post = 
```

The following template:

```vue-html

```

Will be equivalent to:

```vue-html

```

### Merge Behavior When Combining Bindings 

When `v-bind` is used alongside explicit bindings on the same component, Vue internally calls `mergeProps()` to combine them. The merging strategy depends on the key type:

- **Regular props** — the last value wins:

```vue-html

```

- **Event listeners** — when passing listeners in a `v-bind` object, [use the `onEventName` key convention](/guide/extras/render-function#v-on). All handlers for the same event will be called (see [`v-on` Listener Inheritance](/guide/components/attrs#v-on-listener-inheritance)):

```vue-html

 console.log(2) }" />
```

- **`class` and `style`** follow a similar merge strategy (see [`class` and `style` Merging](/guide/components/attrs#class-and-style-merging)).

:::tip
The full merging rules are described in the [`mergeProps()`](/api/render-function#mergeprops) API reference.
:::

## One-Way Data Flow 

All props form a **one-way-down binding** between the child property and the parent one: when the parent property updates, it will flow down to the child, but not the other way around. This prevents child components from accidentally mutating the parent's state, which can make your app's data flow harder to understand.

In addition, every time the parent component is updated, all props in the child component will be refreshed with the latest value. This means you should **not** attempt to mutate a prop inside a child component. If you do, Vue will warn you in the console:

```js
const props = defineProps(['foo'])

// ❌ warning, props are readonly!
props.foo = 'bar'
```

```js
export default {
  props: ['foo'],
  created() 
}
```

There are usually two cases where it's tempting to mutate a prop:

1. **The prop is used to pass in an initial value; the child component wants to use it as a local data property afterwards.** In this case, it's best to define a local data property that uses the prop as its initial value:

   

   ```js
   const props = defineProps(['initialCounter'])

   // counter only uses props.initialCounter as the initial value;
   // it is disconnected from future prop updates.
   const counter = ref(props.initialCounter)
   ```

   
   

   ```js
   export default {
     props: ['initialCounter'],
     data() {
       return 
     }
   }
   ```

   

2. **The prop is passed in as a raw value that needs to be transformed.** In this case, it's best to define a computed property using the prop's value:

   

   ```js
   const props = defineProps(['size'])

   // computed property that auto-updates when the prop changes
   const normalizedSize = computed(() => props.size.trim().toLowerCase())
   ```

   
   

   ```js
   export default {
     props: ['size'],
     computed: {
       // computed property that auto-updates when the prop changes
       normalizedSize() 
     }
   }
   ```

   

### Mutating Object / Array Props 

When objects and arrays are passed as props, while the child component cannot mutate the prop binding, it **will** be able to mutate the object or array's nested properties. This is because in JavaScript objects and arrays are passed by reference, and it is unreasonably expensive for Vue to prevent such mutations.

The main drawback of such mutations is that it allows the child component to affect parent state in a way that isn't obvious to the parent component, potentially making it more difficult to reason about the data flow in the future. As a best practice, you should avoid such mutations unless the parent and child are tightly coupled by design. In most cases, the child should [emit an event](/guide/components/events) to let the parent perform the mutation.

## Prop Validation 

Components can specify requirements for their props, such as the types you've already seen. If a requirement is not met, Vue will warn you in the browser's JavaScript console. This is especially useful when developing a component that is intended to be used by others.

To specify prop validations, you can provide an object with validation requirements to the `defineProps()` macro`props` option, instead of an array of strings. For example:

```js
defineProps({
  // Basic type check
  //  (`null` and `undefined` values will allow any type)
  propA: Number,
  // Multiple possible types
  propB: [String, Number],
  // Required string
  propC: ,
  // Required but nullable string
  propD: ,
  // Number with a default value
  propE: ,
  // Object with a default value
  propF: {
    type: Object,
    // Object or array defaults must be returned from
    // a factory function. The function receives the raw
    // props received by the component as the argument.
    default(rawProps) {
      return 
    }
  },
  // Custom validator function
  // full props passed as 2nd argument in 3.4+
  propG: {
    validator(value, props) 
  },
  // Function with a default value
  propH: {
    type: Function,
    // Unlike object or array default, this is not a factory
    // function - this is a function to serve as a default value
    default() 
  }
})
```

:::tip
Code inside the `defineProps()` argument **cannot access other variables declared in ``**, because the entire expression is moved to an outer function scope when compiled.
:::

```js
export default {
  props: {
    // Basic type check
    //  (`null` and `undefined` values will allow any type)
    propA: Number,
    // Multiple possible types
    propB: [String, Number],
    // Required string
    propC: ,
    // Required but nullable string
    propD: ,
    // Number with a default value
    propE: ,
    // Object with a default value
    propF: {
      type: Object,
      // Object or array defaults must be returned from
      // a factory function. The function receives the raw
      // props received by the component as the argument.
      default(rawProps) {
        return 
      }
    },
    // Custom validator function
    // full props passed as 2nd argument in 3.4+
    propG: {
      validator(value, props) 
    },
    // Function with a default value
    propH: {
      type: Function,
      // Unlike object or array default, this is not a factory
      // function - this is a function to serve as a default value
      default() 
    }
  }
}
```

Additional details:

- All props are optional by default, unless `required: true` is specified.

- An absent optional prop other than `Boolean` will have `undefined` value.

- The `Boolean` absent props will be cast to `false`. You can change this by setting a `default` for it — i.e.: `default: undefined` to behave as a non-Boolean prop.

- If a `default` value is specified, it will be used if the resolved prop value is `undefined` - this includes both when the prop is absent, or an explicit `undefined` value is passed.

When prop validation fails, Vue will produce a console warning (if using the development build).

If using [Type-based props declarations](/api/sfc-script-setup#type-only-props-emit-declarations) , Vue will try its best to compile the type annotations into equivalent runtime prop declarations. For example, `defineProps<>` will be compiled into `{ msg: }`.

::: tip Note
Note that props are validated **before** a component instance is created, so instance properties (e.g. `data`, `computed`, etc.) will not be available inside `default` or `validator` functions.
:::

### Runtime Type Checks 

The `type` can be one of the following native constructors:

- `String`
- `Number`
- `Boolean`
- `Array`
- `Object`
- `Date`
- `Function`
- `Symbol`
- `Error`

In addition, `type` can also be a custom class or constructor function and the assertion will be made with an `instanceof` check. For example, given the following class:

```js
class Person {
  constructor(firstName, lastName) 
}
```

You could use it as a prop's type:

```js
defineProps()
```

```js
export default {
  props: 
}
```

Vue will use `instanceof Person` to validate whether the value of the `author` prop is indeed an instance of the `Person` class.

### Nullable Type 

If the type is required but nullable, you can use the array syntax that includes `null`:

```js
defineProps({
  id: 
})
```

```js
export default {
  props: {
    id: 
  }
}
```

Note that if `type` is just `null` without using the array syntax, it will allow any type.

## Boolean Casting 

Props with `Boolean` type have special casting rules to mimic the behavior of native boolean attributes. Given a `` with the following declaration:

```js
defineProps()
```

```js
export default {
  props: 
}
```

The component can be used like this:

```vue-html

```

When a prop is declared to allow multiple types, the casting rules for `Boolean` will also be applied. However, there is an edge when both `String` and `Boolean` are allowed - the Boolean casting rule only applies if Boolean appears before String:

```js
// disabled will be casted to true
defineProps()

// disabled will be casted to true
defineProps()

// disabled will be casted to true
defineProps()

// disabled will be parsed as an empty string (disabled="")
defineProps()
```

```js
// disabled will be casted to true
export default {
  props: 
}

// disabled will be casted to true
export default {
  props: 
}

// disabled will be casted to true
export default {
  props: 
}

// disabled will be parsed as an empty string (disabled="")
export default {
  props: 
}
```

