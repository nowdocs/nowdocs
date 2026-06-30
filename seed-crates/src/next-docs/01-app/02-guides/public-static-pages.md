---
title: Building public pages
description: Learn how to build public, "static" pages that share data across users, such as landing pages, list pages (products, blogs, etc.), marketing and news sites.
nav_title: Public pages
related:
  links:
    - app/guides/incremental-static-regeneration-cache-components
---

Public pages show the same content to every user. Common examples include landing pages, marketing pages, and product pages.

Since data is shared, these kind of pages can be [prerendered](/docs/app/glossary#prerendering) ahead of time and reused. This leads to faster page loads and lower server costs.

This guide will show you how to build public pages that share data across users.

## Example

As an example, we'll build a product list page.

We'll start with a static header, add a product list with async external data, and learn how to render it without blocking the response. Finally, we'll add a user-specific promotion banner without switching the entire page to [dynamic rendering](/docs/app/glossary#dynamic-rendering).

You can find the resources used in this example here:

- [Video](https://youtu.be/F6romq71KtI)
- [Demo](https://cache-components-public-pages.labs.vercel.dev/)
- [Code](https://github.com/vercel-labs/cache-components-public-pages)

### Step 1: Add a simple header

Let's start with a simple header.

```tsx filename="app/products/page.tsx"
// Static component
function Header() 

export default async function Page() 
```

The fallback is prerendered alongside the rest of our static and cached content. The inner component streams in later, once its async work completes.

With this change, Next.js can separate prerenderable work from request-time work and the route becomes [partially prerendered](/docs/app/glossary#partial-prerendering-ppr).

Again, we can confirm this by running `next build`:

```bash filename="Terminal"
Route (app)      Revalidate  Expire
┌ ◐ /products    15m      1y
└ ◐ /_not-found

◐  (Partial Prerender)  Prerendered as static HTML with dynamic server-streamed content
```

At [**build time**](/docs/app/glossary#build-time), most of the page, including the header, product list and promotion fallback, is rendered, cached and pushed to a content delivery network.

At [**request time**](/docs/app/glossary#dynamic-rendering), the prerendered part is served instantly from a CDN node close to the user.

In parallel, the user specific promotion is rendered on the server, streamed to the client, and swapped into the fallback slot.

If we refresh the page one last time, we can see most of the page loads instantly, while the dynamic parts stream in as they become available.

### Next steps

We've learned how to build mostly static pages that include pockets of dynamic content.

We started with a static page, added async work, and resolved the blocking behavior by caching what could be prerendered, and streaming what couldn't.

In future guides, we'll learn how to:

- Revalidate prerendered pages or cached data.
- Create variants of the same page with route params.
- Create private pages with personalized user data.
