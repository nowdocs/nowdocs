---
title: Responding to Events
---

## Adding event handlers 

To add an event handler, you will first define a function and then [pass it as a prop](/learn/passing-props-to-a-component) to the appropriate JSX tag. For example, here is a button that doesn't do anything yet:

You can make it show a message when a user clicks by following these three steps:

1. Declare a function called `handleClick` *inside* your `Button` component.
2. Implement the logic inside that function (use `alert` to show the message).
3. Add `onClick=` to the `` JSX.

You defined the `handleClick` function and then [passed it as a prop](/learn/passing-props-to-a-component) to ``.  `handleClick` is an **event handler.** Event handler functions:

* Are usually defined *inside* your components.
* Have names that start with `handle`, followed by the name of the event.

By convention, it is common to name event handlers as `handle` followed by the event name. You'll often see `onClick=`, `onMouseEnter=`, and so on.

Alternatively, you can define an event handler inline in the JSX:

```jsx

```

Or, more concisely, using an arrow function:

```jsx
 }>
```

All of these styles are equivalent. Inline event handlers are convenient for short functions.

### Reading props in event handlers 

Because event handlers are declared inside of a component, they have access to the component's props. Here is a button that, when clicked, shows an alert with its `message` prop:

      
    
  );
}
```

```css
button 
```

This lets these two buttons show different messages. Try changing the messages passed to them.

### Passing event handlers as props 

Often you'll want the parent component to specify a child's event handler. Consider buttons: depending on where you're using a `Button` component, you might want to execute a different function—perhaps one plays a movie and another uploads an image.

To do this, pass a prop the component receives from its parent as the event handler like so:

  );
}

function UploadButton() 

export default function Toolbar() {
  return (
    
      

Here, the `Toolbar` component renders a `PlayButton` and an `UploadButton`:

- `PlayButton` passes `handlePlayClick` as the `onClick` prop to the `Button` inside.
- `UploadButton` passes `() => alert('Uploading!')` as the `onClick` prop to the `Button` inside.

Finally, your `Button` component accepts a prop called `onClick`. It passes that prop directly to the built-in browser `` with `onClick=`. This tells React to call the passed function on click.

If you use a [design system](https://uxdesign.cc/everything-you-need-to-know-about-design-systems-54b109851969), it's common for components like buttons to contain styling but not specify behavior. Instead, components like `PlayButton` and `UploadButton` will pass event handlers down.

### Naming event handler props 

Built-in components like `` and `` only support [browser event names](/reference/react-dom/components/common#common-props) like `onClick`. However, when you're building your own components, you can name their event handler props any way that you like.

By convention, event handler props should start with `on`, followed by a capital letter.

For example, the `Button` component's `onClick` prop could have been called `onSmash`:

      
    
  );
}
```

```css
button 
```

In this example, `` shows that the browser `` (lowercase) still needs a prop called `onClick`, but the prop name received by your custom `Button` component is up to you!

When your component supports multiple interactions, you might name event handler props for app-specific concepts. For example, this `Toolbar` component receives `onPlayMovie` and `onUploadImage` event handlers:

      
    
  );
}

function Button() {
  return (
    
      
    
  );
}
```

```css
button 
```

Notice how the `App` component does not need to know *what* `Toolbar` will do with `onPlayMovie` or `onUploadImage`. That's an implementation detail of the `Toolbar`. Here, `Toolbar` passes them down as `onClick` handlers to its `Button`s, but it could later also trigger them on a keyboard shortcut. Naming props after app-specific interactions like `onPlayMovie` gives you the flexibility to change how they're used later.

## Event propagation 

Event handlers will also catch events from any children your component might have. We say that an event "bubbles" or "propagates" up the tree: it starts with where the event happened, and then goes up the tree.

This `` contains two buttons. Both the `` *and* each button have their own `onClick` handlers. Which handlers do you think will fire when you click a button?

If you click on either button, its `onClick` will run first, followed by the parent ``'s `onClick`. So two messages will appear. If you click the toolbar itself, only the parent ``'s `onClick` will run.

### Stopping propagation 

Event handlers receive an **event object** as their only argument. By convention, it's usually called `e`, which stands for "event". You can use this object to read information about the event.

That event object also lets you stop the propagation. If you want to prevent an event from reaching parent components, you need to call `e.stopPropagation()` like this `Button` component does:

      
    
  );
}
```

```css
.Toolbar 
button 
```

When you click on a button:

1. React calls the `onClick` handler passed to ``.
2. That handler, defined in `Button`, does the following:
   * Calls `e.stopPropagation()`, preventing the event from bubbling further.
   * Calls the `onClick` function, which is a prop passed from the `Toolbar` component.
3. That function, defined in the `Toolbar` component, displays the button's own alert.
4. Since the propagation was stopped, the parent ``'s `onClick` handler does *not* run.

As a result of `e.stopPropagation()`, clicking on the buttons now only shows a single alert (from the ``) rather than the two of them (from the `` and the parent toolbar ``). Clicking a button is not the same thing as clicking the surrounding toolbar, so stopping the propagation makes sense for this UI.

### Passing handlers as alternative to propagation 

Notice how this click handler runs a line of code _and then_ calls the `onClick` prop passed by the parent:

```js 
function Button() {
  return (
     }>
      
    
  );
}
```

You could add more code to this handler before calling the parent `onClick` event handler, too. This pattern provides an *alternative* to propagation. It lets the child component handle the event, while also letting the parent component specify some additional behavior. Unlike propagation, it's not automatic. But the benefit of this pattern is that you can clearly follow the whole chain of code that executes as a result of some event.

If you rely on propagation and it's difficult to trace which handlers execute and why, try this approach instead.

### Preventing default behavior 

Some browser events have default behavior associated with them. For example, a `` submit event, which happens when a button inside of it is clicked, will reload the whole page by default:

You can call `e.preventDefault()` on the event object to stop this from happening:

Don't confuse `e.stopPropagation()` and `e.preventDefault()`. They are both useful, but are unrelated:

* [`e.stopPropagation()`](https://developer.mozilla.org/docs/Web/API/Event/stopPropagation) stops the event handlers attached to the tags above from firing.
* [`e.preventDefault()` ](https://developer.mozilla.org/docs/Web/API/Event/preventDefault) prevents the default browser behavior for the few events that have it.

## Can event handlers have side effects? 

Absolutely! Event handlers are the best place for side effects.

Unlike rendering functions, event handlers don't need to be [pure](/learn/keeping-components-pure), so it's a great place to *change* something—for example, change an input's value in response to typing, or change a list in response to a button press. However, in order to change some information, you first need some way to store it. In React, this is done by using [state, a component's memory.](/learn/state-a-components-memory) You will learn all about it on the next page.

Alternatively, you could wrap the call into another function, like ` handleClick()}>`:

#### Wire up the events 

This `ColorSwitch` component renders a button. It's supposed to change the page color. Wire it up to the `onChangeColor` event handler prop it receives from the parent so that clicking the button changes the color.

After you do this, notice that clicking the button also increments the page click counter. Your colleague who wrote the parent component insists that `onChangeColor` does not increment any counters. What else might be happening? Fix it so that clicking the button *only* changes the color, and does _not_ increment the counter.

