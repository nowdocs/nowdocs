---
title: How to migrate from Pages to the App Router
nav_title: App Router
description: Learn how to upgrade your existing Next.js application from the Pages Router to the App Router.
---

This guide will help you:

- [Update your Next.js application from version 12 to version 13](#nextjs-version)
- [Upgrade features that work in both the `pages` and the `app` directories](#upgrading-new-features)
- [Incrementally migrate your existing application from `pages` to `app`](#migrating-from-pages-to-app)

## Upgrading

### Node.js Version

The minimum Node.js version is now **v18.17**. See the [Node.js documentation](https://nodejs.org/docs/latest-v18.x/api/) for more information.

### Next.js Version

To update to Next.js version 13, run the following command using your preferred package manager:

```bash package="pnpm"
pnpm add next@latest react@latest react-dom@latest
```

```bash package="npm"
npm install next@latest react@latest react-dom@latest
```

```bash package="yarn"
yarn add next@latest react@latest react-dom@latest
```

```bash package="bun"
bun add next@latest react@latest react-dom@latest
```

### ESLint Version

If you're using ESLint, you need to upgrade your ESLint version:

```bash package="pnpm"
pnpm add -D eslint-config-next@latest
```

```bash package="npm"
npm install -D eslint-config-next@latest
```

```bash package="yarn"
yarn add -D eslint-config-next@latest
```

```bash package="bun"
bun add -D eslint-config-next@latest
```

> **Good to know**: You may need to restart the ESLint server in VS Code for the ESLint changes to take effect. Open the Command Palette (`cmd+shift+p` on Mac; `ctrl+shift+p` on Windows) and search for `ESLint: Restart ESLint Server`.

## Next Steps

After you've updated, see the following sections for next steps:

- [Upgrade new features](#upgrading-new-features): A guide to help you upgrade to new features such as the improved Image and Link Components.
- [Migrate from the `pages` to `app` directory](#migrating-from-pages-to-app): A step-by-step guide to help you incrementally migrate from the `pages` to the `app` directory.

## Upgrading New Features

Next.js 13 introduced the new [App Router](/docs/app) with new features and conventions. The new Router is available in the `app` directory and co-exists with the `pages` directory.

Upgrading to Next.js 13 does **not** require using the App Router. You can continue using `pages` with new features that work in both directories, such as the updated [Image component](#image-component), [Link component](#link-component), [Script component](#script-component), and [Font optimization](#font-optimization).

### `

// Next.js 13: `
```

To upgrade your links to Next.js 13, you can use the [`new-link` codemod](/docs/app/guides/upgrading/codemods#new-link).

### `
}
```

**After**

- Remove the `Page.getLayout` property from `pages/dashboard/index.js` and follow the [steps for migrating pages](#step-4-migrating-pages) to the `app` directory.

  ```jsx filename="app/dashboard/page.js"
  export default function Page() 
  ```

- Move the contents of `DashboardLayout` into a new [Client Component](/docs/app/getting-started/server-and-client-components) to retain `pages` directory behavior.

  ```jsx filename="app/dashboard/DashboardLayout.js"
  'use client' // this directive should be at top of the file, before any imports.

  // This is a Client Component
  export default function DashboardLayout() {
    return (
      
        My Dashboard
        
      
    )
  }
  ```

- Import the `DashboardLayout` into a new `layout.js` file inside the `app` directory.

  ```jsx filename="app/dashboard/layout.js"
  import DashboardLayout from './DashboardLayout'

  // This is a Server Component
  export default function Layout() 
  ```

- You can incrementally move non-interactive parts of `DashboardLayout.js` (Client Component) into `layout.js` (Server Component) to reduce the amount of component JavaScript you send to the client.

### Step 3: Migrating `next/head`

In the `pages` directory, the `next/head` React component is used to manage `` HTML elements such as `title` and `meta` . In the `app` directory, `next/head` is replaced with the new [built-in SEO support](/docs/app/getting-started/metadata-and-og-images).

**Before:**

```tsx filename="pages/index.tsx" switcher
import Head from 'next/head'

export default function Page() 
```

```jsx filename="pages/index.js" switcher
import Head from 'next/head'

export default function Page() 
```

**After:**

```tsx filename="app/page.tsx" switcher
import type  from 'next'

export const metadata: Metadata = 

export default function Page() 
```

```jsx filename="app/page.js" switcher
export const metadata = 

export default function Page() 
```

[See all metadata options](/docs/app/api-reference/functions/generate-metadata).

### Step 4: Migrating Pages

- Pages in the [`app` directory](/docs/app) are [Server Components](/docs/app/getting-started/server-and-client-components) by default. This is different from the `pages` directory where pages are [Client Components](/docs/app/getting-started/server-and-client-components).
- [Data fetching](/docs/app/getting-started/fetching-data) has changed in `app`. `getServerSideProps`, `getStaticProps` and `getInitialProps` have been replaced with a simpler API.
- The `app` directory uses nested folders to define routes and a special `page.js` file to make a route segment publicly accessible.
- | `pages` Directory | `app` Directory       | Route          |
  | ----------------- | --------------------- | -------------- |
  | `index.js`        | `page.js`             | `/`            |
  | `about.js`        | `about/page.js`       | `/about`       |
  | `blog/[slug].js`  | `blog/[slug]/page.js` | `/blog/post-1` |

We recommend breaking down the migration of a page into two main steps:

- Step 1: Move the default exported Page Component into a new Client Component.
- Step 2: Import the new Client Component into a new `page.js` file inside the `app` directory.

> **Good to know**: This is the easiest migration path because it has the most comparable behavior to the `pages` directory.

**Step 1: Create a new Client Component**

- Create a new separate file inside the `app` directory (i.e. `app/home-page.tsx` or similar) that exports a Client Component. To define Client Components, add the `'use client'` directive to the top of the file (before any imports).
  - Similar to the Pages Router, there is an [optimization step](/docs/app/getting-started/server-and-client-components#on-the-client-first-load) to prerender Client Components to static HTML on the initial page load.
- Move the default exported page component from `pages/index.js` to `app/home-page.tsx`.

```tsx filename="app/home-page.tsx" switcher
'use client'

// This is a Client Component (same as components in the `pages` directory)
// It receives data as props, has access to state and effects, and is
// prerendered on the server during the initial page load.
export default function HomePage() {
  return (
    
      {recentPosts.map((post) => (
        
      ))}
    
  )
}
```

```jsx filename="app/home-page.js" switcher
'use client'

// This is a Client Component. It receives data as props and
// has access to state and effects just like Page components
// in the `pages` directory.
export default function HomePage() {
  return (
    
      {recentPosts.map((post) => (
        
      ))}
    
  )
}
```

**Step 2: Create a new page**

- Create a new `app/page.tsx` file inside the `app` directory. This is a Server Component by default.
- Import the `home-page.tsx` Client Component into the page.
- If you were fetching data in `pages/index.js`, move the data fetching logic directly into the Server Component using the new [data fetching APIs](/docs/app/getting-started/fetching-data). See the [data fetching upgrade guide](#step-6-migrating-data-fetching-methods) for more details.

  ```tsx filename="app/page.tsx" switcher
  // Import your Client Component
  import HomePage from './home-page'

  async function getPosts() 

  export default async function Page() 
```

In the `app` directory, data fetching with [`fetch()`](/docs/app/api-reference/functions/fetch) can use `revalidate`, which will cache the request for the specified amount of seconds.

```jsx filename="app/page.js"
// `app` directory

async function getPosts() {
  const res = await fetch(`https://.../posts`, { next:  })
  const data = await res.json()

  return data.posts
}

export default async function PostList() {
  const posts = await getPosts()

  return posts.map((post) => )
}
```

#### API Routes

API Routes continue to work in the `pages/api` directory without any changes. However, they have been replaced by [Route Handlers](/docs/app/api-reference/file-conventions/route) in the `app` directory.

Route Handlers allow you to create custom request handlers for a given route using the Web [Request](https://developer.mozilla.org/docs/Web/API/Request) and [Response](https://developer.mozilla.org/docs/Web/API/Response) APIs.

```ts filename="app/api/route.ts" switcher
export async function GET(request: Request) 
```

```js filename="app/api/route.js" switcher
export async function GET(request) 
```

> **Good to know**: If you previously used API routes to call an external API from the client, you can now use [Server Components](/docs/app/getting-started/server-and-client-components) instead to securely fetch data. Learn more about [data fetching](/docs/app/getting-started/fetching-data).

#### Single-Page Applications

If you are also migrating to Next.js from a Single-Page Application (SPA) at the same time, see our [documentation](/docs/app/guides/single-page-applications) to learn more.

### Step 7: Styling

In the `pages` directory, global stylesheets are restricted to only `pages/_app.js`. With the `app` directory, this restriction has been lifted. Global styles can be added to any layout, page, or component.

- [CSS Modules](/docs/app/getting-started/css#css-modules)
- [Tailwind CSS](/docs/app/getting-started/css#tailwind-css)
- [Global Styles](/docs/app/getting-started/css#global-css)
- [CSS-in-JS](/docs/app/guides/css-in-js)
- [External Stylesheets](/docs/app/getting-started/css#external-stylesheets)
- [Sass](/docs/app/guides/sass)

#### Tailwind CSS

If you're using Tailwind CSS, you'll need to add the `app` directory to your `tailwind.config.js` file:

```js filename="tailwind.config.js"
module.exports = {
  content: [
    './app/**/*.', // <-- Add this line
    './pages/**/*.',
    './components/**/*.',
  ],
}
```

You'll also need to import your global styles in your `app/layout.js` file:

```jsx filename="app/layout.js"
import '../styles/globals.css'

export default function RootLayout() {
  return (
    
      
    
  )
}
```

Learn more about [styling with Tailwind CSS](/docs/app/getting-started/css#tailwind-css)

## Using App Router together with Pages Router

When navigating between routes served by the different Next.js routers, there will be a hard navigation. Automatic link prefetching with `next/link` will not prefetch across routers.

Instead, you can [optimize navigations](https://vercel.com/guides/optimizing-hard-navigations) between App Router and Pages Router to retain the prefetched and fast page transitions. [Learn more](https://vercel.com/guides/optimizing-hard-navigations).

## Codemods

Next.js provides Codemod transformations to help upgrade your codebase when a feature is deprecated. See [Codemods](/docs/app/guides/upgrading/codemods) for more information.
