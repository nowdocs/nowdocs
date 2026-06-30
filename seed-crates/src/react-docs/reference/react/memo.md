---
title: memo
---

---

### Updating a memoized component using state 

Even when a component is memoized, it will still re-render when its own state changes. Memoization only has to do with props that are passed to the component from its parent.

If you set a state variable to its current value, React will skip re-rendering your component even without `memo`. You may still see your component function being called an extra time, but the result will be discarded.

---

### Updating a memoized component using a context 

Even when a component is memoized, it will still re-render when a context that it's using changes. Memoization only has to do with props that are passed to the component from its parent.

  );
}

const Greeting = memo(function Greeting() {
  console.log("Greeting was rendered at", new Date().toLocaleTimeString());
  const theme = useContext(ThemeContext);
  return (
    Hello, !
  );
});
```

```css
label 

.light 

.dark 
```

To make your component re-render only when a _part_ of some context changes, split your component in two. Read what you need from the context in the outer component, and pass it down to a memoized child as a prop.

---

### Minimizing props changes 

When you use `memo`, your component re-renders whenever any prop is not *shallowly equal* to what it was previously. This means that React compares every prop in your component with its previous value using the [`Object.is`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/is) comparison. Note that `Object.is(3, 3)` is `true`, but `Object.is(, )` is `false`.

To get the most out of `memo`, minimize the times that the props change. For example, if the prop is an object, prevent the parent component from re-creating that object every time by using [`useMemo`:](/reference/react/useMemo)

```js 
function Page() {
  const [name, setName] = useState('Taylor');
  const [age, setAge] = useState(42);

  const person = useMemo(
    () => (),
    [name, age]
  );

  return 

---

### Do I still need React.memo if I use React Compiler? 

When you enable [React Compiler](/learn/react-compiler), you typically don't need `React.memo` anymore. The compiler automatically optimizes component re-rendering for you.

Here's how it works:

**Without React Compiler**, you need `React.memo` to prevent unnecessary re-renders:

```js
// Parent re-renders every second
function Parent() {
  const [seconds, setSeconds] = useState(0);

  useEffect(() => {
    const interval = setInterval(() => , 1000);
    return () => clearInterval(interval);
  }, []);

  return (
    <>
      Seconds: 
      

---

## Troubleshooting 
### My component re-renders when a prop is an object, array, or function 

React compares old and new props by shallow equality: that is, it considers whether each new prop is reference-equal to the old prop. If you create a new object or array each time the parent is re-rendered, even if the individual elements are each the same, React will still consider it to be changed. Similarly, if you create a new function when rendering the parent component, React will consider it to have changed even if the function has the same definition. To avoid this, [simplify props or memoize props in the parent component](#minimizing-props-changes).
