---
title: JavaScript in JSX with Curly Braces
---

## Passing strings with quotes 

When you want to pass a string attribute to JSX, you put it in single or double quotes:

Here, `"https://react.dev/images/docs/scientists/7vQD0fPs.jpg"` and `"Gregorio Y. Zara"` are being passed as strings.

But what if you want to dynamically specify the `src` or `alt` text? You could **use a value from JavaScript by replacing `"` and `"` with ``**:

Notice the difference between `className="avatar"`, which specifies an `"avatar"` CSS class name that makes the image round, and `src=` that reads the value of the JavaScript variable called `avatar`. That's because curly braces let you work with JavaScript right there in your markup!

## Using curly braces: A window into the JavaScript world 

JSX is a special way of writing JavaScript. That means it’s possible to use JavaScript inside it—with curly braces ``. The example below first declares a name for the scientist, `name`, then embeds it with curly braces inside the ``:

Try changing the `name`'s value from `'Gregorio Y. Zara'` to `'Hedy Lamarr'`. See how the list title changes?

Any JavaScript expression will work between curly braces, including function calls like `formatDate()`:

### Where to use curly braces 

You can only use curly braces in two ways inside JSX:

1. **As text** directly inside a JSX tag: `'s To Do List` works, but `<>Gregorio Y. Zara's To Do List</>`  will not.
2. **As attributes** immediately following the `=` sign: `src=` will read the `avatar` variable, but `src=""` will pass the string `""`.

## Using "double curlies": CSS and other objects in JSX 

In addition to strings, numbers, and other JavaScript expressions, you can even pass objects in JSX. Objects are also denoted with curly braces, like ``. Therefore, to pass a JS object in JSX, you must wrap the object in another pair of curly braces: `person={}`.

You may see this with inline CSS styles in JSX. React does not require you to use inline styles (CSS classes work great for most cases). But when you need an inline style, you pass an object to the `style` attribute:

Try changing the values of `backgroundColor` and `color`.

You can really see the JavaScript object inside the curly braces when you write it like this:

```js 

```

The next time you see `` in JSX, know that it's nothing more than an object inside the JSX curlies!

## More fun with JavaScript objects and curly braces 

You can move several expressions into one object, and reference them in your JSX inside curly braces:

In this example, the `person` JavaScript object contains a `name` string and a `theme` object:

```js
const person = {
  name: 'Gregorio Y. Zara',
  theme: 
};
```

The component can use these values from `person` like so:

```js

  's Todos
```

JSX is very minimal as a templating language because it lets you organize data and logic using JavaScript.

Can you find the problem?

#### Extract information into an object 

Extract the image URL into the `person` object.

#### Write an expression inside JSX curly braces 

In the object below, the full image URL is split into four parts: base URL, `imageId`, `imageSize`, and file extension.

We want the image URL to combine these attributes together: base URL (always `'https://react.dev/images/docs/scientists/'`), `imageId` (`'7vQD0fP'`), `imageSize` (`'s'`), and file extension (always `'.jpg'`). However, something is wrong with how the `` tag specifies its `src`.

Can you fix it?

To check that your fix worked, try changing the value of `imageSize` to `'b'`. The image should resize after your edit.

You can also move this expression into a separate function like `getImageUrl` below:

Variables and functions can help you keep the markup simple!

