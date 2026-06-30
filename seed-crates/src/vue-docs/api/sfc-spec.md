# SFC Syntax Specification 

## Overview 

A Vue Single-File Component (SFC), conventionally using the `*.vue` file extension, is a custom file format that uses an HTML-like syntax to describe a Vue component. A Vue SFC is syntactically compatible with HTML.

Each `*.vue` file consists of three types of top-level language blocks: ``, ``, and ``, and optionally additional custom blocks:

```vue

  {}

export default {
  data() {
    return 
  }
}

.example 

  This could be e.g. documentation for the component.

```

## Language Blocks 

### `` 

- Each `*.vue` file can contain at most one top-level `` block.

- Contents will be extracted and passed on to `@vue/compiler-dom`, pre-compiled into JavaScript render functions, and attached to the exported component as its `render` option.

### `` 

- Each `*.vue` file can contain at most one `` block (excluding [``](/api/sfc-script-setup)).

- The script is executed as an ES Module.

- The **default export** should be a Vue component options object, either as a plain object or as the return value of [defineComponent](/api/general#definecomponent).

### `` 

- Each `*.vue` file can contain at most one `` block (excluding normal ``).

- The script is pre-processed and used as the component's `setup()` function, which means it will be executed **for each instance of the component**. Top-level bindings in `` are automatically exposed to the template. For more details, see [dedicated documentation on ``](/api/sfc-script-setup).

### `` 

- A single `*.vue` file can contain multiple `` tags.

- A `` tag can have `scoped` or `module` attributes (see [SFC Style Features](/api/sfc-css-features) for more details) to help encapsulate the styles to the current component. Multiple `` tags with different encapsulation modes can be mixed in the same component.

### Custom Blocks 

Additional custom blocks can be included in a `*.vue` file for any project-specific needs, for example a `` block. Some real-world examples of custom blocks include:

- [Gridsome: ``](https://gridsome.org/docs/querying-data/)
- [vite-plugin-vue-gql: ``](https://github.com/wheatjs/vite-plugin-vue-gql)
- [vue-i18n: ``](https://github.com/intlify/bundle-tools/tree/main/packages/unplugin-vue-i18n#i18n-custom-block)

Handling of Custom Blocks will depend on tooling - if you want to build your own custom block integrations, see the [SFC custom block integrations tooling section](/guide/scaling-up/tooling#sfc-custom-block-integrations) for more details.

## Automatic Name Inference 

An SFC automatically infers the component's name from its **filename** in the following cases:

- Dev warning formatting
- DevTools inspection
- Recursive self-reference, e.g. a file named `FooBar.vue` can refer to itself as `` in its template. This has lower priority than explicitly registered/imported components.

## Pre-Processors 

Blocks can declare pre-processor languages using the `lang` attribute. The most common case is using TypeScript for the `` block:

```vue-html

  // use TypeScript

```

`lang` can be applied to any block - for example we can use `` with [Sass](https://sass-lang.com/) and `` with [Pug](https://pugjs.org/api/getting-started.html):

```vue-html

p {}

  $primary-color: #333;
  body 

```

Note that integration with various pre-processors may differ by toolchain. Check out the respective documentation for examples:

- [Vite](https://vite.dev/guide/features.html#css-pre-processors)
- [Vue CLI](https://cli.vuejs.org/guide/css.html#pre-processors)
- [webpack + vue-loader](https://vue-loader.vuejs.org/guide/pre-processors.html#using-pre-processors)

## `src` Imports 

If you prefer splitting up your `*.vue` components into multiple files, you can use the `src` attribute to import an external file for a language block:

```vue

```

Beware that `src` imports follow the same path resolution rules as webpack module requests, which means:

- Relative paths need to start with `./`
- You can import resources from npm dependencies:

```vue

```

`src` imports also work with custom blocks, e.g.:

```vue

```

:::warning Note
While using aliases in `src`, don't start with `~`, anything after it is interpreted as a module request. This means you can reference assets inside node modules:
```vue

```
:::

## Comments 

Inside each block you shall use the comment syntax of the language being used (HTML, CSS, JavaScript, Pug, etc.). For top-level comments, use HTML comment syntax: ``
