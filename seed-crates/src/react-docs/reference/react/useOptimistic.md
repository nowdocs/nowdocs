---
title: useOptimistic
---

**How the final state is determined**

The `value` argument to `useOptimistic` determines what displays after the Action finishes. How this works depends on the pattern you use:

- **Hardcoded values** like `useOptimistic(false)`: After the Action, `state` is still `false`, so the UI shows `false`. This is useful for pending states where you always start from `false`.

- **Props or state passed in** like `useOptimistic(isLiked)`: If the parent updates `isLiked` during the Action, the new value is used after the Action completes. This is how the UI reflects the result of the Action.

- **Reducer pattern** like `useOptimistic(items, fn)`: If `items` changes while the Action is pending, React re-runs your `reducer` with the new `items` to recalculate the state. This keeps your optimistic additions on top of the latest data.

**What happens when the Action fails**

If the Action throws an error, the Transition still ends, and React renders with whatever `value` currently is. Since the parent typically only updates `value` on success, a failure means `value` hasn't changed, so the UI shows what it showed before the optimistic update. You can catch the error to show a message to the user.

---

## Usage 

### Adding optimistic state to a component 

Call `useOptimistic` at the top level of your component to declare one or more optimistic states.

```js [[1, 4, "age"], [1, 5, "name"], [1, 6, "todos"], [2, 4, "optimisticAge"], [2, 5, "optimisticName"], [2, 6, "optimisticTodos"], [3, 4, "setOptimisticAge"], [3, 5, "setOptimisticName"], [3, 6, "setOptimisticTodos"], [4, 6, "reducer"]]
import  from 'react';

function MyComponent() {
  const [optimisticAge, setOptimisticAge] = useOptimistic(age);
  const [optimisticName, setOptimisticName] = useOptimistic(name);
  const [optimisticTodos, setOptimisticTodos] = useOptimistic(todos, reducer);
  // ...
```

`useOptimistic` returns an array with exactly two items:

