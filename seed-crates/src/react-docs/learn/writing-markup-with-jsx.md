---
title: Writing Markup with JSX
---

## JSX: Putting markup into JavaScript 

The Web has been built on HTML, CSS, and JavaScript. For many years, web developers kept content in HTML, design in CSS, and logic in JavaScript—often in separate files! Content was marked up inside HTML while the page's logic lived separately in JavaScript:

But as the Web became more interactive, logic increasingly determined content. JavaScript was in charge of the HTML! This is why **in React, rendering logic and markup live together in the same place—components.**

Keeping a button's rendering logic and markup together ensures that they stay in sync with each other on every edit. Conversely, details that are unrelated, such as the button's markup and a sidebar's markup, are isolated from each other, making it safer to change either of them on their own.

Each React component is a JavaScript function that may contain some markup that React renders into the browser. React components use a syntax extension called JSX to represent that markup. JSX looks a lot like HTML, but it is a bit stricter and can display dynamic information. The best way to understand this is to convert some HTML markup to JSX markup.

## Converting HTML to JSX 

Suppose that you have some (perfectly valid) HTML:

```html
Hedy Lamarr's Todos

    Invent new traffic lights
    Rehearse a movie scene
    Improve the spectrum technology

```

And you want to put it into your component:

```js
export default function TodoList() 
```

If you copy and paste it as is, it will not work:

This is because JSX is stricter and has a few more rules than HTML! If you read the error messages above, they'll guide you to fix the markup, or you can follow the guide below.

## The Rules of JSX 

### 1. Return a single root element 

To return multiple elements from a component, **wrap them with a single parent tag.**

For example, you can use a ``:

```js 

  Hedy Lamarr's Todos
  
  
    ...
  

```

If you don't want to add an extra `` to your markup, you can write `<>` and `</>` instead:

```js 
<>
  Hedy Lamarr's Todos
  
  
    ...
  
</>
```

This empty tag is called a *[Fragment.](/reference/react/Fragment)* Fragments let you group things without leaving any trace in the browser HTML tree.

### 2. Close all the tags 

JSX requires tags to be explicitly closed: self-closing tags like `` must become ``, and wrapping tags like `oranges` must be written as `oranges`.

This is how Hedy Lamarr's image and list items look closed:

```js 
<>
  
  
    Invent new traffic lights
    Rehearse a movie scene
    Improve the spectrum technology
  
</>
```

### 3. camelCase all most of the things! 

JSX turns into JavaScript and attributes written in JSX become keys of JavaScript objects. In your own components, you will often want to read those attributes into variables. But JavaScript has limitations on variable names. For example, their names can't contain dashes or be reserved words like `class`.

This is why, in React, many HTML and SVG attributes are written in camelCase. For example, instead of `stroke-width` you use `strokeWidth`. Since `class` is a reserved word, in React you write `className` instead, named after the [corresponding DOM property](https://developer.mozilla.org/en-US/docs/Web/API/Element/className):

```js 

```

You can [find all these attributes in the list of DOM component props.](/reference/react-dom/components/common) If you get one wrong, don't worry—React will print a message with a possible correction to the [browser console.](https://developer.mozilla.org/docs/Tools/Browser_Console)

### Pro-tip: Use a JSX Converter 

Converting all these attributes in existing markup can be tedious! We recommend using a [converter](https://transform.tools/html-to-jsx) to translate your existing HTML and SVG to JSX. Converters are very useful in practice, but it's still worth understanding what is going on so that you can comfortably write JSX on your own.

Here is your final result:

Whether to do it by hand or using the converter is up to you!

