---
title: useImperativeHandle
---

#### Returns 

`useImperativeHandle` returns `undefined`.

---

## Usage 

### Exposing a custom ref handle to the parent component 

To expose a DOM node to the parent element, pass in the `ref` prop to the node.

```js 
function MyInput() ;
```

With the code above, [a ref to `MyInput` will receive the `` DOM node.](/learn/manipulating-the-dom-with-refs) However, you can expose a custom value instead. To customize the exposed handle, call `useImperativeHandle` at the top level of your component:

```js 
import  from 'react';

function MyInput() {
  useImperativeHandle(ref, () => {
    return ;
  }, []);

  return ;
};
```

Note that in the code above, the `ref` is no longer passed to the ``.

For example, suppose you don't want to expose the entire `` DOM node, but you want to expose two of its methods: `focus` and `scrollIntoView`. To do this, keep the real browser DOM in a separate ref. Then use `useImperativeHandle` to expose a handle with only the methods that you want the parent component to call:

```js 
import  from 'react';

function MyInput() {
  const inputRef = useRef(null);

  useImperativeHandle(ref, () => {
    return {
      focus() ,
      scrollIntoView() ,
    };
  }, []);

  return ;
};
```

Now, if the parent component gets a ref to `MyInput`, it will be able to call the `focus` and `scrollIntoView` methods on it. However, it will not have full access to the underlying `` DOM node.

---

### Exposing your own imperative methods 

The methods you expose via an imperative handle don't have to match the DOM methods exactly. For example, this `Post` component exposes a `scrollAndFocusAddComment` method via an imperative handle. This lets the parent `Page` scroll the list of comments *and* focus the input field when you click the button:

