---
title: globals
---

## Rule Details 

Global variables exist outside React's control. When you modify them during render, you break React's assumption that rendering is pure. This can cause components to behave differently in development vs production, break Fast Refresh, and make your app impossible to optimize with features like React Compiler.

### Invalid 

Examples of incorrect code for this rule:

```js
// ❌ Global counter
let renderCount = 0;
function Component() {
  renderCount++; // Mutating global
  return Count: ;
}

// ❌ Modifying window properties
function Component() {
  window.currentUser = userId; // Global mutation
  return User: ;
}

// ❌ Global array push
const events = [];
function Component() {
  events.push(event); // Mutating global array
  return Events: ;
}

// ❌ Cache manipulation
const cache = ;
function Component() {
  if (!cache[id]) 
  return ;
}
```

### Valid 

Examples of correct code for this rule:

```js
// ✅ Use state for counters
function Component() {
  const [clickCount, setClickCount] = useState(0);

  const handleClick = () => ;

  return (
    
      Clicked:  times
    
  );
}

// ✅ Use context for global values
function Component() {
  const user = useContext(UserContext);
  return User: ;
}

// ✅ Synchronize external state with React
function Component() {
  useEffect(() => , [title]);

  return Page: ;
}
```
