---
title: taint
description: Enable tainting Objects and Values.
version: experimental
---

## Usage

The `taint` option enables support for experimental React APIs for tainting objects and values. This feature helps prevent sensitive data from being accidentally passed to the client. When enabled, you can use:

- [`experimental_taintObjectReference`](https://react.dev/reference/react/experimental_taintObjectReference) taint objects references.
- [`experimental_taintUniqueValue`](https://react.dev/reference/react/experimental_taintUniqueValue) to taint unique values.

> **Good to know**: Activating this flag also enables the React `experimental` channel for `app` directory.

```ts filename="next.config.ts" switcher
import type  from 'next'

const nextConfig: NextConfig = {
  experimental: ,
}

export default nextConfig
```

```js filename="next.config.js" switcher
/** @type  */
const nextConfig = {
  experimental: ,
}

module.exports = nextConfig
```

> **Warning:** Do not rely on the taint API as your only mechanism to prevent exposing sensitive data to the client. See our [security recommendations](/blog/security-nextjs-server-components-actions).

The taint APIs allows you to be defensive, by declaratively and explicitly marking data that is not allowed to pass through the Server-Client boundary. When an object or value, is passed through the Server-Client boundary, React throws an error.

This is helpful for cases where:

- The methods to read data are out of your control
- You have to work with sensitive data shapes not defined by you
- Sensitive data is accessed during Server Component rendering

It is recommended to model your data and APIs so that sensitive data is not returned to contexts where it is not needed.

## Caveats

- Tainting can only keep track of objects by reference. Copying an object creates an untainted version, which loses all guarantees given by the API. You'll need to taint the copy.
- Tainting cannot keep track of data derived from a tainted value. You also need to taint the derived value.
- Values are tainted for as long as their lifetime reference is within scope. See the [`experimental_taintUniqueValue` parameters reference](https://react.dev/reference/react/experimental_taintUniqueValue#parameters), for more information.

## Examples

### Tainting an object reference

In this case, the `getUserDetails` function returns data about a given user. We taint the user object reference, so that it cannot cross a Server-Client boundary. For example, assuming `UserCard` is a Client Component.

```ts switcher
import  from 'react'

function getUserDetails(id: string): UserDetails 
```

```js switcher
import  from 'react'

function getUserDetails(id) 
```

We can still access individual fields from the tainted `userDetails` object.

```tsx filename="app/contact/page.tsx switcher
export async function ContactPage(: {
  params: Promise<>
}) {
  const  = await params
  const userDetails = await getUserDetails(id)

  return (
    
  )
}
```

```jsx filename="app/contact/page.js switcher
export async function ContactPage() {
  const  = await params
  const userDetails = await getUserDetails(id)

  return (
    
  )
}
```

Now, passing the entire object to the Client Component will throw an error.

```tsx switcher
export async function ContactPage(: {
  params: Promise<>
}) 
```

```jsx switcher
export async function ContactPage() {
  const  = await params
  const userDetails = await getUserDetails(id)

  // Throws an error
  return 
}
```

### Tainting a unique value

In this case, we can access the server configuration by awaiting calls to `config.getConfigDetails`. However, the system configuration contains the `SERVICE_API_KEY` that we don't want to expose to clients.

We can taint the `config.SERVICE_API_KEY` value.

```ts switcher
import  from 'react'

function getSystemConfig(): SystemConfig 
```

```js switcher
import  from 'react'

function getSystemConfig() 
```

We can still access other properties of the `systemConfig` object.

```tsx
export async function Dashboard() 
```

However, passing `SERVICE_API_KEY` to `ClientDashboard` throws an error.

```tsx
export async function Dashboard() 
```

Note that, even though, `systemConfig.SERVICE_API_KEY` is reassigned to a new variable. Passing it to a Client Component still throws an error.

Whereas, a value derived from a tainted unique value, will be exposed to the client.

```tsx
export async function Dashboard() {
  const systemConfig = await getSystemConfig()
  // Someone makes a mistake in a PR
  const version = `version::$`

  return 
}
```

A better approach is to remove `SERVICE_API_KEY` from the data returned by `getSystemConfig`.
