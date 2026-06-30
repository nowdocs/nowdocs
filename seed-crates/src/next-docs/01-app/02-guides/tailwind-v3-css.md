---
title: How to install Tailwind CSS v3 in your Next.js application
nav_title: Tailwind CSS v3
description: Style your Next.js Application using Tailwind CSS v3 for broader browser support.
---

This guide will walk you through how to install [Tailwind CSS v3](https://v3.tailwindcss.com/) in your Next.js application.

> **Good to know:** For the latest Tailwind 4 setup, see the [Tailwind CSS setup instructions](/docs/app/getting-started/css#tailwind-css).

## Installing Tailwind v3

Install Tailwind CSS and its peer dependencies, then run the `init` command to generate both `tailwind.config.js` and `postcss.config.js` files:

```bash package="pnpm"
pnpm add -D tailwindcss@^3 postcss autoprefixer
npx tailwindcss init -p
```

```bash package="npm"
npm install -D tailwindcss@^3 postcss autoprefixer
npx tailwindcss init -p
```

```bash package="yarn"
yarn add -D tailwindcss@^3 postcss autoprefixer
npx tailwindcss init -p
```

```bash package="bun"
bun add -D tailwindcss@^3 postcss autoprefixer
bunx tailwindcss init -p
```

## Configuring Tailwind v3

Configure your template paths in your `tailwind.config.js` file:

## Using classes

After installing Tailwind CSS and adding the global styles, you can use Tailwind's utility classes in your application.

## Usage with Turbopack

As of Next.js 13.1, Tailwind CSS and PostCSS are supported with [Turbopack](https://turbo.build/pack/docs/features/css#tailwind-css).
