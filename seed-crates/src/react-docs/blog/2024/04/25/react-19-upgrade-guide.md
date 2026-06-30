---
title: "React 19 Upgrade Guide"
author: Ricky Hanlon
date: 2024/04/25
description: The improvements added to React 19 require some breaking changes, but we've worked to make the upgrade as smooth as possible and we don't expect the changes to impact most apps. In this post, we will guide you through the steps for upgrading apps and libraries to React 19.
---

April 25, 2024 by [Ricky Hanlon](https://twitter.com/rickhanlonii)

---

In this post, we will guide you through the steps for upgrading to React 19:

- [Installing](#installing)
- [Codemods](#codemods)
- [Breaking changes](#breaking-changes)
- [New deprecations](#new-deprecations)
- [Notable changes](#notable-changes)
- [TypeScript changes](#typescript-changes)
- [Changelog](#changelog)

If you'd like to help us test React 19, follow the steps in this upgrade guide and [report any issues](https://github.com/facebook/react/issues/new?assignees=&labels=React+19&projects=&template=19.md&title=%5BReact+19%5D) you encounter. For a list of new features added to React 19, see the [React 19 release post](/blog/2024/12/05/react-19).

---
## Installing 

We expect most apps will not be affected since the transform is enabled in most environments already. For manual instructions on how to upgrade, please see the [announcement post](https://legacy.reactjs.org/blog/2020/09/22/introducing-the-new-jsx-transform.html).

To install the latest version of React and React DOM:

```bash
npm install --save-exact react@^19.0.0 react-dom@^19.0.0
```

Or, if you're using Yarn:

```bash
yarn add --exact react@^19.0.0 react-dom@^19.0.0
```

If you're using TypeScript, you also need to update the types.
```bash
npm install --save-exact @types/react@^19.0.0 @types/react-dom@^19.0.0
```

Or, if you're using Yarn:
```bash
yarn add --exact @types/react@^19.0.0 @types/react-dom@^19.0.0
```

We're also including a codemod for the most common replacements. See [TypeScript changes](#typescript-changes) below.

## Codemods 

To help with the upgrade, we've worked with the team at [codemod.com](https://codemod.com) to publish codemods that will automatically update your code to many of the new APIs and patterns in React 19.

All codemods are available in the [`react-codemod` repo](https://github.com/reactjs/react-codemod) and the Codemod team have joined in helping maintain the codemods. To run these codemods, we recommend using the `codemod` command instead of the `react-codemod` because it runs faster, handles more complex code migrations, and provides better support for TypeScript.

Changes that include a codemod include the command below.

For a list of all available codemods, see the [`react-codemod` repo](https://github.com/reactjs/react-codemod).

## Breaking changes 

### Errors in render are not re-thrown 

In previous versions of React, errors thrown during render were caught and rethrown. In DEV, we would also log to `console.error`, resulting in duplicate error logs.

In React 19, we've [improved how errors are handled](/blog/2024/04/25/react-19#error-handling) to reduce duplication by not re-throwing:

- **Uncaught Errors**: Errors that are not caught by an Error Boundary are reported to `window.reportError`.
- **Caught Errors**: Errors that are caught by an Error Boundary are reported to `console.error`.

This change should not impact most apps, but if your production error reporting relies on errors being re-thrown, you may need to update your error handling. To support this, we've added new methods to `createRoot` and `hydrateRoot` for custom error handling:

```js [[1, 2, "onUncaughtError"], [2, 5, "onCaughtError"]]
const root = createRoot(container, {
  onUncaughtError: (error, errorInfo) => ,
  onCaughtError: (error, errorInfo) => 
});
```

For more info, see the docs for [`createRoot`](https://react.dev/reference/react-dom/client/createRoot) and [`hydrateRoot`](https://react.dev/reference/react-dom/client/hydrateRoot).

### Removed deprecated React APIs 

#### Removed: `propTypes` and `defaultProps` for functions 
`PropTypes` were deprecated in [April 2017 (v15.5.0)](https://legacy.reactjs.org/blog/2017/04/07/react-v15.5.0.html#new-deprecation-warnings).

In React 19, we're removing the `propType` checks from the React package, and using them will be silently ignored. If you're using `propTypes`, we recommend migrating to TypeScript or another type-checking solution.

We're also removing `defaultProps` from function components in place of ES6 default parameters. Class components will continue to support `defaultProps` since there is no ES6 alternative.

```js
// Before
import PropTypes from 'prop-types';

function Heading() {
  return ;
}
Heading.propTypes = ;
Heading.defaultProps = ;
```
```ts
// After
interface Props 
function Heading(: Props) {
  return ;
}
```

#### Removed: Legacy Context using `contextTypes` and `getChildContext` 

Legacy Context was deprecated in [October 2018 (v16.6.0)](https://legacy.reactjs.org/blog/2018/10/23/react-v-16-6.html).

Legacy Context was only available in class components using the APIs `contextTypes` and `getChildContext`, and was replaced with `contextType` due to subtle bugs that were easy to miss. In React 19, we're removing Legacy Context to make React slightly smaller and faster.

If you're still using Legacy Context in class components, you'll need to migrate to the new `contextType` API:

```js 
// Before
import PropTypes from 'prop-types';

class Parent extends React.Component {
  static childContextTypes = ;

  getChildContext() {
    return ;
  }

  render() 
}

class Child extends React.Component {
  static contextType = FooContext;

  render() {
    return ;
  }
}
```

#### Removed: string refs 
String refs were deprecated in [March, 2018 (v16.3.0)](https://legacy.reactjs.org/blog/2018/03/27/update-on-async-rendering.html).

Class components supported string refs before being replaced by ref callbacks due to [multiple downsides](https://github.com/facebook/react/issues/1373). In React 19, we're removing string refs to make React simpler and easier to understand.

If you're still using string refs in class components, you'll need to migrate to ref callbacks:

```js 
// Before
class MyComponent extends React.Component {
  componentDidMount() 

  render() 
}
```

```js 
// After
class MyComponent extends React.Component {
  componentDidMount() 

  render()  />;
  }
}
```

#### Removed: Module pattern factories 
Module pattern factories were deprecated in [August 2019 (v16.9.0)](https://legacy.reactjs.org/blog/2019/08/08/react-v16.9.0.html#deprecating-module-pattern-factories).

This pattern was rarely used and supporting it causes React to be slightly larger and slower than necessary. In React 19, we're removing support for module pattern factories, and you'll need to migrate to regular functions:

```js
// Before
function FactoryComponent() {
  return { render()  }
}
```

```js
// After
function FactoryComponent() 
```

#### Removed: `React.createFactory` 
`createFactory` was deprecated in [February 2020 (v16.13.0)](https://legacy.reactjs.org/blog/2020/02/26/react-v16.13.0.html#deprecating-createfactory).

Using `createFactory` was common before broad support for JSX, but it's rarely used today and can be replaced with JSX. In React 19, we're removing `createFactory` and you'll need to migrate to JSX:

```js
// Before
import  from 'react';

const button = createFactory('button');
```

```js
// After
const button = ;
```

#### Removed: `react-test-renderer/shallow` 

In React 18, we updated `react-test-renderer/shallow` to re-export [react-shallow-renderer](https://github.com/enzymejs/react-shallow-renderer). In React 19, we're removing `react-test-render/shallow` to prefer installing the package directly:

```bash
npm install react-shallow-renderer --save-dev
```
```diff
- import ShallowRenderer from 'react-test-renderer/shallow';
+ import ShallowRenderer from 'react-shallow-renderer';
```

### Removed deprecated React DOM APIs 

#### Removed: `react-dom/test-utils` 

We've moved `act` from `react-dom/test-utils` to the `react` package:

To fix this warning, you can import `act` from `react`:

```diff
- import  from 'react-dom/test-utils'
+ import  from 'react';
```

All other `test-utils` functions have been removed. These utilities were uncommon, and made it too easy to depend on low level implementation details of your components and React. In React 19, these functions will error when called and their exports will be removed in a future version.

See the [warning page](https://react.dev/warnings/react-dom-test-utils) for alternatives.

#### Removed: `ReactDOM.render` 

`ReactDOM.render` was deprecated in [March 2022 (v18.0.0)](https://react.dev/blog/2022/03/08/react-18-upgrade-guide). In React 19, we're removing `ReactDOM.render` and you'll need to migrate to using [`ReactDOM.createRoot`](https://react.dev/reference/react-dom/client/createRoot):

```js
// Before
import  from 'react-dom';
render(

#### Removed: `ReactDOM.hydrate` 

`ReactDOM.hydrate` was deprecated in [March 2022 (v18.0.0)](https://react.dev/blog/2022/03/08/react-18-upgrade-guide). In React 19, we're removing `ReactDOM.hydrate` you'll need to migrate to using [`ReactDOM.hydrateRoot`](https://react.dev/reference/react-dom/client/hydrateRoot),

```js
// Before
import  from 'react-dom';
hydrate(

#### Removed: `unmountComponentAtNode` 

`ReactDOM.unmountComponentAtNode` was deprecated in [March 2022 (v18.0.0)](https://react.dev/blog/2022/03/08/react-18-upgrade-guide). In React 19, you'll need to migrate to using `root.unmount()`.

```js
// Before
unmountComponentAtNode(document.getElementById('root'));

// After
root.unmount();
```

For more see `root.unmount()` for [`createRoot`](https://react.dev/reference/react-dom/client/createRoot#root-unmount) and [`hydrateRoot`](https://react.dev/reference/react-dom/client/hydrateRoot#root-unmount).

#### Removed: `ReactDOM.findDOMNode` 

`ReactDOM.findDOMNode` was [deprecated in October 2018 (v16.6.0)](https://legacy.reactjs.org/blog/2018/10/23/react-v-16-6.html#deprecations-in-strictmode).

We're removing `findDOMNode` because it was a legacy escape hatch that was slow to execute, fragile to refactoring, only returned the first child, and broke abstraction levels (see more [here](https://legacy.reactjs.org/docs/strict-mode.html#warning-about-deprecated-finddomnode-usage)). You can replace `ReactDOM.findDOMNode` with [DOM refs](/learn/manipulating-the-dom-with-refs):

```js
// Before
import  from 'react-dom';

function AutoselectingInput() {
  useEffect(() => , []);

  return ;
}
```

```js
// After
function AutoselectingInput() {
  const ref = useRef(null);
  useEffect(() => , []);

  return 
}
```

## New deprecations 

### Deprecated: `element.ref` 

React 19 supports [`ref` as a prop](/blog/2024/04/25/react-19#ref-as-a-prop), so we're deprecating the `element.ref` in place of `element.props.ref`.

Accessing `element.ref` will warn:

### Deprecated: `react-test-renderer` 

We are deprecating `react-test-renderer` because it implements its own renderer environment that doesn't match the environment users use, promotes testing implementation details, and relies on introspection of React's internals.

The test renderer was created before there were more viable testing strategies available like [React Testing Library](https://testing-library.com), and we now recommend using a modern testing library instead.

In React 19, `react-test-renderer` logs a deprecation warning, and has switched to concurrent rendering. We recommend migrating your tests to [@testing-library/react](https://testing-library.com/docs/react-testing-library/intro/) or [@testing-library/react-native](https://testing-library.com/docs/react-native-testing-library/intro) for a modern and well supported testing experience.

## Notable changes 

### StrictMode changes 

React 19 includes several fixes and improvements to Strict Mode.

When double rendering in Strict Mode in development, `useMemo` and `useCallback` will reuse the memoized results from the first render during the second render. Components that are already Strict Mode compatible should not notice a difference in behavior.

As with all Strict Mode behaviors, these features are designed to proactively surface bugs in your components during development so you can fix them before they are shipped to production. For example, during development, Strict Mode will double-invoke ref callback functions on initial mount, to simulate what happens when a mounted component is replaced by a Suspense fallback.

### Improvements to Suspense 

In React 19, when a component suspends, React will immediately commit the fallback of the nearest Suspense boundary without waiting for the entire sibling tree to render. After the fallback commits, React schedules another render for the suspended siblings to "pre-warm" lazy requests in the rest of the tree:

This change means Suspense fallbacks display faster, while still warming lazy requests in the suspended tree.

### UMD builds removed 

UMD was widely used in the past as a convenient way to load React without a build step. Now, there are modern alternatives for loading modules as scripts in HTML documents. Starting with React 19, React will no longer produce UMD builds to reduce the complexity of its testing and release process.

To load React 19 with a script tag, we recommend using an ESM-based CDN such as [esm.sh](https://esm.sh/).

```html

  import React from "https://esm.sh/react@19/?dev"
  import ReactDOMClient from "https://esm.sh/react-dom@19/client?dev"
  ...

```

### Libraries depending on React internals may block upgrades 

This release includes changes to React internals that may impact libraries that ignore our pleas to not use internals like `SECRET_INTERNALS_DO_NOT_USE_OR_YOU_WILL_BE_FIRED`. These changes are necessary to land improvements in React 19, and will not break libraries that follow our guidelines.

Based on our [Versioning Policy](https://react.dev/community/versioning-policy#what-counts-as-a-breaking-change), these updates are not listed as breaking changes, and we are not including docs for how to upgrade them. The recommendation is to remove any code that depends on internals.

To reflect the impact of using internals, we have renamed the `SECRET_INTERNALS` suffix to:

`_DO_NOT_USE_OR_WARN_USERS_THEY_CANNOT_UPGRADE`

In the future we will more aggressively block accessing internals from React to discourage usage and ensure users are not blocked from upgrading.

## TypeScript changes 

### Removed deprecated TypeScript types 

We've cleaned up the TypeScript types based on the removed APIs in React 19. Some of the removed have types been moved to more relevant packages, and others are no longer needed to describe React's behavior.

Check out [`types-react-codemod`](https://github.com/eps1lon/types-react-codemod/) for a list of supported replacements. If you feel a codemod is missing, it can be tracked in the [list of missing React 19 codemods](https://github.com/eps1lon/types-react-codemod/issues?q=is%3Aissue+is%3Aopen+sort%3Aupdated-desc+label%3A%22React+19%22+label%3Aenhancement).

### `ref` cleanups required 

_This change is included in the `react-19` codemod preset as [`no-implicit-ref-callback-return
`](https://github.com/eps1lon/types-react-codemod/#no-implicit-ref-callback-return)._

Due to the introduction of ref cleanup functions, returning anything else from a ref callback will now be rejected by TypeScript. The fix is usually to stop using implicit returns:

```diff [[1, 1, "("], [1, 1, ")"], [2, 2, "", 1]]
-  (instance = current)} />
+  } />
```

The original code returned the instance of the `HTMLDivElement` and TypeScript wouldn't know if this was supposed to be a cleanup function or not.

### `useRef` requires an argument 

_This change is included in the `react-19` codemod preset as [`refobject-defaults`](https://github.com/eps1lon/types-react-codemod/#refobject-defaults)._

A long-time complaint of how TypeScript and React work has been `useRef`. We've changed the types so that `useRef` now requires an argument. This significantly simplifies its type signature. It'll now behave more like `createContext`.

```ts
// @ts-expect-error: Expected 1 argument but saw none
useRef();
// Passes
useRef(undefined);
// @ts-expect-error: Expected 1 argument but saw none
createContext();
// Passes
createContext(undefined);
```

This now also means that all refs are mutable. You'll no longer hit the issue where you can't mutate a ref because you initialised it with `null`:

```ts
const ref = useRef(null);

// Cannot assign to 'current' because it is a read-only property
ref.current = 1;
```

`MutableRef` is now deprecated in favor of a single `RefObject` type which `useRef` will always return:

```ts
interface RefObject 

declare function useRef: RefObject
```

`useRef` still has a convenience overload for `useRef(null)` that automatically returns `RefObject`. To ease migration due to the required argument for `useRef`, a convenience overload for `useRef(undefined)` was added that automatically returns `RefObject`.

Check out [[RFC] Make all refs mutable](https://github.com/DefinitelyTyped/DefinitelyTyped/pull/64772) for prior discussions about this change.

### Changes to the `ReactElement` TypeScript type 

_This change is included in the [`react-element-default-any-props`](https://github.com/eps1lon/types-react-codemod#react-element-default-any-props) codemod._

The `props` of React elements now default to `unknown` instead of `any` if the element is typed as `ReactElement`. This does not affect you if you pass a type argument to `ReactElement`:

```ts
type Example2 = ReactElement<>["props"];
//   ^? 
```

But if you relied on the default, you now have to handle `unknown`:

```ts
type Example = ReactElement["props"];
//   ^? Before, was 'any', now 'unknown'
```

You should only need it if you have a lot of legacy code relying on unsound access of element props. Element introspection only exists as an escape hatch, and you should make it explicit that your props access is unsound via an explicit `any`.

### The JSX namespace in TypeScript 
This change is included in the `react-19` codemod preset as [`scoped-jsx`](https://github.com/eps1lon/types-react-codemod#scoped-jsx)

A long-time request is to remove the global `JSX` namespace from our types in favor of `React.JSX`. This helps prevent pollution of global types which prevents conflicts between different UI libraries that leverage JSX.

You'll now need to wrap module augmentation of the JSX namespace in `declare module "....":

```diff
// global.d.ts
+ declare module "react" {
    namespace JSX {
      interface IntrinsicElements {
        "my-element": ;
      }
    }
+ }
```

The exact module specifier depends on the JSX runtime you specified in the `compilerOptions` of your `tsconfig.json`:

- For `"jsx": "react-jsx"` it would be `react/jsx-runtime`.
- For `"jsx": "react-jsxdev"` it would be `react/jsx-dev-runtime`.
- For `"jsx": "react"` and `"jsx": "preserve"` it would be `react`.

### Better `useReducer` typings 

`useReducer` now has improved type inference thanks to [@mfp22](https://github.com/mfp22).

However, this required a breaking change where `useReducer` doesn't accept the full reducer type as a type parameter but instead either needs none (and rely on contextual typing) or needs both the state and action type.

The new best practice is _not_ to pass type arguments to `useReducer`.
```diff
- useReducer>(reducer)
+ useReducer(reducer)
```
This may not work in edge cases where you can explicitly type the state and action, by passing in the `Action` in a tuple:
```diff
- useReducer>(reducer)
+ useReducer(reducer)
```
If you define the reducer inline, we encourage to annotate the function parameters instead:
```diff
- useReducer>((state, action) => state)
+ useReducer((state: State, action: Action) => state)
```
This is also what you'd also have to do if you move the reducer outside of the `useReducer` call:

```ts
const reducer = (state: State, action: Action) => state;
```

## Changelog 

### Other breaking changes 

- **react-dom**: Error for javascript URLs in `src` and `href` [#26507](https://github.com/facebook/react/pull/26507)
- **react-dom**: Remove `errorInfo.digest` from `onRecoverableError` [#28222](https://github.com/facebook/react/pull/28222)
- **react-dom**: Remove `unstable_flushControlled` [#26397](https://github.com/facebook/react/pull/26397)
- **react-dom**: Remove `unstable_createEventHandle` [#28271](https://github.com/facebook/react/pull/28271)
- **react-dom**: Remove `unstable_renderSubtreeIntoContainer` [#28271](https://github.com/facebook/react/pull/28271)
- **react-dom**: Remove `unstable_runWithPriority` [#28271](https://github.com/facebook/react/pull/28271)
- **react-is**: Remove deprecated methods from `react-is` [28224](https://github.com/facebook/react/pull/28224)

### Other notable changes 

- **react**: Batch sync, default and continuous lanes [#25700](https://github.com/facebook/react/pull/25700)
- **react**: Don't prerender siblings of suspended component [#26380](https://github.com/facebook/react/pull/26380)
- **react**: Detect infinite update loops caused by render phase updates [#26625](https://github.com/facebook/react/pull/26625)
- **react-dom**: Transitions in popstate are now synchronous [#26025](https://github.com/facebook/react/pull/26025)
- **react-dom**: Remove layout effect warning during SSR [#26395](https://github.com/facebook/react/pull/26395)
- **react-dom**: Warn and don’t set empty string for src/href (except anchor tags) [#28124](https://github.com/facebook/react/pull/28124)

For a full list of changes, please see the [Changelog](https://github.com/facebook/react/blob/main/CHANGELOG.md#1900-december-5-2024).

---

Thanks to [Andrew Clark](https://twitter.com/acdlite), [Eli White](https://twitter.com/Eli_White), [Jack Pope](https://github.com/jackpope), [Jan Kassens](https://github.com/kassens), [Josh Story](https://twitter.com/joshcstory), [Matt Carroll](https://twitter.com/mattcarrollcode), [Noah Lemen](https://twitter.com/noahlemen), [Sophie Alpert](https://twitter.com/sophiebits), and [Sebastian Silbermann](https://twitter.com/sebsilbermann) for reviewing and editing this post.
