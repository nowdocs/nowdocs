---
title: Compiling Libraries
---

Configure your build tool to compile your library. For example, with Babel:

```js
// babel.config.js
module.exports = ;
```

## Backwards Compatibility 

If your library supports React versions below 19, you'll need additional configuration:

### 1. Install the runtime package 

We recommend installing react-compiler-runtime as a direct dependency:

```json
{
  "dependencies": ,
  "peerDependencies": 
}
```

### 2. Configure the target version 

Set the minimum React version your library supports:

```js

```

## Testing Strategy 

Test your library both with and without compilation to ensure compatibility. Run your existing test suite against the compiled code, and also create a separate test configuration that bypasses the compiler. This helps catch any issues that might arise from the compilation process and ensures your library works correctly in all scenarios.

## Troubleshooting 

### Library doesn't work with older React versions 

If your compiled library throws errors in React 17 or 18:

1. Verify you've installed `react-compiler-runtime` as a dependency
2. Check that your `target` configuration matches your minimum supported React version
3. Ensure the runtime package is included in your published bundle

### Compilation conflicts with other Babel plugins 

Some Babel plugins may conflict with React Compiler:

1. Place `babel-plugin-react-compiler` early in your plugin list
2. Disable conflicting optimizations in other plugins
3. Test your build output thoroughly

### Runtime module not found 

If users see "Cannot find module 'react-compiler-runtime'":

1. Ensure the runtime is listed in `dependencies`, not `devDependencies`
2. Check that your bundler includes the runtime in the output
3. Verify the package is published to npm with your library

## Next Steps 

- Learn about [debugging techniques](/learn/react-compiler/debugging) for compiled code
- Check the [configuration options](/reference/react-compiler/configuration) for all compiler options
- Explore [compilation modes](/reference/react-compiler/compilationMode) for selective optimization