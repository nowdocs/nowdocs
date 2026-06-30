---
title: How to load and optimize scripts
nav_title: Scripts
description: Optimize 3rd party scripts with the built-in Script component.
related:
  title: API Reference
  description: Learn more about the next/script API.
  links:
    - app/api-reference/components/script
---

### Application Scripts

This script will load and execute when _any_ route in your application is accessed. Next.js will ensure the script will **only load once**, even if a user navigates between multiple pages.

> **Recommendation**: We recommend only including third-party scripts in specific pages or layouts in order to minimize any unnecessary impact to performance.

### Strategy

Although the default behavior of `next/script` allows you to load third-party scripts in any page or layout, you can fine-tune its loading behavior by using the `strategy` property:

- `beforeInteractive`: Load the script before any Next.js code and before any page hydration occurs.
- `afterInteractive`: (**default**) Load the script early but after some hydration on the page occurs.
- `lazyOnload`: Load the script later during browser idle time.
- `worker`: (experimental) Load the script in a web worker.

Refer to the [`next/script`](/docs/app/api-reference/components/script#strategy) API reference documentation to learn more about each strategy and their use cases.

### Offloading Scripts To A Web Worker (experimental)

> **Warning:** The `worker` strategy is not yet stable and does not yet work with the App Router. Use with caution.

Scripts that use the `worker` strategy are offloaded and executed in a web worker with [Partytown](https://partytown.qwik.dev/). This can improve the performance of your site by dedicating the main thread to the rest of your application code.

This strategy is still experimental and can only be used if the `nextScriptWorkers` flag is enabled in `next.config.js`:

```js filename="next.config.js"
module.exports = {
  experimental: ,
}
```

Then, run the development server and Next.js will guide you through the installation of the required packages to finish the setup:

```bash package="pnpm"
pnpm dev
```

```bash package="npm"
npm run dev
```

```bash package="yarn"
yarn dev
```

```bash package="bun"
bun dev
```

You'll see instructions like these: Please install Partytown by running `npm install @qwik.dev/partytown`

Once setup is complete, defining `strategy="worker"` will automatically instantiate Partytown in your application and offload the script to a web worker.

```tsx filename="pages/home.tsx" switcher
import Script from 'next/script'

export default function Home() 
```

In order to modify Partytown's configuration, the following conditions must be met:

1. The `data-partytown-config` attribute must be used in order to overwrite the default configuration used by Next.js
2. Unless you decide to save Partytown's library files in a separate directory, the `lib: "/_next/static/~partytown/"` property and value must be included in the configuration object in order to let Partytown know where Next.js stores the necessary static files.

> **Note**: If you are using an [asset prefix](/docs/pages/api-reference/config/next-config-js/assetPrefix) and would like to modify Partytown's default configuration, you must include it as part of the `lib` path.

Take a look at Partytown's [configuration options](https://partytown.qwik.dev/configuration) to see the full list of other properties that can be added.

### Inline Scripts

Inline scripts, or scripts not loaded from an external file, are also supported by the Script component. They can be written by placing the JavaScript within curly braces:

```jsx

```

Or by using the `dangerouslySetInnerHTML` property:

```jsx

### Additional Attributes

There are many DOM attributes that can be assigned to a `` element that are not used by the Script component, like [`nonce`](https://developer.mozilla.org/docs/Web/HTML/Global_attributes/nonce) or [custom data attributes](https://developer.mozilla.org/docs/Web/HTML/Global_attributes/data-*). Including any additional attributes will automatically forward it to the final, optimized `` element that is included in the HTML.

