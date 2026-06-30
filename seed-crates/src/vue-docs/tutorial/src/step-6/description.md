# Conditional Rendering 

We can use the `v-if` directive to conditionally render an element:

```vue-html
Vue is awesome!
```

This `` will be rendered only if the value of `awesome` is [truthy](https://developer.mozilla.org/en-US/docs/Glossary/Truthy). If `awesome` changes to a [falsy](https://developer.mozilla.org/en-US/docs/Glossary/Falsy) value, it will be removed from the DOM.

We can also use `v-else` and `v-else-if` to denote other branches of the condition:

```vue-html
Vue is awesome!
Oh no 😢
```

Currently, the demo is showing both ``s at the same time, and the button does nothing. Try to add `v-if` and `v-else` directives to them, and implement the `toggle()` method so that we can use the button to toggle between them.

More details on `v-if`: Guide - Conditional Rendering
