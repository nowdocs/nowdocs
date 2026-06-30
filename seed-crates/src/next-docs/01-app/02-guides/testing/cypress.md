---
title: How to set up Cypress with Next.js
nav_title: Cypress
description: Learn how to set up Cypress with Next.js for End-to-End (E2E) and Component Testing.
---

[Cypress](https://www.cypress.io/) is a test runner used for **End-to-End (E2E)** and **Component Testing**. This page will show you how to set up Cypress with Next.js and write your first tests.

> **Warning:**
>
> - Cypress versions below 13.6.3 do not support [TypeScript version 5](https://github.com/cypress-io/cypress/issues/27731) with `moduleResolution:"bundler"`. However, this issue has been resolved in Cypress version 13.6.3 and later. [cypress v13.6.3](https://docs.cypress.io/guides/references/changelog#13-6-3)

## Manual setup

To manually set up Cypress, install `cypress` as a dev dependency:

```bash package="pnpm"
pnpm add -D cypress
```

```bash package="npm"
npm install -D cypress
```

```bash package="yarn"
yarn add -D cypress
```

```bash package="bun"
bun add -D cypress
```

Add the Cypress `open` command to the `package.json` scripts field:

```json filename="package.json"
{
  "scripts": 
}
```

Run Cypress for the first time to open the Cypress testing suite:

```bash package="pnpm"
pnpm cypress:open
```

```bash package="npm"
npm run cypress:open
```

```bash package="yarn"
yarn cypress:open
```

```bash package="bun"
bun run cypress:open
```

You can choose to configure **E2E Testing** and/or **Component Testing**. Selecting any of these options will automatically create a `cypress.config.js` file and a `cypress` folder in your project.

## Creating your first Cypress E2E test

Ensure your `cypress.config` file has the following configuration:

```ts filename="cypress.config.ts" switcher
import  from 'cypress'

export default defineConfig({
  e2e: {
    setupNodeEvents(on, config) ,
  },
})
```

```js filename="cypress.config.js" switcher
const  = require('cypress')

module.exports = defineConfig({
  e2e: {
    setupNodeEvents(on, config) ,
  },
})
```

Then, create two new Next.js files:

    
  )
}
```

```jsx filename="app/about/page.js"
import Link from 'next/link'

export default function Page() 
```

    
  )
}
```

```jsx filename="pages/about.js"
import Link from 'next/link'

export default function About() 
```

Add a test to check your navigation is working correctly:

```js filename="cypress/e2e/app.cy.js"
describe('Navigation', () => {
  it('should navigate to the about page', () => )
})
```

### Running E2E Tests

Cypress will simulate a user navigating your application, this requires your Next.js server to be running. We recommend running your tests against your production code to more closely resemble how your application will behave.

Run `npm run build && npm run start` to build your Next.js application, then run `npm run cypress:open` in another terminal window to start Cypress and run your E2E Testing suite.

> **Good to know:**
>
> - You can use `cy.visit("/")` instead of `cy.visit("http://localhost:3000/")` by adding `baseUrl: 'http://localhost:3000'` to the `cypress.config.js` configuration file.
> - Alternatively, you can install the [`start-server-and-test`](https://www.npmjs.com/package/start-server-and-test) package to run the Next.js production server in conjunction with Cypress. After installation, add `"test": "start-server-and-test start http://localhost:3000 cypress"` to your `package.json` scripts field. Remember to rebuild your application after new changes.

## Creating your first Cypress component test

Component tests build and mount a specific component without having to bundle your whole application or start a server.

Select **Component Testing** in the Cypress app, then select **Next.js** as your front-end framework. A `cypress/component` folder will be created in your project, and a `cypress.config.js` file will be updated to enable Component Testing.

Ensure your `cypress.config` file has the following configuration:

```ts filename="cypress.config.ts" switcher
import  from 'cypress'

export default defineConfig({
  component: {
    devServer: ,
  },
})
```

```js filename="cypress.config.js" switcher
const  = require('cypress')

module.exports = defineConfig({
  component: {
    devServer: ,
  },
})
```

Assuming the same components from the previous section, add a test to validate a component is rendering the expected output:

> **Good to know**:
>
> - Cypress currently doesn't support Component Testing for `async` Server Components. We recommend using E2E testing.
> - Since component tests do not require a Next.js server, features like `` that rely on a server being available may not function out-of-the-box.

### Running Component Tests

Run `npm run cypress:open` in your terminal to start Cypress and run your Component Testing suite.

## Continuous Integration (CI)

In addition to interactive testing, you can also run Cypress headlessly using the `cypress run` command, which is better suited for CI environments:

```json filename="package.json"
{
  "scripts": 
}
```

You can learn more about Cypress and Continuous Integration from these resources:

- [Next.js with Cypress example](https://github.com/vercel/next.js/tree/canary/examples/with-cypress)
- [Cypress Continuous Integration Docs](https://docs.cypress.io/guides/continuous-integration/introduction)
- [Cypress GitHub Actions Guide](https://on.cypress.io/github-actions)
- [Official Cypress GitHub Action](https://github.com/cypress-io/github-action)
- [Cypress Discord](https://discord.com/invite/cypress)
