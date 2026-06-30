---
title: cloneElement
---

,
  ,
  'Goodbye'
);

console.log(clonedElement); // 
```

[See more examples below.](#usage)

#### Parameters 

* `element`: The `element` argument must be a valid React element. For example, it could be a JSX node like `, pass it to `cloneElement` with the :

```js [[1, 5, " will be `
```

By cloning its children, the `List` can pass extra information to every `Row` inside. The result looks like this:

```js 

```

Notice how pressing "Next" updates the state of the `List`, and highlights a different row:

  );
}
```

```js src/List.js active
import  from 'react';

export default function List() {
  const [selectedIndex, setSelectedIndex] = useState(0);
  return (
    
      {Children.map(children, (child, index) =>
        cloneElement(child, )
      )}
      
       }>
        Next
      
    
  );
}
```

```js src/Row.js
export default function Row() {
  return (
    
      
    
  );
}
```

```js src/data.js
export const products = [
  ,
  ,
  ,
];
```

```css
.List 

.Row 

.RowHighlighted 

button 
```

To summarize, the `List` cloned the `

---

## Alternatives 

### Passing data with a render prop 

Instead of using `cloneElement`, consider accepting a *render prop* like `renderItem`. Here, `List` receives `renderItem` as a prop. `List` calls `renderItem` for every item and passes `isHighlighted` as an argument:

```js 
export default function List() {
  const [selectedIndex, setSelectedIndex] = useState(0);
  return (
    
      {items.map((item, index) => )}
```

The `renderItem` prop is called a "render prop" because it's a prop that specifies how to render something. For example, you can pass a `renderItem` implementation that renders a `
```

However, you can clearly trace where the `isHighlighted` value is coming from.

This pattern is preferred to `cloneElement` because it is more explicit.

---

### Passing data through context 

Another alternative to `cloneElement` is to [pass data through context.](/learn/passing-data-deeply-with-context)

For example, you can call [`createContext`](/reference/react/createContext) to define a `HighlightContext`:

```js
export const HighlightContext = createContext(false);
```

Your `List` component can wrap every item it renders into a `HighlightContext` provider:

```js 
export default function List() {
  const [selectedIndex, setSelectedIndex] = useState(0);
  return (
    
      {items.map((item, index) => )}
```

With this approach, `Row` does not need to receive an `isHighlighted` prop at all. Instead, it reads the context:

```js src/Row.js 
export default function Row() )}
      
       }>
        Next
      
    
  );
}
```

```js src/Row.js
import  from 'react';
import  from './HighlightContext.js';

export default function Row() {
  const isHighlighted = useContext(HighlightContext);
  return (
    
      
    
  );
}
```

```js src/HighlightContext.js
import  from 'react';

export const HighlightContext = createContext(false);
```

```js src/data.js
export const products = [
  ,
  ,
  ,
];
```

```css
.List 

.Row 

.RowHighlighted 

button 
```

[Learn more about passing data through context.](/reference/react/useContext#passing-data-deeply-into-the-tree)

---

### Extracting logic into a custom Hook 

Another approach you can try is to extract the "non-visual" logic into your own Hook, and use the information returned by your Hook to decide what to render. For example, you could write a `useList` custom Hook like this:

```js
import  from 'react';

export default function useList(items) {
  const [selectedIndex, setSelectedIndex] = useState(0);

  function onNext() 

  const selected = items[selectedIndex];
  return [selected, onNext];
}
```

Then you could use it like this:

```js 
export default function App() {
  const [selected, onNext] = useList(products);
  return (
    
      {products.map(product =>
        

This approach is particularly useful if you want to reuse this logic between different components.
