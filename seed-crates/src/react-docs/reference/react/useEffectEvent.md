---
title: useEffectEvent
---

---

## Usage 

### Using an event in an Effect 

Call `useEffectEvent` at the top level of your component to create an *Effect Event*:

```js [[1, 1, "onConnected"]]
const onConnected = useEffectEvent(() => {
  if (!muted) 
});
```

`useEffectEvent` accepts an `event callback` and returns an . The Effect Event is a function that can be called inside of Effects without re-connecting the Effect:

```js [[1, 3, "onConnected"]]
useEffect(() => {
  const connection = createConnection(roomId);
  connection.on('connected', onConnected);
  connection.connect();
  return () => 
}, [roomId]);
```

Since `onConnected` is an , `muted` and `onConnect` are not in the Effect dependencies.

---

### Using a timer with latest values 

When you use `setInterval` or `setTimeout` in an Effect, you often want to read the latest values from render without restarting the timer whenever those values change.

This counter increments `count` by the current `increment` value every second. The `onTick` Effect Event reads the latest `count` and `increment` without causing the interval to restart:

Try changing the increment value while the timer is running. The counter immediately uses the new increment value, but the timer keeps ticking smoothly without restarting.

---

### Using an event listener with latest values 

When you set up an event listener in an Effect, you often need to read the latest values from render in the callback. Without `useEffectEvent`, you would need to include the values in your dependencies, causing the listener to be removed and re-added on every change.

This example shows a dot that follows the cursor, but only when "Can move" is checked. The `onMove` Effect Event always reads the latest `canMove` value without re-running the Effect:

Toggle the checkbox and move your cursor. The dot responds immediately to the checkbox state, but the event listener is only set up once when the component mounts.

---

### Avoid reconnecting to external systems 

A common use case for `useEffectEvent` is when you want to do something in response to an Effect, but that "something" depends on a value you don't want to react to.

In this example, a chat component connects to a room and shows a notification when connected. The user can mute notifications with a checkbox. However, you don't want to reconnect to the chat room every time the user changes the settings:

Try switching rooms. The chat reconnects and shows a notification. Now mute the notifications. Since `muted` is read inside the Effect Event rather than the Effect, the chat stays connected.

---

### Using Effect Events in custom Hooks 

You can use `useEffectEvent` inside your own custom Hooks. This lets you create reusable Hooks that encapsulate Effects while keeping some values non-reactive:

In this example, `useInterval` is a custom Hook that sets up an interval. The `callback` passed to it is wrapped in an Effect Event, so the interval does not reset even if a new `callback` is passed in every render.

---

## Troubleshooting 

### I'm getting an error: "A function wrapped in useEffectEvent can't be called during rendering" 

This error means you're calling an Effect Event function during the render phase of your component. Effect Events can only be called from inside Effects or other Effect Events.

```js
function MyComponent() {
  const onLog = useEffectEvent(() => );

  // 🔴 Wrong: calling during render
  onLog();

  // ✅ Correct: call from an Effect
  useEffect(() => , []);

  return ;
}
```

If you need to run logic during render, don't wrap it in `useEffectEvent`. Call the logic directly or move it into an Effect.

---

### I'm getting a lint error: "Functions returned from useEffectEvent must not be included in the dependency array" 

If you see a warning like "Functions returned from `useEffectEvent` must not be included in the dependency array", remove the Effect Event from your dependencies:

```js
const onSomething = useEffectEvent(() => );

// 🔴 Wrong: Effect Event in dependencies
useEffect(() => , [onSomething]);

// ✅ Correct: no Effect Event in dependencies
useEffect(() => , []);
```

Effect Events are designed to be called from Effects without being listed as dependencies. The linter enforces this because the function identity is [intentionally not stable](#why-are-effect-events-not-stable). Including it would cause your Effect to re-run on every render.

---

### I'm getting a lint error: "... is a function created with useEffectEvent, and can only be called from Effects" 

If you see a warning like "... is a function created with React Hook `useEffectEvent`, and can only be called from Effects and Effect Events", you're calling the function from the wrong place:

```js
const onSomething = useEffectEvent(() => );

// 🔴 Wrong: calling from event handler
function handleClick() 

// 🔴 Wrong: passing to child component
return ;

// ✅ Correct: calling from Effect
useEffect(() => , []);
```

Effect Events are specifically designed to be used in Effects local to the component they're defined in. If you need a callback for event handlers or to pass to children, use a regular function or `useCallback` instead.