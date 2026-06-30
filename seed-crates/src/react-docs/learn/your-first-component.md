---
title: Your First Component
---

## Components: UI building blocks 

On the Web, HTML lets us create rich structured documents with its built-in set of tags like `` and ``:

```html

  My First Component
  
    Components: UI Building Blocks
    Defining a Component
    Using a Component
  

```

This markup represents this article ``, its heading ``, and an (abbreviated) table of contents as an ordered list ``. Markup like this, combined with CSS for style, and JavaScript for interactivity, lies behind every sidebar, avatar, modal, dropdown—every piece of UI you see on the Web.

React lets you combine your markup, CSS, and JavaScript into custom "components", **reusable UI elements for your app.** The table of contents code you saw above could be turned into a `
  
  

```

As your project grows, you will notice that many of your designs can be composed by reusing components you already wrote, speeding up your development. Our table of contents above could be added to any screen with `

And here's how to build a component:

### Step 1: Export the component 

The `export default` prefix is a [standard JavaScript syntax](https://developer.mozilla.org/docs/web/javascript/reference/statements/export) (not specific to React). It lets you mark the main function in a file so that you can later import it from other files. (More on importing in [Importing and Exporting Components](/learn/importing-and-exporting-components)!)

### Step 2: Define the function 

With `function Profile() ` you define a JavaScript function with the name `Profile`.

### Step 3: Add markup 

The component returns an `` tag with `src` and `alt` attributes. `` is written like HTML, but it is actually JavaScript under the hood! This syntax is called [JSX](/learn/writing-markup-with-jsx), and it lets you embed markup inside JavaScript.

Return statements can be written all on one line, as in this component:

```js
return ;
```

But if your markup isn't all on the same line as the `return` keyword, you must wrap it in a pair of parentheses:

```js
return (
  
    
  
);
```

## Using a component 

Now that you've defined your `Profile` component, you can nest it inside other components. For example, you can export a `Gallery` component that uses multiple `Profile` components:

### What the browser sees 

Notice the difference in casing:

* `` is lowercase, so React knows we refer to an HTML tag.
* `

Try to fix it yourself before looking at the solution!

You might be wondering why writing `export` alone is not enough to fix this example. You can learn the difference between `export` and `export default` in [Importing and Exporting Components.](/learn/importing-and-exporting-components)

#### Fix the return statement 

Something isn't right about this `return` statement. Can you fix it?

Or by wrapping the returned JSX markup in parentheses that open right after `return`:

#### Spot the mistake 

Something's wrong with how the `Profile` component is declared and used. Can you spot the mistake? (Try to remember how React distinguishes components from the regular HTML tags!)

#### Your own component 

Write a component from scratch. You can give it any valid name and return any markup. If you're out of ideas, you can write a `Congratulations` component that shows `Good job!`. Don't forget to export it!

