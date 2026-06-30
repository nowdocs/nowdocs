---
title: createContext
---

  );
}
```

#### Props 

* `value`: The value that you want to pass to all the components reading this context inside this provider, no matter how deep. The context value can be of any type. A component calling [`useContext(SomeContext)`](/reference/react/useContext) inside of the provider receives the `value` of the innermost corresponding context provider above it.

---

### `SomeContext.Consumer` 

Before `useContext` existed, there was an older way to read context:

```js
function Button() {
  // 🟡 Legacy way (not recommended)
  return (
    . Components can read context by passing it to [`useContext()`](/reference/react/useContext):

```js [[1, 2, "ThemeContext"], [1, 7, "AuthContext"]]
function Button() 

function Profile() 
```

By default, the values they receive will be the  you have specified when creating the contexts. However, by itself this isn't useful because the default values never change.

Context is useful because you can **provide other, dynamic values from your components:**

```js 
function App() {
  const [theme, setTheme] = useState('dark');
  const [currentUser, setCurrentUser] = useState();

  // ...

  return (
    
    
  );
}
```

Now the `Page` component and any components inside it, no matter how deep, will "see" the passed context values. If the passed context values change, React will re-render the components reading the context as well.

[Read more about reading and providing context and see examples.](/reference/react/useContext)

---

### Importing and exporting context from a file 

Often, components in different files will need access to the same context. This is why it's common to declare contexts in a separate file. Then you can use the [`export` statement](https://developer.mozilla.org/en-US/docs/web/javascript/reference/statements/export) to make context available for other files:

```js 
// Contexts.js
import  from 'react';

export const ThemeContext = createContext('light');
export const AuthContext = createContext(null);
```

Components declared in other files can then use the [`import`](https://developer.mozilla.org/en-US/docs/web/javascript/reference/statements/import) statement to read or provide this context:

```js 
// Button.js
import  from './Contexts.js';

function Button() 
```

```js 
// App.js
import  from './Contexts.js';

function App() 
```

This works similar to [importing and exporting components.](/learn/importing-and-exporting-components)

---

## Troubleshooting 

### I can't find a way to change the context value 

Code like this specifies the *default* context value:

```js
const ThemeContext = createContext('light');
```

This value never changes. React only uses this value as a fallback if it can't find a matching provider above.

To make context change over time, [add state and wrap components in a context provider.](/reference/react/useContext#updating-data-passed-via-context)
