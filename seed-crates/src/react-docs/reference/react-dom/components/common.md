---
title: "Common components (e.g. )"
---

#### Returns 

* **optional** `cleanup function`: When the `ref` is detached, React will call the cleanup function. If a function is not returned by the `ref` callback, React will call the callback again with `null` as the argument when the `ref` gets detached. This behavior will be removed in a future version.

#### Caveats 

* When Strict Mode is on, React will **run one extra development-only setup+cleanup cycle** before the first real setup. This is a stress-test that ensures that your cleanup logic "mirrors" your setup logic and that it stops or undoes whatever the setup is doing. If this causes a problem, implement the cleanup function.
* When you pass a *different* `ref` callback, React will call the *previous* callback's cleanup function if provided. If no cleanup function is defined, the `ref` callback will be called with `null` as the argument. The *next* function will be called with the DOM node.

---

### React event object 

Your event handlers will receive a *React event object.* It is also sometimes known as a "synthetic event".

```js
 } />
```

It conforms to the same standard as the underlying DOM events, but fixes some browser inconsistencies.

Some React events do not map directly to the browser's native events. For example in `onMouseLeave`, `e.nativeEvent` will point to a `mouseout` event. The specific mapping is not part of the public API and may change in the future. If you need the underlying browser event for some reason, read it from `e.nativeEvent`.

#### Properties 

