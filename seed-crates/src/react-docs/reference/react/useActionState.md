---
title: useActionState
---

---

### `reducerAction` function 

The `reducerAction` function passed to `useActionState` receives the previous state and returns a new state.

Unlike reducers in `useReducer`, the `reducerAction` can be async and perform side effects:

```js
async function reducerAction(previousState, actionPayload) 
```

Each time you call `dispatchAction`, React calls the `reducerAction` with the `actionPayload`. The reducer will perform side effects such as posting data, and return the new state. If `dispatchAction` is called multiple times, React queues and executes them in order so the result of the previous call is passed as `previousState` for the current call.

#### Parameters 

* `previousState`: The last state. Initially this is equal to the `initialState`. After the first call to `dispatchAction`, it's equal to the last state returned.

* **optional** `actionPayload`: The argument passed to `dispatchAction`. It can be a value of any type. Similar to `useReducer` conventions, it is usually an object with a `type` property identifying it and, optionally, other properties with additional information.

#### Returns 

`reducerAction` returns the new state, and triggers a Transition to re-render with that state.

#### Caveats 

* `reducerAction` can be sync or async. It can perform sync actions like showing a notification, or async actions like posting updates to a server.
* `reducerAction` is not invoked twice in `

---

## Usage 

### Adding state to an Action 

Call `useActionState` at the top level of your component to create state for the result of an Action.

```js [[1, 7, "count"], [2, 7, "dispatchAction"], [3, 7, "isPending"]]
import  from 'react';

async function addToCartAction(prevCount) 
function Counter() 
```

`useActionState` returns an array with exactly three items:

1. The , initially set to the initial state you provided.
2. The  that lets you trigger `reducerAction`.
3. A  that tells you whether the Action is in progress.

To call `addToCartAction`, call the . React will queue calls to `addToCartAction` with the previous count.

Every time you click "Add Ticket," React queues a call to `addToCartAction`. React shows the pending state until all the tickets are added, and then re-renders with the final state.

---

### Using multiple Action types 

To handle multiple types, you can pass an argument to `dispatchAction`.

By convention, it is common to write it as a switch statement. For each case in the switch, calculate and return some next state. The argument can have any shape, but it is common to pass objects with a `type` property identifying the action.

When you click to increase or decrease the quantity, an `"ADD"` or `"REMOVE"` is dispatched. In the `reducerAction`, different APIs are called to update the quantity.

In this example, we use the pending state of the Actions to replace both the quantity and the total. If you want to provide immediate feedback, such as immediately updating the quantity, you can use `useOptimistic`.

---

### Using with `useOptimistic` 

You can combine `useActionState` with [`useOptimistic`](/reference/react/useOptimistic) to show immediate UI feedback:

`setOptimisticCount` immediately updates the quantity, and `dispatchAction()` queues the `updateCartAction`. A pending indicator appears on both the quantity and total to give the user feedback that their update is still being applied.

---

### Using with Action props 

When you pass the `dispatchAction` function to a component that exposes an [Action prop](/reference/react/useTransition#exposing-action-props-from-components), you don't need to call `startTransition` or `useOptimistic` yourself.

This example shows using the `increaseAction` and `decreaseAction` props of a QuantityStepper component:

Since `

Try clicking increase or decrease multiple times, and notice that the total updates within 1 second no matter how many times you click. This works because it uses an `AbortController` to "complete" the previous Action so the next Action can proceed.

---

### Using with `` Action props 

You can pass the `dispatchAction` function as the `action` prop to a ``.

When used this way, React automatically wraps the submission in a Transition, so you don't need to call `startTransition` yourself. The `reducerAction` receives the previous state and the submitted `FormData`:

In this example, when the user clicks the stepper arrows, the button submits the form and `useActionState` calls `updateCartAction` with the form data. The example uses `useOptimistic` to immediately show the new quantity while the server confirms the update.

See the [``](/reference/react-dom/components/form#handle-form-submission-with-a-server-function) docs for more information on using Actions with forms.

---

### Handling errors 

There are two ways to handle errors with `useActionState`.

For known errors, such as "quantity not available" validation errors from your backend, you can return it as part of your `reducerAction` state and display it in the UI.

For unknown errors, such as `undefined is not a function`, you can throw an error. React will cancel all queued Actions and shows the nearest [Error Boundary](/reference/react/Component#catching-rendering-errors-with-an-error-boundary) by rethrowing the error from the `useActionState` hook.

  );
}
```

```js src/Total.js
const formatter = new Intl.NumberFormat('en-US', );

