---
title: Script Component
description: Optimize third-party scripts in your Next.js application using the built-in `next/script` Component.
---

This API reference will help you understand how to use [props](#props) available for the Script Component. For features and usage, please see the [Optimizing Scripts](/docs/app/guides/scripts) page.

```tsx filename="app/dashboard/page.tsx" switcher
import Script from 'next/script'

export default function Dashboard() 
```

> **Good to know**: Scripts with `beforeInteractive` will always be injected inside the `head` of the HTML document regardless of where it's placed in the component.

Some examples of scripts that should be fetched as soon as possible with `beforeInteractive` include:

- Bot detectors
- Cookie consent managers

### `afterInteractive`

Scripts that use the `afterInteractive` strategy are injected into the HTML client-side and will load after some (or all) hydration occurs on the page. **This is the default strategy** of the Script component and should be used for any script that needs to load as soon as possible but not before any first-party Next.js code.

`afterInteractive` scripts can be placed inside of any page or layout and will only load and execute when that page (or group of pages) is opened in the browser.

```jsx filename="app/page.js"
import Script from 'next/script'

export default function Page() {
  return (
    <>
      

### `onError`

> **Warning:** `onError` does not yet work with Server Components and can only be used in Client Components. `onError` cannot be used with the `beforeInteractive` loading strategy.

Sometimes it is helpful to catch when a script fails to load. These errors can be handled with the `onError` property:

## Version History

| Version   | Changes                                                                   |
| --------- | ------------------------------------------------------------------------- |
| `v13.0.0` | `beforeInteractive` and `afterInteractive` is modified to support `app`.  |
| `v12.2.4` | `onReady` prop added.                                                     |
| `v12.2.2` | Allow `next/script` with `beforeInteractive` to be placed in `_document`. |
| `v11.0.0` | `next/script` introduced.                                                 |
