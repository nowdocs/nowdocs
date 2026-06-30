---
title: Using TypeScript
re: https://github.com/reactjs/react.dev/issues/5960
---

## Installation 

All [production-grade React frameworks](/learn/creating-a-react-app#full-stack-frameworks) offer support for using TypeScript. Follow the framework specific guide for installation:

- [Next.js](https://nextjs.org/docs/app/building-your-application/configuring/typescript)
- [Remix](https://remix.run/docs/en/1.19.2/guides/typescript)
- [Gatsby](https://www.gatsbyjs.com/docs/how-to/custom-configuration/typescript/)
- [Expo](https://docs.expo.dev/guides/typescript/)

### Adding TypeScript to an existing React project 

To install the latest version of React's type definitions:

The following compiler options need to be set in your `tsconfig.json`:

1. `dom` must be included in [`lib`](https://www.typescriptlang.org/tsconfig/#lib) (Note: If no `lib` option is specified, `dom` is included by default).
2. [`jsx`](https://www.typescriptlang.org/tsconfig/#jsx) must be set to one of the valid options. `preserve` should suffice for most applications.
  If you're publishing a library, consult the [`jsx` documentation](https://www.typescriptlang.org/tsconfig/#jsx) on what value to choose.

## TypeScript with React Components 

Writing TypeScript with React is very similar to writing JavaScript with React. The key difference when working with a component is that you can provide types for your component's props. These types can be used for correctness checking and providing inline documentation in editors.

Taking the [`MyButton` component](/learn#components) from the [Quick Start](/learn) guide, we can add a type describing the `title` for the button:

 

This inline syntax is the simplest way to provide types for a component, though once you start to have a few fields to describe it can become unwieldy. Instead, you can use an `interface` or `type` to describe the component's props:

The type describing your component's props can be as simple or as complex as you need, though they should be an object type described with either a `type` or `interface`. You can learn about how TypeScript describes objects in [Object Types](https://www.typescriptlang.org/docs/handbook/2/objects.html) but you may also be interested in using [Union Types](https://www.typescriptlang.org/docs/handbook/2/everyday-types.html#union-types) to describe a prop that can be one of a few different types and the [Creating Types from Types](https://www.typescriptlang.org/docs/handbook/2/types-from-types.html) guide for more advanced use cases.

## Example Hooks 

The type definitions from `@types/react` include types for the built-in Hooks, so you can use them in your components without any additional setup. They are built to take into account the code you write in your component, so you will get [inferred types](https://www.typescriptlang.org/docs/handbook/type-inference.html) a lot of the time and ideally do not need to handle the minutiae of providing the types.

However, we can look at a few examples of how to provide types for Hooks.

### `useState` 

The [`useState` Hook](/reference/react/useState) will re-use the value passed in as the initial state to determine what the type of the value should be. For example:

```ts
// Infer the type as "boolean"
const [enabled, setEnabled] = useState(false);
```

This will assign the type of `boolean` to `enabled`, and `setEnabled` will be a function accepting either a `boolean` argument, or a function that returns a `boolean`. If you want to explicitly provide a type for the state, you can do so by providing a type argument to the `useState` call:

```ts
// Explicitly set the type to "boolean"
const [enabled, setEnabled] = useState(false);
```

This isn't very useful in this case, but a common case where you may want to provide a type is when you have a union type. For example, `status` here can be one of a few different strings:

```ts
type Status = "idle" | "loading" | "success" | "error";

const [status, setStatus] = useState

We are using TypeScript in a few key places:

 - `interface State` describes the shape of the reducer's state.
 - `type CounterAction` describes the different actions which can be dispatched to the reducer.
 - `const initialState: State` provides a type for the initial state, and also the type which is used by `useReducer` by default.
 - `stateReducer(state: State, action: CounterAction): State` sets the types for the reducer function's arguments and return value.

A more explicit alternative to setting the type on `initialState` is to provide a type argument to `useReducer`:

```ts
import  from './your-reducer-implementation';

const initialState = ;

export default function App() 

function MyComponent() {
  const theme = useGetTheme();

  return (
    
      Current theme: 
    
  )
}
```

```js src/App.js hidden
import AppTSX from "./App.tsx";
export default App = AppTSX;
```

This technique works when you have a default value which makes sense - but there are occasionally cases when you do not, and in those cases `null` can feel reasonable as a default value. However, to allow the type-system to understand your code, you need to explicitly set `ContextShape | null` on the `createContext`.

This causes the issue that you need to eliminate the `| null` in the type for context consumers. Our recommendation is to have the Hook do a runtime check for it's existence and throw an error when not present:

```js 
import  from 'react';

// This is a simpler example, but you can imagine a more complex object here
type ComplexObject = ;

// The context is created with `| null` in the type, to accurately reflect the default value.
const Context = createContext
  )
}

function MyComponent() {
  const object = useGetComplexObject();

  return (
    
      Current object: 
    
  )
}
```

### `useMemo` 

The [`useMemo`](/reference/react/useMemo) Hooks will create/re-access a memorized value from a function call, re-running the function only when dependencies passed as the 2nd parameter are changed. The result of calling the Hook is inferred from the return value from the function in the first parameter. You can be more explicit by providing a type argument to the Hook.

```ts
// The type of visibleTodos is inferred from the return value of filterTodos
const visibleTodos = useMemo(() => filterTodos(todos, tab), [todos, tab]);
```

### `useCallback` 

The [`useCallback`](/reference/react/useCallback) provide a stable reference to a function as long as the dependencies passed into the second parameter are the same. Like `useMemo`, the function's type is inferred from the return value of the function in the first parameter, and you can be more explicit by providing a type argument to the Hook.

```ts
const handleClick = useCallback(() => , [todos]);
```

When working in TypeScript strict mode `useCallback` requires adding types for the parameters in your callback. This is because the type of the callback is inferred from the return value of the function, and without parameters the type cannot be fully understood.

Depending on your code-style preferences, you could use the `*EventHandler` functions from the React types to provide the type for the event handler at the same time as defining the callback:

```ts
import  from 'react';

export default function Form() {
  const [value, setValue] = useState("Change me");

  const handleChange = useCallback

There are many types of events provided in the React types - the full list can be found [here](https://github.com/DefinitelyTyped/DefinitelyTyped/blob/b580df54c0819ec9df62b0835a315dd48b8594a9/types/react/index.d.ts#L1247C1-L1373) which is based on the [most popular events from the DOM](https://developer.mozilla.org/en-US/docs/Web/Events).

When determining the type you are looking for you can first look at the hover information for the event handler you are using, which will show the type of the event.

If you need to use an event that is not included in this list, you can use the `React.SyntheticEvent` type, which is the base type for all events.

### Children 

There are two common paths to describing the children of a component. The first is to use the `React.ReactNode` type, which is a union of all the possible types that can be passed as children in JSX:

```ts
interface ModalRendererProps 
```

This is a very broad definition of children. The second is to use the `React.ReactElement` type, which is only JSX elements and not JavaScript primitives like strings or numbers:

```ts
interface ModalRendererProps 
```

Note, that you cannot use TypeScript to describe that the children are a certain type of JSX elements, so you cannot use the type-system to describe a component which only accepts `` children.

You can see an example of both `React.ReactNode` and `React.ReactElement` with the type-checker in [this TypeScript playground](https://www.typescriptlang.org/play?#code/JYWwDg9gTgLgBAJQKYEMDG8BmUIjgIilQ3wChSB6CxYmAOmXRgDkIATJOdNJMGAZzgwAFpxAR+8YADswAVwGkZMJFEzpOjDKw4AFHGEEBvUnDhphwADZsi0gFw0mDWjqQBuUgF9yaCNMlENzgAXjgACjADfkctFnYkfQhDAEpQgD44AB42YAA3dKMo5P46C2tbJGkvLIpcgt9-QLi3AEEwMFCItJDMrPTTbIQ3dKywdIB5aU4kKyQQKpha8drhhIGzLLWODbNs3b3s8YAxKBQAcwXpAThMaGWDvbH0gFloGbmrgQfBzYpd1YjQZbEYARkB6zMwO2SHSAAlZlYIBCdtCRkZpHIrFYahQYQD8UYYFA5EhcfjyGYqHAXnJAsIUHlOOUbHYhMIIHJzsI0Qk4P9SLUBuRqXEXEwAKKfRZcNA8PiCfxWACecAAUgBlAAacFm80W-CU11U6h4TgwUv11yShjgJjMLMqDnN9Dilq+nh8pD8AXgCHdMrCkWisVoAet0R6fXqhWKhjKllZVVxMcavpd4Zg7U6Qaj+2hmdG4zeRF10uu-Aeq0LBfLMEe-V+T2L7zLVu+FBWLdLeq+lc7DYFf39deFVOotMCACNOCh1dq219a+30uC8YWoZsRyuEdjkevR8uvoVMdjyTWt4WiSSydXD4NqZP4AymeZE072ZzuUeZQKheQgA).

### Style Props 

When using inline styles in React, you can use `React.CSSProperties` to describe the object passed to the `style` prop. This type is a union of all the possible CSS properties, and is a good way to ensure you are passing valid CSS properties to the `style` prop, and to get auto-complete in your editor.

```ts
interface MyComponentProps 
```

## Further learning 

This guide has covered the basics of using TypeScript with React, but there is a lot more to learn.
Individual API pages on the docs may contain more in-depth documentation on how to use them with TypeScript.

We recommend the following resources:

 - [The TypeScript handbook](https://www.typescriptlang.org/docs/handbook/) is the official documentation for TypeScript, and covers most key language features.

 - [The TypeScript release notes](https://devblogs.microsoft.com/typescript/) cover new features in depth.

 - [React TypeScript Cheatsheet](https://react-typescript-cheatsheet.netlify.app/) is a community-maintained cheatsheet for using TypeScript with React, covering a lot of useful edge cases and providing more breadth than this document.

 - [TypeScript Community Discord](https://discord.com/invite/typescript) is a great place to ask questions and get help with TypeScript and React issues.
