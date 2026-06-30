---
title: Extracting State Logic into a Reducer
---

## Consolidate state logic with a reducer 

As your components grow in complexity, it can get harder to see at a glance all the different ways in which a component's state gets updated. For example, the `TaskApp` component below holds an array of `tasks` in state and uses three different event handlers to add, remove, and edit tasks:

Each of its event handlers calls `setTasks` in order to update the state. As this component grows, so does the amount of state logic sprinkled throughout it. To reduce this complexity and keep all your logic in one easy-to-access place, you can move that state logic into a single function outside your component, **called a "reducer".**

Reducers are a different way to handle state. You can migrate from `useState` to `useReducer` in three steps:

1. **Move** from setting state to dispatching actions.
2. **Write** a reducer function.
3. **Use** the reducer from your component.

### Step 1: Move from setting state to dispatching actions 

Your event handlers currently specify _what to do_ by setting state:

```js
function handleAddTask(text) {
  setTasks([
    ...tasks,
    ,
  ]);
}

function handleChangeTask(task) {
  setTasks(
    tasks.map((t) => {
      if (t.id === task.id)  else 
    })
  );
}

function handleDeleteTask(taskId) 
```

Remove all the state setting logic. What you are left with are three event handlers:

- `handleAddTask(text)` is called when the user presses "Add".
- `handleChangeTask(task)` is called when the user toggles a task or presses "Save".
- `handleDeleteTask(taskId)` is called when the user presses "Delete".

Managing state with reducers is slightly different from directly setting state. Instead of telling React "what to do" by setting state, you specify "what the user just did" by dispatching "actions" from your event handlers. (The state update logic will live elsewhere!) So instead of "setting `tasks`" via an event handler, you're dispatching an "added/changed/deleted a task" action. This is more descriptive of the user's intent.

```js
function handleAddTask(text) {
  dispatch();
}

function handleChangeTask(task) {
  dispatch();
}

function handleDeleteTask(taskId) {
  dispatch();
}
```

The object you pass to `dispatch` is called an "action":

```js 
function handleDeleteTask(taskId) {
  dispatch(
    // "action" object:
    
  );
}
```

It is a regular JavaScript object. You decide what to put in it, but generally it should contain the minimal information about _what happened_. (You will add the `dispatch` function itself in a later step.)

### Step 2: Write a reducer function 

A reducer function is where you will put your state logic. It takes two arguments, the current state and the action object, and it returns the next state:

```js
function yourReducer(state, action) 
```

React will set the state to what you return from the reducer.

To move your state setting logic from your event handlers to a reducer function in this example, you will:

1. Declare the current state (`tasks`) as the first argument.
2. Declare the `action` object as the second argument.
3. Return the _next_ state from the reducer (which React will set the state to).

Here is all the state setting logic migrated to a reducer function:

```js
function tasksReducer(tasks, action) {
  if (action.type === 'added') {
    return [
      ...tasks,
      ,
    ];
  } else if (action.type === 'changed') {
    return tasks.map((t) => {
      if (t.id === action.task.id)  else 
    });
  } else if (action.type === 'deleted')  else 
}
```

Because the reducer function takes state (`tasks`) as an argument, you can **declare it outside of your component.** This decreases the indentation level and can make your code easier to read.

You probably won't need to do this yourself, but this is similar to what React does!

### Step 3: Use the reducer from your component 

Finally, you need to hook up the `tasksReducer` to your component. Import the `useReducer` Hook from React:

```js
import  from 'react';
```

Then you can replace `useState`:

```js
const [tasks, setTasks] = useState(initialTasks);
```

with `useReducer` like so:

```js
const [tasks, dispatch] = useReducer(tasksReducer, initialTasks);
```

The `useReducer` Hook is similar to `useState`—you must pass it an initial state and it returns a stateful value and a way to set state (in this case, the dispatch function). But it's a little different.

The `useReducer` Hook takes two arguments:

1. A reducer function
2. An initial state

And it returns:

1. A stateful value
2. A dispatch function (to "dispatch" user actions to the reducer)

Now it's fully wired up! Here, the reducer is declared at the bottom of the component file:

If you want, you can even move the reducer to a different file:

Component logic can be easier to read when you separate concerns like this. Now the event handlers only specify _what happened_ by dispatching actions, and the reducer function determines _how the state updates_ in response to them.

## Comparing `useState` and `useReducer` 

Reducers are not without downsides! Here's a few ways you can compare them:

