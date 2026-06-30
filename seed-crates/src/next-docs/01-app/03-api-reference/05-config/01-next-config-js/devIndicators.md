---
title: devIndicators
description: Configuration options for the on-screen indicator that gives context about the current route you're viewing during development.
---

`devIndicators` allows you to configure the on-screen indicator that gives context about the current route you're viewing during development.

Open `next.config.ts` and set `position` to choose where the indicator renders. The default is `bottom-left`.

```ts filename="next.config.ts"
import type  from 'next'

const nextConfig: NextConfig = {
  devIndicators: ,
}

export default nextConfig
```

To hide the indicator entirely, set `devIndicators` to `false`. Next.js will still surface any compile or runtime errors that were encountered.

```ts filename="next.config.ts"
const nextConfig: NextConfig = 

export default nextConfig
```

## Troubleshooting

### Indicator not marking a route as static

If you expect a route to be static and the indicator has marked it as dynamic, it's likely the route has opted out of prerendering.

You can confirm if a route is [prerendered](/docs/app/glossary#prerendering) or [dynamically rendered](/docs/app/glossary#dynamic-rendering) by building your application using `next build --debug`, and checking the output in your terminal. Static (or prerendered) routes will display a `○` symbol, whereas dynamic routes will display a `ƒ` symbol. For example:

```bash filename="Build Output"
Route (app)
┌ ○ /_not-found
└ ƒ /products/[id]

○  (Static)   prerendered as static content
ƒ  (Dynamic)  server-rendered on demand
```

## Version History

| Version   | Changes                                                                                                                                             |
| --------- | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| `v16.0.0` | `appIsrStatus`, `buildActivity`, and `buildActivityPosition` options have been removed.                                                             |
| `v15.2.0` | Improved on-screen indicator with new `position` option. `appIsrStatus`, `buildActivity`, and `buildActivityPosition` options have been deprecated. |
| `v15.0.0` | Static on-screen indicator added with `appIsrStatus` option.                                                                                        |
