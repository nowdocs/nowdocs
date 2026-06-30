---
title: ""
---

---

### Providing a label for an input 

Typically, you will place every `` inside a [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/label) tag. This tells the browser that this label is associated with that input. When the user clicks the label, the browser will automatically focus the input. It's also essential for accessibility: a screen reader will announce the label caption when the user focuses the associated input.

If you can't nest `` into a ``, associate them by passing the same ID to `` and [``.](https://developer.mozilla.org/en-US/docs/Web/API/HTMLLabelElement/htmlFor) To avoid conflicts between multiple instances of one component, generate such an ID with [`useId`.](/reference/react/useId)

---

### Providing an initial value for an input 

You can optionally specify the initial value for any input. Pass it as the `defaultValue` string for text inputs. Checkboxes and radio buttons should specify the initial value with the `defaultChecked` boolean instead.

---

### Reading the input values when submitting a form 

Add a [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/form) around your inputs with a [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/button) inside. It will call your `` event handler. By default, the browser will send the form data to the current URL and refresh the page. You can override that behavior by calling `e.preventDefault()`. Read the form data with [`new FormData(e.target)`](https://developer.mozilla.org/en-US/docs/Web/API/FormData).

---

### Controlling an input with a state variable 

An input like `` is *uncontrolled.* Even if you [pass an initial value](#providing-an-initial-value-for-an-input) like ``, your JSX only specifies the initial value. It does not control what the value should be right now.

**To render a _controlled_ input, pass the `value` prop to it (or `checked` for checkboxes and radios).** React will force the input to always have the `value` you passed. Usually, you would do this by declaring a [state variable:](/reference/react/useState)

```js 
function Form()  // ... and update the state variable on any edits!
    />
  );
}
```

A controlled input makes sense if you needed state anyway--for example, to re-render your UI on every edit:

```js 
function Form()  />
      
      {firstName !== '' && Your name is .}
      ...
```

It's also useful if you want to offer multiple ways to adjust the input state (for example, by clicking a button):

```js 
function Form() 
          type="number"
        />
         setAge(ageAsNumber + 10)}>
          Add 10 years
        
```

The `value` you pass to controlled components should not be `undefined` or `null`. If you need the initial value to be empty (such as with the `firstName` field below), initialize your state variable to an empty string (`''`).

---

### Optimizing re-rendering on every keystroke 

When you use a controlled input, you set the state on every keystroke. If the component containing your state re-renders a large tree, this can get slow. There's a few ways you can optimize re-rendering performance.

For example, suppose you start with a form that re-renders all page content on every keystroke:

```js 
function App()  />
      
      

As the error message suggests, if you only wanted to [specify the *initial* value,](#providing-an-initial-value-for-an-input) pass `defaultValue` instead:

```js
// ✅ Good: uncontrolled input with an initial value

```

If you want [to control this input with a state variable,](#controlling-an-input-with-a-state-variable) specify an `onChange` handler:

```js
// ✅ Good: controlled input with onChange
 setSomething(e.target.value)} />
```

If the value is intentionally read-only, add a `readOnly` prop to suppress the error:

```js
// ✅ Good: readonly controlled input without on change

```

---

### My checkbox doesn't update when I click on it 

If you render a checkbox with `checked` but no `onChange`, you will see an error in the console:

```js
// 🔴 Bug: controlled checkbox with no onChange handler

```

As the error message suggests, if you only wanted to [specify the *initial* value,](#providing-an-initial-value-for-an-input) pass `defaultChecked` instead:

```js
// ✅ Good: uncontrolled checkbox with an initial value

```

If you want [to control this checkbox with a state variable,](#controlling-an-input-with-a-state-variable) specify an `onChange` handler:

```js
// ✅ Good: controlled checkbox with onChange
 setSomething(e.target.checked)} />
```

If the checkbox is intentionally read-only, add a `readOnly` prop to suppress the error:

```js
// ✅ Good: readonly controlled input without on change

```

---

### My input caret jumps to the beginning on every keystroke 

If you [control an input,](#controlling-an-input-with-a-state-variable) you must update its state variable to the input's value from the DOM during `onChange`.

You can't update it to something other than `e.target.value` (or `e.target.checked` for checkboxes):

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

If this doesn't fix the problem, it's possible that the input gets removed and re-added from the DOM on every keystroke. This can happen if you're accidentally [resetting state](/learn/preserving-and-resetting-state) on every re-render, for example if the input or one of its parents always receives a different `key` attribute, or if you nest component function definitions (which is not supported and causes the "inner" component to always be considered a different tree).

---

### I'm getting an error: "A component is changing an uncontrolled input to be controlled" 

If you provide a `value` to the component, it must remain a string throughout its lifetime.

You cannot pass `value=` first and later pass `value="some string"` because React won't know whether you want the component to be uncontrolled or controlled. A controlled component should always receive a string `value`, not `null` or `undefined`.

If your `value` is coming from an API or a state variable, it might be initialized to `null` or `undefined`. In that case, either set it to an empty string (`''`) initially, or pass `value=` to ensure `value` is a string.

Similarly, if you pass `checked` to a checkbox, ensure it's always a boolean.
