# Built-in Directives 

## v-text 

Update the element's text content.

- **Expects:** `string`

- **Details**

  `v-text` works by setting the element's [textContent](https://developer.mozilla.org/en-US/docs/Web/API/Node/textContent) property, so it will overwrite any existing content inside the element. If you need to update only part of the `textContent`, you should use [mustache interpolations](/guide/essentials/template-syntax#text-interpolation) instead (ie. `Keep this but update a {}`).

- **Example**

  ```vue-html
  
  
  {}
  ```

- **See also** [Template Syntax - Text Interpolation](/guide/essentials/template-syntax#text-interpolation)

## v-html 

Update the element's [innerHTML](https://developer.mozilla.org/en-US/docs/Web/API/Element/innerHTML).

- **Expects:** `string`

- **Details**

  Contents of `v-html` are inserted as plain HTML - Vue template syntax will not be processed. If you find yourself trying to compose templates using `v-html`, try to rethink the solution by using components instead.

  ::: warning Security Note
  Dynamically rendering arbitrary HTML on your website can be very dangerous because it can easily lead to [XSS attacks](https://en.wikipedia.org/wiki/Cross-site_scripting). Only use `v-html` on trusted content and **never** on user-provided content.
  :::

  In [Single-File Components](/guide/scaling-up/sfc), `scoped` styles will not apply to content inside `v-html`, because that HTML is not processed by Vue's template compiler. If you want to target `v-html` content with scoped CSS, you can instead use [CSS modules](./sfc-css-features#css-modules) or an additional, global `` element with a manual scoping strategy such as BEM.

- **Example**

  ```vue-html
  
  ```

- **See also** [Template Syntax - Raw HTML](/guide/essentials/template-syntax#raw-html)

## v-show 

Toggle the element's visibility based on the truthy-ness of the expression value.

- **Expects:** `any`

- **Details**

  `v-show` works by setting the `display` CSS property via inline styles, and will try to respect the initial `display` value when the element is visible. It also triggers transitions when its condition changes.

- **See also** [Conditional Rendering - v-show](/guide/essentials/conditional#v-show)

## v-if 

Conditionally render an element or a template fragment based on the truthy-ness of the expression value.

- **Expects:** `any`

- **Details**

  When a `v-if` element is toggled, the element and its contained directives / components are destroyed and re-constructed. If the initial condition is falsy, then the inner content won't be rendered at all.

  Can be used on `` to denote a conditional block containing only text or multiple elements.

  This directive triggers transitions when its condition changes.

  When used together, `v-if` has a higher priority than `v-for`. We don't recommend using these two directives together on one element — see the [list rendering guide](/guide/essentials/list#v-for-with-v-if) for details.

- **See also** [Conditional Rendering - v-if](/guide/essentials/conditional#v-if)

## v-else 

Denote the "else block" for `v-if` or a `v-if` / `v-else-if` chain.

- **Does not expect expression**

- **Details**

  - Restriction: previous sibling element must have `v-if` or `v-else-if`.

  - Can be used on `` to denote a conditional block containing only text or multiple elements.

- **Example**

  ```vue-html
   0.5">
    Now you see me
  
  
    Now you don't
  
  ```

- **See also** [Conditional Rendering - v-else](/guide/essentials/conditional#v-else)

## v-else-if 

Denote the "else if block" for `v-if`. Can be chained.

- **Expects:** `any`

- **Details**

  - Restriction: previous sibling element must have `v-if` or `v-else-if`.

  - Can be used on `` to denote a conditional block containing only text or multiple elements.

- **Example**

  ```vue-html
  
    A
  
  
    B
  
  
    C
  
  
    Not A/B/C
  
  ```

- **See also** [Conditional Rendering - v-else-if](/guide/essentials/conditional#v-else-if)

## v-for 

Render the element or template block multiple times based on the source data.

- **Expects:** `Array | Object | number | string | Iterable`

- **Details**

  The directive's value must use the special syntax `alias in expression` to provide an alias for the current element being iterated on:

  ```vue-html
  
    {}
  
  ```

  Alternatively, you can also specify an alias for the index (or the key if used on an Object):

  ```vue-html
  
  
  
  ```

  The default behavior of `v-for` will try to patch the elements in-place without moving them. To force it to reorder elements, you should provide an ordering hint with the `key` special attribute:

  ```vue-html
  
    {}
  
  ```

  `v-for` can also work on values that implement the [Iterable Protocol](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Iteration_protocols#The_iterable_protocol), including native `Map` and `Set`.

- **See also**
  - [List Rendering](/guide/essentials/list)

## v-on 

Attach an event listener to the element.

- **Shorthand:** `@`

- **Expects:** `Function | Inline Statement | Object (without argument)`

- **Argument:** `event` (optional if using Object syntax)

- **Modifiers**

  - `.stop` - call `event.stopPropagation()`.
  - `.prevent` - call `event.preventDefault()`.
  - `.capture` - add event listener in capture mode.
  - `.self` - only trigger handler if event was dispatched from this element.
  - `.` - only trigger handler on certain keys.
  - `.once` - trigger handler at most once.
  - `.left` - only trigger handler for left button mouse events.
  - `.right` - only trigger handler for right button mouse events.
  - `.middle` - only trigger handler for middle button mouse events.
  - `.passive` - attaches a DOM event with ``.

- **Details**

  The event type is denoted by the argument. The expression can be a method name, an inline statement, or omitted if there are modifiers present.

  When used on a normal element, it listens to [**native DOM events**](https://developer.mozilla.org/en-US/docs/Web/Events) only. When used on a custom element component, it listens to **custom events** emitted on that child component.

  When listening to native DOM events, the method receives the native event as the only argument. If using inline statement, the statement has access to the special `$event` property: `v-on:click="handle('ok', $event)"`.

  `v-on` also supports binding to an object of event / listener pairs without an argument. Note when using the object syntax, it does not support any modifiers.

- **Example**

  ```vue-html
  
  

  
  

  
  

  
  

  
  

  
  

  
  

  
  

  
  

  
  

  
  

  
  
  ```

  Listening to custom events on a child component (the handler is called when "my-event" is emitted on the child):

  ```vue-html
  

  
  

  
  
  ```

- **See also**
  - [Components - Slots](/guide/components/slots)

## v-pre 

Skip compilation for this element and all its children.

- **Does not expect expression**

- **Details**

  Inside the element with `v-pre`, all Vue template syntax will be preserved and rendered as-is. The most common use case of this is displaying raw mustache tags.

- **Example**

  ```vue-html
  {}
  ```

## v-once 

Render the element and component once only, and skip future updates.

- **Does not expect expression**

- **Details**

  On subsequent re-renders, the element/component and all its children will be treated as static content and skipped. This can be used to optimize update performance.

  ```vue-html
  
  This will never change: {}
  
  
    Comment
    {}
  
  
  
  
  
    {}
  
  ```

  Since 3.2, you can also memoize part of the template with invalidation conditions using [`v-memo`](#v-memo).

- **See also**
  - [Data Binding Syntax - interpolations](/guide/essentials/template-syntax#text-interpolation)
  - [v-memo](#v-memo)

## v-memo 

- Only supported in 3.2+

- **Expects:** `any[]`

- **Details**

  Memoize a sub-tree of the template. Can be used on both elements and components. The directive expects a fixed-length array of dependency values to compare for the memoization. If every value in the array was the same as last render, then updates for the entire sub-tree will be skipped. For example:

  ```vue-html
  
    ...
  
  ```

  When the component re-renders, if both `valueA` and `valueB` remain the same, all updates for this `` and its children will be skipped. In fact, even the Virtual DOM VNode creation will also be skipped since the memoized copy of the sub-tree can be reused.

  It is important to specify the memoization array correctly, otherwise we may skip updates that should indeed be applied. `v-memo` with an empty dependency array (`v-memo="[]"`) would be functionally equivalent to `v-once`.

  **Usage with `v-for`**

  `v-memo` is provided solely for micro optimizations in performance-critical scenarios and should be rarely needed. The most common case where this may prove helpful is when rendering large `v-for` lists (where `length > 1000`):

  ```vue-html
  
    ID: {} - selected: {}
    ...more child nodes
  
  ```

  When the component's `selected` state changes, a large amount of VNodes will be created even though most of the items remained exactly the same. The `v-memo` usage here is essentially saying "only update this item if it went from non-selected to selected, or the other way around". This allows every unaffected item to reuse its previous VNode and skip diffing entirely. Note we don't need to include `item.id` in the memo dependency array here since Vue automatically infers it from the item's `:key`.

  :::warning
  When using `v-memo` with `v-for`, make sure they are used on the same element. **`v-memo` does not work inside `v-for`.**
  :::

  `v-memo` can also be used on components to manually prevent unwanted updates in certain edge cases where the child component update check has been de-optimized. But again, it is the developer's responsibility to specify correct dependency arrays to avoid skipping necessary updates.

- **See also**
  - [v-once](#v-once)

## v-cloak 

Used to hide un-compiled template until it is ready.

- **Does not expect expression**

- **Details**

  **This directive is only needed in no-build-step setups.**

  When using in-DOM templates, there can be a "flash of un-compiled templates": the user may see raw mustache tags until the mounted component replaces them with rendered content.

  `v-cloak` will remain on the element until the associated component instance is mounted. Combined with CSS rules such as `[v-cloak] `, it can be used to hide the raw templates until the component is ready.

- **Example**

  ```css
  [v-cloak] 
  ```

  ```vue-html
  
    {}
  
  ```

  The `` will not be visible until the compilation is done.
