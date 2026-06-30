---
title: createElement
---

, , and :

```js [[1, 5, "'h1'"], [2, 6, ""], [3, 7, "'Hello ',"], [3, 8, "createElement('i', null, name),"], [3, 9, "'. Welcome!'"]]
import  from 'react';

function Greeting() {
  return createElement(
    'h1',
    ,
    'Hello ',
    createElement('i', null, name),
    '. Welcome!'
  );
}
```

The  are optional, and you can pass as many as you need (the example above has three children). This code will display a `` header with a greeting. For comparison, here is the same example rewritten with JSX:

```js [[1, 3, "h1"], [2, 3, "className=\\"greeting\\""], [3, 4, "Hello . Welcome!"], [1, 5, "h1"]]
function Greeting() {
  return (
    
      Hello . Welcome!
    
  );
}
```

To render your own React component, pass a function like `Greeting` as the  instead of a string like `'h1'`:

```js [[1, 2, "Greeting"], [2, 2, ""]]
export default function App() {
  return createElement(Greeting, );
}
```

With JSX, it would look like this:

```js [[1, 2, "Greeting"], [2, 2, "name=\\"Taylor\\""]]
export default function App() {
  return 

And here is the same example written using JSX:

Both coding styles are fine, so you can use whichever one you prefer for your project. The main benefit of using JSX compared to `createElement` is that it's easy to see which closing tag corresponds to which opening tag.

