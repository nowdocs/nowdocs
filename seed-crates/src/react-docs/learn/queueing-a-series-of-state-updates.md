---
title: Queueing a Series of State Updates
---

## React batches state updates 

You might expect that clicking the "+3" button will increment the counter three times because it calls `setNumber(number + 1)` three times:

However, as you might recall from the previous section, [each render's state values are fixed](/learn/state-as-a-snapshot#rendering-takes-a-snapshot-in-time), so the value of `number` inside the first render's event handler is always `0`, no matter how many times you call `setNumber(1)`:

```js
setNumber(0 + 1);
setNumber(0 + 1);
setNumber(0 + 1);
```

But there is one other factor at play here. **React waits until *all* code in the event handlers has run before processing your state updates.** This is why the re-render only happens *after* all these `setNumber()` calls.

This might remind you of a waiter taking an order at the restaurant. A waiter doesn't run to the kitchen at the mention of your first dish! Instead, they let you finish your order, let you make changes to it, and even take orders from other people at the table.

Here, `n => n + 1` is called an **updater function.** When you pass it to a state setter:

1. React queues this function to be processed after all the other code in the event handler has run.
2. During the next render, React goes through the queue and gives you the final updated state.

```js
setNumber(n => n + 1);
setNumber(n => n + 1);
setNumber(n => n + 1);
```

Here's how React works through these lines of code while executing the event handler:

1. `setNumber(n => n + 1)`: `n => n + 1` is a function. React adds it to a queue.
1. `setNumber(n => n + 1)`: `n => n + 1` is a function. React adds it to a queue.
1. `setNumber(n => n + 1)`: `n => n + 1` is a function. React adds it to a queue.

When you call `useState` during the next render, React goes through the queue. The previous `number` state was `0`, so that's what React passes to the first updater function as the `n` argument. Then React takes the return value of your previous updater function and passes it to the next updater as `n`, and so on:

|  queued update | `n` | returns |
|--------------|---------|-----|
| `n => n + 1` | `0` | `0 + 1 = 1` |
| `n => n + 1` | `1` | `1 + 1 = 2` |
| `n => n + 1` | `2` | `2 + 1 = 3` |

React stores `3` as the final result and returns it from `useState`.

This is why clicking "+3" in the above example correctly increments the value by 3.
### What happens if you update state after replacing it 

What about this event handler? What do you think `number` will be in the next render?

```js
 }>
```

Here's what this event handler tells React to do:

1. `setNumber(number + 5)`: `number` is `0`, so `setNumber(0 + 5)`. React adds *"replace with `5`"* to its queue.
2. `setNumber(n => n + 1)`: `n => n + 1` is an updater function. React adds *that function* to its queue.

During the next render, React goes through the state queue:

|   queued update       | `n` | returns |
|--------------|---------|-----|
| "replace with `5`" | `0` (unused) | `5` |
| `n => n + 1` | `5` | `5 + 1 = 6` |

React stores `6` as the final result and returns it from `useState`.

### What happens if you replace state after updating it 

Let's try one more example. What do you think `number` will be in the next render?

```js
 }>
```

Here's how React works through these lines of code while executing this event handler:

1. `setNumber(number + 5)`: `number` is `0`, so `setNumber(0 + 5)`. React adds *"replace with `5`"* to its queue.
2. `setNumber(n => n + 1)`: `n => n + 1` is an updater function. React adds *that function* to its queue.
3. `setNumber(42)`: React adds *"replace with `42`"* to its queue.

During the next render, React goes through the state queue:

|   queued update       | `n` | returns |
|--------------|---------|-----|
| "replace with `5`" | `0` (unused) | `5` |
| `n => n + 1` | `5` | `5 + 1 = 6` |
| "replace with `42`" | `6` (unused) | `42` |

Then React stores `42` as the final result and returns it from `useState`.

To summarize, here's how you can think of what you're passing to the `setNumber` state setter:

* **An updater function** (e.g. `n => n + 1`) gets added to the queue.
* **Any other value** (e.g. number `5`) adds "replace with `5`" to the queue, ignoring what's already queued.

After the event handler completes, React will trigger a re-render. During the re-render, React will process the queue. Updater functions run during rendering, so **updater functions must be [pure](/learn/keeping-components-pure)** and only *return* the result. Don't try to set state from inside of them or run other side effects. In Strict Mode, React will run each updater function twice (but discard the second result) to help you find mistakes.

### Naming conventions 

It's common to name the updater function argument by the first letters of the corresponding state variable:

```js
setEnabled(e => !e);
setLastName(ln => ln.reverse());
setFriendCount(fc => fc * 2);
```

If you prefer more verbose code, another common convention is to repeat the full state variable name, like `setEnabled(enabled => !enabled)`, or to use a prefix like `setEnabled(prevEnabled => !prevEnabled)`.

This ensures that when you increment or decrement a counter, you do it in relation to its *latest* state rather than what the state was at the time of the click.

#### Implement the state queue yourself 

In this challenge, you will reimplement a tiny part of React from scratch! It's not as hard as it sounds.

Scroll through the sandbox preview. Notice that it shows **four test cases.** They correspond to the examples you've seen earlier on this page. Your task is to implement the `getFinalState` function so that it returns the correct result for each of those cases. If you implement it correctly, all four tests should pass.

You will receive two arguments: `baseState` is the initial state (like `0`), and the `queue` is an array which contains a mix of numbers (like `5`) and updater functions (like `n => n + 1`) in the order they were added.

Your task is to return the final state, just like the tables on this page show!

Now you know how this part of React works!

