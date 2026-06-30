---
title: 'Referencing Values with Refs'
---

## Adding a ref to your component 

You can add a ref to your component by importing the `useRef` Hook from React:

```js
import  from 'react';
```

Inside your component, call the `useRef` Hook and pass the initial value that you want to reference as the only argument. For example, here is a ref to the value `0`:

```js
const ref = useRef(0);
```

`useRef` returns an object like this:

```js

```

The ref points to a number, but, like [state](/learn/state-a-components-memory), you could point to anything: a string, an object, or even a function. Unlike state, ref is a plain JavaScript object with the `current` property that you can read and modify.

Note that **the component doesn't re-render with every increment.** Like state, refs are retained by React between re-renders. However, setting state re-renders a component. Changing a ref does not!

## Example: building a stopwatch 

You can combine refs and state in a single component. For example, let's make a stopwatch that the user can start or stop by pressing a button. In order to display how much time has passed since the user pressed "Start", you will need to keep track of when the Start button was pressed and what the current time is. **This information is used for rendering, so you'll keep it in state:**

```js
const [startTime, setStartTime] = useState(null);
const [now, setNow] = useState(null);
```

When the user presses "Start", you'll use [`setInterval`](https://developer.mozilla.org/docs/Web/API/setInterval) in order to update the time every 10 milliseconds:

When the "Stop" button is pressed, you need to cancel the existing interval so that it stops updating the `now` state variable. You can do this by calling [`clearInterval`](https://developer.mozilla.org/en-US/docs/Web/API/clearInterval), but you need to give it the interval ID that was previously returned by the `setInterval` call when the user pressed Start. You need to keep the interval ID somewhere. **Since the interval ID is not used for rendering, you can keep it in a ref:**

When a piece of information is used for rendering, keep it in state. When a piece of information is only needed by event handlers and changing it doesn't require a re-render, using a ref may be more efficient.

## Differences between refs and state 

Perhaps you're thinking refs seem less "strict" than state—you can mutate them instead of always having to use a state setting function, for instance. But in most cases, you'll want to use state. Refs are an "escape hatch" you won't need often. Here's how state and refs compare:

| refs                                                                                  | state                                                                                                                     |
| ------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| `useRef(initialValue)` returns ``                            | `useState(initialValue)` returns the current value of a state variable and a state setter function ( `[value, setValue]`) |
| Doesn't trigger re-render when you change it.                                         | Triggers re-render when you change it.                                                                                    |
| Mutable—you can modify and update `current`'s value outside of the rendering process. | "Immutable"—you must use the state setting function to modify state variables to queue a re-render.                       |
| You shouldn't read (or write) the `current` value during rendering. | You can read state at any time. However, each render has its own [snapshot](/learn/state-as-a-snapshot) of state which does not change.

Here is a counter button that's implemented with state:

Because the `count` value is displayed, it makes sense to use a state value for it. When the counter's value is set with `setCount()`, React re-renders the component and the screen updates to reflect the new count.

If you tried to implement this with a ref, React would never re-render the component, so you'd never see the count change! See how clicking this button **does not update its text**:

This is why reading `ref.current` during render leads to unreliable code. If you need that, use state instead.

## When to use refs 

Typically, you will use a ref when your component needs to "step outside" React and communicate with external APIs—often a browser API that won't impact the appearance of the component. Here are a few of these rare situations:

- Storing [timeout IDs](https://developer.mozilla.org/docs/Web/API/setTimeout)
- Storing and manipulating [DOM elements](https://developer.mozilla.org/docs/Web/API/Element), which we cover on [the next page](/learn/manipulating-the-dom-with-refs)
- Storing other objects that aren't necessary to calculate the JSX.

If your component needs to store some value, but it doesn't impact the rendering logic, choose refs.

## Best practices for refs 

Following these principles will make your components more predictable:

- **Treat refs as an escape hatch.** Refs are useful when you work with external systems or browser APIs. If much of your application logic and data flow relies on refs, you might want to rethink your approach.
- **Don't read or write `ref.current` during rendering.** If some information is needed during rendering, use [state](/learn/state-a-components-memory) instead. Since React doesn't know when `ref.current` changes, even reading it while rendering makes your component's behavior difficult to predict. (The only exception to this is code like `if (!ref.current) ref.current = new Thing()` which only sets the ref once during the first render.)

Limitations of React state don't apply to refs. For example, state acts like a [snapshot for every render](/learn/state-as-a-snapshot) and [doesn't update synchronously.](/learn/queueing-a-series-of-state-updates) But when you mutate the current value of a ref, it changes immediately:

```js
ref.current = 5;
console.log(ref.current); // 5
```

This is because **the ref itself is a regular JavaScript object,** and so it behaves like one.

You also don't need to worry about [avoiding mutation](/learn/updating-objects-in-state) when you work with a ref. As long as the object you're mutating isn't used for rendering, React doesn't care what you do with the ref or its contents.

## Refs and the DOM 

You can point a ref to any value. However, the most common use case for a ref is to access a DOM element. For example, this is handy if you want to focus an input programmatically. When you pass a ref to a `ref` attribute in JSX, like ``, React will put the corresponding DOM element into `myRef.current`. Once the element is removed from the DOM, React will update `myRef.current` to be `null`. You can read more about this in [Manipulating the DOM with Refs.](/learn/manipulating-the-dom-with-refs)

#### Fix a component failing to re-render 

This button is supposed to toggle between showing "On" and "Off". However, it always shows "Off". What is wrong with this code? Fix it.

#### Fix debouncing 

In this example, all button click handlers are ["debounced".](https://kettanaito.com/blog/debounce-vs-throttle) To see what this means, press one of the buttons. Notice how the message appears a second later. If you press the button while waiting for the message, the timer will reset. So if you keep clicking the same button fast many times, the message won't appear until a second *after* you stop clicking. Debouncing lets you delay some action until the user "stops doing things".

This example works, but not quite as intended. The buttons are not independent. To see the problem, click one of the buttons, and then immediately click another button. You'd expect that after a delay, you would see both button's messages. But only the last button's message shows up. The first button's message gets lost.

Why are the buttons interfering with each other? Find and fix the issue.

      
      
    </>
  )
}
```

```css
button 
```

      
      
    </>
  )
}
```

```css
button 
```

#### Read the latest state 

In this example, after you press "Send", there is a small delay before the message is shown. Type "hello", press Send, and then quickly edit the input again. Despite your edits, the alert would still show "hello" (which was the value of state [at the time](/learn/state-as-a-snapshot#state-over-time) the button was clicked).

Usually, this behavior is what you want in an app. However, there may be occasional cases where you want some asynchronous code to read the *latest* version of some state. Can you think of a way to make the alert show the *current* input text rather than what it was at the time of the click?

