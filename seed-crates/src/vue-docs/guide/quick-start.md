---
footer: false
---

import  from '@vue/theme'

# Quick Start 

## Try Vue Online 

- To quickly get a taste of Vue, you can try it directly in our [Playground](https://play.vuejs.org/#eNo9jcEKwjAMhl/lt5fpQYfXUQfefAMvvRQbddC1pUuHUPrudg4HIcmXjyRZXEM4zYlEJ+T0iEPgXjn6BB8Zhp46WUZWDjCa9f6w9kAkTtH9CRinV4fmRtZ63H20Ztesqiylphqy3R5UYBqD1UyVAPk+9zkvV1CKbCv9poMLiTEfR2/IXpSoXomqZLtti/IFwVtA9A==).

- If you prefer a plain HTML setup without any build steps, you can use this [JSFiddle](https://jsfiddle.net/yyx990803/2ke1ab0z/) as your starting point.

- If you are already familiar with Node.js and the concept of build tools, you can also try a complete build setup right within your browser on [StackBlitz](https://vite.new/vue).

- To get a walkthrough of the recommended setup, watch this interactive [Scrimba](http://scrimba.com/links/vue-quickstart) tutorial that shows you how to run, edit, and deploy your first Vue app.

## Creating a Vue Application 

:::tip Prerequisites

- Familiarity with the command line
- Install [Node.js](https://nodejs.org/) version `^20.19.0 || >=22.12.0`
  :::

In this section we will introduce how to scaffold a Vue [Single Page Application](/guide/extras/ways-of-using-vue#single-page-application-spa) on your local machine. The created project will be using a build setup based on [Vite](https://vite.dev/) and allow us to use Vue [Single-File Components](/guide/scaling-up/sfc) (SFCs).

Make sure you have an up-to-date version of [Node.js](https://nodejs.org/) installed and your current working directory is the one where you intend to create a project. Run the following command in your command line (without the `$` sign):

::: code-group

```sh [npm]
$ npm create vue@latest
```

```sh [pnpm]
$ pnpm create vue@latest
```

```sh [yarn]
# For Yarn (v1+)
$ yarn create vue

# For Yarn Modern (v2+)
$ yarn create vue@latest
  
# For Yarn ^v4.11
$ yarn dlx create-vue@latest
```

```sh [bun]
$ bun create vue@latest
```
:::

This command will install and execute [create-vue](https://github.com/vuejs/create-vue), the official Vue project scaffolding tool. You will be presented with prompts for several optional features such as TypeScript and testing support:

✔ Project name: … &lt;your-project-name&gt;
✔ Add TypeScript? … No / Yes
✔ Add JSX Support? … No / Yes
✔ Add Vue Router for Single Page Application development? … No / Yes
✔ Add Pinia for state management? … No / Yes
✔ Add Vitest for Unit testing? … No / Yes
✔ Add an End-to-End Testing Solution? … No / Cypress / Nightwatch / Playwright
✔ Add ESLint for code quality? … No / Yes
✔ Add Prettier for code formatting? … No / Yes
✔ Add Vue DevTools 7 extension for debugging? (experimental) … No / Yes

Scaffolding project in ./&lt;your-project-name&gt;...
Done.

If you are unsure about an option, simply choose `No` by hitting enter for now. Once the project is created, follow the instructions to install dependencies and start the dev server:

::: code-group

```sh-vue [npm]
$ cd {}
$ npm install
$ npm run dev
```

```sh-vue [pnpm]
$ cd {}
$ pnpm install
$ pnpm run dev
```

```sh-vue [yarn]
$ cd {}
$ yarn
$ yarn dev
```

```sh-vue [bun]
$ cd {}
$ bun install
$ bun run dev
```

:::

You should now have your first Vue project running! Note that the example components in the generated project are written using the [Composition API](/guide/introduction#composition-api) and ``, rather than the [Options API](/guide/introduction#options-api). Here are some additional tips:

- The recommended IDE setup is [Visual Studio Code](https://code.visualstudio.com/) + [Vue - Official extension](https://marketplace.visualstudio.com/items?itemName=Vue.volar). If you use other editors, check out the [IDE support section](/guide/scaling-up/tooling#ide-support).
- More tooling details, including integration with backend frameworks, are discussed in the [Tooling Guide](/guide/scaling-up/tooling).
- To learn more about the underlying build tool Vite, check out the [Vite docs](https://vite.dev/).
- If you choose to use TypeScript, check out the [TypeScript Usage Guide](typescript/overview).

When you are ready to ship your app to production, run the following:

::: code-group

```sh [npm]
$ npm run build
```

```sh [pnpm]
$ pnpm run build
```

```sh [yarn]
$ yarn build
```

```sh [bun]
$ bun run build
```

:::

This will create a production-ready build of your app in the project's `./dist` directory. Check out the [Production Deployment Guide](/guide/best-practices/production-deployment) to learn more about shipping your app to production.

[Next Steps >](#next-steps)

## Using Vue from CDN 

You can use Vue directly from a CDN via a script tag:

```html

```

Here we are using [unpkg](https://unpkg.com/), but you can also use any CDN that serves npm packages, for example [jsdelivr](https://www.jsdelivr.com/package/npm/vue) or [cdnjs](https://cdnjs.com/libraries/vue). Of course, you can also download this file and serve it yourself.

When using Vue from a CDN, there is no "build step" involved. This makes the setup a lot simpler, and is suitable for enhancing static HTML or integrating with a backend framework. However, you won't be able to use the Single-File Component (SFC) syntax.

### Using the Global Build 

The above link loads the _global build_ of Vue, where all top-level APIs are exposed as properties on the global `Vue` object. Here is a full example using the global build:

```html

{}

  const  = Vue

  createApp({
    data() {
      return 
    }
  }).mount('#app')

```

[CodePen Demo >](https://codepen.io/vuejs-examples/pen/QWJwJLp)

```html

{}

  const  = Vue

  createApp({
    setup() {
      const message = ref('Hello vue!')
      return 
    }
  }).mount('#app')

```

[CodePen Demo >](https://codepen.io/vuejs-examples/pen/eYQpQEG)

:::tip
Many of the examples for Composition API throughout the guide will be using the `` syntax, which requires build tools. If you intend to use Composition API without a build step, consult the usage of the [`setup()` option](/api/composition-api-setup).
:::

### Using the ES Module Build 

Throughout the rest of the documentation, we will be primarily using [ES modules](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Modules) syntax. Most modern browsers now support ES modules natively, so we can use Vue from a CDN via native ES modules like this:

```html
{}

  import  from 'https://unpkg.com/vue@3/dist/vue.esm-browser.js'

  createApp({
    data() {
      return 
    }
  }).mount('#app')

```

```html
{}

  import  from 'https://unpkg.com/vue@3/dist/vue.esm-browser.js'

  createApp({
    setup() {
      const message = ref('Hello Vue!')
      return 
    }
  }).mount('#app')

```

Notice that we are using ``, and the imported CDN URL is pointing to the **ES modules build** of Vue instead.

[CodePen Demo >](https://codepen.io/vuejs-examples/pen/VwVYVZO)

[CodePen Demo >](https://codepen.io/vuejs-examples/pen/MWzazEv)

### Enabling Import maps 

In the above example, we are importing from the full CDN URL, but in the rest of the documentation you will see code like this:

```js
import  from 'vue'
```

We can teach the browser where to locate the `vue` import by using [Import Maps](https://caniuse.com/import-maps):

```html

  {
    "imports": 
  }

{}

  import  from 'vue'

  createApp({
    data() {
      return 
    }
  }).mount('#app')

```

[CodePen Demo >](https://codepen.io/vuejs-examples/pen/wvQKQyM)

```html

  {
    "imports": 
  }

{}

  import  from 'vue'

  createApp({
    setup() {
      const message = ref('Hello Vue!')
      return 
    }
  }).mount('#app')

```

[CodePen Demo >](https://codepen.io/vuejs-examples/pen/YzRyRYM)

You can also add entries for other dependencies to the import map - but make sure they point to the ES modules version of the library you intend to use.

:::tip Import Maps Browser Support
Import Maps is a relatively new browser feature. Make sure to use a browser within its [support range](https://caniuse.com/import-maps). In particular, it is only supported in Safari 16.4+.
:::

:::warning Notes on Production Use
The examples so far are using the development build of Vue - if you intend to use Vue from a CDN in production, make sure to check out the [Production Deployment Guide](/guide/best-practices/production-deployment#without-build-tools).

While it is possible to use Vue without a build system, an alternative approach to consider is using [`vuejs/petite-vue`](https://github.com/vuejs/petite-vue) that could better suit the context where [`jquery/jquery`](https://github.com/jquery/jquery) (in the past) or [`alpinejs/alpine`](https://github.com/alpinejs/alpine) (in the present) might be used instead.
:::

### Splitting Up the Modules 

As we dive deeper into the guide, we may need to split our code into separate JavaScript files so that they are easier to manage. For example:

```html [index.html]

  import  from 'vue'
  import MyComponent from './my-component.js'

  createApp(MyComponent).mount('#app')

```

```js [my-component.js]
export default {
  data() {
    return 
  },
  template: `Count is: {}`
}
```

```js [my-component.js]
import  from 'vue'
export default {
  setup() {
    const count = ref(0)
    return 
  },
  template: `Count is: {}`
}
```

If you directly open the above `index.html` in your browser, you will find that it throws an error because ES modules cannot work over the `file://` protocol, which is the protocol the browser uses when you open a local file.

Due to security reasons, ES modules can only work over the `http://` protocol, which is what the browsers use when opening pages on the web. In order for ES modules to work on our local machine, we need to serve the `index.html` over the `http://` protocol, with a local HTTP server.

To start a local HTTP server, first make sure you have [Node.js](https://nodejs.org/en/) installed, then run `npx serve` from the command line in the same directory where your HTML file is. You can also use any other HTTP server that can serve static files with the correct MIME types.

You may have noticed that the imported component's template is inlined as a JavaScript string. If you are using VS Code, you can install the [es6-string-html](https://marketplace.visualstudio.com/items?itemName=Tobermory.es6-string-html) extension and prefix the strings with a `/*html*/` comment to get syntax highlighting for them.

## Next Steps 

If you skipped the [Introduction](/guide/introduction), we strongly recommend reading it before moving on to the rest of the documentation.

  
    Continue with the Guide
    The guide walks you through every aspect of the framework in full detail.
  
  
    Try the Tutorial
    For those who prefer learning things hands-on.
  
  
    Check out the Examples
    Explore examples of core features and common UI tasks.
  

