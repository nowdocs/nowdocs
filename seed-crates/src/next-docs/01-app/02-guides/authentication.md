---
title: How to implement authentication in Next.js
nav_title: Authentication
description: Learn how to implement authentication in your Next.js application.
---

Understanding authentication is crucial for protecting your application's data. This page will guide you through what React and Next.js features to use to implement auth.

Before starting, it helps to break down the process into three concepts:

1. **[Authentication](#authentication)**: Verifies if the user is who they say they are. It requires the user to prove their identity with something they have, such as a username and password.
2. **[Session Management](#session-management)**: Tracks the user's auth state across requests.
3. **[Authorization](#authorization)**: Decides what routes and data the user can access.

This diagram shows the authentication flow using React and Next.js features:

## Session Management

Session management ensures that the user's authenticated state is preserved across requests. It involves creating, storing, refreshing, and deleting sessions or tokens.

There are two types of sessions:

1. [**Stateless**](#stateless-sessions): Session data (or a token) is stored in the browser's cookies. The cookie is sent with each request, allowing the session to be verified on the server. This method is simpler, but can be less secure if not implemented correctly.
2. [**Database**](#database-sessions): Session data is stored in a database, with the user's browser only receiving the encrypted session ID. This method is more secure, but can be complex and use more server resources.

> **Good to know:** While you can use either method, or both, we recommend using a session management library such as [iron-session](https://github.com/vvo/iron-session) or [Jose](https://github.com/panva/jose).

### Stateless Sessions

### Database Sessions

To create and manage database sessions, you'll need to follow these steps:

1. Create a table in your database to store session and data (or check if your Auth Library handles this).
2. Implement functionality to insert, update, and delete sessions
3. Encrypt the session ID before storing it in the user's browser, and ensure the database and cookie stay in sync (this is optional, but recommended for optimistic auth checks in [Proxy](#optimistic-checks-with-proxy-optional)).

## Authorization

Once a user is authenticated and a session is created, you can implement authorization to control what the user can access and do within your application.

There are two main types of authorization checks:

1. **Optimistic**: Checks if the user is authorized to access a route or perform an action using the session data stored in the cookie. These checks are useful for quick operations, such as showing/hiding UI elements or redirecting users based on permissions or roles.
2. **Secure**: Checks if the user is authorized to access a route or perform an action using the session data stored in the database. These checks are more secure and are used for operations that require access to sensitive data or actions.

For both cases, we recommend:

- Creating a [Data Access Layer](#creating-a-data-access-layer-dal) to centralize your authorization logic
- Using [Data Transfer Objects (DTO)](#using-data-transfer-objects-dto) to only return the necessary data
- Optionally use [Proxy](#optimistic-checks-with-proxy-optional) to perform optimistic checks.

### Optimistic checks with Proxy (Optional)

There are some cases where you may want to use [Proxy](/docs/app/api-reference/file-conventions/proxy) and redirect users based on permissions:

- To perform optimistic checks. Since Proxy runs on every route, it's a good way to centralize redirect logic and pre-filter unauthorized users.
- To protect static routes that share data between users (e.g. content behind a paywall).

However, since Proxy runs on every route, including [prefetched](/docs/app/getting-started/linking-and-navigating#prefetching) routes, it's important to only read the session from the cookie (optimistic checks), and avoid database checks to prevent performance issues.

For example:

```tsx filename="proxy.ts" switcher
import  from 'next/server'
import  from '@/app/lib/session'
import  from 'next/headers'

// 1. Specify protected and public routes
const protectedRoutes = ['/dashboard']
const publicRoutes = ['/login', '/signup', '/']

export default async function proxy(req: NextRequest) {
  // 2. Check if the current route is protected or public
  const path = req.nextUrl.pathname
  const isProtectedRoute = protectedRoutes.includes(path)
  const isPublicRoute = publicRoutes.includes(path)

  // 3. Decrypt the session from the cookie
  const cookie = (await cookies()).get('session')?.value
  const session = await decrypt(cookie)

  // 4. Redirect to /login if the user is not authenticated
  if (isProtectedRoute && !session?.userId) 

  // 5. Redirect to /dashboard if the user is authenticated
  if (
    isPublicRoute &&
    session?.userId &&
    !req.nextUrl.pathname.startsWith('/dashboard')
  ) 

  return NextResponse.next()
}

// Routes Proxy should not run on
export const config = 
```

```js filename="proxy.js" switcher
import  from 'next/server'
import  from '@/app/lib/session'
import  from 'next/headers'

// 1. Specify protected and public routes
const protectedRoutes = ['/dashboard']
const publicRoutes = ['/login', '/signup', '/']

export default async function proxy(req) {
  // 2. Check if the current route is protected or public
  const path = req.nextUrl.pathname
  const isProtectedRoute = protectedRoutes.includes(path)
  const isPublicRoute = publicRoutes.includes(path)

  // 3. Decrypt the session from the cookie
  const cookie = (await cookies()).get('session')?.value
  const session = await decrypt(cookie)

  // 5. Redirect to /login if the user is not authenticated
  if (isProtectedRoute && !session?.userId) 

  // 6. Redirect to /dashboard if the user is authenticated
  if (
    isPublicRoute &&
    session?.userId &&
    !req.nextUrl.pathname.startsWith('/dashboard')
  ) 

  return NextResponse.next()
}

// Routes Proxy should not run on
export const config = 
```

While Proxy can be useful for initial checks, it should not be your only line of defense in protecting your data. The majority of security checks should be performed as close as possible to your data source, see [Data Access Layer](#creating-a-data-access-layer-dal) for more information.

> **Tips**:
>
> - In Proxy, you can also read cookies using `req.cookies.get('session')?.value`.
> - Proxy uses the Node.js runtime, check if your Auth library and session management library are compatible.
> - You can use the `matcher` property in the Proxy to specify which routes Proxy should run on. Although, for auth, it's recommended Proxy runs on all routes.

      
    
  )
}
```

```tsx filename="app/ui/profile.tsx" switcher
'use client'

import  from "auth-lib";

export default function Profile() {
  const  = useSession();
  const  = useSWR(`/api/user/$`, fetcher)

  return (
    // ...
  );
}
```

```jsx filename="app/ui/profile.jsx" switcher
'use client'

import  from "auth-lib";

export default function Profile() {
  const  = useSession();
  const  = useSWR(`/api/user/$`, fetcher)

  return (
    // ...
  );
}
```

If session data is needed in Client Components (e.g. for client-side data fetching), use React’s [`taintUniqueValue`](https://react.dev/reference/react/experimental_taintUniqueValue) API to prevent sensitive session data from being exposed to the client.

## Resources

Now that you've learned about authentication in Next.js, here are Next.js-compatible libraries and resources to help you implement secure authentication and session management:

### Auth Libraries

- [Auth0](https://auth0.com/docs/quickstart/webapp/nextjs)
- [Better Auth](https://www.better-auth.com/docs/integrations/next)
- [Clerk](https://clerk.com/docs/quickstarts/nextjs)
- [Descope](https://docs.descope.com/getting-started/nextjs)
- [Kinde](https://kinde.com/docs/developer-tools/nextjs-sdk)
- [Logto](https://docs.logto.io/quick-starts/next-app-router)
- [NextAuth.js](https://authjs.dev/getting-started/installation?framework=next.js)
- [Ory](https://www.ory.sh/docs/getting-started/integrate-auth/nextjs)
- [Stack Auth](https://docs.stack-auth.com/getting-started/setup)
- [Supabase](https://supabase.com/docs/guides/getting-started/quickstarts/nextjs)
- [Stytch](https://stytch.com/docs/guides/quickstarts/nextjs)
- [WorkOS](https://workos.com/docs/user-management/nextjs)

### Session Management Libraries

- [Iron Session](https://github.com/vvo/iron-session)
- [Jose](https://github.com/panva/jose)

## Further Reading

To continue learning about authentication and security, check out the following resources:

- [How to think about security in Next.js](/blog/security-nextjs-server-components-actions)
- [Understanding XSS Attacks](https://vercel.com/guides/understanding-xss-attacks)
- [Understanding CSRF Attacks](https://vercel.com/guides/understanding-csrf-attacks)
- [The Copenhagen Book](https://thecopenhagenbook.com/)
