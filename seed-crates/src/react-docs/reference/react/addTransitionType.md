---
title: addTransitionType
version: canary
---

 inside the scope of , React will associate  as one of the causes for the Transition.

Currently, Transition Types can be used to customize different animations based on what caused the Transition. You have three different ways to choose from for how to use them:

- [Customize animations using browser view transition types](#customize-animations-using-browser-view-transition-types)
- [Customize animations using `View Transition` Class](#customize-animations-using-view-transition-class)
- [Customize animations using `ViewTransition` events](#customize-animations-using-viewtransition-events)

In the future, we plan to support more use cases for using the cause of a transition.

---
### Customize animations using browser view transition types 

When a [`ViewTransition`](/reference/react/ViewTransition) activates from a transition, React adds all the Transition Types as browser [view transition types](https://www.w3.org/TR/css-view-transitions-2/#active-view-transition-pseudo-examples) to the element.

This allows you to customize different animations based on CSS scopes:

```js [11]
function Component() 

startTransition(() => );
```

```css
:root:active-view-transition-type(my-transition-type) {
  &::view-transition-...(...) 
}
```

---

### Customize animations using `View Transition` Class 

You can customize animations for an activated `ViewTransition` based on type by passing an object to the View Transition Class:

```js
function Component() 

// ...
startTransition(() => );
```

If multiple types match, then they're joined together. If no types match then the special "default" entry is used instead. If any type has the value "none" then that wins and the ViewTransition is disabled (not assigned a name).

These can be combined with enter/exit/update/layout/share props to match based on kind of trigger and Transition Type.

```js

```

---

### Customize animations using `ViewTransition` events 

You can imperatively customize animations for an activated `ViewTransition` based on type using View Transition events:

```
 {
  if (types.includes('navigation-back'))  else if (types.includes('navigation-forward'))  else 
}}>
```

This allows you to pick different imperative Animations based on the cause.
