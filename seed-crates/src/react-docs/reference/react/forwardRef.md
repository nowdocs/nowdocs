---
title: forwardRef
---

 as the second argument after props. Pass it to the DOM node that you want to expose:

```js  [[1, 3, "ref"], [1, 8, "ref", 30]]
import  from 'react';

const MyInput = forwardRef(function MyInput(props, ref) {
  const  = props;
  return (
    
      
      
    
  );
});
```

This lets the parent `Form` component access the  exposed by `MyInput`:

```js [[1, 2, "ref"], [1, 10, "ref", 41], [2, 5, "ref.current"]]
function Form() {
  const ref = useRef(null);

  function handleClick() 

  return (
    
      

---

### Forwarding a ref through multiple components 

Instead of forwarding a `ref` to a DOM node, you can forward it to your own component like `MyInput`:

```js 
const FormField = forwardRef(function FormField(props, ref) {
  // ...
  return (
    <>
      

---

### Exposing an imperative handle instead of a DOM node 

Instead of exposing an entire DOM node, you can expose a custom object, called an *imperative handle,* with a more constrained set of methods. To do this, you'd need to define a separate ref to hold the DOM node:

```js 
const MyInput = forwardRef(function MyInput(props, ref) );
```

Pass the `ref` you received to [`useImperativeHandle`](/reference/react/useImperativeHandle) and specify the value you want to expose to the `ref`:

```js 
import  from 'react';

const MyInput = forwardRef(function MyInput(props, ref) {
  const inputRef = useRef(null);

  useImperativeHandle(ref, () => {
    return {
      focus() ,
      scrollIntoView() ,
    };
  }, []);

  return ;
});
```

If some component gets a ref to `MyInput`, it will only receive your `` object instead of the DOM node. This lets you limit the information you expose about your DOM node to the minimum.

[Read more about using imperative handles.](/reference/react/useImperativeHandle)

---

## Troubleshooting 

### My component is wrapped in `forwardRef`, but the `ref` to it is always `null` 

This usually means that you forgot to actually use the `ref` that you received.

For example, this component doesn't do anything with its `ref`:

```js 
const MyInput = forwardRef(function MyInput(, ref) {
  return (
    
      
      
    
  );
});
```

To fix it, pass the `ref` down to a DOM node or another component that can accept a ref:

```js 
const MyInput = forwardRef(function MyInput(, ref) {
  return (
    
      
      
    
  );
});
```

The `ref` to `MyInput` could also be `null` if some of the logic is conditional:

```js 
const MyInput = forwardRef(function MyInput(, ref) {
  return (
    
      
      
    
  );
});
```

If `showInput` is `false`, then the ref won't be forwarded to any node, and a ref to `MyInput` will remain empty. This is particularly easy to miss if the condition is hidden inside another component, like `Panel` in this example:

```js 
const MyInput = forwardRef(function MyInput(, ref) {
  return (
    
      
      
    
  );
});
```
