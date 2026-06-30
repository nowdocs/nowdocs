---
title: ""
---

---

### Providing a label for a text area 

Typically, you will place every `` inside a [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/label) tag. This tells the browser that this label is associated with that text area. When the user clicks the label, the browser will focus the text area. It's also essential for accessibility: a screen reader will announce the label caption when the user focuses the text area.

If you can't nest `` into a ``, associate them by passing the same ID to `` and [``.](https://developer.mozilla.org/en-US/docs/Web/API/HTMLLabelElement/htmlFor) To avoid conflicts between instances of one component, generate such an ID with [`useId`.](/reference/react/useId)

---

### Providing an initial value for a text area 

You can optionally specify the initial value for the text area. Pass it as the `defaultValue` string.

---

### Reading the text area value when submitting a form 

Add a [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/form) around your textarea with a [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/button) inside. It will call your `` event handler. By default, the browser will send the form data to the current URL and refresh the page. You can override that behavior by calling `e.preventDefault()`. Read the form data with [`new FormData(e.target)`](https://developer.mozilla.org/en-US/docs/Web/API/FormData).

---

### Controlling a text area with a state variable 

A text area like `` is *uncontrolled.* Even if you [pass an initial value](#providing-an-initial-value-for-a-text-area) like ``, your JSX only specifies the initial value, not the value right now.

**To render a _controlled_ text area, pass the `value` prop to it.** React will force the text area to always have the `value` you passed. Typically, you will control a text area by declaring a [state variable:](/reference/react/useState)

```js 
function NewPost()  // ... and update the state variable on any edits!
    />
  );
}
```

This is useful if you want to re-render some part of the UI in response to every keystroke.

---

## Troubleshooting 

### My text area doesn't update when I type into it 

If you render a text area with `value` but no `onChange`, you will see an error in the console:

```js
// 🔴 Bug: controlled text area with no onChange handler

```

As the error message suggests, if you only wanted to [specify the *initial* value,](#providing-an-initial-value-for-a-text-area) pass `defaultValue` instead:

```js
// ✅ Good: uncontrolled text area with an initial value

```

If you want [to control this text area with a state variable,](#controlling-a-text-area-with-a-state-variable) specify an `onChange` handler:

```js
// ✅ Good: controlled text area with onChange
 setSomething(e.target.value)} />
```

If the value is intentionally read-only, add a `readOnly` prop to suppress the error:

```js
// ✅ Good: readonly controlled text area without on change

```

---

### My text area caret jumps to the beginning on every keystroke 

If you [control a text area,](#controlling-a-text-area-with-a-state-variable) you must update its state variable to the text area's value from the DOM during `onChange`.

You can't update it to something other than `e.target.value`:

```js
function handleChange(e) 
```

You also can't update it asynchronously:

```js
function handleChange(e) {
  // 🔴 Bug: updating an input asynchronously
  setTimeout(() => , 100);
}
```

To fix your code, update it synchronously to `e.target.value`:

```js
function handleChange(e) 
```

If this doesn't fix the problem, it's possible that the text area gets removed and re-added from the DOM on every keystroke. This can happen if you're accidentally [resetting state](/learn/preserving-and-resetting-state) on every re-render. For example, this can happen if the text area or one of its parents always receives a different `key` attribute, or if you nest component definitions (which is not allowed in React and causes the "inner" component to remount on every render).

---

### I'm getting an error: "A component is changing an uncontrolled input to be controlled" 

If you provide a `value` to the component, it must remain a string throughout its lifetime.

You cannot pass `value=` first and later pass `value="some string"` because React won't know whether you want the component to be uncontrolled or controlled. A controlled component should always receive a string `value`, not `null` or `undefined`.

If your `value` is coming from an API or a state variable, it might be initialized to `null` or `undefined`. In that case, either set it to an empty string (`''`) initially, or pass `value=` to ensure `value` is a string.
