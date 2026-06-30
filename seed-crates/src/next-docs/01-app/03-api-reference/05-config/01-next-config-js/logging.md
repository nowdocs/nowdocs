---
title: logging
description: Configure logging behavior in the terminal when running Next.js in development mode, including fetch logging, incoming requests, and forwarding browser console logs to the terminal.
---

## Options

### Incoming Requests

By default all the incoming requests will be logged in the console during development. You can use the `incomingRequests` option to decide which requests to ignore.
Since this is only logged in development, this option doesn't affect production builds.

```js filename="next.config.js"
module.exports = {
  logging: {
    incomingRequests: ,
  },
}
```

Or you can disable incoming request logging by setting `incomingRequests` to `false`.

```js filename="next.config.js"
module.exports = {
  logging: ,
}
```

### Browser Console Logs

You can forward browser console logs (such as `console.log`, `console.warn`, `console.error`) to the terminal during development. This is useful for debugging client-side code without needing to check the browser's developer tools.

```js filename="next.config.js"
module.exports = {
  logging: ,
}
```

#### Options

The `browserToTerminal` option accepts the following values:

| Value     | Description                                         |
| --------- | --------------------------------------------------- |
| `'warn'`  | Forward only warnings and errors, by default        |
| `'error'` | Forward only errors                                 |
| `true`    | Forward all console output (log, info, warn, error) |
| `false`   | Disable browser log forwarding                      |

```js filename="next.config.js"
module.exports = {
  logging: ,
}
```

#### Source Location

When enabled, browser logs include source location information (file path and line number) by default. For example:

### Disabling Logging

In addition, you can disable the development logging by setting `logging` to `false`.

```js filename="next.config.js"
module.exports = 
```

## Version History

| Version   | Changes                                                                          |
| --------- | -------------------------------------------------------------------------------- |
| `v16.2.0` | `browserToTerminal` added (moved from `experimental.browserDebugInfoInTerminal`) |
| `v15.4.0` | `experimental.browserDebugInfoInTerminal` introduced                             |
| `v15.2.0` | `incomingRequests` added                                                         |
| `v15.0.0` | `logging: false` option added, `fetches.hmrRefreshes` added for App Router       |
| `v14.0.0` | `logging.fetches` moved to stable for App Router                                 |
