---
title: How to use Sass
nav_title: Sass
description: Style your Next.js application using Sass.
---

Next.js has built-in support for integrating with Sass after the package is installed using both the `.scss` and `.sass` extensions. You can use component-level Sass via CSS Modules and the `.module.scss`or `.module.sass` extension.

First, install [`sass`](https://github.com/sass/sass):

```bash package="pnpm"
pnpm add -D sass
```

```bash package="npm"
npm install --save-dev sass
```

```bash package="yarn"
yarn add -D sass
```

```bash package="bun"
bun add -D sass
```

> **Good to know**:
>
> Sass supports [two different syntaxes](https://sass-lang.com/documentation/syntax), each with their own extension.
> The `.scss` extension requires you use the [SCSS syntax](https://sass-lang.com/documentation/syntax#scss),
> while the `.sass` extension requires you use the [Indented Syntax ("Sass")](https://sass-lang.com/documentation/syntax#the-indented-syntax).
>
> If you're not sure which to choose, start with the `.scss` extension which is a superset of CSS, and doesn't require you learn the
> Indented Syntax ("Sass").

### Customizing Sass Options

If you want to configure your Sass options, use `sassOptions` in `next.config`.

```ts filename="next.config.ts" switcher
import type  from 'next'

const nextConfig: NextConfig = {
  sassOptions: ,
}

export default nextConfig
```

```js filename="next.config.js" switcher
/** @type  */

const nextConfig = {
  sassOptions: ,
}

module.exports = nextConfig
```

#### Implementation

You can use the `implementation` property to specify the Sass implementation to use. By default, Next.js uses the [`sass`](https://www.npmjs.com/package/sass) package.

```ts filename="next.config.ts" switcher
import type  from 'next'

const nextConfig: NextConfig = {
  sassOptions: ,
}

export default nextConfig
```

```js filename="next.config.js" switcher
/** @type  */

const nextConfig = {
  sassOptions: ,
}

module.exports = nextConfig
```

### Sass Variables

Next.js supports Sass variables exported from CSS Module files.

For example, using the exported `primaryColor` Sass variable:

```scss filename="app/variables.module.scss"
$primary-color: #64ff00;

:export 
```

  )
}
```

