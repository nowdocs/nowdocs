---
title: Children
---

```

Then, with the `RowList` implementation above, the final rendered result will look like this:

```js

  
    This is the first item.
  
  
    This is the second item.
  
  
    This is the third item.
  

```

`Children.map` is similar to [to transforming arrays with `map()`.](/learn/rendering-lists) The difference is that the `children` data structure is considered *opaque.* This means that even if it's sometimes an array, you should not assume it's an array or any other particular data type. This is why you should use `Children.map` if you need to transform it.

  );
}
```

```js src/RowList.js active
import  from 'react';

export default function RowList() {
  return (
    
      {Children.map(children, child =>
        
          
        
      )}
    
  );
}
```

```css
.RowList 

.Row 
```

  );
}

function MoreRows() 
```

```js src/RowList.js
import  from 'react';

export default function RowList() {
  return (
    
      {Children.map(children, child =>
        
          
        
      )}
    
  );
}
```

```css
.RowList 

.Row 
```

**There is no way to get the rendered output of an inner component** like `

---

### Running some code for each child 

Call `Children.forEach` to iterate over each child in the `children` data structure. It does not return any value and is similar to the [array `forEach` method.](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/forEach) You can use it to run custom logic like constructing your own array.

  );
}
```

```js src/SeparatorList.js active
import  from 'react';

export default function SeparatorList() {
  const result = [];
  Children.forEach(children, (child, index) => );
  result.pop(); // Remove the last separator
  return result;
}
```

---

### Counting children 

Call `Children.count(children)` to calculate the number of children.

  );
}
```

```js src/RowList.js active
import  from 'react';

export default function RowList() {
  return (
    
      
        Total rows: 
      
      {Children.map(children, child =>
        
          
        
      )}
    
  );
}
```

```css
.RowList 

.RowListHeader 

.Row 
```

---

### Converting children to an array 

Call `Children.toArray(children)` to turn the `children` data structure into a regular JavaScript array. This lets you manipulate the array with built-in array methods like [`filter`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/filter), [`sort`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/sort), or [`reverse`.](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/reverse)

  );
}
```

```js src/ReversedList.js active
import  from 'react';

export default function ReversedList() 
```

---

## Alternatives 

### Exposing multiple components 

Manipulating children with the `Children` methods often leads to fragile code. When you pass children to a component in JSX, you don't usually expect the component to manipulate or transform the individual children.

When you can, try to avoid using the `Children` methods. For example, if you want every child of `RowList` to be wrapped in ``, export a `Row` component, and manually wrap every row into it like this:

      
      
    
  );
}
```

```js src/RowList.js
export function RowList() {
  return (
    
      
    
  );
}

export function Row() {
  return (
    
      
    
  );
}
```

```css
.RowList 

.Row 
```

Unlike using `Children.map`, this approach does not wrap every child automatically. **However, this approach has a significant benefit compared to the [earlier example with `Children.map`](#transforming-children) because it works even if you keep extracting more components.** For example, it still works if you extract your own `MoreRows` component:

      
  );
}

function MoreRows() 
```

```js src/RowList.js
export function RowList() {
  return (
    
      
    
  );
}

export function Row() {
  return (
    
      
    
  );
}
```

```css
.RowList 

.Row 
```

This wouldn't work with `Children.map` because it would "see" `

Since `rows` is a regular JavaScript array, the `RowList` component can use built-in array methods like [`map`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/map) on it.

This pattern is especially useful when you want to be able to pass more information as structured data together with children. In the below example, the `TabSwitcher` component receives an array of objects as the `tabs` prop:

Unlike passing the children as JSX, this approach lets you associate some extra data like `header` with each item. Because you are working with the `tabs` directly, and it is an array, you do not need the `Children` methods.

---

### Calling a render prop to customize rendering 

Instead of producing JSX for every single item, you can also pass a function that returns JSX, and call that function when necessary. In this example, the `App` component passes a `renderContent` function to the `TabSwitcher` component. The `TabSwitcher` component calls `renderContent` only for the selected tab:

A prop like `renderContent` is called a *render prop* because it is a prop that specifies how to render a piece of the user interface. However, there is nothing special about it: it is a regular prop which happens to be a function.

Render props are functions, so you can pass information to them. For example, this `RowList` component passes the `id` and the `index` of each row to the `renderRow` render prop, which uses `index` to highlight even rows:

        );
      }}
    />
  );
}
```

```js src/RowList.js
import  from 'react';

export function RowList() {
  return (
    
      
        Total rows: 
      
      
    
  );
}

export function Row() {
  return (
    
      
    
  );
}
```

```css
.RowList 

.RowListHeader 

.Row 

.RowHighlighted 
```

This is another example of how parent and child components can cooperate without manipulating the children.

---

## Troubleshooting 

### I pass a custom component, but the `Children` methods don't show its render result 

Suppose you pass two children to `RowList` like this:

```js

```

If you do `Children.count(children)` inside `RowList`, you will get `2`. Even if `MoreRows` renders 10 different items, or if it returns `null`, `Children.count(children)` will still be `2`. From the `RowList`'s perspective, it only "sees" the JSX it has received. It does not "see" the internals of the `MoreRows` component.

The limitation makes it hard to extract a component. This is why [alternatives](#alternatives) are preferred to using `Children`.
