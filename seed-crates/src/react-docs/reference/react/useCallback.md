---
title: useCallback
---

 including every value within your component that's used inside your function.

On the initial render, the  you'll get from `useCallback` will be the function you passed.

On the following renders, React will compare the  with the dependencies you passed during the previous render. If none of the dependencies have changed (compared with [`Object.is`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/is)), `useCallback` will return the same function as before. Otherwise, `useCallback` will return the function you passed on *this* render.

In other words, `useCallback` caches a function between re-renders until its dependencies change.

**Let's walk through an example to see when this is useful.**

Say you're passing a `handleSubmit` function down from the `ProductPage` to the `ShippingForm` component:

```js 
function ProductPage() {
  // ...
  return (
    
      

However, here is the same code **with the artificial slowdown removed.** Does the lack of `useCallback` feel noticeable or not?

Quite often, code without memoization works fine. If your interactions are fast enough, you don't need memoization.

Keep in mind that you need to run React in production mode, disable [React Developer Tools](/learn/react-developer-tools), and use devices similar to the ones your app's users have in order to get a realistic sense of what's actually slowing down your app.

---

### Updating state from a memoized callback 

Sometimes, you might need to update state based on previous state from a memoized callback.

This `handleAddTodo` function specifies `todos` as a dependency because it computes the next todos from it:

```js 
function TodoList() {
  const [todos, setTodos] = useState([]);

  const handleAddTodo = useCallback((text) => {
    const newTodo = ;
    setTodos([...todos, newTodo]);
  }, [todos]);
  // ...
```

You'll usually want memoized functions to have as few dependencies as possible. When you read some state only to calculate the next state, you can remove that dependency by passing an [updater function](/reference/react/useState#updating-state-based-on-the-previous-state) instead:

```js 
function TodoList() {
  const [todos, setTodos] = useState([]);

  const handleAddTodo = useCallback((text) => {
    const newTodo = ;
    setTodos(todos => [...todos, newTodo]);
  }, []); // ✅ No need for the todos dependency
  // ...
```

Here, instead of making `todos` a dependency and reading it inside, you pass an instruction about *how* to update the state (`todos => [...todos, newTodo]`) to React. [Read more about updater functions.](/reference/react/useState#updating-state-based-on-the-previous-state)

---

### Preventing an Effect from firing too often 

Sometimes, you might want to call a function from inside an [Effect:](/learn/synchronizing-with-effects)

```js 
function ChatRoom() {
  const [message, setMessage] = useState('');

  function createOptions() {
    return ;
  }

  useEffect(() => {
    const options = createOptions();
    const connection = createConnection(options);
    connection.connect();
    // ...
```

This creates a problem. [Every reactive value must be declared as a dependency of your Effect.](/learn/lifecycle-of-reactive-effects#react-verifies-that-you-specified-every-reactive-value-as-a-dependency) However, if you declare `createOptions` as a dependency, it will cause your Effect to constantly reconnect to the chat room:

```js 
  useEffect(() => , [createOptions]); // 🔴 Problem: This dependency changes on every render
  // ...
```

To solve this, you can wrap the function you need to call from an Effect into `useCallback`:

```js 
function ChatRoom() {
  const [message, setMessage] = useState('');

  const createOptions = useCallback(() => {
    return ;
  }, [roomId]); // ✅ Only changes when roomId changes

  useEffect(() => , [createOptions]); // ✅ Only changes when createOptions changes
  // ...
```

This ensures that the `createOptions` function is the same between re-renders if the `roomId` is the same. **However, it's even better to remove the need for a function dependency.** Move your function *inside* the Effect:

```js 
function ChatRoom() {
  const [message, setMessage] = useState('');

  useEffect(() => {
    function createOptions() { // ✅ No need for useCallback or function dependencies!
      return ;
    }

    const options = createOptions();
    const connection = createConnection(options);
    connection.connect();
    return () => connection.disconnect();
  }, [roomId]); // ✅ Only changes when roomId changes
  // ...
```

Now your code is simpler and doesn't need `useCallback`. [Learn more about removing Effect dependencies.](/learn/removing-effect-dependencies#move-dynamic-objects-and-functions-inside-your-effect)

---

### Optimizing a custom Hook 

If you're writing a [custom Hook,](/learn/reusing-logic-with-custom-hooks) it's recommended to wrap any functions that it returns into `useCallback`:

```js 
function useRouter() {
  const  = useContext(RouterStateContext);

  const navigate = useCallback((url) => {
    dispatch();
  }, [dispatch]);

  const goBack = useCallback(() => {
    dispatch();
  }, [dispatch]);

  return ;
}
```

This ensures that the consumers of your Hook can optimize their own code when needed.

---

## Troubleshooting 

### Every time my component renders, `useCallback` returns a different function 

Make sure you've specified the dependency array as a second argument!

If you forget the dependency array, `useCallback` will return a new function every time:

```js 
function ProductPage() {
  const handleSubmit = useCallback((orderDetails) => {
    post('/product/' + productId + '/buy', );
  }); // 🔴 Returns a new function every time: no dependency array
  // ...
```

This is the corrected version passing the dependency array as a second argument:

```js 
function ProductPage() {
  const handleSubmit = useCallback((orderDetails) => {
    post('/product/' + productId + '/buy', );
  }, [productId, referrer]); // ✅ Does not return a new function unnecessarily
  // ...
```

If this doesn't help, then the problem is that at least one of your dependencies is different from the previous render. You can debug this problem by manually logging your dependencies to the console:

```js 
  const handleSubmit = useCallback((orderDetails) => , [productId, referrer]);

  console.log([productId, referrer]);
```

You can then right-click on the arrays from different re-renders in the console and select "Store as a global variable" for both of them. Assuming the first one got saved as `temp1` and the second one got saved as `temp2`, you can then use the browser console to check whether each dependency in both arrays is the same:

```js
Object.is(temp1[0], temp2[0]); // Is the first dependency the same between the arrays?
Object.is(temp1[1], temp2[1]); // Is the second dependency the same between the arrays?
Object.is(temp1[2], temp2[2]); // ... and so on for every dependency ...
```

When you find which dependency is breaking memoization, either find a way to remove it, or [memoize it as well.](/reference/react/useMemo#memoizing-a-dependency-of-another-hook)

---

### I need to call `useCallback` for each list item in a loop, but it's not allowed 

Suppose the `Chart` component is wrapped in [`memo`](/reference/react/memo). You want to skip re-rendering every `Chart` in the list when the `ReportList` component re-renders. However, you can't call `useCallback` in a loop:

```js {expectedErrors: } 
function ReportList() {
  return (
    
      {items.map(item => {
        // 🔴 You can't call useCallback in a loop like this:
        const handleClick = useCallback(() => , [item]);

        return (
          
            
          
        );
      })}
    
  );
}
```

Instead, extract a component for an individual item, and put `useCallback` there:

```js 
function ReportList() {
  return (
    
      
    
  );
}

function Report() {
  // ✅ Call useCallback at the top level:
  const handleClick = useCallback(() => , [item]);

  return (
    
      
    
  );
}
```

Alternatively, you could remove `useCallback` in the last snippet and instead wrap `Report` itself in [`memo`.](/reference/react/memo) If the `item` prop does not change, `Report` will skip re-rendering, so `Chart` will skip re-rendering too:

```js 
function ReportList() 

const Report = memo(function Report() {
  function handleClick() 

  return (
    
      
    
  );
});
```
