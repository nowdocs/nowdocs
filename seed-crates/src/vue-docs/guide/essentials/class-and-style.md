# Class and Style Bindings 

A common need for data binding is manipulating an element's class list and inline styles. Since `class` and `style` are both attributes, we can use `v-bind` to assign them a string value dynamically, much like with other attributes. However, trying to generate those values using string concatenation can be annoying and error-prone. For this reason, Vue provides special enhancements when `v-bind` is used with `class` and `style`. In addition to strings, the expressions can also evaluate to objects or arrays.

## Binding HTML Classes 

  

  

### Binding to Objects 

We can pass an object to `:class` (short for `v-bind:class`) to dynamically toggle classes:

```vue-html

```

The above syntax means the presence of the `active` class will be determined by the [truthiness](https://developer.mozilla.org/en-US/docs/Glossary/Truthy) of the data property `isActive`.

You can have multiple classes toggled by having more fields in the object. In addition, the `:class` directive can also co-exist with the plain `class` attribute. So given the following state:

```js
const isActive = ref(true)
const hasError = ref(false)
```

```js
data() {
  return 
}
```

And the following template:

```vue-html

```

It will render:

```vue-html

```

When `isActive` or `hasError` changes, the class list will be updated accordingly. For example, if `hasError` becomes `true`, the class list will become `"static active text-danger"`.

The bound object doesn't have to be inline:

```js
const classObject = reactive()
```

```js
data() {
  return {
    classObject: 
  }
}
```

```vue-html

```

This will render:

```vue-html

```

We can also bind to a [computed property](./computed) that returns an object. This is a common and powerful pattern:

```js
const isActive = ref(true)
const error = ref(null)

const classObject = computed(() => ())
```

```js
data() {
  return 
},
computed: {
  classObject() {
    return 
  }
}
```

```vue-html

```

### Binding to Arrays 

We can bind `:class` to an array to apply a list of classes:

```js
const activeClass = ref('active')
const errorClass = ref('text-danger')
```

```js
data() {
  return 
}
```

```vue-html

```

Which will render:

```vue-html

```

If you would like to also toggle a class in the list conditionally, you can do it with a ternary expression:

```vue-html

```

This will always apply `errorClass`, but `activeClass` will only be applied when `isActive` is truthy.

However, this can be a bit verbose if you have multiple conditional classes. That's why it's also possible to use the object syntax inside the array syntax:

```vue-html

```

### With Components 

> This section assumes knowledge of [Components](/guide/essentials/component-basics). Feel free to skip it and come back later.

When you use the `class` attribute on a component with a single root element, those classes will be added to the component's root element and merged with any existing class already on it.

For example, if we have a component named `MyComponent` with the following template:

```vue-html

Hi!
```

Then add some classes when using it:

```vue-html

```

The rendered HTML will be:

```vue-html
Hi!
```

The same is true for class bindings:

```vue-html

```

When `isActive` is truthy, the rendered HTML will be:

```vue-html
Hi!
```

If your component has multiple root elements, you would need to define which element will receive this class. You can do this using the `$attrs` component property:

```vue-html

Hi!
This is a child component
```

```vue-html

```

Will render:

```html
Hi!
This is a child component
```

You can learn more about component attribute inheritance in [Fallthrough Attributes](/guide/components/attrs) section.

## Binding Inline Styles 

### Binding to Objects 

`:style` supports binding to JavaScript object values - it corresponds to an [HTML element's `style` property](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/style):

```js
const activeColor = ref('red')
const fontSize = ref(30)
```

```js
data() {
  return 
}
```

```vue-html

```

Although camelCase keys are recommended, `:style` also supports kebab-cased CSS property keys (corresponds to how they are used in actual CSS) - for example:

```vue-html

```

It is often a good idea to bind to a style object directly so that the template is cleaner:

```js
const styleObject = reactive()
```

```js
data() {
  return {
    styleObject: 
  }
}
```

```vue-html

```

Again, object style binding is often used in conjunction with computed properties that return objects.

`:style` directives can also coexist with regular style attributes, just like `:class`.

Template:

```vue-html
hello
```

It will render:

```vue-html
hello
```

### Binding to Arrays 

We can bind `:style` to an array of multiple style objects. These objects will be merged and applied to the same element:

```vue-html

```

### Auto-prefixing 

When you use a CSS property that requires a [vendor prefix](https://developer.mozilla.org/en-US/docs/Glossary/Vendor_Prefix) in `:style`, Vue will automatically add the appropriate prefix. Vue does this by checking at runtime to see which style properties are supported in the current browser. If the browser doesn't support a particular property then various prefixed variants will be tested to try to find one that is supported.

### Multiple Values 

You can provide an array of multiple (prefixed) values to a style property, for example:

```vue-html

```

This will only render the last value in the array which the browser supports. In this example, it will render `display: flex` for browsers that support the unprefixed version of flexbox.
