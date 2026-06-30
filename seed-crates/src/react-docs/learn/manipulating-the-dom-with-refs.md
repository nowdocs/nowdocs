---
title: 'Manipulating the DOM with Refs'
---

## Getting a ref to the node 

To access a DOM node managed by React, first, import the `useRef` Hook:

```js
import  from 'react';
```

Then, use it to declare a ref inside your component:

```js
const myRef = useRef(null);
```

Finally, pass your ref as the `ref` attribute to the JSX tag for which you want to get the DOM node:

```js

```

The `useRef` Hook returns an object with a single property called `current`. Initially, `myRef.current` will be `null`. When React creates a DOM node for this ``, React will put a reference to this node into `myRef.current`. You can then access this DOM node from your [event handlers](/learn/responding-to-events) and use the built-in [browser APIs](https://developer.mozilla.org/docs/Web/API/Element) defined on it.

```js
// You can use any browser APIs, for example:
myRef.current.scrollIntoView();
```

### Example: Focusing a text input 

In this example, clicking the button will focus the input:

To implement this:

1. Declare `inputRef` with the `useRef` Hook.
2. Pass it as ``. This tells React to **put this ``'s DOM node into `inputRef.current`.**
3. In the `handleClick` function, read the input DOM node from `inputRef.current` and call [`focus()`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/focus) on it with `inputRef.current.focus()`.
4. Pass the `handleClick` event handler to `` with `onClick`.

While DOM manipulation is the most common use case for refs, the `useRef` Hook can be used for storing other things outside React, like timer IDs. Similarly to state, refs remain between renders. Refs are like state variables that don't trigger re-renders when you set them. Read about refs in [Referencing Values with Refs.](/learn/referencing-values-with-refs)

### Example: Scrolling to an element 

You can have more than a single ref in a component. In this example, there is a carousel of three images. Each button centers an image by calling the browser [`scrollIntoView()`](https://developer.mozilla.org/en-US/docs/Web/API/Element/scrollIntoView) method on the corresponding DOM node:

In this example, `itemsRef` doesn't hold a single DOM node. Instead, it holds a [Map](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Map) from item ID to a DOM node. ([Refs can hold any values!](/learn/referencing-values-with-refs)) The [`ref` callback](/reference/react-dom/components/common#ref-callback) on every list item takes care to update the Map:

```js
 {
    const map = getMap();
    // Add to the Map
    map.set(cat, node);

    return () => ;
  }}
>
```

This lets you read individual DOM nodes from the Map later.

## Accessing another component's DOM nodes 

You can pass refs from parent component to child components [just like any other prop](/learn/passing-props-to-a-component).

```js 
import  from 'react';

function MyInput() 

function MyForm() {
  const inputRef = useRef(null);
  return 

Here, `realInputRef` inside `MyInput` holds the actual input DOM node. However, [`useImperativeHandle`](/reference/react/useImperativeHandle) instructs React to provide your own special object as the value of a ref to the parent component. So `inputRef.current` inside the `Form` component will only have the `focus` method. In this case, the ref "handle" is not the DOM node, but the custom object you create inside [`useImperativeHandle`](/reference/react/useImperativeHandle) call.

## When React attaches the refs 

In React, every update is split in [two phases](/learn/render-and-commit#step-3-react-commits-changes-to-the-dom):

* During **render,** React calls your components to figure out what should be on the screen.
* During **commit,** React applies changes to the DOM.

In general, you [don't want](/learn/referencing-values-with-refs#best-practices-for-refs) to access refs during rendering. That goes for refs holding DOM nodes as well. During the first render, the DOM nodes have not yet been created, so `ref.current` will be `null`. And during the rendering of updates, the DOM nodes haven't been updated yet. So it's too early to read them.

React sets `ref.current` during the commit. Before updating the DOM, React sets the affected `ref.current` values to `null`. After updating the DOM, React immediately sets them to the corresponding DOM nodes.

**Usually, you will access refs from event handlers.** If you want to do something with a ref, but there is no particular event to do it in, you might need an Effect. We will discuss Effects on the next pages.

The issue is with these two lines:

```js
setTodos([ ...todos, newTodo]);
listRef.current.lastChild.scrollIntoView();
```

In React, [state updates are queued.](/learn/queueing-a-series-of-state-updates) Usually, this is what you want. However, here it causes a problem because `setTodos` does not immediately update the DOM. So the time you scroll the list to its last element, the todo has not yet been added. This is why scrolling always "lags behind" by one item.

To fix this issue, you can force React to update ("flush") the DOM synchronously. To do this, import `flushSync` from `react-dom` and **wrap the state update** into a `flushSync` call:

```js
flushSync(() => );
listRef.current.lastChild.scrollIntoView();
```

This will instruct React to update the DOM synchronously right after the code wrapped in `flushSync` executes. As a result, the last todo will already be in the DOM by the time you try to scroll to it:

## Best practices for DOM manipulation with refs 

Refs are an escape hatch. You should only use them when you have to "step outside React". Common examples of this include managing focus, scroll position, or calling browser APIs that React does not expose.

If you stick to non-destructive actions like focusing and scrolling, you shouldn't encounter any problems. However, if you try to **modify** the DOM manually, you can risk conflicting with the changes React is making.

To illustrate this problem, this example includes a welcome message and two buttons. The first button toggles its presence using [conditional rendering](/learn/conditional-rendering) and [state](/learn/state-a-components-memory), as you would usually do in React. The second button uses the [`remove()` DOM API](https://developer.mozilla.org/en-US/docs/Web/API/Element/remove) to forcefully remove it from the DOM outside of React's control.

Try pressing "Toggle with setState" a few times. The message should disappear and appear again. Then press "Remove from the DOM". This will forcefully remove it. Finally, press "Toggle with setState":

After you've manually removed the DOM element, trying to use `setState` to show it again will lead to a crash. This is because you've changed the DOM, and React doesn't know how to continue managing it correctly.

**Avoid changing DOM nodes managed by React.** Modifying, adding children to, or removing children from elements that are managed by React can lead to inconsistent visual results or crashes like above.

However, this doesn't mean that you can't do it at all. It requires caution. **You can safely modify parts of the DOM that React has _no reason_ to update.** For example, if some `` is always empty in the JSX, React won't have a reason to touch its children list. Therefore, it is safe to manually add or remove elements there.

For an extra challenge, keep the "Play" button in sync with whether the video is playing even if the user right-clicks the video and plays it using the built-in browser media controls. You might want to listen to `onPlay` and `onPause` on the video to do that.

In order to handle the built-in browser controls, you can add `onPlay` and `onPause` handlers to the `` element and call `setIsPlaying` from them. This way, if the user plays the video using the browser controls, the state will adjust accordingly.

#### Focus the search field 

Make it so that clicking the "Search" button puts focus into the field.

#### Scrolling an image carousel 

This image carousel has a "Next" button that switches the active image. Make the gallery scroll horizontally to the active image on click. You will want to call [`scrollIntoView()`](https://developer.mozilla.org/en-US/docs/Web/API/Element/scrollIntoView) on the DOM node of the active image:

```js
node.scrollIntoView();
```

#### Focus the search field with separate components 

Make it so that clicking the "Search" button puts focus into the field. Note that each component is defined in a separate file and shouldn't be moved out of it. How do you connect them together?

