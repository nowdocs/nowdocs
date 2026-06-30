---
title: createPortal
---

 and the :

```js [[1, 8, "This child is placed in the document body."], [2, 9, "document.body"]]
import  from 'react-dom';

function MyComponent() {
  return (
    
      This child is placed in the parent div.
      
    
  );
}
```

React will put the DOM nodes for  inside of the .

Without a portal, the second `` would be placed inside the parent ``, but the portal "teleported" it into the [`document.body`:](https://developer.mozilla.org/en-US/docs/Web/API/Document/body)

Notice how the second paragraph visually appears outside the parent `` with the border. If you inspect the DOM structure with developer tools, you'll see that the second `` got placed directly into the ``:

```html 

  
    ...
      
        This child is placed inside the parent div.
      
    ...
  
  This child is placed in the document body.

```

A portal only changes the physical placement of the DOM node. In every other way, the JSX you render into a portal acts as a child node of the React component that renders it. For example, the child can access the context provided by the parent tree, and events still bubble up from children to parents according to the React tree.

---

### Rendering a modal dialog with a portal 

You can use a portal to create a modal dialog that floats above the rest of the page, even if the component that summons the dialog is inside a container with `overflow: hidden` or other styles that interfere with the dialog.

In this example, the two containers have styles that disrupt the modal dialog, but the one rendered into a portal is unaffected because, in the DOM, the modal is not contained within the parent JSX elements.

---

### Rendering React components into non-React server markup 

Portals can be useful if your React root is only part of a static or server-rendered page that isn't built with React. For example, if your page is built with a server framework like Rails, you can create areas of interactivity within static areas such as sidebars. Compared with having [multiple separate React roots,](/reference/react-dom/client/createRoot#rendering-a-page-partially-built-with-react) portals let you treat the app as a single React tree with shared state even though its parts render to different parts of the DOM.

);
```

```js src/App.js active
import  from 'react-dom';

const sidebarContentEl = document.getElementById('sidebar-content');

export default function App() {
  return (
    <>
      

---

### Rendering React components into non-React DOM nodes 

You can also use a portal to manage the content of a DOM node that's managed outside of React. For example, suppose you're integrating with a non-React map widget and you want to render React content inside a popup. To do this, declare a `popupContainer` state variable to store the DOM node you're going to render into:

```js
const [popupContainer, setPopupContainer] = useState(null);
```

When you create the third-party widget, store the DOM node returned by the widget so you can render into it:

```js 
useEffect(() => {
  if (mapRef.current === null) 
}, []);
```

This lets you use `createPortal` to render React content into `popupContainer` once it becomes available:

```js 
return (
  
    
  
);
```

Here is a complete example you can play with:

