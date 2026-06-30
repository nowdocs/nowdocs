---
title: useMemo
---

---

## Usage 

### Skipping expensive recalculations 

To cache a calculation between re-renders, wrap it in a `useMemo` call at the top level of your component:

```js [[3, 4, "visibleTodos"], [1, 4, "() => filterTodos(todos, tab)"], [2, 4, "[todos, tab]"]]
import  from 'react';

function TodoList() 
```

You need to pass two things to `useMemo`:

1. A  that takes no arguments, like `() =>`, and returns what you wanted to calculate.
2. A  including every value within your component that's used inside your calculation.

On the initial render, the  you'll get from `useMemo` will be the result of calling your .

On every subsequent render, React will compare the  with the dependencies you passed during the last render. If none of the dependencies have changed (compared with [`Object.is`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/is)), `useMemo` will return the value you already calculated before. Otherwise, React will re-run your calculation and return the new value.

In other words, `useMemo` caches a calculation result between re-renders until its dependencies change.

**Let's walk through an example to see when this is useful.**

By default, React will re-run the entire body of your component every time that it re-renders. For example, if this `TodoList` updates its state or receives new props from its parent, the `filterTodos` function will re-run:

```js 
function TodoList() 
```

Usually, this isn't a problem because most calculations are very fast. However, if you're filtering or transforming a large array, or doing some expensive computation, you might want to skip doing it again if data hasn't changed. If both `todos` and `tab` are the same as they were during the last render, wrapping the calculation in `useMemo` like earlier lets you reuse `visibleTodos` you've already calculated before.

