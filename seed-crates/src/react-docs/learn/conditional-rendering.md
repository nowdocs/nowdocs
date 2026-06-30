---
title: Conditional Rendering
---

## Conditionally returning JSX 

Let’s say you have a `PackingList` component rendering several `Item`s, which can be marked as packed or not:

Notice that some of the `Item` components have their `isPacked` prop set to `true` instead of `false`. You want to add a checkmark (✅) to packed items if `isPacked=`.

You can write this as an [`if`/`else` statement](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/if...else) like so:

```js
if (isPacked) {
  return  ✅;
}
return ;
```

If the `isPacked` prop is `true`, this code **returns a different JSX tree.** With this change, some of the items get a checkmark at the end:

Try editing what gets returned in either case, and see how the result changes!

Notice how you're creating branching logic with JavaScript's `if` and `return` statements. In React, control flow (like conditions) is handled by JavaScript.

### Conditionally returning nothing with `null` 

In some situations, you won't want to render anything at all. For example, say you don't want to show packed items at all. A component must return something. In this case, you can return `null`:

```js
if (isPacked) 
return ;
```

If `isPacked` is true, the component will return nothing, `null`. Otherwise, it will return JSX to render.

In practice, returning `null` from a component isn't common because it might surprise a developer trying to render it. More often, you would conditionally include or exclude the component in the parent component's JSX. Here's how to do that!

## Conditionally including JSX 

In the previous example, you controlled which (if any!) JSX tree would be returned by the component. You may already have noticed some duplication in the render output:

```js
 ✅
```

is very similar to

```js

```

Both of the conditional branches return `...`:

```js
if (isPacked) {
  return  ✅;
}
return ;
```

While this duplication isn't harmful, it could make your code harder to maintain. What if you want to change the `className`? You'd have to do it in two places in your code! In such a situation, you could conditionally include a little JSX to make your code more [DRY.](https://en.wikipedia.org/wiki/Don%27t_repeat_yourself)

### Conditional (ternary) operator (`? :`) 

JavaScript has a compact syntax for writing a conditional expression -- the [conditional operator](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Conditional_Operator) or "ternary operator".

Instead of this:

```js
if (isPacked) {
  return  ✅;
}
return ;
```

You can write this:

```js
return (
  
    
  
);
```

You can read it as *"if `isPacked` is true, then (`?`) render `name + ' ✅'`, otherwise (`:`) render `name`"*.

Now let's say you want to wrap the completed item's text into another HTML tag, like `` to strike it out. You can add even more newlines and parentheses so that it's easier to nest more JSX in each of the cases:

This style works well for simple conditions, but use it in moderation. If your components get messy with too much nested conditional markup, consider extracting child components to clean things up. In React, markup is a part of your code, so you can use tools like variables and functions to tidy up complex expressions.

### Logical AND operator (`&&`) 

Another common shortcut you'll encounter is the [JavaScript logical AND (`&&`) operator.](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Logical_AND#:~:text=The%20logical%20AND%20(%20%26%26%20)%20operator,it%20returns%20a%20Boolean%20value.) Inside React components, it often comes up when you want to render some JSX when the condition is true, **or render nothing otherwise.** With `&&`, you could conditionally render the checkmark only if `isPacked` is `true`:

```js
return (
  
     
  
);
```

You can read this as *"if `isPacked`, then (`&&`) render the checkmark, otherwise, render nothing"*.

Here it is in action:

A [JavaScript && expression](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Logical_AND) returns the value of its right side (in our case, the checkmark) if the left side (our condition) is `true`. But if the condition is `false`, the whole expression becomes `false`. React considers `false` as a "hole" in the JSX tree, just like `null` or `undefined`, and doesn't render anything in its place.

### Conditionally assigning JSX to a variable 

When the shortcuts get in the way of writing plain code, try using an `if` statement and a variable. You can reassign variables defined with [`let`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/let), so start by providing the default content you want to display, the name:

```js
let itemContent = name;
```

Use an `if` statement to reassign a JSX expression to `itemContent` if `isPacked` is `true`:

```js
if (isPacked) 
```

[Curly braces open the "window into JavaScript".](/learn/javascript-in-jsx-with-curly-braces#using-curly-braces-a-window-into-the-javascript-world) Embed the variable with curly braces in the returned JSX tree, nesting the previously calculated expression inside of JSX:

```js

  

```

This style is the most verbose, but it's also the most flexible. Here it is in action:

Like before, this works not only for text, but for arbitrary JSX too:

If you're not familiar with JavaScript, this variety of styles might seem overwhelming at first. However, learning them will help you read and write any JavaScript code -- and not just React components! Pick the one you prefer for a start, and then consult this reference again if you forget how the other ones work.

#### Show the item importance with `&&` 

In this example, each `Item` receives a numerical `importance` prop. Use the `&&` operator to render "_(Importance: X)_" in italics, but only for items that have non-zero importance. Your item list should end up looking like this:

* Space suit _(Importance: 9)_
* Helmet with a golden leaf
* Photo of Tam _(Importance: 6)_

Don't forget to add a space between the two labels!

Note that you must write `importance > 0 && ...` rather than `importance && ...` so that if the `importance` is `0`, `0` isn't rendered as the result!

In this solution, two separate conditions are used to insert a space between the name and the importance label. Alternatively, you could use a Fragment with a leading space: `importance > 0 && <> ...</>` or add a space immediately inside the ``:  `importance > 0 &&  ...`.

#### Refactor a series of `? :` to `if` and variables 

This `Drink` component uses a series of `? :` conditions to show different information depending on whether the `name` prop is `"tea"` or `"coffee"`. The problem is that the information about each drink is spread across multiple conditions. Refactor this code to use a single `if` statement instead of three `? :` conditions.

Once you've refactored the code to use `if`, do you have further ideas on how to simplify it?

Here the information about each drink is grouped together instead of being spread across multiple conditions. This makes it easier to add more drinks in the future.

Another solution would be to remove the condition altogether by moving the information into objects:

