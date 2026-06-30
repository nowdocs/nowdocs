---
title: Preserving and Resetting State
---

## State is tied to a position in the render tree 

React builds [render trees](learn/understanding-your-ui-as-a-tree#the-render-tree) for the component structure in your UI.

When you give a component state, you might think the state "lives" inside the component. But the state is actually held inside React. React associates each piece of state it's holding with the correct component by where that component sits in the render tree.

Here, there is only one `

Here's how these look as a tree:

**These are two separate counters because each is rendered at its own position in the tree.** You don't usually have to think about these positions to use React, but it can be useful to understand how it works.

In React, each component on the screen has fully isolated state. For example, if you render two `Counter` components side by side, each of them will get its own, independent, `score` and `hover` states.

Try clicking both counters and notice they don't affect each other:

As you can see, when one counter is updated, only the state for that component is updated:

React will keep the state around for as long as you render the same component at the same position in the tree. To see this, increment both counters, then remove the second component by unchecking "Render the second counter" checkbox, and then add it back by ticking it again:

Notice how the moment you stop rendering the second counter, its state disappears completely. That's because when React removes a component, it destroys its state.

When you tick "Render the second counter", a second `Counter` and its state are initialized from scratch (`score = 0`) and added to the DOM.

**React preserves a component's state for as long as it's being rendered at its position in the UI tree.** If it gets removed, or a different component gets rendered at the same position, React discards its state.

## Same component at the same position preserves state 

In this example, there are two different `

When you tick or clear the checkbox, the counter state does not get reset. Whether `isFancy` is `true` or `false`, you always have a `

It's the same component at the same position, so from React's perspective, it's the same counter.

You might expect the state to reset when you tick checkbox, but it doesn't! This is because **both of these `

## Different components at the same position reset state 

In this example, ticking the checkbox will replace `

Here, you switch between _different_ component types at the same position. Initially, the first child of the `` contained a `Counter`. But when you swapped in a `p`, React removed the `Counter` from the UI tree and destroyed its state.

Also, **when you render a different component in the same position, it resets the state of its entire subtree.** To see how this works, increment the counter and then tick the checkbox:

The counter state gets reset when you click the checkbox. Although you render a `Counter`, the first child of the `div` changes from a `section` to a `div`. When the child `section` was removed from the DOM, the whole tree below it (including the `Counter` and its state) was destroyed as well.

As a rule of thumb, **if you want to preserve the state between re-renders, the structure of your tree needs to "match up"** from one render to another. If the structure is different, the state gets destroyed because React destroys state when it removes a component from the tree.

Every time you click the button, the input state disappears! This is because a *different* `MyTextField` function is created for every render of `MyComponent`. You're rendering a *different* component in the same position, so React resets all state below. This leads to bugs and performance problems. To avoid this problem, **always declare component functions at the top level, and don't nest their definitions.**

## Resetting state at the same position 

By default, React preserves state of a component while it stays at the same position. Usually, this is exactly what you want, so it makes sense as the default behavior. But sometimes, you may want to reset a component's state. Consider this app that lets two players keep track of their scores during each turn:

Currently, when you change the player, the score is preserved. The two `Counter`s appear in the same position, so React sees them as *the same* `Counter` whose `person` prop has changed.

But conceptually, in this app they should be two separate counters. They might appear in the same place in the UI, but one is a counter for Taylor, and another is a counter for Sarah.

There are two ways to reset state when switching between them:

1. Render components in different positions
2. Give each component an explicit identity with `key`

### Option 1: Rendering a component in different positions 

If you want these two `Counter`s to be independent, you can render them in two different positions:

* Initially, `isPlayerA` is `true`. So the first position contains `Counter` state, and the second one is empty.
* When you click the "Next player" button the first position clears but the second one now contains a `Counter`.

Each `Counter`'s state gets destroyed each time it's removed from the DOM. This is why they reset every time you click the button.

This solution is convenient when you only have a few independent components rendered in the same place. In this example, you only have two, so it's not a hassle to render both separately in the JSX.

### Option 2: Resetting state with a key 

There is also another, more generic, way to reset a component's state.

You might have seen `key`s when [rendering lists.](/learn/rendering-lists#keeping-list-items-in-order-with-key) Keys aren't just for lists! You can use keys to make React distinguish between any components. By default, React uses order within the parent ("first counter", "second counter") to discern between components. But keys let you tell React that this is not just a *first* counter, or a *second* counter, but a specific counter--for example, *Taylor's* counter. This way, React will know *Taylor's* counter wherever it appears in the tree!

In this example, the two `

Switching between Taylor and Sarah does not preserve the state. This is because **you gave them different `key`s:**

```js
{isPlayerA ? (
  

### Resetting a form with a key 

Resetting state with a key is particularly useful when dealing with forms.

In this chat app, the `

Try entering something into the input, and then press "Alice" or "Bob" to choose a different recipient. You will notice that the input state is preserved because the `

Technically, you could also add `null` before `

This way, `Form` is always the second child, so it stays in the same position and keeps its state. But this approach is much less obvious and introduces a risk that someone else will remove that `null`.

#### Swap two form fields 

This form lets you enter first and last name. It also has a checkbox controlling which field goes first. When you tick the checkbox, the "Last name" field will appear before the "First name" field.

It almost works, but there is a bug. If you fill in the "First name" input and tick the checkbox, the text will stay in the first input (which is now "Last name"). Fix it so that the input text *also* moves when you reverse the order.

#### Reset a detail form 

This is an editable contact list. You can edit the selected contact's details and then either press "Save" to update it, or "Reset" to undo your changes.

When you select a different contact (for example, Alice), the state updates but the form keeps showing the previous contact's details. Fix it so that the form gets reset when the selected contact changes.

#### Clear an image while it's loading 

When you press "Next", the browser starts loading the next image. However, because it's displayed in the same `` tag, by default you would still see the previous image until the next one loads. This may be undesirable if it's important for the text to always match the image. Change it so that the moment you press "Next", the previous image immediately clears.

#### Fix misplaced state in the list 

In this list, each `Contact` has state that determines whether "Show email" has been pressed for it. Press "Show email" for Alice, and then tick the "Show in reverse order" checkbox. You will notice that it's _Taylor's_ email that is expanded now, but Alice's--which has moved to the bottom--appears collapsed.

Fix it so that the expanded state is associated with each contact, regardless of the chosen ordering.

State is associated with the tree position. A `key` lets you specify a named position instead of relying on order.

