---
link: ""
---

  );
}
```

### Linking to a stylesheet 

If a component depends on a certain stylesheet in order to be displayed correctly, you can render a link to that stylesheet within the component. Your component will [suspend](/reference/react/Suspense) while the stylesheet is loading. You must supply the `precedence` prop, which tells React where to place this stylesheet relative to others — stylesheets with higher precedence can override those with lower precedence.

  );
}
```

### Controlling stylesheet precedence 

Stylesheets can conflict with each other, and when they do, the browser goes with the one that comes later in the document. React lets you control the order of stylesheets with the `precedence` prop. In this example, three components render stylesheets, and the ones with the same precedence are grouped together in the ``.

  );
}

function FirstComponent() 

function SecondComponent() 

function ThirdComponent() 

```

Note the `precedence` values themselves are arbitrary and their naming is up to you. React will infer that precedence values it discovers first are "lower" and precedence values it discovers later are "higher".

### Deduplicated stylesheet rendering 

If you render the same stylesheet from multiple components, React will place only a single `` in the document head.

  );
}

function Component() 
```

### Annotating specific items within the document with links 

You can use the `` component with the `itemProp` prop to annotate specific items within the document with links to related resources. In this case, React will *not* place these annotations within the document `` but will place them like any other React component.

```js

  Annotating specific items
  
  ...

```