export default function Total() {
  return (
    
      Total
      
        
      
    
  );
}
```

```js src/api.js hidden
export async function addToCart(count, quantity) {
  await new Promise((resolve) => setTimeout(resolve, 1000));
  if (quantity > 5) {
    return ;
  } else if (isNaN(quantity)) 
  return ;
}
```

```css
.checkout 

.checkout h2 

.row 

.total 

hr 

button 

.buttons 

.error 
```

```json package.json hidden
{
  "dependencies": ,
  "main": "/index.js"
}
```

In this example, "Add 10" simulates an API that returns a validation error, which `updateCartAction` stores in state and displays inline. "Add NaN" results in an invalid count, so `updateCartAction` throws, which propagates through `useActionState` to the `ErrorBoundary` and shows a reset UI.

---

## Troubleshooting 

### My `isPending` flag is not updating 

If you're calling `dispatchAction` manually (not through an Action prop), make sure you wrap the call in [`startTransition`](/reference/react/startTransition):

```js
import  from 'react';

function MyComponent() {
  const [state, dispatchAction, isPending] = useActionState(myAction, null);

  function handleClick() {
    // ✅ Correct: wrap in startTransition
    startTransition(() => );
  }

  // ...
}
```

When `dispatchAction` is passed to an Action prop, React automatically wraps it in a Transition.

---

### My Action cannot read form data 

When you use `useActionState`, the `reducerAction` receives an extra argument as its first argument: the previous or initial state. The submitted form data is therefore its second argument instead of its first.

```js 
// Without useActionState
function action(formData) 

// With useActionState
function action(prevState, formData) 
```

---

### My actions are being skipped 

If you call `dispatchAction` multiple times and some of them don't run, it may be because an earlier `dispatchAction` call threw an error.

When a `reducerAction` throws, React skips all subsequently queued `dispatchAction` calls.

To handle this, catch errors within your `reducerAction` and return an error state instead of throwing:

```js
async function myReducerAction(prevState, data) {
  try {
    const result = await submitData(data);
    return ;
  } catch (error) {
    // ✅ Return error state instead of throwing
    return ;
  }
}
```

---

### My state doesn't reset 

`useActionState` doesn't provide a built-in reset function. To reset the state, you can design your `reducerAction` to handle a reset signal:

```js
const initialState = ;

async function formAction(prevState, payload) {
  // Handle reset
  if (payload === null) 
  // Normal action logic
  const result = await submitData(payload);
  return result;
}

function MyComponent() {
  const [state, dispatchAction, isPending] = useActionState(formAction, initialState);

  function handleReset() {
    startTransition(() => );
  }

  // ...
}
```

Alternatively, you can add a `key` prop to the component using `useActionState` to force it to remount with fresh state, or a `` `action` prop, which resets automatically after submission.

---

### I'm getting an error: "An async function with useActionState was called outside of a transition." 

A common mistake is to forget to call `dispatchAction` from inside a Transition:

This error happens because `dispatchAction` must run inside a Transition:

```js
function MyComponent() {
  const [state, dispatchAction, isPending] = useActionState(myAsyncAction, null);

  function handleClick() 

  // ...
}
```

To fix, either wrap the call in [`startTransition`](/reference/react/startTransition):

```js
import  from 'react';

function MyComponent() {
  const [state, dispatchAction, isPending] = useActionState(myAsyncAction, null);

  function handleClick() {
    // ✅ Correct: wrap in startTransition
    startTransition(() => );
  }

  // ...
}
```

Or pass `dispatchAction` to an Action prop, is call in a Transition:

```js
function MyComponent() 
```

---

### I'm getting an error: "Cannot update action state while rendering" 

You cannot call `dispatchAction` during render:

This causes an infinite loop because calling `dispatchAction` schedules a state update, which triggers a re-render, which calls `dispatchAction` again.

```js
function MyComponent() 
```

To fix, only call `dispatchAction` in response to user events (like form submissions or button clicks).
