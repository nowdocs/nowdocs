---
title: Render and Commit
---

Imagine that your components are cooks in the kitchen, assembling tasty dishes from ingredients. In this scenario, React is the waiter who puts in requests from customers and brings them their orders. This process of requesting and serving UI has three steps:

1. **Triggering** a render (delivering the guest's order to the kitchen)
2. **Rendering** the component (preparing the order in the kitchen)
3. **Committing** to the DOM (placing the order on the table)

## Step 1: Trigger a render 

There are two reasons for a component to render:

1. It's the component's **initial render.**
2. The component's (or one of its ancestors') **state has been updated.**

### Initial render 

When your app starts, you need to trigger the initial render. Frameworks and sandboxes sometimes hide this code, but it's done by calling [`createRoot`](/reference/react-dom/client/createRoot) with the target DOM node, and then calling its `render` method with your component:

Try commenting out the `root.render()` call and see the component disappear!

### Re-renders when state updates 

Once the component has been initially rendered, you can trigger further renders by updating its state with the [`set` function.](/reference/react/useState#setstate) Updating your component's state automatically queues a render. (You can imagine these as a restaurant guest ordering tea, dessert, and all sorts of things after putting in their first order, depending on the state of their thirst or hunger.)

## Step 2: React renders your components 

After you trigger a render, React calls your components to figure out what to display on screen. **"Rendering" is React calling your components.**

* **On initial render,** React will call the root component.
* **For subsequent renders,** React will call the function component whose state update triggered the render.

This process is recursive: if the updated component returns some other component, React will render _that_ component next, and if that component also returns something, it will render _that_ component next, and so on. The process will continue until there are no more nested components and React knows exactly what should be displayed on screen.

In the following example, React will call `Gallery()` and `Image()` several times:

* **During the initial render,** React will [create the DOM nodes](https://developer.mozilla.org/docs/Web/API/Document/createElement) for ``, ``, and three `` tags.
* **During a re-render,** React will calculate which of their properties, if any, have changed since the previous render. It won't do anything with that information until the next step, the commit phase.

## Step 3: React commits changes to the DOM 

After rendering (calling) your components, React will modify the DOM.

* **For the initial render,** React will use the [`appendChild()`](https://developer.mozilla.org/docs/Web/API/Node/appendChild) DOM API to put all the DOM nodes it has created on screen.
* **For re-renders,** React will apply the minimal necessary operations (calculated while rendering!) to make the DOM match the latest rendering output.

**React only changes the DOM nodes if there's a difference between renders.** For example, here is a component that re-renders with different props passed from its parent every second. Notice how you can add some text into the ``, updating its `value`, but the text doesn't disappear when the component re-renders:

This works because during this last step, React only updates the content of `` with the new `time`. It sees that the `` appears in the JSX in the same place as last time, so React doesn't touch the ``—or its `value`!
## Epilogue: Browser paint 

After rendering is done and React updated the DOM, the browser will repaint the screen. Although this process is known as "browser rendering", we'll refer to it as "painting" to avoid confusion throughout the docs.

