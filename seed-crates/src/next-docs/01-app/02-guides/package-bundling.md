---
title: Optimizing package bundling
nav_title: Package Bundling
description: Learn how to analyze and optimize your application's server and client bundles with the Next.js Bundle Analyzer for Turbopack, and the `@next/bundle-analyzer` plugin for Webpack.
related:
  description: Learn more about optimizing your application for production.
  links:
    - app/guides/production-checklist
---

Bundling is the process of combining your application code and its dependencies into optimized output files for the client and server. Smaller bundles load faster, reduce JavaScript execution time, improve [Core Web Vitals](https://web.dev/articles/vitals), and lower server cold start times.

Next.js automatically optimizes bundles by code splitting, tree-shaking, and other techniques. However, there are some cases where you may need to optimize your bundles manually.

There are two tools for analyzing your application's bundles:

- [Next.js Bundle Analyzer for Turbopack (experimental)](#nextjs-bundle-analyzer-experimental)
- [`@next/bundle-analyzer` plugin for Webpack](#nextbundle-analyzer-for-webpack)

This guide will walk you through how to use each tool and how to [optimize large bundles](#optimizing-large-bundles).

## Next.js Bundle Analyzer (Experimental)

> Available in v16.1 and later. You can share feedback in the [dedicated GitHub discussion](https://github.com/vercel/next.js/discussions/86731) and view the demo at [turbopack-bundle-analyzer-demo.vercel.sh](https://turbopack-bundle-analyzer-demo.vercel.sh/).

The Next.js Bundle Analyzer is integrated with Turbopack's module graph. You can inspect server and client modules with precise import tracing, making it easier to find large dependencies. Open the interactive Bundle Analyzer demo to explore the module graph.

### Step 1: Run the Turbopack Bundle Analyzer

To get started, run the following command and open up the interactive view in your browser.

```bash filename="Terminal" package="npm"
npx next experimental-analyze
```

```bash filename="Terminal" package="yarn"
yarn next experimental-analyze
```

```bash filename="Terminal" package="pnpm"
pnpm next experimental-analyze
```

```bash filename="Terminal" package="bun"
bunx next experimental-analyze
```

### Step 2: Filter and inspect modules

Within the UI, you can filter by route, environment (client or server), and type (JavaScript, CSS, JSON), or search by file:

    
  )
}
```

This increases bundle size because the client must download and execute the highlighting library, even though the result is static HTML.

Instead, move the highlighting logic to a Server Component and render the final HTML on the server. The client will only receive the rendered markup.

```tsx filename="app/blog/[slug]/page.tsx"
import  from 'shiki'

export default async function Page() {
  const code = `export function hello() `

  // The Shiki package runs on the server and is never bundled for the client.
  const highlightedHtml = await codeToHtml(code, )

  return (
    
      Blog Post Title

      
      
        
      
    
  )
}
```

