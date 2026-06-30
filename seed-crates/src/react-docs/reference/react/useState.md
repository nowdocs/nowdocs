---
title: useState
---

 of this state variable, initially set to the  you provided.
2. The  that lets you change it to any other value in response to interaction.

To update what’s on the screen, call the `set` function with some next state:

```js [[2, 2, "setName"]]
function handleClick() 
```

React will store the next state, render your component again with the new values, and update the UI.

---

### Updating state based on the previous state 

Suppose the `age` is `42`. This handler calls `setAge(age + 1)` three times:

```js
function handleClick() 
```

However, after one click, `age` will only be `43` rather than `45`! This is because calling the `set` function [does not update](/learn/state-as-a-snapshot) the `age` state variable in the already running code. So each `setAge(age + 1)` call becomes `setAge(43)`.

To solve this problem, **you may pass an *updater function*** to `setAge` instead of the next state:

```js [[1, 2, "a", 0], [2, 2, "a + 1"], [1, 3, "a", 0], [2, 3, "a + 1"], [1, 4, "a", 0], [2, 4, "a + 1"]]
function handleClick() 
```

Here, `a => a + 1` is your updater function. It takes the  and calculates the  from it.

React puts your updater functions in a [queue.](/learn/queueing-a-series-of-state-updates) Then, during the next render, it will call them in the same order:

1. `a => a + 1` will receive `42` as the pending state and return `43` as the next state.
1. `a => a + 1` will receive `43` as the pending state and return `44` as the next state.
1. `a => a + 1` will receive `44` as the pending state and return `45` as the next state.

There are no other queued updates, so React will store `45` as the current state in the end.

By convention, it's common to name the pending state argument for the first letter of the state variable name, like `a` for `age`. However, you may also call it like `prevAge` or something else that you find clearer.

