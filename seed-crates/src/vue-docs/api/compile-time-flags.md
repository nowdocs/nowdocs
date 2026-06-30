---
outline: deep
---

# Compile-Time Flags 

:::tip
Compile-time flags only apply when using the `esm-bundler` build of Vue (i.e. `vue/dist/vue.esm-bundler.js`).
:::

When using Vue with a build step, it is possible to configure a number of compile-time flags to enable / disable certain features. The benefit of using compile-time flags is that features disabled this way can be removed from the final bundle via tree-shaking.

Vue will work even if these flags are not explicitly configured. However, it is recommended to always configure them so that the relevant features can be properly removed when possible.

See [Configuration Guides](#configuration-guides) on how to configure them depending on your build tool.

## `__VUE_OPTIONS_API__` 

- **Default:** `true`

  Enable / disable Options API support. Disabling this will result in smaller bundles, but may affect compatibility with 3rd party libraries if they rely on Options API.

## `__VUE_PROD_DEVTOOLS__` 

- **Default:** `false`

  Enable / disable devtools support in production builds. This will result in more code included in the bundle, so it is recommended to only enable this for debugging purposes.

## `__VUE_PROD_HYDRATION_MISMATCH_DETAILS__` 

- **Default:** `false`

  Enable/disable detailed warnings for hydration mismatches in production builds. This will result in more code included in the bundle, so it is recommended to only enable this for debugging purposes.

- Only available in 3.4+

## Configuration Guides 

### Vite 

`@vitejs/plugin-vue` automatically provides default values for these flags. To change the default values, use Vite's [`define` config option](https://vite.dev/config/shared-options.html#define):

```js [vite.config.js]
import  from 'vite'

export default defineConfig({
  define: 
})
```

### vue-cli 

`@vue/cli-service` automatically provides default values for some of these flags. To configure /change the values:

```js [vue.config.js]
module.exports = {
  chainWebpack: (config) => {
    config.plugin('define').tap((definitions) => {
      Object.assign(definitions[0], )
      return definitions
    })
  }
}
```

### webpack 

Flags should be defined using webpack's [DefinePlugin](https://webpack.js.org/plugins/define-plugin/):

```js [webpack.config.js]
module.exports = {
  // ...
  plugins: [
    new webpack.DefinePlugin()
  ]
}
```

### Rollup 

Flags should be defined using [@rollup/plugin-replace](https://github.com/rollup/plugins/tree/master/packages/replace):

```js [rollup.config.js]
import replace from '@rollup/plugin-replace'

export default {
  plugins: [
    replace()
  ]
}
```
