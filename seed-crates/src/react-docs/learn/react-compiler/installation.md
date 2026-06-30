---
title: Installation
---

## Prerequisites 

React Compiler is designed to work best with React 19, but it also supports React 17 and 18. Learn more about [React version compatibility](/reference/react-compiler/target).

## Installation 

Install React Compiler as a `devDependency`:

Or with Yarn:

Or with pnpm:

## Basic Setup 

React Compiler is designed to work by default without any configuration. However, if you need to configure it in special circumstances (for example, to target React versions below 19), refer to the [compiler options reference](/reference/react-compiler/configuration).

The setup process depends on your build tool. React Compiler includes a Babel plugin that integrates with your build pipeline.

### Babel 

Create or update your `babel.config.js`:

```js 
module.exports = ;
```

### Vite 

If you use Vite with version 6.0.0 or later of `@vitejs/plugin-react`, you can use the `reactCompilerPreset`:

```js 
// vite.config.js
import  from 'vite';
import react,  from '@vitejs/plugin-react';
import babel from '@rolldown/plugin-babel';

export default defineConfig({
  plugins: [
    react(),
    babel(),
  ],
});
```

Alternatively, you can use the Babel plugin directly with `@rolldown/plugin-babel`:

```js 
// vite.config.js
import  from 'vite';
import react from '@vitejs/plugin-react';
import babel from '@rolldown/plugin-babel';

export default defineConfig({
  plugins: [
    react(),
    babel(),
  ],
});
```

### Next.js 

Please refer to the [Next.js docs](https://nextjs.org/docs/app/api-reference/next-config-js/reactCompiler) for more information.

### React Router 
Install `vite-plugin-babel`, and add the compiler's Babel plugin to it:

```js 
// vite.config.js
import  from "vite";
import babel from "vite-plugin-babel";
import  from "@react-router/dev/vite";

const ReactCompilerConfig = ;

export default defineConfig({
  plugins: [
    reactRouter(),
    babel({
      filter: /\.[jt]sx?$/,
      babelConfig: ,
    }),
  ],
});
```

### Webpack 

A community webpack loader is [now available here](https://github.com/SukkaW/react-compiler-webpack).

### Expo 

Please refer to [Expo's docs](https://docs.expo.dev/guides/react-compiler/) to enable and use the React Compiler in Expo apps.

### Metro (React Native) 

React Native uses Babel via Metro, so refer to the [Usage with Babel](#babel) section for installation instructions.

### Rspack 

Please refer to [Rspack's docs](https://rspack.dev/guide/tech/react#react-compiler) to enable and use the React Compiler in Rspack apps.

### Rsbuild 

Please refer to [Rsbuild's docs](https://rsbuild.dev/guide/framework/react#react-compiler) to enable and use the React Compiler in Rsbuild apps.

## ESLint Integration 

React Compiler includes an ESLint rule that helps identify code that can't be optimized. When the ESLint rule reports an error, it means the compiler will skip optimizing that specific component or hook. This is safe: the compiler will continue optimizing other parts of your codebase. You don't need to fix all violations immediately. Address them at your own pace to gradually increase the number of optimized components.

Install the ESLint plugin:

If you haven't already configured eslint-plugin-react-hooks, follow the [installation instructions in the readme](https://github.com/facebook/react/blob/main/packages/eslint-plugin-react-hooks/README.md#installation). The compiler rules are available in the `recommended-latest` preset.

The ESLint rule will:
- Identify violations of the [Rules of React](/reference/rules)
- Show which components can't be optimized
- Provide helpful error messages for fixing issues

## Verify Your Setup 

After installation, verify that React Compiler is working correctly.

### Check React DevTools 

Components optimized by React Compiler will show a "Memo ✨" badge in React DevTools:

1. Install the [React Developer Tools](/learn/react-developer-tools) browser extension
2. Open your app in development mode
3. Open React DevTools
4. Look for the ✨ emoji next to component names

If the compiler is working:
- Components will show a "Memo ✨" badge in React DevTools
- Expensive calculations will be automatically memoized
- No manual `useMemo` is required

### Check Build Output 

You can also verify the compiler is running by checking your build output. The compiled code will include automatic memoization logic that the compiler adds automatically.

```js
import  from "react/compiler-runtime";
export default function MyApp() {
  const $ = _c(1);
  let t0;
  if ($[0] === Symbol.for("react.memo_cache_sentinel"))  else 
  return t0;
}

```

## Troubleshooting 

### Opting out specific components 

If a component is causing issues after compilation, you can temporarily opt it out using the `"use no memo"` directive:

```js
function ProblematicComponent() 
```

This tells the compiler to skip optimization for this specific component. You should fix the underlying issue and remove the directive once resolved.

For more troubleshooting help, see the [debugging guide](/learn/react-compiler/debugging).

## Next Steps 

Now that you have React Compiler installed, learn more about:

- [React version compatibility](/reference/react-compiler/target) for React 17 and 18
- [Configuration options](/reference/react-compiler/configuration) to customize the compiler
- [Incremental adoption strategies](/learn/react-compiler/incremental-adoption) for existing codebases
- [Debugging techniques](/learn/react-compiler/debugging) for troubleshooting issues
- [Compiling Libraries guide](/reference/react-compiler/compiling-libraries) for compiling your React library
