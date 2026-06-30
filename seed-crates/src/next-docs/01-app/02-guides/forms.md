---
title: How to create forms with Server Actions
nav_title: Forms
description: Learn how to create forms in Next.js with React Server Actions.
---

React Server Actions are [Server Functions](https://react.dev/reference/rsc/server-functions) that execute on the server. They can be called in Server and Client Components to handle form submissions. This guide will walk you through how to create forms in Next.js with Server Actions. For Server Action behaviors beyond forms (single-roundtrip response, sequential dispatch, security, deployment), see [Server Actions and Mutations](/docs/app/guides/server-actions).

> [!WARNING]
> Always verify [authentication and authorization](/docs/app/guides/authentication) inside each Server Action, even if the form is only rendered on an authenticated page. See the [Data Security guide](/docs/app/guides/data-security) for more details.

## How it works

React extends the HTML [``](https://developer.mozilla.org/docs/Web/HTML/Element/form) element to allow Server Actions to be invoked with the [`action`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/form#action) attribute.

When used in a form, the function automatically receives the [`FormData`](https://developer.mozilla.org/docs/Web/API/FormData/FormData) object. You can then extract the data using the native [`FormData` methods](https://developer.mozilla.org/en-US/docs/Web/API/FormData#instance_methods):

```tsx filename="app/invoices/page.tsx" switcher
import  from '@/lib/auth'

export default function Page() {
  async function createInvoice(formData: FormData) {
    'use server'

    const session = await auth()
    if (!session?.user) 

    const rawFormData = 

    // mutate data
    // revalidate the cache
  }

  return ...
}
```

```jsx filename="app/invoices/page.js" switcher
import  from '@/lib/auth'

export default function Page() {
  async function createInvoice(formData) {
    'use server'

    const session = await auth()
    if (!session?.user) 

    const rawFormData = 

    // mutate data
    // revalidate the cache
  }

  return ...
}
```

> **Good to know:** When working with forms that have multiple fields, use JavaScript's [`Object.fromEntries()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/fromEntries). For example: `const rawFormData = Object.fromEntries(formData)`. Note that this object will contain extra properties prefixed with `$ACTION_`.

## Passing additional arguments

Outside of form fields, you can pass additional arguments to a Server Function using the JavaScript [`bind`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function/bind) method. For example, to pass the `userId` argument to the `updateUser` Server Function:

```tsx filename="app/client-component.tsx" highlight= switcher
'use client'

import  from './actions'

export function UserProfile(: ) 
```

```jsx filename="app/client-component.js" highlight= switcher
'use client'

import  from './actions'

export function UserProfile() 
```

The Server Function will receive the `userId` as an additional argument:

```ts filename="app/actions.ts" switcher
'use server'

export async function updateUser(userId: string, formData: FormData) 
```

```js filename="app/actions.js" switcher
'use server'

export async function updateUser(userId, formData) 
```

> **Good to know**:
>
> - An alternative is to pass arguments as hidden input fields in the form (e.g. ``). However, the value will be part of the rendered HTML and will not be encoded.
> - `bind` works in both Server and Client Components and supports progressive enhancement.

## Form validation

Forms can be validated on the client or server.

- For **client-side validation**, you can use the HTML attributes like `required` and `type="email"` for basic validation.
- For **server-side validation**, you can use a library like [zod](https://zod.dev/) to validate the form fields. For example:

```tsx filename="app/actions.ts" switcher
'use server'

import  from 'zod'

const schema = z.object({
  email: z.string(),
})

export default async function createUser(formData: FormData) {
  const validatedFields = schema.safeParse()

  // Return early if the form data is invalid
  if (!validatedFields.success) {
    return 
  }

  // Mutate data
}
```

```jsx filename="app/actions.js" switcher
'use server'

import  from 'zod'

const schema = z.object({
  email: z.string(),
})

export default async function createUser(formData) {
  const validatedFields = schema.safeParse()

  // Return early if the form data is invalid
  if (!validatedFields.success) {
    return 
  }

  // Mutate data
}
```

## Validation errors

To display validation errors or messages, turn the component that defines the `` into a Client Component and use React [`useActionState`](https://react.dev/reference/react/useActionState).

When using `useActionState`, the Server function signature will change to receive a new `prevState` or `initialState` parameter as its first argument.

```tsx filename="app/actions.ts" highlight= switcher
'use server'

import  from 'zod'

export async function createUser(initialState: any, formData: FormData) {
  const validatedFields = schema.safeParse()
  // ...
}
```

```jsx filename="app/actions.ts" highlight= switcher
'use server'

import  from 'zod'

// ...

export async function createUser(initialState, formData) {
  const validatedFields = schema.safeParse()
  // ...
}
```

You can then conditionally render the error message based on the `state` object.

```tsx filename="app/ui/signup.tsx" highlight= switcher
'use client'

import  from 'react'
import  from '@/app/actions'

const initialState = 

export function Signup() {
  const [state, formAction, pending] = useActionState(createUser, initialState)

  return (
    
      Email
      
      
      
      Sign up
    
  )
}
```

```jsx filename="app/ui/signup.js" highlight= switcher
'use client'

import  from 'react'
import  from '@/app/actions'

const initialState = 

export function Signup() {
  const [state, formAction, pending] = useActionState(createUser, initialState)

  return (
    
      Email
      
      
      
      Sign up
    
  )
}
```

## Pending states

The [`useActionState`](https://react.dev/reference/react/useActionState) hook exposes a `pending` boolean that can be used to show a loading indicator or disable the submit button while the action is being executed.

```tsx filename="app/ui/signup.tsx" highlight= switcher
'use client'

import  from 'react'
import  from '@/app/actions'

export function Signup() {
  const [state, formAction, pending] = useActionState(createUser, initialState)

  return (
    
      
      Sign up
    
  )
}
```

```jsx filename="app/ui/signup.js" highlight= switcher
'use client'

import  from 'react'
import  from '@/app/actions'

export function Signup() {
  const [state, formAction, pending] = useActionState(createUser, initialState)

  return (
    
      
      Sign up
    
  )
}
```

Alternatively, you can use the [`useFormStatus`](https://react.dev/reference/react-dom/hooks/useFormStatus) hook to show a loading indicator while the action is being executed. When using this hook, you'll need to create a separate component to render the loading indicator. For example, to disable the button when the action is pending:

```tsx filename="app/ui/button.tsx" highlight= switcher
'use client'

import  from 'react-dom'

export function SubmitButton() {
  const  = useFormStatus()

  return (
    
      Sign Up
    
  )
}
```

```jsx filename="app/ui/button.js" highlight= switcher
'use client'

import  from 'react-dom'

export function SubmitButton() {
  const  = useFormStatus()

  return (
    
      Sign Up
    
  )
}
```

You can then nest the `SubmitButton` component inside the form:

```tsx filename="app/ui/signup.tsx" switcher
import  from './button'
import  from '@/app/actions'

export function Signup() {
  return (
    
      
      
    
  )
}
```

```jsx filename="app/ui/signup.js" switcher
import  from './button'
import  from '@/app/actions'

export function Signup() {
  return (
    
      
      
    
  )
}
```

> **Good to know:** In React 19, `useFormStatus` includes additional keys on the returned object, like data, method, and action. If you are not using React 19, only the `pending` key is available.

## Optimistic updates

You can use the React [`useOptimistic`](https://react.dev/reference/react/useOptimistic) hook to optimistically update the UI before the Server Function finishes executing, rather than waiting for the response:

```tsx filename="app/page.tsx" switcher
'use client'

import  from 'react'
import  from './actions'

type Message = 

export function Thread(: ) {
  const [optimisticMessages, addOptimisticMessage] = useOptimistic<
    Message[],
    string
  >(messages, (state, newMessage) => [...state, ])

  const formAction = async (formData: FormData) => 

  return (
    
      {optimisticMessages.map((m, i) => (
        
      ))}
      
        
        Send
      
    
  )
}
```

```jsx filename="app/page.js" switcher
'use client'

import  from 'react'
import  from './actions'

export function Thread() {
  const [optimisticMessages, addOptimisticMessage] = useOptimistic(
    messages,
    (state, newMessage) => [...state, ]
  )

  const formAction = async (formData) => 

  return (
    
      {optimisticMessages.map((m) => (
        
      ))}
      
        
        Send
      
    
  )
}
```

## Nested form elements

You can call Server Actions in elements nested inside `` such as ``, ``, and ``. These elements accept the `formAction` prop or event handlers.

This is useful in cases where you want to call multiple Server Actions within a form. For example, you can create a specific `` element for saving a post draft in addition to publishing it. See the [React `` docs](https://react.dev/reference/react-dom/components/form#handling-multiple-submission-types) for more information.

## Programmatic form submission

You can trigger a form submission programmatically using the [`requestSubmit()`](https://developer.mozilla.org/en-US/docs/Web/API/HTMLFormElement/requestSubmit) method. For example, when the user submits a form using the `⌘` + `Enter` keyboard shortcut, you can listen for the `onKeyDown` event:

```tsx filename="app/entry.tsx" switcher
'use client'

export function Entry() {
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (
      (e.ctrlKey || e.metaKey) &&
      (e.key === 'Enter' || e.key === 'NumpadEnter')
    ) 
  }

  return (
    
      
    
  )
}
```

```jsx filename="app/entry.js" switcher
'use client'

export function Entry() {
  const handleKeyDown = (e) => {
    if (
      (e.ctrlKey || e.metaKey) &&
      (e.key === 'Enter' || e.key === 'NumpadEnter')
    ) 
  }

  return (
    
      
    
  )
}
```

This will trigger the submission of the nearest `` ancestor, which will invoke the Server Function.
