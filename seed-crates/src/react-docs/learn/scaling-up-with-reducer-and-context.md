---
title: Scaling Up with Reducer and Context
---

## Combining a reducer with context 

In this example from [the introduction to reducers](/learn/extracting-state-logic-into-a-reducer), the state is managed by a reducer. The reducer function contains all of the state update logic and is declared at the bottom of this file:

A reducer helps keep the event handlers short and concise. However, as your app grows, you might run into another difficulty. **Currently, the `tasks` state and the `dispatch` function are only available in the top-level `TaskApp` component.** To let other components read the list of tasks or change it, you have to explicitly [pass down](/learn/passing-props-to-a-component) the current state and the event handlers that change it as props.

For example, `TaskApp` passes a list of tasks and the event handlers to `TaskList`:

```js

Here, you're passing `null` as the default value to both contexts. The actual values will be provided by the `TaskApp` component.

### Step 2: Put state and dispatch into context 

Now you can import both contexts in your `TaskApp` component. Take the `tasks` and `dispatch` returned by `useReducer()` and [provide them](/learn/passing-data-deeply-with-context#step-3-provide-the-context) to the entire tree below:

```js 
import  from './TasksContext.js';

export default function TaskApp() 
```

For now, you pass the information both via props and in context:

    
  );
}

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

let nextId = 3;
const initialTasks = [
  ,
  ,
  
];
```

```js src/TasksContext.js
import  from 'react';

export const TasksContext = createContext(null);
export const TasksDispatchContext = createContext(null);
```

```js src/AddTask.js
import  from 'react';

export default function AddTask() 
      />
       }>Add
    </>
  )
}
```

```js src/TaskList.js
import  from 'react';

export default function TaskList() {
  return (
    
      {tasks.map(task => (
        
          

In the next step, you will remove prop passing.

### Step 3: Use context anywhere in the tree 

Now you don't need to pass the list of tasks or the event handlers down the tree:

```js 

```

Instead, any component that needs the task list can read it from the `TasksContext`:

```js 
export default function TaskList() {
  const tasks = useContext(TasksContext);
  // ...
```

To update the task list, any component can read the `dispatch` function from context and call it:

```js 
export default function AddTask() {
  const [text, setText] = useState('');
  const dispatch = useContext(TasksDispatchContext);
  // ...
  return (
    // ...
     {
      setText('');
      dispatch();
    }}>Add
    // ...
```

**The `TaskApp` component does not pass any event handlers down, and the `TaskList` does not pass any event handlers to the `Task` component either.** Each component reads the context that it needs:

    
  );
}

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

```js src/TasksContext.js
import  from 'react';

export const TasksContext = createContext(null);
export const TasksDispatchContext = createContext(null);
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

```js src/TaskList.js active
import  from 'react';
import  from './TasksContext.js';

export default function TaskList() {
  const tasks = useContext(TasksContext);
  return (
    
      {tasks.map(task => (
        
          

**The state still "lives" in the top-level `TaskApp` component, managed with `useReducer`.** But its `tasks` and `dispatch` are now available to every component below in the tree by importing and using these contexts.

## Moving all wiring into a single file 

You don't have to do this, but you could further declutter the components by moving both reducer and context into a single file. Currently, `TasksContext.js` contains only two context declarations:

```js
import  from 'react';

export const TasksContext = createContext(null);
export const TasksDispatchContext = createContext(null);
```

This file is about to get crowded! You'll move the reducer into that same file. Then you'll declare a new `TasksProvider` component in the same file. This component will tie all the pieces together:

1. It will manage the state with a reducer.
2. It will provide both contexts to components below.
3. It will [take `children` as a prop](/learn/passing-props-to-a-component#passing-jsx-as-children) so you can pass JSX to it.

```js
export function TasksProvider() 
```

**This removes all the complexity and wiring from your `TaskApp` component:**

  );
}
```

```js src/TasksContext.js
import  from 'react';

export const TasksContext = createContext(null);
export const TasksDispatchContext = createContext(null);

export function TasksProvider() 

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
  const tasks = useContext(TasksContext);
  return (
    
      {tasks.map(task => (
        
          

You can also export functions that _use_ the context from `TasksContext.js`:

```js
export function useTasks() 

export function useTasksDispatch() 
```

When a component needs to read context, it can do it through these functions:

```js
const tasks = useTasks();
const dispatch = useTasksDispatch();
```

This doesn't change the behavior in any way, but it lets you later split these contexts further or add some logic to these functions. **Now all of the context and reducer wiring is in `TasksContext.js`. This keeps the components clean and uncluttered, focused on what they display rather than where they get the data:**

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

```js src/TaskList.js active
import  from 'react';
import  from './TasksContext.js';

export default function TaskList() {
  const tasks = useTasks();
  return (
    
      {tasks.map(task => (
        
          

You can think of `TasksProvider` as a part of the screen that knows how to deal with tasks, `useTasks` as a way to read them, and `useTasksDispatch` as a way to update them from any component below in the tree.

As your app grows, you may have many context-reducer pairs like this. This is a powerful way to scale your app and [lift state up](/learn/sharing-state-between-components) without too much work whenever you want to access the data deep in the tree.

