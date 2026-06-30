---
title: ""
---

---

### Providing a label for a select box 

Typically, you will place every `` inside a [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/label) tag. This tells the browser that this label is associated with that select box. When the user clicks the label, the browser will automatically focus the select box. It's also essential for accessibility: a screen reader will announce the label caption when the user focuses the select box.

If you can't nest `` into a ``, associate them by passing the same ID to `` and [``.](https://developer.mozilla.org/en-US/docs/Web/API/HTMLLabelElement/htmlFor) To avoid conflicts between multiple instances of one component, generate such an ID with [`useId`.](/reference/react/useId)

---

### Providing an initially selected option 

By default, the browser will select the first `` in the list. To select a different option by default, pass that ``'s `value` as the `defaultValue` to the `` element.

---

### Enabling multiple selection 

Pass `multiple=` to the `` to let the user select multiple options. In that case, if you also specify `defaultValue` to choose the initially selected options, it must be an array.

---

### Reading the select box value when submitting a form 

Add a [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/form) around your select box with a [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/button) inside. It will call your `` event handler. By default, the browser will send the form data to the current URL and refresh the page. You can override that behavior by calling `e.preventDefault()`. Read the form data with [`new FormData(e.target)`](https://developer.mozilla.org/en-US/docs/Web/API/FormData).

---

### Controlling a select box with a state variable 

A select box like `` is *uncontrolled.* Even if you [pass an initially selected value](#providing-an-initially-selected-option) like ``, your JSX only specifies the initial value, not the value right now.

**To render a _controlled_ select box, pass the `value` prop to it.** React will force the select box to always have the `value` you passed. Typically, you will control a select box by declaring a [state variable:](/reference/react/useState)

```js 
function FruitPicker()  // ... and update the state variable on any change!
    >
      Apple
      Banana
      Orange
    
  );
}
```

This is useful if you want to re-render some part of the UI in response to every selection.

