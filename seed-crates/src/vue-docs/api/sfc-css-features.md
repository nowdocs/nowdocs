# SFC CSS Features 

## Scoped CSS 

When a `` tag has the `scoped` attribute, its CSS will apply to elements of the current component only. This is similar to the style encapsulation found in Shadow DOM. It comes with some caveats, but doesn't require any polyfills. It is achieved by using PostCSS to transform the following:

```vue

.example 

  hi

```

Into the following:

```vue

.example[data-v-f3f3eg9] 

  hi

```

### Child Component Root Elements 

With `scoped`, the parent component's styles will not leak into child components. However, a child component's root node will be affected by both the parent's scoped CSS and the child's scoped CSS. This is by design so that the parent can style the child root element for layout purposes.

### Deep Selectors 

If you want a selector in `scoped` styles to be "deep", i.e. affecting child components, you can use the `:deep()` pseudo-class:

```vue

.a :deep(.b) 

```

The above will be compiled into:

```css
.a[data-v-f3f3eg9] .b 
```

:::tip
DOM content created with `v-html` are not affected by scoped styles, but you can still style them using deep selectors.
:::

### Slotted Selectors 

By default, scoped styles do not affect contents rendered by ``, as they are considered to be owned by the parent component passing them in. To explicitly target slot content, use the `:slotted` pseudo-class:

```vue

:slotted(div) 

```

### Global Selectors 

If you want just one rule to apply globally, you can use the `:global` pseudo-class rather than creating another `` (see below):

```vue

:global(.red) 

```

### Mixing Local and Global Styles 

You can also include both scoped and non-scoped styles in the same component:

```vue

/* global styles */

/* local styles */

```

### Scoped Style Tips 

- **Scoped styles do not eliminate the need for classes**. Due to the way browsers render various CSS selectors, `p ` will be many times slower when scoped (i.e. when combined with an attribute selector). If you use classes or ids instead, such as in `.example `, then you virtually eliminate that performance hit.

- **Be careful with descendant selectors in recursive components!** For a CSS rule with the selector `.a .b`, if the element that matches `.a` contains a recursive child component, then all `.b` in that child component will be matched by the rule.

## CSS Modules 

A `` tag is compiled as [CSS Modules](https://github.com/css-modules/css-modules) and exposes the resulting CSS classes to the component as an object under the key of `$style`:

```vue

  This should be red

.red 

```

The resulting classes are hashed to avoid collision, achieving the same effect of scoping the CSS to the current component only.

Refer to the [CSS Modules spec](https://github.com/css-modules/css-modules) for more details such as [global exceptions](https://github.com/css-modules/css-modules/blob/master/docs/composition.md#exceptions) and [composition](https://github.com/css-modules/css-modules/blob/master/docs/composition.md#composition).

### Custom Inject Name 

You can customize the property key of the injected classes object by giving the `module` attribute a value:

```vue

  red

.red 

```

### Usage with Composition API 

The injected classes can be accessed in `setup()` and `` via the `useCssModule` API. For `` blocks with custom injection names, `useCssModule` accepts the matching `module` attribute value as the first argument:

```js
import  from 'vue'

// inside setup() scope...
// default, returns classes for 
useCssModule()

// named, returns classes for 
useCssModule('classes')
```

- **Example**

```vue

import  from 'vue'

const classes = useCssModule()

  red

.red 

```

## `v-bind()` in CSS 

SFC `` tags support linking CSS values to dynamic component state using the `v-bind` CSS function:

```vue

  hello

export default {
  data() {
    return 
  }
}

.text 

```

The syntax works with [``](./sfc-script-setup), and supports JavaScript expressions (must be wrapped in quotes):

```vue

import  from 'vue'
const theme = ref()

  hello

p 

```

The actual value will be compiled into a hashed CSS custom property, so the CSS is still static. The custom property will be applied to the component's root element via inline styles and reactively updated if the source value changes.
