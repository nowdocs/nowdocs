---
title: Passing Data Deeply with Context
---

## The problem with passing props 

[Passing props](/learn/passing-props-to-a-component) is a great way to explicitly pipe data through your UI tree to the components that use it.

But passing props can become verbose and inconvenient when you need to pass some prop deeply through the tree, or if many components need the same prop. The nearest common ancestor could be far removed from the components that need data, and [lifting state up](/learn/sharing-state-between-components) that high can lead to a situation called "prop drilling".

Wouldn't it be great if there were a way to "teleport" data to the components in the tree that need it without passing props? With React's context feature, there is!

## Context: an alternative to passing props 

Context lets a parent component provide data to the entire tree below it. There are many uses for context. Here is one example. Consider this `Heading` component that accepts a `level` for its size:

      
      
      
      
      
    
  );
}
```

```js src/Section.js
export default function Section() {
  return (
    
      
    
  );
}
```

```js src/Heading.js
export default function Heading() {
  switch (level) {
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

```css
.section 
```

Let's say you want multiple headings within the same `Section` to always have the same size:

      
        
        
        
          
          
          
            
            
          
        
      
    
  );
}
```

```js src/Section.js
export default function Section() {
  return (
    
      
    
  );
}
```

```js src/Heading.js
export default function Heading() {
  switch (level) {
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

```css
.section 
```

Currently, you pass the `level` prop to each `
  
  

```

It would be nice if you could pass the `level` prop to the `
  
  

```

But how can the `

### Step 1: Create the context 

First, you need to create the context. You'll need to **export it from a file** so that your components can use it:

      
        
        
        
          
          
          
            
            
          
        
      
    
  );
}
```

```js src/Section.js
export default function Section() {
  return (
    
      
    
  );
}
```

```js src/Heading.js
export default function Heading() {
  switch (level) {
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

```js src/LevelContext.js active
import  from 'react';

export const LevelContext = createContext(1);
```

```css
.section 
```

The only argument to `createContext` is the _default_ value. Here, `1` refers to the biggest heading level, but you could pass any kind of value (even an object). You will see the significance of the default value in the next step.

### Step 2: Use the context 

Import the `useContext` Hook from React and your context:

```js
import  from 'react';
import  from './LevelContext.js';
```

Currently, the `Heading` component reads `level` from props:

```js
export default function Heading() 
```

Instead, remove the `level` prop and read the value from the context you just imported, `LevelContext`:

```js 
export default function Heading() 
```

`useContext` is a Hook. Just like `useState` and `useReducer`, you can only call a Hook immediately inside a React component (not inside loops or conditions). **`useContext` tells React that the `Heading` component wants to read the `LevelContext`.**

Now that the `Heading` component doesn't have a `level` prop, you don't need to pass the level prop to `Heading` in your JSX like this anymore:

```js

  
  

```

Update the JSX so that it's the `Section` that receives it instead:

```jsx

  
  

```

As a reminder, this is the markup that you were trying to get working:

      
        
        
        
          
          
          
            
            
          
        
      
    
  );
}
```

```js src/Section.js
export default function Section() {
  return (
    
      
    
  );
}
```

```js src/Heading.js
import  from 'react';
import  from './LevelContext.js';

export default function Heading() {
  const level = useContext(LevelContext);
  switch (level) {
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

export const LevelContext = createContext(1);
```

```css
.section 
```

Notice this example doesn't quite work, yet! All the headings have the same size because **even though you're *using* the context, you have not *provided* it yet.** React doesn't know where to get it!

If you don't provide the context, React will use the default value you've specified in the previous step. In this example, you specified `1` as the argument to `createContext`, so `useContext(LevelContext)` returns `1`, setting all those headings to ``. Let's fix this problem by having each `Section` provide its own context.

### Step 3: Provide the context 

The `Section` component currently renders its children:

```js
export default function Section() {
  return (
    
      
    
  );
}
```

**Wrap them with a context provider** to provide the `LevelContext` to them:

```js 
import  from './LevelContext.js';

export default function Section() 
```

This tells React: "if any component inside this `
      
        
        
        
          
          
          
            
            
          
        
      
    
  );
}
```

```js src/Section.js
import  from './LevelContext.js';

export default function Section() 
```

```js src/Heading.js
import  from 'react';
import  from './LevelContext.js';

export default function Heading() {
  const level = useContext(LevelContext);
  switch (level) {
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

export const LevelContext = createContext(1);
```

```css
.section 
```

It's the same result as the original code, but you did not need to pass the `level` prop to each `Heading` component! Instead, it "figures out" its heading level by asking the closest `Section` above:

1. You pass a `level` prop to the `
    
  );
}
```

With this change, you don't need to pass the `level` prop *either* to the `
      
        
        
        
          
          
          
            
            
          
        
      
    
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

Now both `Heading` and `Section` read the `LevelContext` to figure out how "deep" they are. And the `Section` wraps its children into the `LevelContext` to specify that anything inside of it is at a "deeper" level.

## Context passes through intermediate components 

You can insert as many components as you like between the component that provides context and the one that uses it. This includes both built-in components like `` and components you might build yourself.

In this example, the same `Post` component (with a dashed border) is rendered at two different nesting levels. Notice that the `
      
  );
}

function AllPosts() 

function RecentPosts() 

function Post() {
  return (
    
      
    
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

.fancy 
```

You didn't do anything special for this to work. A `Section` specifies the context for the tree inside it, so you can insert a ``. This reduces the number of layers between the component specifying the data and the one that needs it.

If neither of these approaches works well for you, consider context.

## Use cases for context 

* **Theming:** If your app lets the user change its appearance (e.g. dark mode), you can put a context provider at the top of your app, and use that context in components that need to adjust their visual look.
* **Current account:** Many components might need to know the currently logged in user. Putting it in context makes it convenient to read it anywhere in the tree. Some apps also let you operate multiple accounts at the same time (e.g. to leave a comment as a different user). In those cases, it can be convenient to wrap a part of the UI into a nested provider with a different current account value.
* **Routing:** Most routing solutions use context internally to hold the current route. This is how every link "knows" whether it's active or not. If you build your own router, you might want to do it too.
* **Managing state:** As your app grows, you might end up with a lot of state closer to the top of your app. Many distant components below may want to change it. It is common to [use a reducer together with context](/learn/scaling-up-with-reducer-and-context) to manage complex state and pass it down to distant components without too much hassle.

Context is not limited to static values. If you pass a different value on the next render, React will update all the components reading it below! This is why context is often used in combination with state.

In general, if some information is needed by distant components in different parts of the tree, it's a good indication that context will help you.

  )
}

function List() {
  const listItems = places.map(place =>
    
      

Note how components in the middle don't need to pass `imageSize` anymore.