- **Code size:** Generally, with `useState` you have to write less code upfront. With `useReducer`, you have to write both a reducer function _and_ dispatch actions. However, `useReducer` can help cut down on the code if many event handlers modify state in a similar way.
- **Readability:** `useState` is very easy to read when the state updates are simple. When they get more complex, they can bloat your component's code and make it difficult to scan. In this case, `useReducer` lets you cleanly separate the _how_ of update logic from the _what happened_ of event handlers.
- **Debugging:** When you have a bug with `useState`, it can be difficult to tell _where_ the state was set incorrectly, and _why_. With `useReducer`, you can add a console log into your reducer to see every state update, and _why_ it happened (due to which `action`). If each `action` is correct, you'll know that the mistake is in the reducer logic itself. However, you have to step through more code than with `useState`.
- **Testing:** A reducer is a pure function that doesn't depend on your component. This means that you can export and test it separately in isolation. While generally it's best to test components in a more realistic environment, for complex state update logic it can be useful to assert that your reducer returns a particular state for a particular initial state and action.
- **Personal preference:** Some people like reducers, others don't. That's okay. It's a matter of preference. You can always convert between `useState` and `useReducer` back and forth: they are equivalent!

We recommend using a reducer if you often encounter bugs due to incorrect state updates in some component, and want to introduce more structure to its code. You don't have to use reducers for everything: feel free to mix and match! You can even `useState` and `useReducer` in the same component.

## Writing reducers well 

Keep these two tips in mind when writing reducers:

- **Reducers must be pure.** Similar to [state updater functions](/learn/queueing-a-series-of-state-updates), reducers run during rendering! (Actions are queued until the next render.) This means that reducers [must be pure](/learn/keeping-components-pure)—same inputs always result in the same output. They should not send requests, schedule timeouts, or perform any side effects (operations that impact things outside the component). They should update [objects](/learn/updating-objects-in-state) and [arrays](/learn/updating-arrays-in-state) without mutations.
- **Each action describes a single user interaction, even if that leads to multiple changes in the data.** For example, if a user presses "Reset" on a form with five fields managed by a reducer, it makes more sense to dispatch one `reset_form` action rather than five separate `set_field` actions. If you log every action in a reducer, that log should be clear enough for you to reconstruct what interactions or responses happened in what order. This helps with debugging!

## Writing concise reducers with Immer 

Just like with [updating objects](/learn/updating-objects-in-state#write-concise-update-logic-with-immer) and [arrays](/learn/updating-arrays-in-state#write-concise-update-logic-with-immer) in regular state, you can use the Immer library to make reducers more concise. Here, [`useImmerReducer`](https://github.com/immerjs/use-immer#useimmerreducer) lets you mutate the state with `push` or `arr[i] =` assignment:

Reducers must be pure, so they shouldn't mutate state. But Immer provides you with a special `draft` object which is safe to mutate. Under the hood, Immer will create a copy of your state with the changes you made to the `draft`. This is why reducers managed by `useImmerReducer` can mutate their first argument and don't need to return state.

#### Clear the input on sending a message 

Currently, pressing "Send" doesn't do anything. Add an event handler to the "Send" button that will:

1. Show an `alert` with the recipient's email and the message.
2. Clear the message input.

This works and clears the input when you hit "Send".

However, _from the user's perspective_, sending a message is a different action than editing the field. To reflect that, you could instead create a _new_ action called `sent_message`, and handle it separately in the reducer:

The resulting behavior is the same. But keep in mind that action types should ideally describe "what the user did" rather than "how you want the state to change". This makes it easier to later add more features.

With either solution, it's important that you **don't** place the `alert` inside a reducer. The reducer should be a pure function--it should only calculate the next state. It should not "do" anything, including displaying messages to the user. That should happen in the event handler. (To help catch mistakes like this, React will call your reducers multiple times in Strict Mode. This is why, if you put an alert in a reducer, it fires twice.)

#### Restore input values when switching between tabs 

In this example, switching between different recipients always clears the text input:

```js
case 'changed_selection': {
  return ;
```

This is because you don't want to share a single message draft between several recipients. But it would be better if your app "remembered" a draft for each contact separately, restoring them when you switch contacts.

Your task is to change the way the state is structured so that you remember a separate message draft _per contact_. You would need to make a few changes to the reducer, the initial state, and the components.

Notably, you didn't need to change any of the event handlers to implement this different behavior. Without a reducer, you would have to change every event handler that updates the state.

#### Implement `useReducer` from scratch 

In the earlier examples, you imported the `useReducer` Hook from React. This time, you will implement _the `useReducer` Hook itself!_ Here is a stub to get you started. It shouldn't take more than 10 lines of code.

To test your changes, try typing into the input or select a contact.

Though it doesn't matter in most cases, a slightly more accurate implementation looks like this:

```js
function dispatch(action) 
```

This is because the dispatched actions are queued until the next render, [similar to the updater functions.](/learn/queueing-a-series-of-state-updates)

