---
title: Introduction
---

## What does React Compiler do? 

React Compiler automatically optimizes your React application at build time. React is often fast enough without optimization, but sometimes you need to manually memoize components and values to keep your app responsive. This manual memoization is tedious, easy to get wrong, and adds extra code to maintain. React Compiler does this optimization automatically for you, freeing you from this mental burden so you can focus on building features.

### Before React Compiler 

Without the compiler, you need to manually memoize components and values to optimize re-renders:

```js
import  from 'react';

const ExpensiveComponent = memo(function ExpensiveComponent() {
  const processedData = useMemo(() => , [data]);

  const handleClick = useCallback((item) => , [onClick]);

  return (
    
      {processedData.map(item => (
        

### After React Compiler 

With React Compiler, you write the same code without manual memoization:

```js
function ExpensiveComponent() {
  const processedData = expensiveProcessing(data);

  const handleClick = (item) => ;

  return (
    
      {processedData.map(item => (
        

## Should I try out the compiler? 

We encourage everyone to start using React Compiler. While the compiler is still an optional addition to React today, in the future some features may require the compiler in order to fully work.

### Is it safe to use? 

React Compiler is now stable and has been tested extensively in production. While it has been used in production at companies like Meta, rolling out the compiler to production for your app will depend on the health of your codebase and how well you've followed the [Rules of React](/reference/rules).

## What build tools are supported? 

React Compiler can be installed across [several build tools](/learn/react-compiler/installation) such as Babel, Vite, Metro, and Rsbuild.

React Compiler is primarily a light Babel plugin wrapper around the core compiler, which was designed to be decoupled from Babel itself. While the initial stable version of the compiler will remain primarily a Babel plugin, we are working with the swc and [oxc](https://github.com/oxc-project/oxc/issues/10048) teams to build first class support for React Compiler so you won't have to add Babel back to your build pipelines in the future.

Next.js users can enable the swc-invoked React Compiler by using [v15.3.1](https://github.com/vercel/next.js/releases/tag/v15.3.1) and up.

## What should I do about useMemo, useCallback, and React.memo? 

By default, React Compiler will memoize your code based on its analysis and heuristics. In most cases, this memoization will be as precise, or moreso, than what you may have written.

However, in some cases developers may need more control over memoization. The `useMemo` and `useCallback` hooks can continue to be used with React Compiler as an escape hatch to provide control over which values are memoized. A common use-case for this is if a memoized value is used as an effect dependency, in order to ensure that an effect does not fire repeatedly even when its dependencies do not meaningfully change.

For new code, we recommend relying on the compiler for memoization and using `useMemo`/`useCallback` where needed to achieve precise control.

For existing code, we recommend either leaving existing memoization in place (removing it can change compilation output) or carefully testing before removing the memoization.

## Try React Compiler 

This section will help you get started with React Compiler and understand how to use it effectively in your projects.

* **[Installation](/learn/react-compiler/installation)** - Install React Compiler and configure it for your build tools
* **[React Version Compatibility](/reference/react-compiler/target)** - Support for React 17, 18, and 19
* **[Configuration](/reference/react-compiler/configuration)** - Customize the compiler for your specific needs
* **[Incremental Adoption](/learn/react-compiler/incremental-adoption)** - Strategies for gradually rolling out the compiler in existing codebases
* **[Debugging and Troubleshooting](/learn/react-compiler/debugging)** - Identify and fix issues when using the compiler
* **[Compiling Libraries](/reference/react-compiler/compiling-libraries)** - Best practices for shipping compiled code
* **[API Reference](/reference/react-compiler/configuration)** - Detailed documentation of all configuration options

## Additional resources 

In addition to these docs, we recommend checking the [React Compiler Working Group](https://github.com/reactwg/react-compiler) for additional information and discussion about the compiler.

