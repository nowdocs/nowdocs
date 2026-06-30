---
title: Server Functions
---

When a Server Function is defined with the [`"use server"`](/reference/rsc/use-server) directive, your framework will automatically create a reference to the Server Function, and pass that reference to the Client Component. When that function is called on the client, React will send a request to the server to execute the function, and return the result.

Server Functions can be created in Server Components and passed as props to Client Components, or they can be imported and used in Client Components.

## Usage 

### Creating a Server Function from a Server Component 

Server Components can define Server Functions with the `"use server"` directive:

```js [[2, 7, "'use server'"], [1, 5, "createNoteAction"], [1, 12, "createNoteAction"]]
// Server Component
import Button from './Button';

function EmptyNote () {
  async function createNoteAction() 

  return  is provided to `useActionState`, React will redirect to the provided URL if the form is submitted before the JavaScript bundle loads.

For more, see the docs for [`useActionState`](/reference/react/useActionState).
