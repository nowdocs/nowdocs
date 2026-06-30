---
title: Understanding Your UI as a Tree
---

## Your UI as a tree 

Trees are a relationship model between items. The UI is often represented using tree structures. For example, browsers use tree structures to model HTML ([DOM](https://developer.mozilla.org/docs/Web/API/Document_Object_Model/Introduction)) and CSS ([CSSOM](https://developer.mozilla.org/docs/Web/API/CSS_Object_Model)). Mobile platforms also use trees to represent their view hierarchy.

Like browsers and mobile platforms, React also uses tree structures to manage and model the relationship between components in a React app. These trees are useful tools to understand how data flows through a React app and how to optimize rendering and app size.

## The Render Tree 

A major feature of components is the ability to compose components of other components. As we [nest components](/learn/your-first-component#nesting-and-organizing-components), we have the concept of parent and child components, where each parent component may itself be a child of another component.

When we render a React app, we can model this relationship in a tree, known as the render tree.

Here is a React app that renders inspirational quotes.

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
import * as React from 'react';
import quotes from './quotes';
import FancyText from './FancyText';

export default function InspirationGenerator() 

```

```js src/FancyText.js
export default function FancyText() {
  return title
    ? 
    : 
}
```

```js src/Color.js
export default function Color() 
```

```js src/InspirationGenerator.js
import * as React from 'react';
import inspirations from './inspirations';
import FancyText from './FancyText';
import Color from './Color';

export default function InspirationGenerator() {
  const [index, setIndex] = React.useState(0);
  const inspiration = inspirations[index];
  const next = () => setIndex((index + 1) % inspirations.length);

  return (
    <>
      Your inspirational  is:
      {inspiration.type === 'quote'
      ? 

In this example, depending on what `inspiration.type` is, we may render `

The root node of the tree is the root module, also known as the entrypoint file. It often is the module that contains the root component.

Comparing to the render tree of the same app, there are similar structures but some notable differences:

* The nodes that make-up the tree represent modules, not components.
* Non-component modules, like `inspirations.js`, are also represented in this tree. The render tree only encapsulates components.
* `Copyright.js` appears under `App.js` but in the render tree, `Copyright`, the component, appears as a child of `InspirationGenerator`. This is because `InspirationGenerator` accepts JSX as [children props](/learn/passing-props-to-a-component#passing-jsx-as-children), so it renders `Copyright` as a child component but does not import the module.

Dependency trees are useful to determine what modules are necessary to run your React app. When building a React app for production, there is typically a build step that will bundle all the necessary JavaScript to ship to the client. The tool responsible for this is called a [bundler](https://developer.mozilla.org/en-US/docs/Learn/Tools_and_testing/Understanding_client-side_tools/Overview#the_modern_tooling_ecosystem), and bundlers will use the dependency tree to determine what modules should be included.

As your app grows, often the bundle size does too. Large bundle sizes are expensive for a client to download and run. Large bundle sizes can delay the time for your UI to get drawn. Getting a sense of your app's dependency tree may help with debugging these issues.

[comment]: <> (perhaps we should also deep dive on conditional imports)

[TODO]: <> (Add challenges)
