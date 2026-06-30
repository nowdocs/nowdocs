---
title: How to set up Vitest with Next.js
nav_title: Vitest
description: Learn how to set up Vitest with Next.js for Unit Testing.
---

Vitest and React Testing Library are frequently used together for **Unit Testing**. This guide will show you how to setup Vitest with Next.js and write your first tests.

> **Good to know:** Since `async` Server Components are new to the React ecosystem, Vitest currently does not support them. While you can still run **unit tests** for synchronous Server and Client Components, we recommend using **E2E tests** for `async` components.

## Quickstart

You can use `create-next-app` with the Next.js [with-vitest](https://github.com/vercel/next.js/tree/canary/examples/with-vitest) example to quickly get started:

```bash package="pnpm"
pnpm create next-app --example with-vitest with-vitest-app
```

```bash package="npm"
npx create-next-app@latest --example with-vitest with-vitest-app
```

```bash package="yarn"
yarn create next-app --example with-vitest with-vitest-app
```

```bash package="bun"
bun create next-app --example with-vitest with-vitest-app
```

## Manual Setup

To manually set up Vitest, install `vitest` and the following packages as dev dependencies:

```bash package="pnpm"
# Using TypeScript
pnpm add -D vitest @vitejs/plugin-react jsdom @testing-library/react @testing-library/dom vite-tsconfig-paths
# Using JavaScript
pnpm add -D vitest @vitejs/plugin-react jsdom @testing-library/react @testing-library/dom
```

```bash package="npm"
# Using TypeScript
npm install -D vitest @vitejs/plugin-react jsdom @testing-library/react @testing-library/dom vite-tsconfig-paths
# Using JavaScript
npm install -D vitest @vitejs/plugin-react jsdom @testing-library/react @testing-library/dom
```

```bash package="yarn"
# Using TypeScript
yarn add -D vitest @vitejs/plugin-react jsdom @testing-library/react @testing-library/dom vite-tsconfig-paths
# Using JavaScript
yarn add -D vitest @vitejs/plugin-react jsdom @testing-library/react @testing-library/dom
```

```bash package="bun"
# Using TypeScript
bun add -D vitest @vitejs/plugin-react jsdom @testing-library/react @testing-library/dom vite-tsconfig-paths
# Using JavaScript
bun add -D vitest @vitejs/plugin-react jsdom @testing-library/react @testing-library/dom
```

Create a `vitest.config.mts|js` file in the root of your project, and add the following options:

```ts filename="vitest.config.mts" switcher
import  from 'vitest/config'
import react from '@vitejs/plugin-react'
import tsconfigPaths from 'vite-tsconfig-paths'

export default defineConfig({
  plugins: [tsconfigPaths(), react()],
  test: ,
})
```

```js filename="vitest.config.js" switcher
import  from 'vitest/config'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  test: ,
})
```

For more information on configuring Vitest, please refer to the [Vitest Configuration](https://vitest.dev/config/#configuration) docs.

Then, add a `test` script to your `package.json`:

```json filename="package.json"
{
  "scripts": 
}
```

When you run `npm run test`, Vitest will **watch** for changes in your project by default.

## Creating your first Vitest Unit Test

Check that everything is working by creating a test to check if the `
    
  )
}
```

```jsx filename="app/page.jsx" switcher
import Link from 'next/link'

export default function Page() 
```

```tsx filename="__tests__/page.test.tsx" switcher
import  from 'vitest'
import  from '@testing-library/react'
import Page from '../app/page'

test('Page', () => 
```

```jsx filename="pages/index.jsx" switcher
import Link from 'next/link'

export default function Page() 
```

```tsx filename="__tests__/index.test.tsx" switcher
import  from 'vitest'
import  from '@testing-library/react'
import Page from '../pages/index'

test('Page', () => {
  render(

## Running your tests

Then, run the following command to run your tests:

```bash package="pnpm"
pnpm test
```

```bash package="npm"
npm run test
```

```bash package="yarn"
yarn test
```

```bash package="bun"
bun run test
```

## Additional Resources

You may find these resources helpful:

- [Next.js with Vitest example](https://github.com/vercel/next.js/tree/canary/examples/with-vitest)
- [Vitest Docs](https://vitest.dev/guide/)
- [React Testing Library Docs](https://testing-library.com/docs/react-testing-library/intro/)
