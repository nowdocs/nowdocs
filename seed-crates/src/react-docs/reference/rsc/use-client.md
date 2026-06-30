---
title: "'use client'"
titleForTitleTag: "'use client' directive"
---

    </>
  );
}

```

```js src/FancyText.js
export default function FancyText() {
  return title
    ? 
    : 
}
```

```js src/InspirationGenerator.js
'use client';

import  from 'react';
import inspirations from './inspirations';
import FancyText from './FancyText';

export default function InspirationGenerator() {
  const [index, setIndex] = useState(0);
  const quote = inspirations[index];
  const next = () => setIndex((index + 1) % inspirations.length);

  return (
    <>
      Your inspirational quote is:
      

In the module dependency tree of this example app, the `'use client'` directive in `InspirationGenerator.js` marks that module and all of its transitive dependencies as Client modules. The subtree starting at `InspirationGenerator.js` is now marked as Client modules.

During render, the framework will server-render the root component and continue through the [render tree](/learn/understanding-your-ui-as-a-tree#the-render-tree), opting-out of evaluating any code imported from client-marked code.

The server-rendered portion of the render tree is then sent to the client. The client, with its client code downloaded, then completes rendering the rest of the tree.

We introduce the following definitions:

* **Client Components** are components in a render tree that are rendered on the client.
* **Server Components** are components in a render tree that are rendered on the server.

Working through the example app, `App`, `FancyText` and `Copyright` are all server-rendered and considered Server Components. As `InspirationGenerator.js` and its transitive dependencies are marked as client code, the component `InspirationGenerator` and its child component `FancyText` are Client Components.

Back to the question of `FancyText`, we see that the component definition does _not_ have a `'use client'` directive and it has two usages.

The usage of `FancyText` as a child of `App`, marks that usage as a Server Component. When `FancyText` is imported and called under `InspirationGenerator`, that usage of `FancyText` is a Client Component as `InspirationGenerator` contains a `'use client'` directive.

This means that the component definition for `FancyText` will both be evaluated on the server and also downloaded by the client to render its Client Component usage.

In the module dependency tree, we see that `App.js` imports and calls `Copyright` from the `Copyright.js` module. As `Copyright.js` does not contain a `'use client'` directive, the component usage is rendered on the server. `App` is rendered on the server as it is the root component.

Client Components can render Server Components because you can pass JSX as props. In this case, `InspirationGenerator` receives `Copyright` as [children](/learn/passing-props-to-a-component#passing-jsx-as-children). However, the `InspirationGenerator` module never directly imports the `Copyright` module nor calls the component, all of that is done by `App`. In fact, the `Copyright` component is fully executed before `InspirationGenerator` starts rendering.

The takeaway is that a parent-child render relationship between components does not guarantee the same render environment.

### When to use `'use client'` 

With `'use client'`, you can determine when components are Client Components. As Server Components are default, here is a brief overview of the advantages and limitations to Server Components to determine when you need to mark something as client rendered.

For simplicity, we talk about Server Components, but the same principles apply to all code in your app that is server run.

#### Advantages of Server Components 
* Server Components can reduce the amount of code sent and run by the client. Only Client modules are bundled and evaluated by the client.
* Server Components benefit from running on the server. They can access the local filesystem and may experience low latency for data fetches and network requests.

#### Limitations of Server Components 
* Server Components cannot support interaction as event handlers must be registered and triggered by a client.
	* For example, event handlers like `onClick` can only be defined in Client Components.
* Server Components cannot use most Hooks.
	* When Server Components are rendered, their output is essentially a list of components for the client to render. Server Components do not persist in memory after render and cannot have their own state.

### Serializable types returned by Server Components 

As in any React app, parent components pass data to child components. As they are rendered in different environments, passing data from a Server Component to a Client Component requires extra consideration.

Prop values passed from a Server Component to Client Component must be serializable.

Serializable props include:
* Primitives
	* [string](https://developer.mozilla.org/en-US/docs/Glossary/String)
	* [number](https://developer.mozilla.org/en-US/docs/Glossary/Number)
	* [bigint](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/BigInt)
	* [boolean](https://developer.mozilla.org/en-US/docs/Glossary/Boolean)
	* [undefined](https://developer.mozilla.org/en-US/docs/Glossary/Undefined)
	* [null](https://developer.mozilla.org/en-US/docs/Glossary/Null)
	* [symbol](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Symbol), only symbols registered in the global Symbol registry via [`Symbol.for`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Symbol/for)
* Iterables containing serializable values
	* [String](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String)
	* [Array](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array)
	* [Map](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map)
	* [Set](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set)
	* [TypedArray](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/TypedArray) and [ArrayBuffer](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/ArrayBuffer)
* [Date](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date)
* Plain [objects](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object): those created with [object initializers](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Object_initializer), with serializable properties
* Functions that are [Server Functions](/reference/rsc/server-functions)
* Client or Server Component elements (JSX)
* [Promises](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise)

Notably, these are not supported:
* [Functions](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function) that are not exported from client-marked modules or marked with [`'use server'`](/reference/rsc/use-server)
* [Classes](https://developer.mozilla.org/en-US/docs/Learn/JavaScript/Objects/Classes_in_JavaScript)
* Objects that are instances of any class (other than the built-ins mentioned) or objects with [a null prototype](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object#null-prototype_objects)
* Symbols not registered globally, ex. `Symbol('my new symbol')`

## Usage 

### Building with interactivity and state 

As `Counter` requires both the `useState` Hook and event handlers to increment or decrement the value, this component must be a Client Component and will require a `'use client'` directive at the top.

In contrast, a component that renders UI without interaction will not need to be a Client Component.

```js
import  from 'node:fs/promises';
import Counter from './Counter';

export default async function CounterContainer() 
```

For example, `Counter`'s parent component, `CounterContainer`, does not require `'use client'` as it is not interactive and does not use state. In addition, `CounterContainer` must be a Server Component as it reads from the local file system on the server, which is possible only in a Server Component.

There are also components that don't use any server or client-only features and can be agnostic to where they render. In our earlier example, `FancyText` is one such component.

```js
export default function FancyText() {
  return title
    ? 
    : 
}
```

In this case, we don't add the `'use client'` directive, resulting in `FancyText`'s _output_ (rather than its source code) to be sent to the browser when referenced from a Server Component. As demonstrated in the earlier Inspirations app example, `FancyText` is used as both a Server or Client Component, depending on where it is imported and used.

But if `FancyText`'s HTML output was large relative to its source code (including dependencies), it might be more efficient to force it to always be a Client Component. Components that return a long SVG path string are one case where it may be more efficient to force a component to be a Client Component.

### Using client APIs 

Your React app may use client-specific APIs, such as the browser's APIs for web storage, audio and video manipulation, and device hardware, among [others](https://developer.mozilla.org/en-US/docs/Web/API).

In this example, the component uses [DOM APIs](https://developer.mozilla.org/en-US/docs/Glossary/DOM) to manipulate a [`canvas`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/canvas) element. Since those APIs are only available in the browser, it must be marked as a Client Component.

```js
'use client';

import  from 'react';

export default function Circle() {
  const ref = useRef(null);
  useLayoutEffect(() => );
  return ;
}
```

### Using third-party libraries 

Often in a React app, you'll leverage third-party libraries to handle common UI patterns or logic.

These libraries may rely on component Hooks or client APIs. Third-party components that use any of the following React APIs must run on the client:
* [createContext](/reference/react/createContext)
* [`react`](/reference/react/hooks) and [`react-dom`](/reference/react-dom/hooks) Hooks, excluding [`use`](/reference/react/use) and [`useId`](/reference/react/useId)
* [forwardRef](/reference/react/forwardRef)
* [memo](/reference/react/memo)
* [startTransition](/reference/react/startTransition)
* If they use client APIs, ex. DOM insertion or native platform views

If these libraries have been updated to be compatible with React Server Components, then they will already include `'use client'` markers of their own, allowing you to use them directly from your Server Components. If a library hasn't been updated, or if a component needs props like event handlers that can only be specified on the client, you may need to add your own Client Component file in between the third-party Client Component and your Server Component where you'd like to use it.

[TODO]: <> (Troubleshooting - need use-cases)
