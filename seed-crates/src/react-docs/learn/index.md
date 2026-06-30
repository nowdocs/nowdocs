---
title: Quick Start
---

## Creating and nesting components 

React apps are made out of *components*. A component is a piece of the UI (user interface) that has its own logic and appearance. A component can be as small as a button, or as large as an entire page.

React components are JavaScript functions that return markup:

```js
function MyButton() 
```

Now that you've declared `MyButton`, you can nest it into another component:

```js 
export default function MyApp() {
  return (
    
      Welcome to my app
      

The `export default` keywords specify the main component in the file. If you're not familiar with some piece of JavaScript syntax, [MDN](https://developer.mozilla.org/en-US/docs/web/javascript/reference/statements/export) and [javascript.info](https://javascript.info/import-export) have great references.

## Writing markup with JSX 

The markup syntax you've seen above is called *JSX*. It is optional, but most React projects use JSX for its convenience. All of the [tools we recommend for local development](/learn/installation) support JSX out of the box.

JSX is stricter than HTML. You have to close tags like ``. Your component also can't return multiple JSX tags. You have to wrap them into a shared parent, like a `...` or an empty `<>...</>` wrapper:

```js 
function AboutPage() 
```

If you have a lot of HTML to port to JSX, you can use an [online converter.](https://transform.tools/html-to-jsx)

## Adding styles 

In React, you specify a CSS class with `className`. It works the same way as the HTML [`class`](https://developer.mozilla.org/en-US/docs/Web/HTML/Global_attributes/class) attribute:

```js

```

Then you write the CSS rules for it in a separate CSS file:

```css
/* In your CSS */
.avatar 
```

React does not prescribe how you add CSS files. In the simplest case, you'll add a [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/link) tag to your HTML. If you use a build tool or a framework, consult its documentation to learn how to add a CSS file to your project.

## Displaying data 

JSX lets you put markup into JavaScript. Curly braces let you "escape back" into JavaScript so that you can embed some variable from your code and display it to the user. For example, this will display `user.name`:

```js 
return (
  
    
  
);
```

You can also "escape into JavaScript" from JSX attributes, but you have to use curly braces *instead of* quotes. For example, `className="avatar"` passes the `"avatar"` string as the CSS class, but `src=` reads the JavaScript `user.imageUrl` variable value, and then passes that value as the `src` attribute:

```js 
return (
  
);
```

You can put more complex expressions inside the JSX curly braces too, for example, [string concatenation](https://javascript.info/operators#string-concatenation-with-binary):

In the above example, `style={}` is not a special syntax, but a regular `` object inside the `style=` JSX curly braces. You can use the `style` attribute when your styles depend on JavaScript variables.

## Conditional rendering 

In React, there is no special syntax for writing conditions. Instead, you'll use the same techniques as you use when writing regular JavaScript code. For example, you can use an [`if`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/if...else) statement to conditionally include JSX:

```js
let content;
if (isLoggedIn) {
  content = 

## Responding to events 

You can respond to events by declaring *event handler* functions inside your components:

```js 
function MyButton() {
  function handleClick() 

  return (
    
      Click me
    
  );
}
```

Notice how `onClick=` has no parentheses at the end! Do not _call_ the event handler function: you only need to *pass it down*. React will call your event handler when the user clicks the button.

## Updating the screen 

Often, you'll want your component to "remember" some information and display it. For example, maybe you want to count the number of times a button is clicked. To do this, add *state* to your component.

First, import [`useState`](/reference/react/useState) from React:

```js
import  from 'react';
```

Now you can declare a *state variable* inside your component:

```js
function MyButton() {
  const [count, setCount] = useState(0);
  // ...
```

You’ll get two things from `useState`: the current state (`count`), and the function that lets you update it (`setCount`). You can give them any names, but the convention is to write `[something, setSomething]`.

The first time the button is displayed, `count` will be `0` because you passed `0` to `useState()`. When you want to change state, call `setCount()` and pass the new value to it. Clicking this button will increment the counter:

```js 
function MyButton() {
  const [count, setCount] = useState(0);

  function handleClick() 

  return (
    
      Clicked  times
    
  );
}
```

React will call your component function again. This time, `count` will be `1`. Then it will be `2`. And so on.

If you render the same component multiple times, each will get its own state. Click each button separately:

Notice how each button "remembers" its own `count` state and doesn't affect other buttons.

## Using Hooks 

Functions starting with `use` are called *Hooks*. `useState` is a built-in Hook provided by React. You can find other built-in Hooks in the [API reference.](/reference/react) You can also write your own Hooks by combining the existing ones.

Hooks are more restrictive than other functions. You can only call Hooks *at the top* of your components (or other Hooks). If you want to use `useState` in a condition or a loop, extract a new component and put it there.

## Sharing data between components 

In the previous example, each `MyButton` had its own independent `count`, and when each button was clicked, only the `count` for the button clicked changed:

However, often you'll need components to *share data and always update together*.

To make both `MyButton` components display the same `count` and update together, you need to move the state from the individual buttons "upwards" to the closest component containing all of them.

In this example, it is `MyApp`:

Now when you click either button, the `count` in `MyApp` will change, which will change both of the counts in `MyButton`. Here's how you can express this in code.

First, *move the state up* from `MyButton` into `MyApp`:

```js 
export default function MyApp() {
  const [count, setCount] = useState(0);

  function handleClick() 

  return (
    
      Counters that update separately
      

## Next Steps 

By now, you know the basics of how to write React code!

Check out the [Tutorial](/learn/tutorial-tic-tac-toe) to put them into practice and build your first mini-app with React.
