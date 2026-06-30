---
title: Components and Hooks must be pure
---

---

## Components and Hooks must be idempotent 

Components must always return the same output with respect to their inputs – props, state, and context. This is known as _idempotency_. [Idempotency](https://en.wikipedia.org/wiki/Idempotence) is a term popularized in functional programming. It refers to the idea that you [always get the same result every time](learn/keeping-components-pure) you run that piece of code with the same inputs.

This means that _all_ code that runs [during render](#how-does-react-run-your-code) must also be idempotent in order for this rule to hold. For example, this line of code is not idempotent (and therefore, neither is the component):

```js 
function Clock() {
  const time = new Date(); // 🔴 Bad: always returns a different result!
  return 
}
```

`new Date()` is not idempotent as it always returns the current date and changes its result every time it's called. When you render the above component, the time displayed on the screen will stay stuck on the time that the component was rendered. Similarly, functions like `Math.random()` also aren't idempotent, because they return different results every time they're called, even when the inputs are the same.

This doesn't mean you shouldn't use non-idempotent functions like `new Date()` _at all_ – you should just avoid using them [during render](#how-does-react-run-your-code). In this case, we can _synchronize_ the latest date to this component using an [Effect](/reference/react/useEffect):

By wrapping the non-idempotent `new Date()` call in an Effect, it moves that calculation [outside of rendering](#how-does-react-run-your-code).

If you don't need to synchronize some external state with React, you can also consider using an [event handler](/learn/responding-to-events) if it only needs to be updated in response to a user interaction.

---

## Side effects must run outside of render 

[Side effects](/learn/keeping-components-pure#side-effects-unintended-consequences) should not run [in render](#how-does-react-run-your-code), as React can render components multiple times to create the best possible user experience.

While render must be kept pure, side effects are necessary at some point in order for your app to do anything interesting, like showing something on the screen! The key point of this rule is that side effects should not run [in render](#how-does-react-run-your-code), as React can render components multiple times. In most cases, you'll use [event handlers](learn/responding-to-events) to handle side effects. Using an event handler explicitly tells React that this code doesn't need to run during render, keeping render pure. If you've exhausted all options – and only as a last resort – you can also handle side effects using `useEffect`.

### When is it okay to have mutation? 

#### Local mutation 
One common example of a side effect is mutation, which in JavaScript refers to changing the value of a non-[primitive](https://developer.mozilla.org/en-US/docs/Glossary/Primitive) value. In general, while mutation is not idiomatic in React, _local_ mutation is absolutely fine:

```js 
function FriendList() {
  const items = []; // ✅ Good: locally created
  for (let i = 0; i < friends.length; i++) 
```

```js 
function Post() 
```

### Don't mutate State 
`useState` returns the state variable and a setter to update that state.

```js
const [stateVariable, setter] = useState(0);
```

Rather than updating the state variable in-place, we need to update it using the setter function that is returned by `useState`. Changing values on the state variable doesn't cause the component to update, leaving your users with an outdated UI. Using the setter function informs React that the state has changed, and that we need to queue a re-render to update the UI.

```js {expectedErrors: } 
function Counter() {
  const [count, setCount] = useState(0);

  function handleClick() 

  return (
    
      You pressed me  times
    
  );
}
```

```js 
function Counter() {
  const [count, setCount] = useState(0);

  function handleClick() 

  return (
    
      You pressed me  times
    
  );
}
```

---

## Return values and arguments to Hooks are immutable 

Once values are passed to a hook, you should not modify them. Like props in JSX, values become immutable when passed to a hook.

```js {expectedErrors: } 
function useIconStyle(icon) {
  const theme = useContext(ThemeContext);
  if (icon.enabled) 
  return icon;
}
```

```js 
function useIconStyle(icon) {
  const theme = useContext(ThemeContext);
  const newIcon = ; // ✅ Good: make a copy instead
  if (icon.enabled) 
  return newIcon;
}
```

One important principle in React is _local reasoning_: the ability to understand what a component or hook does by looking at its code in isolation. Hooks should be treated like "black boxes" when they are called. For example, a custom hook might have used its arguments as dependencies to memoize values inside it:

```js 
function useIconStyle(icon) {
  const theme = useContext(ThemeContext);

  return useMemo(() => {
    const newIcon = ;
    if (icon.enabled) 
    return newIcon;
  }, [icon, theme]);
}
```

If you were to mutate the Hook's arguments, the custom hook's memoization will become incorrect,  so it's important to avoid doing that.

```js 
style = useIconStyle(icon);         // `style` is memoized based on `icon`
icon.enabled = false;               // Bad: 🔴 never mutate hook arguments directly
style = useIconStyle(icon);         // previously memoized result is returned
```

```js 
style = useIconStyle(icon);         // `style` is memoized based on `icon`
icon = ; // Good: ✅ make a copy instead
style = useIconStyle(icon);         // new value of `style` is calculated
```

Similarly, it's important to not modify the return values of Hooks, as they may have been memoized.

---

## Values are immutable after being passed to JSX 

Don't mutate values after they've been used in JSX. Move the mutation to before the JSX is created.

When you use JSX in an expression, React may eagerly evaluate the JSX before the component finishes rendering. This means that mutating values after they've been passed to JSX can lead to outdated UIs, as React won't know to update the component's output.

```js {expectedErrors: } 
function Page() {
  const styles = ;
  const header = ;
  styles.size = "small"; // 🔴 Bad: styles was already used in the JSX above
  const footer = ;
  return (
    <>
      
      
      
    </>
  );
}
```

```js 
function Page() {
  const headerStyles = ;
  const header = ;
  const footerStyles = ; // ✅ Good: we created a new value
  const footer = ;
  return (
    <>
      
      
      
    </>
  );
}
```