1. The , initially set to the  provided.
2. The  that lets you temporarily change the state during an [Action](reference/react/useTransition#functions-called-in-starttransition-are-called-actions).
   * If a  is provided, it will run before returning the optimistic state.

To use the , call the `set` function inside an Action.

Actions are functions called inside `startTransition`:

```js 
function onAgeChange(e) {
  startTransition(async () => );
}
```

React will render the optimistic state `42` first while the `age` remains the current age. The Action waits for POST, and then renders the `newAge` for both `age` and `optimisticAge`.

See [How optimistic state works](#how-optimistic-state-works) for a deep dive.

---

### Using optimistic state in Action props 

In an [Action prop](/reference/react/useTransition#exposing-action-props-from-components), you can call the optimistic setter directly without `startTransition`.

This example sets optimistic state inside a `` `submitAction` prop:

In this example, when the user submits the form, the `optimisticName` updates immediately to show the `newName` optimistically while the server request is in progress. When the request completes, `name` and `optimisticName` are rendered with the actual `updatedName` from the response.

---

### Adding optimistic state to Action props 

When creating an [Action prop](/reference/react/useTransition#exposing-action-props-from-components), you can add `useOptimistic` to show immediate feedback.

Here's a button that shows "Submitting..." while the `action` is pending:

      {count > 0 && Submitted !}
    
  );
}
```

```js src/Button.js active
import  from 'react';

export default function Button() {
  const [isPending, setIsPending] = useOptimistic(false);

  return (
     {
        startTransition(async () => );
      }}
    >
      
    
  );
}
```

```js src/actions.js hidden
export async function submitForm() 
```

When the button is clicked, `setIsPending(true)` uses optimistic state to immediately show "Submitting..." and disable the button. When the Action is done, `isPending` is rendered as `false` automatically.

This pattern automatically shows a pending state however `action` prop is used with `Button`:

```js
// Show pending state for a state update

---

### Updating props or state optimistically 

You can wrap props or state in `useOptimistic` to update it immediately while an Action is in progress.

In this example, `LikeButton` receives `isLiked` as a prop and immediately toggles it when clicked:

When the button is clicked, `setOptimisticIsLiked` immediately updates the displayed state to show the heart as liked. Meanwhile, `await toggleLike` runs in the background. When the `await` completes, `setIsLiked` parent updates the "real" `isLiked` state, and the optimistic state is rendered to match this new value.

---

### Updating multiple values together 

When an optimistic update affects multiple related values, use a reducer to update them together. This ensures the UI stays consistent.

Here's a follow button that updates both the follow state and follower count:

The reducer receives the new `isFollowing` value and calculates both the new follow state and the updated follower count in a single update. This ensures the button text and count always stay in sync.

---

### Optimistically adding to a list 

When you need to optimistically add items to a list, use a `reducer`:

The `reducer` receives the current list of todos and the new todo to add. This is important because if the `todos` prop changes while your add is pending (for example, another user added a todo), React will update your optimistic state by re-running the reducer with the updated list. This ensures your new todo is added to the latest list, not an outdated copy.

---

### Handling multiple `action` types 

When you need to handle multiple types of optimistic updates (like adding and removing items), use a reducer pattern with `action` objects.

This shopping cart example shows how to handle add and remove with a single reducer:

The reducer handles three `action` types (`add`, `remove`, `update_quantity`) and returns the new optimistic state for each. Each `action` sets a `pending: true` flag so you can show visual feedback while the [Server Function](/reference/rsc/server-functions) runs.

---

### Optimistic delete with error recovery 

When deleting items optimistically, you should handle the case where the Action fails.

This example shows how to display an error message when a delete fails, and the UI automatically rolls back to show the item again.

Try deleting 'Deploy to production'. When the delete fails, the item automatically reappears in the list.

---

## Troubleshooting 

### I'm getting an error: "An optimistic state update occurred outside a Transition or Action" 

You may see this error:

The optimistic setter function must be called inside `startTransition`:

```js
// 🚩 Incorrect: outside a Transition
function handleClick() 

// ✅ Correct: inside a Transition
function handleClick() {
  startTransition(async () => );
}

// ✅ Also correct: inside an Action prop
function submitAction(formData) 
```

When you call the setter outside an Action, the optimistic state will briefly appear and then immediately revert back to the original value. This happens because there's no Transition to "hold" the optimistic state while your Action runs.

### I'm getting an error: "Cannot update optimistic state while rendering" 

You may see this error:

This error occurs when you call the optimistic setter during the render phase of a component. You can only call it from event handlers, effects, or other callbacks:

```js
// 🚩 Incorrect: calling during render
function MyComponent() 

// ✅ Correct: calling inside startTransition
function MyComponent() {
  const [isPending, setPending] = useOptimistic(false);

  function handleClick() {
    startTransition(() => );
  }

  // ...
}

// ✅ Also correct: calling from an Action
function MyComponent() {
  const [isPending, setPending] = useOptimistic(false);

  function action() 

  // ...
}
```

### My optimistic updates show stale values 

If your optimistic state seems to be based on old data, consider using an updater function or reducer to calculate the optimistic state relative to the current state.

```js
// May show stale data if state changes during Action
const [optimistic, setOptimistic] = useOptimistic(count);
setOptimistic(5);  // Always sets to 5, even if count changed

// Better: relative updates handle state changes correctly
const [optimistic, adjust] = useOptimistic(count, (current, delta) => current + delta);
adjust(1);  // Always adds 1 to whatever the current count is
```

See [Updating state based on the current state](#updating-state-based-on-current-state) for details.

### I don't know if my optimistic update is pending 

To know when `useOptimistic` is pending, you have three options:

1. **Check if `optimisticValue === value`**

```js
const [optimistic, setOptimistic] = useOptimistic(value);
const isPending = optimistic !== value;
```

If the values are not equal, there's a Transition in progress.

2. **Add a `useTransition`**

```js
const [isPending, startTransition] = useTransition();
const [optimistic, setOptimistic] = useOptimistic(value);

//...
startTransition(() => )
```

Since `useTransition` uses `useOptimistic` for `isPending` under the hood, this is equivalent to option 1.

3. **Add a `pending` flag in your reducer**

```js
const [optimistic, addOptimistic] = useOptimistic(
  items,
  (state, newItem) => [...state, ]
);
```

Since each optimistic item has its own flag, you can show loading state for individual items.
