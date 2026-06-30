---
title: immutability
---

## Rule Details 

A component’s props and state are immutable snapshots. Never mutate them directly. Instead, pass new props down, and use the setter function from `useState`.

## Common Violations 

### Invalid 

```js
// ❌ Array push mutation
function Component() {
  const [items, setItems] = useState([1, 2, 3]);

  const addItem = () => ;
}

// ❌ Object property assignment
function Component() {
  const [user, setUser] = useState();

  const updateName = () => ;
}

// ❌ Sort without spreading
function Component() {
  const [items, setItems] = useState([3, 1, 2]);

  const sortItems = () => ;
}
```

### Valid 

```js
// ✅ Create new array
function Component() {
  const [items, setItems] = useState([1, 2, 3]);

  const addItem = () => ;
}

// ✅ Create new object
function Component() {
  const [user, setUser] = useState();

  const updateName = () => {
    setUser(); // New object
  };
}
```

## Troubleshooting 

### I need to add items to an array 

Mutating arrays with methods like `push()` won't trigger re-renders:

```js
// ❌ Wrong: Mutating the array
function TodoList() {
  const [todos, setTodos] = useState([]);

  const addTodo = (id, text) => {
    todos.push();
    setTodos(todos); // Same array reference!
  };

  return (
    
      {todos.map(todo => )}
    
  );
}
```

Create a new array instead:

```js
// ✅ Better: Create a new array
function TodoList() {
  const [todos, setTodos] = useState([]);

  const addTodo = (id, text) => {
    setTodos([...todos, ]);
    // Or: setTodos(todos => [...todos, ])
  };

  return (
    
      {todos.map(todo => )}
    
  );
}
```

### I need to update nested objects 

Mutating nested properties doesn't trigger re-renders:

```js
// ❌ Wrong: Mutating nested object
function UserProfile() {
  const [user, setUser] = useState({
    name: 'Alice',
    settings: 
  });

  const toggleTheme = () => ;
}
```

Spread at each level that needs updating:

```js
// ✅ Better: Create new objects at each level
function UserProfile() {
  const [user, setUser] = useState({
    name: 'Alice',
    settings: 
  });

  const toggleTheme = () => {
    setUser({
      ...user,
      settings: 
    });
  };
}
```