React event objects implement some of the standard [`Event`](https://developer.mozilla.org/en-US/docs/Web/API/Event) properties:

* [`bubbles`](https://developer.mozilla.org/en-US/docs/Web/API/Event/bubbles): A boolean. Returns whether the event bubbles through the DOM.
* [`cancelable`](https://developer.mozilla.org/en-US/docs/Web/API/Event/cancelable): A boolean. Returns whether the event can be canceled.
* [`currentTarget`](https://developer.mozilla.org/en-US/docs/Web/API/Event/currentTarget): A DOM node. Returns the node to which the current handler is attached in the React tree.
* [`defaultPrevented`](https://developer.mozilla.org/en-US/docs/Web/API/Event/defaultPrevented): A boolean. Returns whether `preventDefault` was called.
* [`eventPhase`](https://developer.mozilla.org/en-US/docs/Web/API/Event/eventPhase): A number. Returns which phase the event is currently in.
* [`isTrusted`](https://developer.mozilla.org/en-US/docs/Web/API/Event/isTrusted): A boolean. Returns whether the event was initiated by user.
* [`target`](https://developer.mozilla.org/en-US/docs/Web/API/Event/target): A DOM node. Returns the node on which the event has occurred (which could be a distant child).
* [`timeStamp`](https://developer.mozilla.org/en-US/docs/Web/API/Event/timeStamp): A number. Returns the time when the event occurred.

Additionally, React event objects provide these properties:

* `nativeEvent`: A DOM [`Event`](https://developer.mozilla.org/en-US/docs/Web/API/Event). The original browser event object.

#### Methods 

React event objects implement some of the standard [`Event`](https://developer.mozilla.org/en-US/docs/Web/API/Event) methods:

* [`preventDefault()`](https://developer.mozilla.org/en-US/docs/Web/API/Event/preventDefault): Prevents the default browser action for the event.
* [`stopPropagation()`](https://developer.mozilla.org/en-US/docs/Web/API/Event/stopPropagation): Stops the event propagation through the React tree.

Additionally, React event objects provide these methods:

* `isDefaultPrevented()`: Returns a boolean value indicating whether `preventDefault` was called.
* `isPropagationStopped()`: Returns a boolean value indicating whether `stopPropagation` was called.
* `persist()`: Not used with React DOM. With React Native, call this to read event's properties after the event.
* `isPersistent()`: Not used with React DOM. With React Native, returns whether `persist` has been called.

#### Caveats 

* The values of `currentTarget`, `eventPhase`, `target`, and `type` reflect the values your React code expects. Under the hood, React attaches event handlers at the root, but this is not reflected in React event objects. For example, `e.currentTarget` may not be the same as the underlying `e.nativeEvent.currentTarget`. For polyfilled events, `e.type` (React event type) may differ from `e.nativeEvent.type` (underlying type).

---

### `AnimationEvent` handler function 

An event handler type for the [CSS animation](https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_Animations/Using_CSS_animations) events.

```js
 console.log('onAnimationStart')}
  onAnimationIteration=
  onAnimationEnd=
/>
```

#### Parameters 

* `e`: A [React event object](#react-event-object) with these extra [`AnimationEvent`](https://developer.mozilla.org/en-US/docs/Web/API/AnimationEvent) properties:
  * [`animationName`](https://developer.mozilla.org/en-US/docs/Web/API/AnimationEvent/animationName)
  * [`elapsedTime`](https://developer.mozilla.org/en-US/docs/Web/API/AnimationEvent/elapsedTime)
  * [`pseudoElement`](https://developer.mozilla.org/en-US/docs/Web/API/AnimationEvent/pseudoElement)

---

### `ClipboardEvent` handler function 

An event handler type for the [Clipboard API](https://developer.mozilla.org/en-US/docs/Web/API/Clipboard_API) events.

```js
 console.log('onCopy')}
  onCut=
  onPaste=
/>
```

#### Parameters 

* `e`: A [React event object](#react-event-object) with these extra [`ClipboardEvent`](https://developer.mozilla.org/en-US/docs/Web/API/ClipboardEvent) properties:

  * [`clipboardData`](https://developer.mozilla.org/en-US/docs/Web/API/ClipboardEvent/clipboardData)

---

### `CompositionEvent` handler function 

An event handler type for the [input method editor (IME)](https://developer.mozilla.org/en-US/docs/Glossary/Input_method_editor) events.

```js
 console.log('onCompositionStart')}
  onCompositionUpdate=
  onCompositionEnd=
/>
```

#### Parameters 

* `e`: A [React event object](#react-event-object) with these extra [`CompositionEvent`](https://developer.mozilla.org/en-US/docs/Web/API/CompositionEvent) properties:
  * [`data`](https://developer.mozilla.org/en-US/docs/Web/API/CompositionEvent/data)

---

### `DragEvent` handler function 

An event handler type for the [HTML Drag and Drop API](https://developer.mozilla.org/en-US/docs/Web/API/HTML_Drag_and_Drop_API) events.

```js
<>
   console.log('onDragStart')}
    onDragEnd=
  >
    Drag source
  

   console.log('onDragEnter')}
    onDragLeave=
    onDragOver={e => }
    onDrop=
  >
    Drop target
  
</>
```

#### Parameters 

* `e`: A [React event object](#react-event-object) with these extra [`DragEvent`](https://developer.mozilla.org/en-US/docs/Web/API/DragEvent) properties:
  * [`dataTransfer`](https://developer.mozilla.org/en-US/docs/Web/API/DragEvent/dataTransfer)

  It also includes the inherited [`MouseEvent`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent) properties:

  * [`altKey`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/altKey)
  * [`button`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/button)
  * [`buttons`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/buttons)
  * [`ctrlKey`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/ctrlKey)
  * [`clientX`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/clientX)
  * [`clientY`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/clientY)
  * [`getModifierState(key)`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/getModifierState)
  * [`metaKey`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/metaKey)
  * [`movementX`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/movementX)
  * [`movementY`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/movementY)
  * [`pageX`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/pageX)
  * [`pageY`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/pageY)
  * [`relatedTarget`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/relatedTarget)
  * [`screenX`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/screenX)
  * [`screenY`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/screenY)
  * [`shiftKey`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/shiftKey)

  It also includes the inherited [`UIEvent`](https://developer.mozilla.org/en-US/docs/Web/API/UIEvent) properties:

  * [`detail`](https://developer.mozilla.org/en-US/docs/Web/API/UIEvent/detail)
  * [`view`](https://developer.mozilla.org/en-US/docs/Web/API/UIEvent/view)

---

### `FocusEvent` handler function 

An event handler type for the focus events.

```js
 console.log('onFocus')}
  onBlur=
/>
```

[See an example.](#handling-focus-events)

#### Parameters 

* `e`: A [React event object](#react-event-object) with these extra [`FocusEvent`](https://developer.mozilla.org/en-US/docs/Web/API/FocusEvent) properties:
  * [`relatedTarget`](https://developer.mozilla.org/en-US/docs/Web/API/FocusEvent/relatedTarget)

  It also includes the inherited [`UIEvent`](https://developer.mozilla.org/en-US/docs/Web/API/UIEvent) properties:

  * [`detail`](https://developer.mozilla.org/en-US/docs/Web/API/UIEvent/detail)
  * [`view`](https://developer.mozilla.org/en-US/docs/Web/API/UIEvent/view)

---

### `Event` handler function 

An event handler type for generic events.

#### Parameters 

* `e`: A [React event object](#react-event-object) with no additional properties.

---

### `InputEvent` handler function 

An event handler type for the `onBeforeInput` event.

```js
 console.log('onBeforeInput')} />
```

#### Parameters 

* `e`: A [React event object](#react-event-object) with these extra [`InputEvent`](https://developer.mozilla.org/en-US/docs/Web/API/InputEvent) properties:
  * [`data`](https://developer.mozilla.org/en-US/docs/Web/API/InputEvent/data)

---

### `KeyboardEvent` handler function 

An event handler type for keyboard events.

```js
 console.log('onKeyDown')}
  onKeyUp=
/>
```

[See an example.](#handling-keyboard-events)

#### Parameters 

* `e`: A [React event object](#react-event-object) with these extra [`KeyboardEvent`](https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent) properties:
  * [`altKey`](https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/altKey)
  * [`charCode`](https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/charCode)
  * [`code`](https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/code)
  * [`ctrlKey`](https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/ctrlKey)
  * [`getModifierState(key)`](https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/getModifierState)
  * [`key`](https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/key)
  * [`keyCode`](https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/keyCode)
  * [`locale`](https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/locale)
  * [`metaKey`](https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/metaKey)
  * [`location`](https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/location)
  * [`repeat`](https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/repeat)
  * [`shiftKey`](https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/shiftKey)
  * [`which`](https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/which)

  It also includes the inherited [`UIEvent`](https://developer.mozilla.org/en-US/docs/Web/API/UIEvent) properties:

  * [`detail`](https://developer.mozilla.org/en-US/docs/Web/API/UIEvent/detail)
  * [`view`](https://developer.mozilla.org/en-US/docs/Web/API/UIEvent/view)

---

### `MouseEvent` handler function 

An event handler type for mouse events.

```js
 console.log('onClick')}
  onMouseEnter=
  onMouseOver=
  onMouseDown=
  onMouseUp=
  onMouseLeave=
/>
```

[See an example.](#handling-mouse-events)

#### Parameters 

* `e`: A [React event object](#react-event-object) with these extra [`MouseEvent`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent) properties:
  * [`altKey`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/altKey)
  * [`button`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/button)
  * [`buttons`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/buttons)
  * [`ctrlKey`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/ctrlKey)
  * [`clientX`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/clientX)
  * [`clientY`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/clientY)
  * [`getModifierState(key)`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/getModifierState)
  * [`metaKey`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/metaKey)
  * [`movementX`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/movementX)
  * [`movementY`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/movementY)
  * [`pageX`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/pageX)
  * [`pageY`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/pageY)
  * [`relatedTarget`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/relatedTarget)
  * [`screenX`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/screenX)
  * [`screenY`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/screenY)
  * [`shiftKey`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/shiftKey)

  It also includes the inherited [`UIEvent`](https://developer.mozilla.org/en-US/docs/Web/API/UIEvent) properties:

  * [`detail`](https://developer.mozilla.org/en-US/docs/Web/API/UIEvent/detail)
  * [`view`](https://developer.mozilla.org/en-US/docs/Web/API/UIEvent/view)

---

### `PointerEvent` handler function 

An event handler type for [pointer events.](https://developer.mozilla.org/en-US/docs/Web/API/Pointer_events)

```js
 console.log('onPointerEnter')}
  onPointerMove=
  onPointerDown=
  onPointerUp=
  onPointerLeave=
/>
```

[See an example.](#handling-pointer-events)

#### Parameters 

* `e`: A [React event object](#react-event-object) with these extra [`PointerEvent`](https://developer.mozilla.org/en-US/docs/Web/API/PointerEvent) properties:
  * [`height`](https://developer.mozilla.org/en-US/docs/Web/API/PointerEvent/height)
  * [`isPrimary`](https://developer.mozilla.org/en-US/docs/Web/API/PointerEvent/isPrimary)
  * [`pointerId`](https://developer.mozilla.org/en-US/docs/Web/API/PointerEvent/pointerId)
  * [`pointerType`](https://developer.mozilla.org/en-US/docs/Web/API/PointerEvent/pointerType)
  * [`pressure`](https://developer.mozilla.org/en-US/docs/Web/API/PointerEvent/pressure)
  * [`tangentialPressure`](https://developer.mozilla.org/en-US/docs/Web/API/PointerEvent/tangentialPressure)
  * [`tiltX`](https://developer.mozilla.org/en-US/docs/Web/API/PointerEvent/tiltX)
  * [`tiltY`](https://developer.mozilla.org/en-US/docs/Web/API/PointerEvent/tiltY)
  * [`twist`](https://developer.mozilla.org/en-US/docs/Web/API/PointerEvent/twist)
  * [`width`](https://developer.mozilla.org/en-US/docs/Web/API/PointerEvent/width)

  It also includes the inherited [`MouseEvent`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent) properties:

  * [`altKey`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/altKey)
  * [`button`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/button)
  * [`buttons`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/buttons)
  * [`ctrlKey`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/ctrlKey)
  * [`clientX`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/clientX)
  * [`clientY`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/clientY)
  * [`getModifierState(key)`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/getModifierState)
  * [`metaKey`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/metaKey)
  * [`movementX`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/movementX)
  * [`movementY`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/movementY)
  * [`pageX`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/pageX)
  * [`pageY`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/pageY)
  * [`relatedTarget`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/relatedTarget)
  * [`screenX`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/screenX)
  * [`screenY`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/screenY)
  * [`shiftKey`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/shiftKey)

  It also includes the inherited [`UIEvent`](https://developer.mozilla.org/en-US/docs/Web/API/UIEvent) properties:

  * [`detail`](https://developer.mozilla.org/en-US/docs/Web/API/UIEvent/detail)
  * [`view`](https://developer.mozilla.org/en-US/docs/Web/API/UIEvent/view)

---

### `TouchEvent` handler function 

An event handler type for [touch events.](https://developer.mozilla.org/en-US/docs/Web/API/Touch_events)

```js
 console.log('onTouchStart')}
  onTouchMove=
  onTouchEnd=
  onTouchCancel=
/>
```

#### Parameters 

* `e`: A [React event object](#react-event-object) with these extra [`TouchEvent`](https://developer.mozilla.org/en-US/docs/Web/API/TouchEvent) properties:
  * [`altKey`](https://developer.mozilla.org/en-US/docs/Web/API/TouchEvent/altKey)
  * [`ctrlKey`](https://developer.mozilla.org/en-US/docs/Web/API/TouchEvent/ctrlKey)
  * [`changedTouches`](https://developer.mozilla.org/en-US/docs/Web/API/TouchEvent/changedTouches)
  * [`getModifierState(key)`](https://developer.mozilla.org/en-US/docs/Web/API/TouchEvent/getModifierState)
  * [`metaKey`](https://developer.mozilla.org/en-US/docs/Web/API/TouchEvent/metaKey)
  * [`shiftKey`](https://developer.mozilla.org/en-US/docs/Web/API/TouchEvent/shiftKey)
  * [`touches`](https://developer.mozilla.org/en-US/docs/Web/API/TouchEvent/touches)
  * [`targetTouches`](https://developer.mozilla.org/en-US/docs/Web/API/TouchEvent/targetTouches)

  It also includes the inherited [`UIEvent`](https://developer.mozilla.org/en-US/docs/Web/API/UIEvent) properties:

  * [`detail`](https://developer.mozilla.org/en-US/docs/Web/API/UIEvent/detail)
  * [`view`](https://developer.mozilla.org/en-US/docs/Web/API/UIEvent/view)

---

### `TransitionEvent` handler function 

An event handler type for the CSS transition events.

```js
 console.log('onTransitionEnd')}
/>
```

#### Parameters 

* `e`: A [React event object](#react-event-object) with these extra [`TransitionEvent`](https://developer.mozilla.org/en-US/docs/Web/API/TransitionEvent) properties:
  * [`elapsedTime`](https://developer.mozilla.org/en-US/docs/Web/API/TransitionEvent/elapsedTime)
  * [`propertyName`](https://developer.mozilla.org/en-US/docs/Web/API/TransitionEvent/propertyName)
  * [`pseudoElement`](https://developer.mozilla.org/en-US/docs/Web/API/TransitionEvent/pseudoElement)

---

### `UIEvent` handler function 

An event handler type for generic UI events.

```js
 console.log('onScroll')}
/>
```

#### Parameters 

* `e`: A [React event object](#react-event-object) with these extra [`UIEvent`](https://developer.mozilla.org/en-US/docs/Web/API/UIEvent) properties:
  * [`detail`](https://developer.mozilla.org/en-US/docs/Web/API/UIEvent/detail)
  * [`view`](https://developer.mozilla.org/en-US/docs/Web/API/UIEvent/view)

---

### `WheelEvent` handler function 

An event handler type for the `onWheel` event.

```js
 console.log('onWheel')}
/>
```

#### Parameters 

* `e`: A [React event object](#react-event-object) with these extra [`WheelEvent`](https://developer.mozilla.org/en-US/docs/Web/API/WheelEvent) properties:
  * [`deltaMode`](https://developer.mozilla.org/en-US/docs/Web/API/WheelEvent/deltaMode)
  * [`deltaX`](https://developer.mozilla.org/en-US/docs/Web/API/WheelEvent/deltaX)
  * [`deltaY`](https://developer.mozilla.org/en-US/docs/Web/API/WheelEvent/deltaY)
  * [`deltaZ`](https://developer.mozilla.org/en-US/docs/Web/API/WheelEvent/deltaZ)

  It also includes the inherited [`MouseEvent`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent) properties:

  * [`altKey`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/altKey)
  * [`button`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/button)
  * [`buttons`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/buttons)
  * [`ctrlKey`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/ctrlKey)
  * [`clientX`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/clientX)
  * [`clientY`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/clientY)
  * [`getModifierState(key)`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/getModifierState)
  * [`metaKey`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/metaKey)
  * [`movementX`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/movementX)
  * [`movementY`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/movementY)
  * [`pageX`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/pageX)
  * [`pageY`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/pageY)
  * [`relatedTarget`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/relatedTarget)
  * [`screenX`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/screenX)
  * [`screenY`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/screenY)
  * [`shiftKey`](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/shiftKey)

  It also includes the inherited [`UIEvent`](https://developer.mozilla.org/en-US/docs/Web/API/UIEvent) properties:

  * [`detail`](https://developer.mozilla.org/en-US/docs/Web/API/UIEvent/detail)
  * [`view`](https://developer.mozilla.org/en-US/docs/Web/API/UIEvent/view)

---

## Usage 

### Applying CSS styles 

In React, you specify a CSS class with [`className`.](https://developer.mozilla.org/en-US/docs/Web/API/Element/className) It works like the `class` attribute in HTML:

```js

```

Then you write the CSS rules for it in a separate CSS file:

```css
/* In your CSS */
.avatar 
```

React does not prescribe how you add CSS files. In the simplest case, you'll add a [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/link) tag to your HTML. If you use a build tool or a framework, consult its documentation to learn how to add a CSS file to your project.

Sometimes, the style values depend on data. Use the `style` attribute to pass some styles dynamically:

```js 

```

In the above example, `style={}` is not a special syntax, but a regular `` object inside the `style=` [JSX curly braces.](/learn/javascript-in-jsx-with-curly-braces) We recommend only using the `style` attribute when your styles depend on JavaScript variables.

---

### Manipulating a DOM node with a ref 

Sometimes, you'll need to get the browser DOM node associated with a tag in JSX. For example, if you want to focus an `` when a button is clicked, you need to call [`focus()`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/focus) on the browser `` DOM node.

To obtain the browser DOM node for a tag, [declare a ref](/reference/react/useRef) and pass it as the `ref` attribute to that tag:

```js 
import  from 'react';

export default function Form() {
  const inputRef = useRef(null);
  // ...
  return (
    
    // ...
```

React will put the DOM node into `inputRef.current` after it's been rendered to the screen.

Read more about [manipulating DOM with refs](/learn/manipulating-the-dom-with-refs) and [check out more examples.](/reference/react/useRef#usage)

For more advanced use cases, the `ref` attribute also accepts a [callback function.](#ref-callback)

---

### Dangerously setting the inner HTML 

You can pass a raw HTML string to an element like so:

```js
const markup = ;
return ;
```

**This is dangerous. As with the underlying DOM [`innerHTML`](https://developer.mozilla.org/en-US/docs/Web/API/Element/innerHTML) property, you must exercise extreme caution! Unless the markup is coming from a completely trusted source, it is trivial to introduce an [XSS](https://en.wikipedia.org/wiki/Cross-site_scripting) vulnerability this way.**

For example, if you use a Markdown library that converts Markdown to HTML, you trust that its parser doesn't contain bugs, and the user only sees their own input, you can display the resulting HTML like this:

The `` object should be created as close to where the HTML is generated as possible, like the above example does in the `renderMarkdownToHTML` function. This ensures that all raw HTML being used in your code is explicitly marked as such, and that only variables that you expect to contain HTML are passed to `dangerouslySetInnerHTML`. It is not recommended to create the object inline like ``.

To see why rendering arbitrary HTML is dangerous, replace the code above with this:

```js 
const post = ;

export default function MarkdownPreview() {
  // 🔴 SECURITY HOLE: passing untrusted input to dangerouslySetInnerHTML
  const markup = ;
  return ;
}
```

The code embedded in the HTML will run. A hacker could use this security hole to steal user information or to perform actions on their behalf. **Only use `dangerouslySetInnerHTML` with trusted and sanitized data.**

---

### Handling mouse events 

This example shows some common [mouse events](#mouseevent-handler) and when they fire.

---

### Handling pointer events 

This example shows some common [pointer events](#pointerevent-handler) and when they fire.

---

### Handling focus events 

In React, [focus events](#focusevent-handler) bubble. You can use the `currentTarget` and `relatedTarget` to differentiate if the focusing or blurring events originated from outside of the parent element. The example shows how to detect focusing a child, focusing the parent element, and how to detect focus entering or leaving the whole subtree.

---

### Handling keyboard events 

This example shows some common [keyboard events](#keyboardevent-handler) and when they fire.

