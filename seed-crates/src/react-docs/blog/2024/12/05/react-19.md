---
title: "React v19"
author: The React Team
date: 2024/12/05
description: React 19 is now available on npm! In this post, we'll give an overview of the new features in React 19, and how you can adopt them.
---

December 05, 2024 by [The React Team](/community/team)

---

In our [React 19 Upgrade Guide](/blog/2024/04/25/react-19-upgrade-guide), we shared step-by-step instructions for upgrading your app to React 19. In this post, we'll give an overview of the new features in React 19, and how you can adopt them.

- [What's new in React 19](#whats-new-in-react-19)
- [Improvements in React 19](#improvements-in-react-19)
- [How to upgrade](#how-to-upgrade)

For a list of breaking changes, see the [Upgrade Guide](/blog/2024/04/25/react-19-upgrade-guide).

---

## What's new in React 19 

### Actions 

A common use case in React apps is to perform a data mutation and then update state in response. For example, when a user submits a form to change their name, you will make an API request, and then handle the response. In the past, you would need to handle pending states, errors, optimistic updates, and sequential requests manually.

For example, you could handle the pending and error state in `useState`:

```js
// Before Actions
function UpdateName() {
  const [name, setName] = useState("");
  const [error, setError] = useState(null);
  const [isPending, setIsPending] = useState(false);

  const handleSubmit = async () => {
    setIsPending(true);
    const error = await updateName(name);
    setIsPending(false);
    if (error) 
    redirect("/path");
  };

  return (
    
       setName(event.target.value)} />
      
        Update
      
      {error && }
    
  );
}
```

In React 19, we're adding support for using async functions in transitions to handle pending states, errors, forms, and optimistic updates automatically.

For example, you can use `useTransition` to handle the pending state for you:

```js
// Using pending state from Actions
function UpdateName() {
  const [name, setName] = useState("");
  const [error, setError] = useState(null);
  const [isPending, startTransition] = useTransition();

  const handleSubmit = () => {
    startTransition(async () => {
      const error = await updateName(name);
      if (error) 
      redirect("/path");
    })
  };

  return (
    
       setName(event.target.value)} />
      
        Update
      
      {error && }
    
  );
}
```

The async transition will immediately set the `isPending` state to true, make the async request(s), and switch `isPending` to false after any transitions. This allows you to keep the current UI responsive and interactive while the data is changing.

Building on top of Actions, React 19 introduces [`useOptimistic`](#new-hook-optimistic-updates) to manage optimistic updates, and a new hook [`React.useActionState`](#new-hook-useactionstate) to handle common cases for Actions. In `react-dom` we're adding [`` Actions](#form-actions) to manage forms automatically and [`useFormStatus`](#new-hook-useformstatus) to support the common cases for Actions in forms.

In React 19, the above example can be simplified to:

```js
// Using  Actions and useActionState
function ChangeName() {
  const [error, submitAction, isPending] = useActionState(
    async (previousState, formData) => {
      const error = await updateName(formData.get("name"));
      if (error) 
      redirect("/path");
      return null;
    },
    null,
  );

  return (
    
      
      Update
      {error && }
    
  );
}
```

In the next section, we'll break down each of the new Action features in React 19.

### New hook: `useActionState` 

To make the common cases easier for Actions, we've added a new hook called `useActionState`:

```js
const [error, submitAction, isPending] = useActionState(
  async (previousState, newName) => {
    const error = await updateName(newName);
    if (error) 

    // handle success
    return null;
  },
  null,
);
```

`useActionState` accepts a function (the "Action"), and returns a wrapped Action to call. This works because Actions compose. When the wrapped Action is called, `useActionState` will return the last result of the Action as `data`, and the pending state of the Action as `pending`.

For more information, see the docs for [`useActionState`](/reference/react/useActionState).

### React DOM: `` Actions 

Actions are also integrated with React 19's new `` features for `react-dom`. We've added support for passing functions as the `action` and `formAction` props of ``, ``, and `` elements to automatically submit forms with Actions:

```js [[1,1,"actionFunction"]]

```

When a `` Action succeeds, React will automatically reset the form for uncontrolled components. If you need to reset the `` manually, you can call the new `requestFormReset` React DOM API.

For more information, see the `react-dom` docs for [``](/reference/react-dom/components/form), [``](/reference/react-dom/components/input), and ``.

### React DOM: New hook: `useFormStatus` 

In design systems, it's common to write design components that need access to information about the `` they're in, without drilling props down to the component. This can be done via Context, but to make the common case easier, we've added a new hook `useFormStatus`:

```js [[1, 4, "pending"], [1, 5, "pending"]]
import  from 'react-dom';

function DesignButton() {
  const  = useFormStatus();
  return 
}
```

`useFormStatus` reads the status of the parent `` as if the form was a Context provider.

For more information, see the `react-dom` docs for [`useFormStatus`](/reference/react-dom/hooks/useFormStatus).

### New hook: `useOptimistic` 

Another common UI pattern when performing a data mutation is to show the final state optimistically while the async request is underway. In React 19, we're adding a new hook called `useOptimistic` to make this easier:

```js 
function ChangeName() {
  const [optimisticName, setOptimisticName] = useOptimistic(currentName);

  const submitAction = async formData => ;

  return (
    
      Your name is: 
      
        Change Name:
        
      
    
  );
}
```

The `useOptimistic` hook will immediately render the `optimisticName` while the `updateName` request is in progress. When the update finishes or errors, React will automatically switch back to the `currentName` value.

For more information, see the docs for [`useOptimistic`](/reference/react/useOptimistic).

### New API: `use` 

In React 19 we're introducing a new API to read resources in render: `use`.

For example, you can read a promise with `use`, and React will Suspend until the promise resolves:

```js 
import  from 'react';

function Comments() {
  // `use` will suspend until the promise resolves.
  const comments = use(commentsPromise);
  return comments.map(comment => );
}

function Page() 
```

To fix, you need to pass a promise from a Suspense powered library or framework that supports caching for promises. In the future we plan to ship features to make it easier to cache promises in render.

You can also read context with `use`, allowing you to read Context conditionally such as after early returns:

```js 
import  from 'react';
import ThemeContext from './ThemeContext'

function Heading() {
  if (children == null) 

  // This would not work with useContext
  // because of the early return.
  const theme = use(ThemeContext);
  return (
    
      
    
  );
}
```

The `use` API can only be called in render, similar to hooks. Unlike hooks, `use` can be called conditionally. In the future we plan to support more ways to consume resources in render with `use`.

For more information, see the docs for [`use`](/reference/react/use).

## New React DOM Static APIs 

We've added two new APIs to `react-dom/static` for static site generation:
- [`prerender`](/reference/react-dom/static/prerender)
- [`prerenderToNodeStream`](/reference/react-dom/static/prerenderToNodeStream)

These new APIs improve on `renderToString` by waiting for data to load for static HTML generation. They are designed to work with streaming environments like Node.js Streams and Web Streams. For example, in a Web Stream environment, you can prerender a React tree to static HTML with `prerender`:

```js
import  from 'react-dom/static';

async function handler(request) {
  const  = await prerender(

For more, see the docs for [React Server Components](/reference/rsc/server-components).

### Server Actions 

Server Actions allow Client Components to call async functions executed on the server.

When a Server Action is defined with the `"use server"` directive, your framework will automatically create a reference to the server function, and pass that reference to the Client Component. When that function is called on the client, React will send a request to the server to execute the function, and return the result.

Server Actions can be created in Server Components and passed as props to Client Components, or they can be imported and used in Client Components.

For more, see the docs for [React Server Actions](/reference/rsc/server-actions).

## Improvements in React 19 

### `ref` as a prop 

Starting in React 19, you can now access `ref` as a prop for function components:

```js [[1, 1, "ref"], [1, 2, "ref", 45], [1, 6, "ref", 14]]
function MyInput() 

//...

### Diffs for hydration errors 

We also improved error reporting for hydration errors in `react-dom`. For example, instead of logging multiple errors in DEV without any information about the mismatch:

We now log a single message with a diff of the mismatch:

### `
  );
}
```

New Context providers can use `

Due to the introduction of ref cleanup functions, returning anything else from a `ref` callback will now be rejected by TypeScript. The fix is usually to stop using implicit returns, for example:

```diff [[1, 1, "("], [1, 1, ")"], [2, 2, "", 1]]
-  (instance = current)} />
+  } />
```

The original code returned the instance of the `HTMLDivElement` and TypeScript wouldn't know if this was _supposed_ to be a cleanup function or if you didn't want to return a cleanup function.

You can codemod this pattern with [`no-implicit-ref-callback-return`](https://github.com/eps1lon/types-react-codemod/#no-implicit-ref-callback-return).

### `useDeferredValue` initial value 

We've added an `initialValue` option to `useDeferredValue`:

```js [[1, 1, "deferredValue"], [1, 4, "deferredValue"], [2, 4, "''"]]
function Search() {
  // On initial render the value is ''.
  // Then a re-render is scheduled with the deferredValue.
  const value = useDeferredValue(deferredValue, '');

  return (
     is provided, `useDeferredValue` will return it as `value` for the initial render of the component, and schedules a re-render in the background with the  returned.

For more, see [`useDeferredValue`](/reference/react/useDeferredValue).

### Support for Document Metadata 

In HTML, document metadata tags like ``, ``, and `` are reserved for placement in the `` section of the document. In React, the component that decides what metadata is appropriate for the app may be very far from the place where you render the `` or React does not render the `` at all. In the past, these elements would need to be inserted manually in an effect, or by libraries like [`react-helmet`](https://github.com/nfl/react-helmet), and required careful handling when server rendering a React application.

In React 19, we're adding support for rendering document metadata tags in components natively:

```js 
function BlogPost() {
  return (
    
      
      
      
      
      
      
        Eee equals em-see-squared...
      
    
  );
}
```

When React renders this component, it will see the `` `` and `` tags, and automatically hoist them to the `` section of document. By supporting these metadata tags natively, we're able to ensure they work with client-only apps, streaming SSR, and Server Components.

For more info, see the docs for [``](/reference/react-dom/components/title), [``](/reference/react-dom/components/link), and [``](/reference/react-dom/components/meta).

### Support for stylesheets 

Stylesheets, both externally linked (``) and inline (`...`), require careful positioning in the DOM due to style precedence rules. Building a stylesheet capability that allows for composability within components is hard, so users often end up either loading all of their styles far from the components that may depend on them, or they use a style library which encapsulates this complexity.

In React 19, we're addressing this complexity and providing even deeper integration into Concurrent Rendering on the Client and Streaming Rendering on the Server with built in support for stylesheets. If you tell React the `precedence` of your stylesheet it will manage the insertion order of the stylesheet in the DOM and ensure that the stylesheet (if external) is loaded before revealing content that depends on those style rules.

```js 
function ComponentOne() 

function ComponentTwo() {
  return (
    
      
        <-- will be inserted between foo & bar
    
  )
}
```

During Server Side Rendering React will include the stylesheet in the ``, which ensures that the browser will not paint until it has loaded. If the stylesheet is discovered late after we've already started streaming, React will ensure that the stylesheet is inserted into the `` on the client before revealing the content of a Suspense boundary that depends on that stylesheet.

During Client Side Rendering React will wait for newly rendered stylesheets to load before committing the render. If you render this component from multiple places within your application React will only include the stylesheet once in the document:

```js 
function App() {
  return <>
    

In React 19, we log a single error with all the error information included:

Additionally, we've added two new root options to complement `onRecoverableError`:

- `onCaughtError`: called when React catches an error in an Error Boundary.
- `onUncaughtError`: called when an error is thrown and not caught by an Error Boundary.
- `onRecoverableError`: called when an error is thrown and automatically recovered.

For more info and examples, see the docs for [`createRoot`](/reference/react-dom/client/createRoot) and [`hydrateRoot`](/reference/react-dom/client/hydrateRoot).

### Support for Custom Elements 

React 19 adds full support for custom elements and passes all tests on [Custom Elements Everywhere](https://custom-elements-everywhere.com/).

In past versions, using Custom Elements in React has been difficult because React treated unrecognized props as attributes rather than properties. In React 19, we've added support for properties that works on the client and during SSR with the following strategy:

- **Server Side Rendering**: props passed to a custom element will render as attributes if their type is a primitive value like `string`, `number`, or the value is `true`. Props with non-primitive types like `object`, `symbol`, `function`, or value `false` will be omitted.
- **Client Side Rendering**: props that match a property on the Custom Element instance will be assigned as properties, otherwise they will be assigned as attributes.

Thanks to [Joey Arhar](https://github.com/josepharhar) for driving the design and implementation of Custom Element support in React.

#### How to upgrade 
See the [React 19 Upgrade Guide](/blog/2024/04/25/react-19-upgrade-guide) for step-by-step instructions and a full list of breaking and notable changes.

_Note: this post was originally published 04/25/2024 and has been updated to 12/05/2024 with the stable release._
