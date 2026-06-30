---
title: Keeping Components Pure
---

## Purity: Components as formulas 

In computer science (and especially the world of functional programming), [a pure function](https://wikipedia.org/wiki/Pure_function) is a function with the following characteristics:

* **It minds its own business.** It does not change any objects or variables that existed before it was called.
* **Same inputs, same output.** Given the same inputs, a pure function should always return the same result.

You might already be familiar with one example of pure functions: formulas in math.

Consider this math formula:  = 2.

If  = 2 then  = 4. Always.

If  = 3 then  = 6. Always.

If  = 3,  won't sometimes be  or  or  depending on the time of day or the state of the stock market.

If  = 2 and  = 3,  will _always_ be .

If we made this into a JavaScript function, it would look like this:

```js
function double(number) 
```

In the above example, `double` is a **pure function.** If you pass it `3`, it will return `6`. Always.

React is designed around this concept. **React assumes that every component you write is a pure function.** This means that React components you write must always return the same JSX given the same inputs:

When you pass `drinkers=` to `Recipe`, it will return JSX containing `2 cups of water`. Always.

If you pass `drinkers=`, it will return JSX containing `4 cups of water`. Always.

Just like a math formula.

You could think of your components as recipes: if you follow them and don't introduce new ingredients during the cooking process, you will get the same dish every time. That "dish" is the JSX that the component serves to React to [render.](/learn/render-and-commit)

This component is reading and writing a `guest` variable declared outside of it. This means that **calling this component multiple times will produce different JSX!** And what's more, if _other_ components read `guest`, they will produce different JSX, too, depending on when they were rendered! That's not predictable.

Going back to our formula  = 2, now even if  = 2, we cannot trust that  = 4. Our tests could fail, our users would be baffled, planes would fall out of the sky—you can see how this would lead to confusing bugs!

You can fix this component by [passing `guest` as a prop instead](/learn/passing-props-to-a-component):

Now your component is pure, as the JSX it returns only depends on the `guest` prop.

In general, you should not expect your components to be rendered in any particular order. It doesn't matter if you call  = 2 before or after  = 5: both formulas will resolve independently of each other. In the same way, each component should only "think for itself", and not attempt to coordinate with or depend upon others during rendering. Rendering is like a school exam: each component should calculate JSX on their own!

 = 2 twice doesn't change what  is. Same inputs, same outputs. Always.

Strict Mode has no effect in production, so it won't slow down the app for your users. To opt into Strict Mode, you can wrap your root component into `

### Local mutation: Your component's little secret 

In the above example, the problem was that the component changed a *preexisting* variable while rendering. This is often called a **"mutation"** to make it sound a bit scarier. Pure functions don't mutate variables outside of the function's scope or objects that were created before the call—that makes them impure!

However, **it's completely fine to change variables and objects that you've *just* created while rendering.** In this example, you create an `[]` array, assign it to a `cups` variable, and then `push` a dozen cups into it:

If the `cups` variable or the `[]` array were created outside the `TeaGathering` function, this would be a huge problem! You would be changing a *preexisting* object by pushing items into that array.

However, it's fine because you've created them *during the same render*, inside `TeaGathering`. No code outside of `TeaGathering` will ever know that this happened. This is called **"local mutation"**—it's like your component's little secret.

## Where you _can_ cause side effects 

While functional programming relies heavily on purity, at some point, somewhere, _something_ has to change. That's kind of the point of programming! These changes—updating the screen, starting an animation, changing the data—are called **side effects.** They're things that happen _"on the side"_, not during rendering.

In React, **side effects usually belong inside [event handlers.](/learn/responding-to-events)** Event handlers are functions that React runs when you perform some action—for example, when you click a button. Even though event handlers are defined *inside* your component, they don't run *during* rendering! **So event handlers don't need to be pure.**

If you've exhausted all other options and can't find the right event handler for your side effect, you can still attach it to your returned JSX with a [`useEffect`](/reference/react/useEffect) call in your component. This tells React to execute it later, after rendering, when side effects are allowed. **However, this approach should be your last resort.**

When possible, try to express your logic with rendering alone. You'll be surprised how far this can take you!

In this example, the side effect (modifying the DOM) was not necessary at all. You only needed to return JSX.

#### Fix a broken profile 

Two `Profile` components are rendered side by side with different data. Press "Collapse" on the first profile, and then "Expand" it. You'll notice that both profiles now show the same person. This is a bug.

Find the cause of the bug and fix it.

  )
}

function Header() {
  return ;
}

function Avatar() 
```

```js src/Panel.js hidden
import  from 'react';

export default function Panel() >
        
      
      
    
  );
}
```

```js src/App.js
import Profile from './Profile.js';

export default function App() 

function Header() {
  return ;
}

function Avatar() 
```

```js src/Panel.js hidden
import  from 'react';

export default function Panel() >
        
      
      
    
  );
}
```

```js src/App.js
import Profile from './Profile.js';

export default function App() {
  return (
    <>
      

Remember that React does not guarantee that component functions will execute in any particular order, so you can't communicate between them by setting variables. All communication must happen through props.

#### Fix a broken story tray 

The CEO of your company is asking you to add "stories" to your online clock app, and you can't say no. You've written a `StoryTray` component that accepts a list of `stories`, followed by a "Create Story" placeholder.

You implemented the "Create Story" placeholder by pushing one more fake story at the end of the `stories` array that you receive as a prop. But for some reason, "Create Story" appears more than once. Fix the issue.

Alternatively, you could create a _new_ array (by copying the existing one) before you push an item into it:

This keeps your mutation local and your rendering function pure. However, you still need to be careful: for example, if you tried to change any of the array's existing items, you'd have to clone those items too.

It is useful to remember which operations on arrays mutate them, and which don't. For example, `push`, `pop`, `reverse`, and `sort` will mutate the original array, but `slice`, `filter`, and `map` will create a new one.