This type of caching is called *[memoization.](https://en.wikipedia.org/wiki/Memoization)*

However, here is the same code **with the artificial slowdown removed.** Does the lack of `useMemo` feel noticeable or not?

Quite often, code without memoization works fine. If your interactions are fast enough, you might not need memoization.

You can try increasing the number of todo items in `utils.js` and see how the behavior changes. This particular calculation wasn't very expensive to begin with, but if the number of todos grows significantly, most of the overhead will be in re-rendering rather than in the filtering. Keep reading below to see how you can optimize re-rendering with `useMemo`.

---

### Skipping re-rendering of components 

In some cases, `useMemo` can also help you optimize performance of re-rendering child components. To illustrate this, let's say this `TodoList` component passes the `visibleTodos` as a prop to the child `List` component:

```js 
export default function TodoList() {
  // ...
  return (
    
      

However, here is the same code **with the artificial slowdown removed.** Does the lack of `useMemo` feel noticeable or not?

Quite often, code without memoization works fine. If your interactions are fast enough, you don't need memoization.

Keep in mind that you need to run React in production mode, disable [React Developer Tools](/learn/react-developer-tools), and use devices similar to the ones your app's users have in order to get a realistic sense of what's actually slowing down your app.

---

### Preventing an Effect from firing too often 

Sometimes, you might want to use a value inside an [Effect:](/learn/synchronizing-with-effects)

```js 
function ChatRoom() {
  const [message, setMessage] = useState('');

  const options = 

  useEffect(() => {
    const connection = createConnection(options);
    connection.connect();
    // ...
```

This creates a problem. [Every reactive value must be declared as a dependency of your Effect.](/learn/lifecycle-of-reactive-effects#react-verifies-that-you-specified-every-reactive-value-as-a-dependency) However, if you declare `options` as a dependency, it will cause your Effect to constantly reconnect to the chat room:

```js 
  useEffect(() => , [options]); // 🔴 Problem: This dependency changes on every render
  // ...
```

To solve this, you can wrap the object you need to call from an Effect in `useMemo`:

```js 
function ChatRoom() {
  const [message, setMessage] = useState('');

  const options = useMemo(() => {
    return ;
  }, [roomId]); // ✅ Only changes when roomId changes

  useEffect(() => , [options]); // ✅ Only changes when options changes
  // ...
```

This ensures that the `options` object is the same between re-renders if `useMemo` returns the cached object.

However, since `useMemo` is performance optimization, not a semantic guarantee, React may throw away the cached value if [there is a specific reason to do that](#caveats). This will also cause the effect to re-fire, **so it's even better to remove the need for a function dependency** by moving your object *inside* the Effect:

```js 
function ChatRoom() {
  const [message, setMessage] = useState('');

  useEffect(() => {
    const options = 

    const connection = createConnection(options);
    connection.connect();
    return () => connection.disconnect();
  }, [roomId]); // ✅ Only changes when roomId changes
  // ...
```

Now your code is simpler and doesn't need `useMemo`. [Learn more about removing Effect dependencies.](/learn/removing-effect-dependencies#move-dynamic-objects-and-functions-inside-your-effect)

### Memoizing a dependency of another Hook 

Suppose you have a calculation that depends on an object created directly in the component body:

```js 
function Dropdown() {
  const searchOptions = ;

  const visibleItems = useMemo(() => , [allItems, searchOptions]); // 🚩 Caution: Dependency on an object created in the component body
  // ...
```

Depending on an object like this defeats the point of memoization. When a component re-renders, all of the code directly inside the component body runs again. **The lines of code creating the `searchOptions` object will also run on every re-render.** Since `searchOptions` is a dependency of your `useMemo` call, and it's different every time, React knows the dependencies are different, and recalculate `searchItems` every time.

To fix this, you could memoize the `searchOptions` object *itself* before passing it as a dependency:

```js 
function Dropdown() {
  const searchOptions = useMemo(() => {
    return ;
  }, [text]); // ✅ Only changes when text changes

  const visibleItems = useMemo(() => , [allItems, searchOptions]); // ✅ Only changes when allItems or searchOptions changes
  // ...
```

In the example above, if the `text` did not change, the `searchOptions` object also won't change. However, an even better fix is to move the `searchOptions` object declaration *inside* of the `useMemo` calculation function:

```js 
function Dropdown() {
  const visibleItems = useMemo(() => {
    const searchOptions = ;
    return searchItems(allItems, searchOptions);
  }, [allItems, text]); // ✅ Only changes when allItems or text changes
  // ...
```

Now your calculation depends on `text` directly (which is a string and can't "accidentally" become different).

---

### Memoizing a function 

Suppose the `Form` component is wrapped in [`memo`.](/reference/react/memo) You want to pass a function to it as a prop:

```js 
export default function ProductPage() {
  function handleSubmit(orderDetails) {
    post('/product/' + productId + '/buy', );
  }

  return ;
}
```

Just as `` creates a different object, function declarations like `function() ` and expressions like `() => ` produce a *different* function on every re-render. By itself, creating a new function is not a problem. This is not something to avoid! However, if the `Form` component is memoized, presumably you want to skip re-rendering it when no props have changed. A prop that is *always* different would defeat the point of memoization.

To memoize a function with `useMemo`, your calculation function would have to return another function:

```js 
export default function Page() {
  const handleSubmit = useMemo(() => {
    return (orderDetails) => {
      post('/product/' + productId + '/buy', );
    };
  }, [productId, referrer]);

  return ;
}
```

This looks clunky! **Memoizing functions is common enough that React has a built-in Hook specifically for that. Wrap your functions into [`useCallback`](/reference/react/useCallback) instead of `useMemo`** to avoid having to write an extra nested function:

```js 
export default function Page() {
  const handleSubmit = useCallback((orderDetails) => {
    post('/product/' + productId + '/buy', );
  }, [productId, referrer]);

  return ;
}
```

The two examples above are completely equivalent. The only benefit to `useCallback` is that it lets you avoid writing an extra nested function inside. It doesn't do anything else. [Read more about `useCallback`.](/reference/react/useCallback)

---

## Troubleshooting 

### My calculation runs twice on every re-render 

In [Strict Mode](/reference/react/StrictMode), React will call some of your functions twice instead of once:

```js 
function TodoList() {
  // This component function will run twice for every render.

  const visibleTodos = useMemo(() => , [todos, tab]);

  // ...
```

This is expected and shouldn't break your code.

This **development-only** behavior helps you [keep components pure.](/learn/keeping-components-pure) React uses the result of one of the calls, and ignores the result of the other call. As long as your component and calculation functions are pure, this shouldn't affect your logic. However, if they are accidentally impure, this helps you notice and fix the mistake.

For example, this impure calculation function mutates an array you received as a prop:

```js 
  const visibleTodos = useMemo(() => {
    // 🚩 Mistake: mutating a prop
    todos.push();
    const filtered = filterTodos(todos, tab);
    return filtered;
  }, [todos, tab]);
```

React calls your function twice, so you'd notice the todo is added twice. Your calculation shouldn't change any existing objects, but it's okay to change any *new* objects you created during the calculation. For example, if the `filterTodos` function always returns a *different* array, you can mutate *that* array instead:

```js 
  const visibleTodos = useMemo(() => {
    const filtered = filterTodos(todos, tab);
    // ✅ Correct: mutating an object you created during the calculation
    filtered.push();
    return filtered;
  }, [todos, tab]);
```

Read [keeping components pure](/learn/keeping-components-pure) to learn more about purity.

Also, check out the guides on [updating objects](/learn/updating-objects-in-state) and [updating arrays](/learn/updating-arrays-in-state) without mutation.

---

### My `useMemo` call is supposed to return an object, but returns undefined 

This code doesn't work:

```js 
  // 🔴 You can't return an object from an arrow function with () => {
  const searchOptions = useMemo(() => , [text]);
```

In JavaScript, `() => {` starts the arrow function body, so the `{` brace is not a part of your object. This is why it doesn't return an object, and leads to mistakes. You could fix it by adding parentheses like `()`:

```js 
  // This works, but is easy for someone to break again
  const searchOptions = useMemo(() => (), [text]);
```

However, this is still confusing and too easy for someone to break by removing the parentheses.

To avoid this mistake, write a `return` statement explicitly:

```js 
  // ✅ This works and is explicit
  const searchOptions = useMemo(() => {
    return ;
  }, [text]);
```

---

### Every time my component renders, the calculation in `useMemo` re-runs 

Make sure you've specified the dependency array as a second argument!

If you forget the dependency array, `useMemo` will re-run the calculation every time:

```js 
function TodoList() {
  // 🔴 Recalculates every time: no dependency array
  const visibleTodos = useMemo(() => filterTodos(todos, tab));
  // ...
```

This is the corrected version passing the dependency array as a second argument:

```js 
function TodoList() {
  // ✅ Does not recalculate unnecessarily
  const visibleTodos = useMemo(() => filterTodos(todos, tab), [todos, tab]);
  // ...
```

If this doesn't help, then the problem is that at least one of your dependencies is different from the previous render. You can debug this problem by manually logging your dependencies to the console:

```js
  const visibleTodos = useMemo(() => filterTodos(todos, tab), [todos, tab]);
  console.log([todos, tab]);
```

You can then right-click on the arrays from different re-renders in the console and select "Store as a global variable" for both of them. Assuming the first one got saved as `temp1` and the second one got saved as `temp2`, you can then use the browser console to check whether each dependency in both arrays is the same:

```js
Object.is(temp1[0], temp2[0]); // Is the first dependency the same between the arrays?
Object.is(temp1[1], temp2[1]); // Is the second dependency the same between the arrays?
Object.is(temp1[2], temp2[2]); // ... and so on for every dependency ...
```

When you find which dependency breaks memoization, either find a way to remove it, or [memoize it as well.](#memoizing-a-dependency-of-another-hook)

---

### I need to call `useMemo` for each list item in a loop, but it's not allowed 

Suppose the `Chart` component is wrapped in [`memo`](/reference/react/memo). You want to skip re-rendering every `Chart` in the list when the `ReportList` component re-renders. However, you can't call `useMemo` in a loop:

```js {expectedErrors: } 
function ReportList() {
  return (
    
      {items.map(item => )}
    
  );
}
```

Instead, extract a component for each item and memoize data for individual items:

```js 
function ReportList() {
  return (
    
      
    
  );
}

function Report() 
```

Alternatively, you could remove `useMemo` and instead wrap `Report` itself in [`memo`.](/reference/react/memo) If the `item` prop does not change, `Report` will skip re-rendering, so `Chart` will skip re-rendering too:

```js 
function ReportList() 

const Report = memo(function Report() );
```
