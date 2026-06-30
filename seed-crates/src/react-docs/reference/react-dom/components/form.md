---
title: ""
---

### Handle form submission with an action prop 

Pass a function to the `action` prop of form to run the function when the form is submitted. [`formData`](https://developer.mozilla.org/en-US/docs/Web/API/FormData) will be passed to the function as an argument so you can access the data submitted by the form. This differs from the conventional [HTML action](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/form#action), which only accepts URLs. Unlike `onSubmit`, an `action` runs in a [Transition](/reference/react/useTransition) and calling `e.preventDefault()` isn't needed. After the `action` function succeeds, all uncontrolled field elements in the form are reset.

### Handle form submission with a Server Function 

Render a `` with an input and submit button. Pass a Server Function (a function marked with [`'use server'`](/reference/rsc/use-server)) to the `action` prop of form to run the function when the form is submitted.

Passing a Server Function to `` allow users to submit forms without JavaScript enabled or before the code has loaded. This is beneficial to users who have a slow connection, device, or have JavaScript disabled and is similar to the way forms work when a URL is passed to the `action` prop.

You can use hidden form fields to provide data to the ``'s action. The Server Function will be called with the hidden form field data as an instance of [`FormData`](https://developer.mozilla.org/en-US/docs/Web/API/FormData).

```jsx
import  from './lib.js';

function AddToCart() {
  async function addToCart(formData) 
  return (
    
        
        Add to Cart
    

  );
}
```

In lieu of using hidden form fields to provide data to the ``'s action, you can call the  method to supply it with extra arguments. This will bind a new argument () to the function in addition to the  that is passed as an argument to the function.

```jsx [[1, 8, "bind"], [2,8, "productId"], [2,4, "productId"], [3,4, "formData"]]
import  from './lib.js';

function AddToCart() {
  async function addToCart(productId, formData) 
  const addProductToCart = addToCart.bind(null, productId);
  return (
    
      Add to Cart
    
  );
}
```

When `` is rendered by a [Server Component](/reference/rsc/use-client), and a [Server Function](/reference/rsc/server-functions) is passed to the ``'s `action` prop, the form is [progressively enhanced](https://developer.mozilla.org/en-US/docs/Glossary/Progressive_Enhancement).

### Display a pending state during form submission 
To display a pending state when a form is being submitted, you can call the `useFormStatus` Hook in a component rendered in a `` and read the `pending` property returned.

Here, we use the `pending` property to indicate the form is submitting.

To learn more about the `useFormStatus` Hook see the [reference documentation](/reference/react-dom/hooks/useFormStatus).

### Optimistically updating form data 
The `useOptimistic` Hook provides a way to optimistically update the user interface before a background operation, like a network request, completes. In the context of forms, this technique helps to make apps feel more responsive. When a user submits a form, instead of waiting for the server's response to reflect the changes, the interface is immediately updated with the expected outcome.

For example, when a user types a message into the form and hits the "Send" button, the `useOptimistic` Hook allows the message to immediately appear in the list with a "Sending..." label, even before the message is actually sent to a server. This "optimistic" approach gives the impression of speed and responsiveness. The form then attempts to truly send the message in the background. Once the server confirms the message has been received, the "Sending..." label is removed.

[//]: # 'Uncomment the next line, and delete this line after the `useOptimistic` reference documentation page is published'
[//]: # 'To learn more about the `useOptimistic` Hook see the [reference documentation](/reference/react/useOptimistic).'

### Handling form submission errors 

In some cases the function called by a ``'s `action` prop throws an error. You can handle these errors by wrapping `` in an Error Boundary. If the function called by a ``'s `action` prop throws an error, the fallback for the error boundary will be displayed.

  );
}

```

```json package.json hidden
{
  "dependencies": ,
  "main": "/index.js",
  "devDependencies": 
}
```

### Display a form submission error without JavaScript 

Displaying a form submission error message before the JavaScript bundle loads for progressive enhancement requires that:

1. `` be rendered by a [Client Component](/reference/rsc/use-client)
1. the function passed to the ``'s `action` prop be a [Server Function](/reference/rsc/server-functions)
1. the `useActionState` Hook be used to display the error message

`useActionState` takes two parameters: a [Server Function](/reference/rsc/server-functions) and an initial state. `useActionState` returns two values, a state variable and an action. The action returned by `useActionState` should be passed to the `action` prop of the form. The state variable returned by `useActionState` can be used to display an error message. The value returned by the Server Function passed to `useActionState` will be used to update the state variable.

Learn more about updating state from a form action with the [`useActionState`](/reference/react/useActionState) docs

### Handling multiple submission types 

Forms can be designed to handle multiple submission actions based on the button pressed by the user. Each button inside a form can be associated with a distinct action or behavior by setting the `formAction` prop.

When a user taps a specific button, the form is submitted, and a corresponding action, defined by that button's attributes and action, is executed. For instance, a form might submit an article for review by default but have a separate button with `formAction` set to save the article as a draft.