React may [call your updaters twice](#my-initializer-or-updater-function-runs-twice) in development to verify that they are [pure.](/learn/keeping-components-pure)

---

### Updating objects and arrays in state 

You can put objects and arrays into state. In React, state is considered read-only, so **you should *replace* it rather than *mutate* your existing objects**. For example, if you have a `form` object in state, don't mutate it:

```js
// 🚩 Don't mutate an object in state like this:
form.firstName = 'Taylor';
```

Instead, replace the whole object by creating a new one:

```js
// ✅ Replace state with a new object
setForm();
```

Read [updating objects in state](/learn/updating-objects-in-state) and [updating arrays in state](/learn/updating-arrays-in-state) to learn more.

---

### Avoiding recreating the initial state 

React saves the initial state once and ignores it on the next renders.

```js
function TodoList() {
  const [todos, setTodos] = useState(createInitialTodos());
  // ...
```

Although the result of `createInitialTodos()` is only used for the initial render, you're still calling this function on every render. This can be wasteful if it's creating large arrays or performing expensive calculations.

To solve this, you may **pass it as an _initializer_ function** to `useState` instead:

```js
function TodoList() {
  const [todos, setTodos] = useState(createInitialTodos);
  // ...
```

Notice that you’re passing `createInitialTodos`, which is the *function itself*, and not `createInitialTodos()`, which is the result of calling it. If you pass a function to `useState`, React will only call it during initialization.

React may [call your initializers twice](#my-initializer-or-updater-function-runs-twice) in development to verify that they are [pure.](/learn/keeping-components-pure)

---

### Resetting state with a key 

You'll often encounter the `key` attribute when [rendering lists.](/learn/rendering-lists) However, it also serves another purpose.

You can **reset a component's state by passing a different `key` to a component.** In this example, the Reset button changes the `version` state variable, which we pass as a `key` to the `Form`. When the `key` changes, React re-creates the `Form` component (and all of its children) from scratch, so its state gets reset.

Read [preserving and resetting state](/learn/preserving-and-resetting-state) to learn more.

---

### Storing information from previous renders 

Usually, you will update state in event handlers. However, in rare cases you might want to adjust state in response to rendering -- for example, you might want to change a state variable when a prop changes.

In most cases, you don't need this:

* **If the value you need can be computed entirely from the current props or other state, [remove that redundant state altogether.](/learn/choosing-the-state-structure#avoid-redundant-state)** If you're worried about recomputing too often, the [`useMemo` Hook](/reference/react/useMemo) can help.
* If you want to reset the entire component tree's state, [pass a different `key` to your component.](#resetting-state-with-a-key)
* If you can, update all the relevant state in the event handlers.

In the rare case that none of these apply, there is a pattern you can use to update state based on the values that have been rendered so far, by calling a `set` function while your component is rendering.

Here's an example. This `CountLabel` component displays the `count` prop passed to it:

```js src/CountLabel.js
export default function CountLabel() {
  return 
}
```

Say you want to show whether the counter has *increased or decreased* since the last change. The `count` prop doesn't tell you this -- you need to keep track of its previous value. Add the `prevCount` state variable to track it. Add another state variable called `trend` to hold whether the count has increased or decreased. Compare `prevCount` with `count`, and if they're not equal, update both `prevCount` and `trend`. Now you can show both the current count prop and *how it has changed since the last render*.

Note that if you call a `set` function while rendering, it must be inside a condition like `prevCount !== count`, and there must be a call like `setPrevCount(count)` inside of the condition. Otherwise, your component would re-render in a loop until it crashes. Also, you can only update the state of the *currently rendering* component like this. Calling the `set` function of *another* component during rendering is an error. Finally, your `set` call should still [update state without mutation](#updating-objects-and-arrays-in-state) -- this doesn't mean you can break other rules of [pure functions.](/learn/keeping-components-pure)

This pattern can be hard to understand and is usually best avoided. However, it's better than updating state in an effect. When you call the `set` function during render, React will re-render that component immediately after your component exits with a `return` statement, and before rendering the children. This way, children don't need to render twice. The rest of your component function will still execute (and the result will be thrown away). If your condition is below all the Hook calls, you may add an early `return;` to restart rendering earlier.

---

## Troubleshooting 

### I've updated the state, but logging gives me the old value 

Calling the `set` function **does not change state in the running code**:

```js 
function handleClick() {
  console.log(count);  // 0

  setCount(count + 1); // Request a re-render with 1
  console.log(count);  // Still 0!

  setTimeout(() => , 5000);
}
```

This is because [states behaves like a snapshot.](/learn/state-as-a-snapshot) Updating state requests another render with the new state value, but does not affect the `count` JavaScript variable in your already-running event handler.

If you need to use the next state, you can save it in a variable before passing it to the `set` function:

```js
const nextCount = count + 1;
setCount(nextCount);

console.log(count);     // 0
console.log(nextCount); // 1
```

---

### I've updated the state, but the screen doesn't update 

React will **ignore your update if the next state is equal to the previous state,** as determined by an [`Object.is`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/is) comparison. This usually happens when you change an object or an array in state directly:

```js
obj.x = 10;  // 🚩 Wrong: mutating existing object
setObj(obj); // 🚩 Doesn't do anything
```

You mutated an existing `obj` object and passed it back to `setObj`, so React ignored the update. To fix this, you need to ensure that you're always [_replacing_ objects and arrays in state instead of _mutating_ them](#updating-objects-and-arrays-in-state):

```js
// ✅ Correct: creating a new object
setObj();
```

---

### I'm getting an error: "Too many re-renders" 

You might get an error that says: `Too many re-renders. React limits the number of renders to prevent an infinite loop.` Typically, this means that you're unconditionally setting state *during render*, so your component enters a loop: render, set state (which causes a render), render, set state (which causes a render), and so on. Very often, this is caused by a mistake in specifying an event handler:

```js 
// 🚩 Wrong: calls the handler during render
return Click me

// ✅ Correct: passes down the event handler
return Click me

// ✅ Correct: passes down an inline function
return  handleClick(e)}>Click me
```

If you can't find the cause of this error, click on the arrow next to the error in the console and look through the JavaScript stack to find the specific `set` function call responsible for the error.

---

### My initializer or updater function runs twice 

In [Strict Mode](/reference/react/StrictMode), React will call some of your functions twice instead of once:

```js 
function TodoList() {
  // This component function will run twice for every render.

  const [todos, setTodos] = useState(() => );

  function handleClick() {
    setTodos(prevTodos => );
  }
  // ...
```

This is expected and shouldn't break your code.

This **development-only** behavior helps you [keep components pure.](/learn/keeping-components-pure) React uses the result of one of the calls, and ignores the result of the other call. As long as your component, initializer, and updater functions are pure, this shouldn't affect your logic. However, if they are accidentally impure, this helps you notice the mistakes.

For example, this impure updater function mutates an array in state:

```js 
setTodos(prevTodos => );
```

Because React calls your updater function twice, you'll see the todo was added twice, so you'll know that there is a mistake. In this example, you can fix the mistake by [replacing the array instead of mutating it](#updating-objects-and-arrays-in-state):

```js 
setTodos(prevTodos => );
```

Now that this updater function is pure, calling it an extra time doesn't make a difference in behavior. This is why React calling it twice helps you find mistakes. **Only component, initializer, and updater functions need to be pure.** Event handlers don't need to be pure, so React will never call your event handlers twice.

Read [keeping components pure](/learn/keeping-components-pure) to learn more.

---

### I'm trying to set state to a function, but it gets called instead 

You can't put a function into state like this:

```js
const [fn, setFn] = useState(someFunction);

function handleClick() 
```

Because you're passing a function, React assumes that `someFunction` is an [initializer function](#avoiding-recreating-the-initial-state), and that `someOtherFunction` is an [updater function](#updating-state-based-on-the-previous-state), so it tries to call them and store the result. To actually *store* a function, you have to put `() =>` before them in both cases. Then React will store the functions you pass.

```js 
const [fn, setFn] = useState(() => someFunction);

function handleClick() 
```
