---
title: use server
description: Learn how to use the use server directive to execute code on the server.
---

The `use server` directive designates a function or file to be executed on the **server side**. It can be used at the top of a file to indicate that all functions in the file are server-side, or inline at the top of a function to mark the function as a [Server Function](https://19.react.dev/reference/rsc/server-functions). This is a React feature.

For Next.js-specific Server Action behaviors (response model, security, configuration, deployment), see [Server Actions and Mutations](/docs/app/guides/server-actions).

## Using `use server` at the top of a file

The following example shows a file with a `use server` directive at the top. All functions in the file are executed on the server.

```tsx filename="app/actions.ts" highlight= switcher
'use server'
import  from '@/lib/db' // Your database client
import  from '@/lib/auth'

export async function createUser(data: ) {
  const session = await auth()
  if (!session?.user) 

  const user = await db.user.create()
  return 
}
```

```jsx filename="app/actions.js" highlight= switcher
'use server'
import  from '@/lib/db' // Your database client
import  from '@/lib/auth'

export async function createUser(data) {
  const session = await auth()
  if (!session?.user) 

  const user = await db.user.create()
  return 
}
```

### Using Server Functions in a Client Component

To use Server Functions in Client Components you need to create your Server Functions in a dedicated file using the `use server` directive at the top of the file. These Server Functions can then be imported into Client and Server Components and executed.

Assuming you have a `fetchUsers` Server Function in `actions.ts`:

```tsx filename="app/actions.ts" highlight= switcher
'use server'
import  from '@/lib/db' // Your database client
import  from '@/lib/auth'

export async function fetchUsers() {
  const session = await auth()
  if (!session?.user) 

  const users = await db.user.findMany({
    select: ,
  })
  return users
}
```

```jsx filename="app/actions.js" highlight= switcher
'use server'
import  from '@/lib/db' // Your database client
import  from '@/lib/auth'

export async function fetchUsers() {
  const session = await auth()
  if (!session?.user) 

  const users = await db.user.findMany({
    select: ,
  })
  return users
}
```

Then you can import the `fetchUsers` Server Function into a Client Component and execute it on the client-side.

```tsx filename="app/components/my-button.tsx" highlight= switcher
'use client'
import  from '../actions'

export default function MyButton() >Fetch Users
}
```

```jsx filename="app/components/my-button.js" highlight= switcher
'use client'
import  from '../actions'

export default function MyButton() >Fetch Users
}
```

## Using `use server` inline

In the following example, `use server` is used inline at the top of a function to mark it as a [Server Function](https://19.react.dev/reference/rsc/server-functions):

```tsx filename="app/posts/[id]/page.tsx" switcher highlight=
import  from './edit-post'
import  from 'next/cache'

export default async function PostPage(: { params:  }) {
  const post = await getPost(params.id)

  async function updatePost(formData: FormData) {
    'use server'
    // Verify auth before saving (e.g. inside savePost)
    await savePost(params.id, formData)
    revalidatePath(`/posts/$`)
  }

  return 
}
```

```jsx filename="app/posts/[id]/page.js" switcher highlight=
import  from './edit-post'
import  from 'next/cache'

export default async function PostPage() {
  const post = await getPost(params.id)

  async function updatePost(formData) {
    'use server'
    // Verify auth before saving (e.g. inside savePost)
    await savePost(params.id, formData)
    revalidatePath(`/posts/$`)
  }

  return 
}
```

## Security considerations

Design your data access functions as secure primitives: validate inputs, check authentication and authorization, and constrain return types to only what the caller needs. When Server Functions delegate to a [Data Access Layer](/docs/app/guides/data-security#using-a-data-access-layer-for-mutations), these guarantees live in one place and apply consistently.

### Authentication and authorization

Always authenticate and authorize users before performing sensitive server-side operations. Read authentication from cookies or headers rather than accepting tokens as function parameters.

```tsx filename="app/actions.ts" highlight= switcher
'use server'

import  from '@/lib/db' // Your database client
import  from '@/lib/auth' // Your authentication library

export async function createUser(data: ) {
  const session = await auth()
  if (!session?.user) 
  const newUser = await db.user.create()
  return 
}
```

```jsx filename="app/actions.js" highlight= switcher
'use server'

import  from '@/lib/db' // Your database client
import  from '@/lib/auth' // Your authentication library

export async function createUser(data) {
  const session = await auth()
  if (!session?.user) 
  const newUser = await db.user.create()
  return 
}
```

### Return values

Server Function return values are serialized and sent to the client. Only return data the UI needs, not raw database records. See the [Data Security guide](/docs/app/guides/data-security#controlling-return-values) for more details.

## Reference

See the [React documentation](https://react.dev/reference/rsc/use-server) for more information on `use server`.
