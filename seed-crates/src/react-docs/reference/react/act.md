---
title: act
---

To prepare a component for assertions, wrap the code rendering it and performing updates inside an `await act()` call. This makes your test run closer to how React works in the browser.

#### Parameters 

* `async actFn`: An async function wrapping renders or interactions for components being tested. Any updates triggered within the `actFn`, are added to an internal act queue, which are then flushed together to process and apply any changes to the DOM. Since it is async, React will also run any code that crosses an async boundary, and flush any updates scheduled.

#### Returns 

`act` does not return anything.

## Usage 

When testing a component, you can use `act` to make assertions about its output.

For example, let’s say we have this `Counter` component, the usage examples below show how to test it:

```js
function Counter() {
  const [count, setCount] = useState(0);
  const handleClick = () => 

  useEffect(() => {
    document.title = `You clicked $ times`;
  }, [count]);

  return (
    
      You clicked  times
      
        Click me
      
    
  )
}
```

### Rendering components in tests 

To test the render output of a component, wrap the render inside `act()`:

```js  
import  from 'react';
import ReactDOMClient from 'react-dom/client';
import Counter from './Counter';

it('can render and update a counter', async () => {
  container = document.createElement('div');
  document.body.appendChild(container);

  // ✅ Render the component inside act().
  await act(() => {
    ReactDOMClient.createRoot(container).render(

## Troubleshooting 

### I'm getting an error: "The current testing environment is not configured to support act(...)" 

Using `act` requires setting `global.IS_REACT_ACT_ENVIRONMENT=true` in your test environment. This is to ensure that `act` is only used in the correct environment.

If you don't set the global, you will see an error like this:

To fix, add this to your global setup file for React tests:

```js
global.IS_REACT_ACT_ENVIRONMENT=true
```

