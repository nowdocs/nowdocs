---
title: useFormStatus
---

 property which tells you if the form is actively submitting.

In the above example, `Submit` uses this information to disable `` presses while the form is submitting.

[See more examples below.](#usage)

#### Parameters 

`useFormStatus` does not take any parameters.

#### Returns 

A `status` object with the following properties:

* `pending`: A boolean. If `true`, this means the parent `` is pending submission. Otherwise, `false`.

* `data`: An object implementing the [`FormData interface`](https://developer.mozilla.org/en-US/docs/Web/API/FormData) that contains the data the parent `` is submitting. If there is no active submission or no parent ``, it will be `null`.

* `method`: A string value of either `'get'` or `'post'`. This represents whether the parent `` is submitting with either a `GET` or `POST` [HTTP method](https://developer.mozilla.org/en-US/docs/Web/HTTP/Methods). By default, a `` will use the `GET` method and can be specified by the [`method`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/form#method) property.

[//]: # (Link to `` documentation. "Read more on the `action` prop on ``.")
* `action`: A reference to the function passed to the `action` prop on the parent ``. If there is no parent ``, the property is `null`. If there is a URI value provided to the `action` prop, or no `action` prop specified, `status.action` will be `null`.

#### Caveats 

* The `useFormStatus` Hook must be called from a component that is rendered inside a ``.
* `useFormStatus` will only return status information for a parent ``. It will not return status information for any `` rendered in that same component or children components.

---

## Usage 

### Display a pending state during form submission 
To display a pending state while a form is submitting, you can call the `useFormStatus` Hook in a component rendered in a `` and read the `pending` property returned.

Here, we use the `pending` property to indicate the form is submitting.

### Read the form data being submitted 

You can use the `data` property of the status information returned from `useFormStatus` to display what data is being submitted by the user.

Here, we have a form where users can request a username. We can use `useFormStatus` to display a temporary status message confirming what username they have requested.

---

## Troubleshooting 

### `status.pending` is never `true` 

`useFormStatus` will only return status information for a parent ``.

If the component that calls `useFormStatus` is not nested in a ``, `status.pending` will always return `false`. Verify `useFormStatus` is called in a component that is a child of a `` element.

`useFormStatus` will not track the status of a `` rendered in the same component. See [Pitfall](#useformstatus-will-not-return-status-information-for-a-form-rendered-in-the-same-component) for more details.
