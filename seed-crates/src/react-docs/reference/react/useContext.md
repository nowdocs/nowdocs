---
title: useContext
---

 for the  you passed. To determine the context value, React searches the component tree and finds **the closest context provider above** for that particular context.

To pass context to a `Button`, wrap it or one of its parent components into the corresponding context provider:

```js [[1, 3, "ThemeContext"], [2, 3, "\\"dark\\""], [1, 5, "ThemeContext"]]
function MyPage() 

function Form() 
```

It doesn't matter how many layers of components there are between the provider and the `Button`. When a `Button` *anywhere* inside of `Form` calls `useContext(ThemeContext)`, it will receive `"dark"` as the value.

  )
}

function Form() 

function Panel() {
  const theme = useContext(ThemeContext);
  const className = 'panel-' + theme;
  return (
    
      
      
    
  )
}

function Button() {
  const theme = useContext(ThemeContext);
  const className = 'button-' + theme;
  return (
    
      
    
  );
}
```

```css
.panel-light,
.panel-dark 
.panel-light 

.panel-dark 

.button-light,
.button-dark 

.button-dark 

.button-light 
```

---

### Updating data passed via context 

Often, you'll want the context to change over time. To update context, combine it with [state.](/reference/react/useState) Declare a state variable in the parent component, and pass the current state down as the  to the provider.

```js  [[1, 4, "ThemeContext"], [2, 4, "theme"], [1, 11, "ThemeContext"]]
function MyPage() 
```

Now any `Button` inside of the provider will receive the current `theme` value. If you call `setTheme` to update the `theme` value that you pass to the provider, all `Button` components will re-render with the new `'light'` value.

  )
}

function Form() 

function Panel() {
  const theme = useContext(ThemeContext);
  const className = 'panel-' + theme;
  return (
    
      
      
    
  )
}

function Button() {
  const theme = useContext(ThemeContext);
  const className = 'button-' + theme;
  return (
    
      
    
  );
}
```

```css
.panel-light,
.panel-dark 
.panel-light 

.panel-dark 

.button-light,
.button-dark 

.button-dark 

.button-light 
```

Note that `value="dark"` passes the `"dark"` string, but `value=` passes the value of the JavaScript `theme` variable with [JSX curly braces.](/learn/javascript-in-jsx-with-curly-braces) Curly braces also let you pass context values that aren't strings.

  );
}

function Form() 

function LoginButton() {
  const  = useContext(CurrentUserContext);

  if (currentUser !== null) {
    return You logged in as .;
  }

  return (
    
  );
}

function Panel() {
  return (
    
      
      
    
  )
}

function Button() {
  return (
    
      
    
  );
}
```

```css
label 

.panel 

.button 
```

    
  )
}

function WelcomePanel() {
  const  = useContext(CurrentUserContext);
  return (
    
  );
}

function Greeting() {
  const  = useContext(CurrentUserContext);
  return (
    You logged in as .
  )
}

function LoginForm() {
  const  = useContext(CurrentUserContext);
  const [firstName, setFirstName] = useState('');
  const [lastName, setLastName] = useState('');
  const canLogin = firstName.trim() !== '' && lastName.trim() !== '';
  return (
    <>
      
        First name
         setFirstName(e.target.value)}
        />
      
      
        Last name
         setLastName(e.target.value)}
        />
      
      
      
    </>
  );
}

function Panel() {
  const theme = useContext(ThemeContext);
  const className = 'panel-' + theme;
  return (
    
      
      
    
  )
}

function Button() {
  const theme = useContext(ThemeContext);
  const className = 'button-' + theme;
  return (
    
      
    
  );
}
```

```css
label 

.panel-light,
.panel-dark 
.panel-light 

.panel-dark 

.button-light,
.button-dark 

.button-dark 

.button-light 
```

  );
}

function MyProviders() 

function WelcomePanel() {
  const  = useContext(CurrentUserContext);
  return (
    
  );
}

function Greeting() {
  const  = useContext(CurrentUserContext);
  return (
    You logged in as .
  )
}

function LoginForm() {
  const  = useContext(CurrentUserContext);
  const [firstName, setFirstName] = useState('');
  const [lastName, setLastName] = useState('');
  const canLogin = firstName !== '' && lastName !== '';
  return (
    <>
      
        First name
         setFirstName(e.target.value)}
        />
      
      
        Last name
         setLastName(e.target.value)}
        />
      
      
      
    </>
  );
}

function Panel() {
  const theme = useContext(ThemeContext);
  const className = 'panel-' + theme;
  return (
    
      
      
    
  )
}

function Button() {
  const theme = useContext(ThemeContext);
  const className = 'button-' + theme;
  return (
    
      
    
  );
}
```

```css
label 

.panel-light,
.panel-dark 
.panel-light 

.panel-dark 

.button-light,
.button-dark 

.button-dark 

.button-light 
```

  );
}
```

```js src/TasksContext.js
import  from 'react';

const TasksContext = createContext(null);

const TasksDispatchContext = createContext(null);

export function TasksProvider() 

export function useTasks() 

export function useTasksDispatch() 

function tasksReducer(tasks, action) {
  switch (action.type) {
    case 'added': {
      return [...tasks, ];
    }
    case 'changed': {
      return tasks.map(t => {
        if (t.id === action.task.id)  else 
      });
    }
    case 'deleted': 
    default: 
  }
}

const initialTasks = [
  ,
  ,
  
];
```

