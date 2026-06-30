---
title: Passing Props to a Component
---

## Familiar props 

Props are the information that you pass to a JSX tag. For example, `className`, `src`, `alt`, `width`, and `height` are some of the props you can pass to an ``:

The props you can pass to an `` tag are predefined (ReactDOM conforms to [the HTML standard](https://www.w3.org/TR/html52/semantics-embedded-content.html#the-img-element)). But you can pass any props to *your own* components, such as `

Now you can read these props inside the `Avatar` component.

### Step 2: Read props inside the child component 

You can read these props by listing their names `person, size` separated by the commas inside `()` directly after `function Avatar`. This lets you use them inside the `Avatar` code, like you would with a variable.

```js
function Avatar() 
```

Add some logic to `Avatar` that uses the `person` and `size` props for rendering, and you're done.

Now you can configure `Avatar` to render in many different ways with different props. Try tweaking the values!

Props let you think about parent and child components independently. For example, you can change the `person` or the `size` props inside `Profile` without having to think about how `Avatar` uses them. Similarly, you can change how the `Avatar` uses these props, without looking at the `Profile`.

You can think of props like "knobs" that you can adjust. They serve the same role as arguments serve for functions—in fact, props _are_ the only argument to your component! React component functions accept a single argument, a `props` object:

```js
function Avatar(props) 
```

Usually you don't need the whole `props` object itself, so you destructure it into individual props.

## Specifying a default value for a prop 

If you want to give a prop a default value to fall back on when no value is specified, you can do it with the destructuring by putting `=` and the default value right after the parameter:

```js
function Avatar() 
```

Now, if `
```

When you nest content inside a JSX tag, the parent component will receive that content in a prop called `children`. For example, the `Card` component below will receive a `children` prop set to `
  );
}
```

```js src/Avatar.js
import  from './utils.js';

export default function Avatar() 
```

```js src/utils.js
export function getImageUrl(person, size = 's') 
```

```css
.card 
.avatar 
```

Try replacing the `

This example illustrates that **a component may receive different props over time.** Props are not always static! Here, the `time` prop changes every second, and the `color` prop changes when you select another color. Props reflect a component's data at any point in time, rather than only in the beginning.

However, props are [immutable](https://en.wikipedia.org/wiki/Immutable_object)—a term from computer science meaning "unchangeable". When a component needs to change its props (for example, in response to a user interaction or new data), it will have to "ask" its parent component to pass it _different props_—a new object! Its old props will then be cast aside, and eventually the JavaScript engine will reclaim the memory taken by them.

**Don't try to "change props".** When you need to respond to the user input (like changing the selected color), you will need to "set state", which you can learn about in [State: A Component's Memory.](/learn/state-a-components-memory)

` will appear as `Card` component's `children` prop.
* Props are read-only snapshots in time: every render receives a new version of props.
* You can't change props. When you need interactivity, you'll need to set state.

Note how you don't need a separate `awardCount` prop if `awards` is an array. Then you can use `awards.length` to count the number of awards. Remember that props can take any values, and that includes arrays too!

Another solution, which is more similar to the earlier examples on this page, is to group all information about a person in a single object, and pass that object as one prop:

Although the syntax looks slightly different because you're describing properties of a JavaScript object rather than a collection of JSX attributes, these examples are mostly equivalent, and you can pick either approach.

#### Adjust the image size based on a prop 

In this example, `Avatar` receives a numeric `size` prop which determines the `` width and height. The `size` prop is set to `40` in this example. However, if you open the image in a new tab, you'll notice that the image itself is larger (`160` pixels). The real image size is determined by which thumbnail size you're requesting.

Change the `Avatar` component to request the closest image size based on the `size` prop. Specifically, if the `size` is less than `90`, pass `'s'` ("small") rather than `'b'` ("big") to the `getImageUrl` function. Verify that your changes work by rendering avatars with different values of the `size` prop and opening images in a new tab.

You could also show a sharper image for high DPI screens by taking [`window.devicePixelRatio`](https://developer.mozilla.org/en-US/docs/Web/API/Window/devicePixelRatio) into account:

Props let you encapsulate logic like this inside the `Avatar` component (and change it later if needed) so that everyone can use the `

#### Passing JSX in a `children` prop 

Extract a `Card` component from the markup below, and use the `children` prop to pass different JSX to it:

      
    
  );
}
```

```css
.card 
.card-content 
.avatar 
h1 
```

You can also make `title` a separate prop if you want every `Card` to always have a title:

      
    
  );
}
```

```css
.card 
.card-content 
.avatar 
h1 
```

