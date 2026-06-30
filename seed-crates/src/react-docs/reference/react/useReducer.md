---
title: useReducer
---

 of this state variable, initially set to the  you provided.
2. The  that lets you change it in response to interaction.

To update what's on the screen, call  with an object representing what the user did, called an *action*:

```js [[2, 2, "dispatch"]]
function handleClick() {
  dispatch();
}
```

React will pass the current state and the action to your . Your reducer will calculate and return the next state. React will store that next state, render your component with it, and update the UI.

`useReducer` is very similar to [`useState`](/reference/react/useState), but it lets you move the state update logic from event handlers into a single function outside of your component. Read more about [choosing between `useState` and `useReducer`.](/learn/extracting-state-logic-into-a-reducer#comparing-usestate-and-usereducer)

---

### Writing the reducer function 

A reducer function is declared like this:

```js
function reducer(state, action) 
```

Then you need to fill in the code that will calculate and return the next state. By convention, it is common to write it as a [`switch` statement.](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/switch) For each `case` in the `switch`, calculate and return some next state.

```js 
function reducer(state, action) {
  switch (action.type) {
    case 'incremented_age': {
      return ;
    }
    case 'changed_name': {
      return ;
    }
  }
  throw Error('Unknown action: ' + action.type);
}
```

Actions can have any shape. By convention, it's common to pass objects with a `type` property identifying the action. It should include the minimal necessary information that the reducer needs to compute the next state.

```js 
function Form() {
  const [state, dispatch] = useReducer(reducer, );

  function handleButtonClick() {
    dispatch();
  }

  function handleInputChange(e) {
    dispatch();
  }
  // ...
```

The action type names are local to your component. [Each action describes a single interaction, even if that leads to multiple changes in data.](/learn/extracting-state-logic-into-a-reducer#writing-reducers-well) The shape of the state is arbitrary, but usually it'll be an object or an array.

Read [extracting state logic into a reducer](/learn/extracting-state-logic-into-a-reducer) to learn more.

---

### Avoiding recreating the initial state 

React saves the initial state once and ignores it on the next renders.

```js
function createInitialState(username) 

function TodoList() {
  const [state, dispatch] = useReducer(reducer, createInitialState(username));
  // ...
```

Although the result of `createInitialState(username)` is only used for the initial render, you're still calling this function on every render. This can be wasteful if it's creating large arrays or performing expensive calculations.

To solve this, you may **pass it as an _initializer_ function** to `useReducer` as the third argument instead:

```js 
function createInitialState(username) 

function TodoList() {
  const [state, dispatch] = useReducer(reducer, username, createInitialState);
  // ...
```

Notice that you’re passing `createInitialState`, which is the *function itself*, and not `createInitialState()`, which is the result of calling it. This way, the initial state does not get re-created after initialization.

In the above example, `createInitialState` takes a `username` argument. If your initializer doesn't need any information to compute the initial state, you may pass `null` as the second argument to `useReducer`.

---

## Troubleshooting 

### I've dispatched an action, but logging gives me the old state value 

Calling the `dispatch` function **does not change state in the running code**:

```js 
function handleClick() {
  console.log(state.age);  // 42

  dispatch(); // Request a re-render with 43
  console.log(state.age);  // Still 42!

  setTimeout(() => , 5000);
}
```

This is because [states behaves like a snapshot.](/learn/state-as-a-snapshot) Updating state requests another render with the new state value, but does not affect the `state` JavaScript variable in your already-running event handler.

If you need to guess the next state value, you can calculate it manually by calling the reducer yourself:

```js
const action = ;
dispatch(action);

const nextState = reducer(state, action);
console.log(state);     // 
console.log(nextState); // 
```

---

### I've dispatched an action, but the screen doesn't update 

React will **ignore your update if the next state is equal to the previous state,** as determined by an [`Object.is`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/is) comparison. This usually happens when you change an object or an array in state directly:

```js 
function reducer(state, action) {
  switch (action.type) {
    case 'incremented_age': 
    case 'changed_name': 
    // ...
  }
}
```

You mutated an existing `state` object and returned it, so React ignored the update. To fix this, you need to ensure that you're always [updating objects in state](/learn/updating-objects-in-state) and [updating arrays in state](/learn/updating-arrays-in-state) instead of mutating them:

```js 
function reducer(state, action) {
  switch (action.type) {
    case 'incremented_age': {
      // ✅ Correct: creating a new object
      return ;
    }
    case 'changed_name': {
      // ✅ Correct: creating a new object
      return ;
    }
    // ...
  }
}
```

---

### A part of my reducer state becomes undefined after dispatching 

Make sure that every `case` branch **copies all of the existing fields** when returning the new state:

```js 
function reducer(state, action) {
  switch (action.type) {
    case 'incremented_age': {
      return ;
    }
    // ...
```

Without `...state` above, the returned next state would only contain the `age` field and nothing else.

---

### My entire reducer state becomes undefined after dispatching 

If your state unexpectedly becomes `undefined`, you're likely forgetting to `return` state in one of the cases, or your action type doesn't match any of the `case` statements. To find why, throw an error outside the `switch`:

```js 
function reducer(state, action) {
  switch (action.type) {
    case 'incremented_age': 
    case 'edited_name': 
  }
  throw Error('Unknown action: ' + action.type);
}
```

You can also use a static type checker like TypeScript to catch such mistakes.

---

### I'm getting an error: "Too many re-renders" 

You might get an error that says: `Too many re-renders. React limits the number of renders to prevent an infinite loop.` Typically, this means that you're unconditionally dispatching an action *during render*, so your component enters a loop: render, dispatch (which causes a render), render, dispatch (which causes a render), and so on. Very often, this is caused by a mistake in specifying an event handler:

```js 
// 🚩 Wrong: calls the handler during render
return Click me

// ✅ Correct: passes down the event handler
return Click me

// ✅ Correct: passes down an inline function
return  handleClick(e)}>Click me
```

If you can't find the cause of this error, click on the arrow next to the error in the console and look through the JavaScript stack to find the specific `dispatch` function call responsible for the error.

---

### My reducer or initializer function runs twice 

In [Strict Mode](/reference/react/StrictMode), React will call your reducer and initializer functions twice. This shouldn't break your code.

This **development-only** behavior helps you [keep components pure.](/learn/keeping-components-pure) React uses the result of one of the calls, and ignores the result of the other call. As long as your component, initializer, and reducer functions are pure, this shouldn't affect your logic. However, if they are accidentally impure, this helps you notice the mistakes.

For example, this impure reducer function mutates an array in state:

```js 
function reducer(state, action) {
  switch (action.type) {
    case 'added_todo': {
      // 🚩 Mistake: mutating state
      state.todos.push();
      return state;
    }
    // ...
  }
}
```

Because React calls your reducer function twice, you'll see the todo was added twice, so you'll know that there is a mistake. In this example, you can fix the mistake by [replacing the array instead of mutating it](/learn/updating-arrays-in-state#adding-to-an-array):

```js 
function reducer(state, action) {
  switch (action.type) {
    case 'added_todo': {
      // ✅ Correct: replacing with new state
      return {
        ...state,
        todos: [
          ...state.todos,
          
        ]
      };
    }
    // ...
  }
}
```

Now that this reducer function is pure, calling it an extra time doesn't make a difference in behavior. This is why React calling it twice helps you find mistakes. **Only component, initializer, and reducer functions need to be pure.** Event handlers don't need to be pure, so React will never call your event handlers twice.

Read [keeping components pure](/learn/keeping-components-pure) to learn more.