```js src/AddTask.js
import  from 'react';
import  from './TasksContext.js';

export default function AddTask() 
      />
       {
        setText('');
        dispatch();
      }}>Add
    </>
  );
}

let nextId = 3;
```

```js src/TaskList.js
import  from 'react';
import  from './TasksContext.js';

export default function TaskList() {
  const tasks = useTasks();
  return (
    
      {tasks.map(task => (
        
          

---

### Specifying a fallback default value 

If React can't find any providers of that particular  in the parent tree, the context value returned by `useContext()` will be equal to the  that you specified when you [created that context](/reference/react/createContext):

```js [[1, 1, "ThemeContext"], [3, 1, "null"]]
const ThemeContext = createContext(null);
```

The default value **never changes**. If you want to update context, use it with state as [described above.](#updating-data-passed-via-context)

Often, instead of `null`, there is some more meaningful value you can use as a default, for example:

```js [[1, 1, "ThemeContext"], [3, 1, "light"]]
const ThemeContext = createContext('light');
```

This way, if you accidentally render some component without a corresponding provider, it won't break. This also helps your components work well in a test environment without setting up a lot of providers in the tests.

In the example below, the "Toggle theme" button is always light because it's **outside any theme context provider** and the default context theme value is `'light'`. Try editing the default theme to be `'dark'`.

      
    </>
  )
}

function Form() 

function Panel() {
  const theme = useContext(ThemeContext);
  const className = 'panel-' + theme;
  return (
    
      
      
    
  )
}

function Button() {
  const theme = useContext(ThemeContext);
  const className = 'button-' + theme;
  return (
    
      
    
  );
}
```

```css
.panel-light,
.panel-dark 
.panel-light 

.panel-dark 

.button-light,
.button-dark 

.button-dark 

.button-light 
```

---

### Overriding context for a part of the tree 

You can override the context for a part of the tree by wrapping that part in a provider with a different value.

```js 

  ...

```

You can nest and override providers as many times as you need.

  )
}

function Form() 

function Footer() 

function Panel() {
  const theme = useContext(ThemeContext);
  const className = 'panel-' + theme;
  return (
    
      {title && }
      
    
  )
}

function Button() {
  const theme = useContext(ThemeContext);
  const className = 'button-' + theme;
  return (
    
      
    
  );
}
```

```css
footer 

.panel-light,
.panel-dark 
.panel-light 

.panel-dark 

.button-light,
.button-dark 

.button-dark 

.button-light 
```

      
        
        
        
          
          
          
            
            
          
        
      
    
  );
}
```

```js src/Section.js
import  from 'react';
import  from './LevelContext.js';

export default function Section() 
```

```js src/Heading.js
import  from 'react';
import  from './LevelContext.js';

export default function Heading() {
  const level = useContext(LevelContext);
  switch (level) {
    case 0:
      throw Error('Heading must be inside a Section!');
    case 1:
      return ;
    case 2:
      return ;
    case 3:
      return ;
    case 4:
      return ;
    case 5:
      return ;
    case 6:
      return ;
    default:
      throw Error('Unknown level: ' + level);
  }
}
```

```js src/LevelContext.js
import  from 'react';

export const LevelContext = createContext(0);
```

```css
.section 
```

---

### Optimizing re-renders when passing objects and functions 

You can pass any values via context, including objects and functions.

```js [[2, 10, ""]]
function MyApp() {
  const [currentUser, setCurrentUser] = useState(null);

  function login(response) 

  return (
    
  );
}
```

Here, the  is a JavaScript object with two properties, one of which is a function. Whenever `MyApp` re-renders (for example, on a route update), this will be a *different* object pointing at a *different* function, so React will also have to re-render all components deep in the tree that call `useContext(AuthContext)`.

In smaller apps, this is not a problem. However, there is no need to re-render them if the underlying data, like `currentUser`, has not changed. To help React take advantage of that fact, you may wrap the `login` function with [`useCallback`](/reference/react/useCallback) and wrap the object creation into [`useMemo`](/reference/react/useMemo). This is a performance optimization:

```js 
import  from 'react';

function MyApp() {
  const [currentUser, setCurrentUser] = useState(null);

  const login = useCallback((response) => , []);

  const contextValue = useMemo(() => (), [currentUser, login]);

  return (
    
  );
}
```

As a result of this change, even if `MyApp` needs to re-render, the components calling `useContext(AuthContext)` won't need to re-render unless `currentUser` has changed.

Read more about [`useMemo`](/reference/react/useMemo#skipping-re-rendering-of-components) and [`useCallback`.](/reference/react/useCallback#skipping-re-rendering-of-components)

---

## Troubleshooting 

### My component doesn't see the value from my provider 

There are a few common ways that this can happen:

1. You're rendering `
```

If you forget to specify `value`, it's like passing `value=`.

You may have also mistakingly used a different prop name by mistake:

```js 
// 🚩 Doesn't work: prop should be called "value"

```

In both of these cases you should see a warning from React in the console. To fix them, call the prop `value`:

```js 
// ✅ Passing the value prop

```

Note that the [default value from your `createContext(defaultValue)` call](#specifying-a-fallback-default-value) is only used **if there is no matching provider above at all.** If there is a `` component somewhere in the parent tree, the component calling `useContext(SomeContext)` *will* receive `undefined` as the context value.
