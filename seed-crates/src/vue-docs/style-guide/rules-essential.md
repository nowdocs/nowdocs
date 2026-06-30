# Priority A Rules: Essential 

These rules help prevent errors, so learn and abide by them at all costs. Exceptions may exist, but should be very rare and only be made by those with expert knowledge of both JavaScript and Vue.

## Use multi-word component names 

User component names should always be multi-word, except for root `App` components. This [prevents conflicts](https://html.spec.whatwg.org/multipage/custom-elements.html#valid-custom-element-name) with existing and future HTML elements, since all HTML elements are a single word.

Bad

```vue-html

```

Good

```vue-html

```

## Use detailed prop definitions 

In committed code, prop definitions should always be as detailed as possible, specifying at least type(s).

::: details Detailed Explanation
Detailed [prop definitions](/guide/components/props#prop-validation) have two advantages:

- They document the API of the component, so that it's easy to see how the component is meant to be used.
- In development, Vue will warn you if a component is ever provided incorrectly formatted props, helping you catch potential sources of error.
  :::

Bad

```js
// This is only OK when prototyping
props: ['status']
```

Good

```js
props: 
```

```js
// Even better!
props: {
  status: {
    type: String,
    required: true,

    validator: value => 
  }
}
```

Bad

```js
// This is only OK when prototyping
const props = defineProps(['status'])
```

Good

```js
const props = defineProps()
```

```js
// Even better!

const props = defineProps({
  status: {
    type: String,
    required: true,

    validator: (value) => 
  }
})
```

## Use keyed `v-for` 

`key` with `v-for` is _always_ required on components, in order to maintain internal component state down the subtree. Even for elements though, it's a good practice to maintain predictable behavior, such as [object constancy](https://bost.ocks.org/mike/constancy/) in animations.

::: details Detailed Explanation
Let's say you have a list of todos:

```js
data() {
  return {
    todos: [
      ,
      
    ]
  }
}
```

```js
const todos = ref([
  ,
  
])
```

Then you sort them alphabetically. When updating the DOM, Vue will optimize rendering to perform the cheapest DOM mutations possible. That might mean deleting the first todo element, then adding it again at the end of the list.

The problem is, there are cases where it's important not to delete elements that will remain in the DOM. For example, you may want to use `` to animate list sorting, or maintain focus if the rendered element is an ``. In these cases, adding a unique key for each item (e.g. `:key="todo.id"`) will tell Vue how to behave more predictably.

In our experience, it's better to _always_ add a unique key, so that you and your team simply never have to worry about these edge cases. Then in the rare, performance-critical scenarios where object constancy isn't necessary, you can make a conscious exception.
:::

Bad

```vue-html

  
    {}
  

```

Good

```vue-html

  
    {}
  

```

## Avoid `v-if` with `v-for` 

**Never use `v-if` on the same element as `v-for`.**

There are two common cases where this can be tempting:

- To filter items in a list (e.g. `v-for="user in users" v-if="user.isActive"`). In these cases, replace `users` with a new computed property that returns your filtered list (e.g. `activeUsers`).

- To avoid rendering a list if it should be hidden (e.g. `v-for="user in users" v-if="shouldShowUsers"`). In these cases, move the `v-if` to a container element (e.g. `ul`, `ol`).

::: details Detailed Explanation
When Vue processes directives, `v-if` has a higher priority than `v-for`, so that this template:

```vue-html

  
    {}
  

```

Will throw an error, because the `v-if` directive will be evaluated first and the iteration variable `user` does not exist at this moment.

This could be fixed by iterating over a computed property instead, like this:

```js
computed: {
  activeUsers() 
}
```

```js
const activeUsers = computed(() => )
```

```vue-html

  
    {}
  

```

Alternatively, we can use a `` tag with `v-for` to wrap the `` element:

```vue-html

  
    
      {}
    
  

```

:::

Bad

```vue-html

  
    {}
  

```

Good

```vue-html

  
    {}
  

```

```vue-html

  
    
      {}
    
  

```

## Use component-scoped styling 

For applications, styles in a top-level `App` component and in layout components may be global, but all other components should always be scoped.

This is only relevant for [Single-File Components](/guide/scaling-up/sfc). It does _not_ require that the [`scoped` attribute](/api/sfc-css-features#scoped-css) be used. Scoping could be through [CSS modules](/api/sfc-css-features#css-modules), a class-based strategy such as [BEM](https://getbem.com/), or another library/convention.

**Component libraries, however, should prefer a class-based strategy instead of using the `scoped` attribute.**

This makes overriding internal styles easier, with human-readable class names that don't have too high specificity, but are still very unlikely to result in a conflict.

::: details Detailed Explanation
If you are developing a large project, working with other developers, or sometimes include 3rd-party HTML/CSS (e.g. from Auth0), consistent scoping will ensure that your styles only apply to the components they are meant for.

Beyond the `scoped` attribute, using unique class names can help ensure that 3rd-party CSS does not apply to your own HTML. For example, many projects use the `button`, `btn`, or `icon` class names, so even if not using a strategy such as BEM, adding an app-specific and/or component-specific prefix (e.g. `ButtonClose-icon`) can provide some protection.
:::

Bad

```vue-html

  ×

.btn-close 

```

Good

```vue-html

  ×

.button 

.button-close 

```

```vue-html

  ×

.button 

.buttonClose 

```

```vue-html

  ×

.c-Button 

.c-Button--close 

```

