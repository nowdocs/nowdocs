---
title: Updating Objects in State
---

## What's a mutation? 

You can store any kind of JavaScript value in state.

```js
const [x, setX] = useState(0);
```

So far you've been working with numbers, strings, and booleans. These kinds of JavaScript values are "immutable", meaning unchangeable or "read-only". You can trigger a re-render to _replace_ a value:

```js
setX(5);
```

The `x` state changed from `0` to `5`, but the _number `0` itself_ did not change. It's not possible to make any changes to the built-in primitive values like numbers, strings, and booleans in JavaScript.

Now consider an object in state:

```js
const [position, setPosition] = useState();
```

Technically, it is possible to change the contents of _the object itself_. **This is called a mutation:**

```js
position.x = 5;
```

However, although objects in React state are technically mutable, you should treat them **as if** they were immutable--like numbers, booleans, and strings. Instead of mutating them, you should always replace them.

## Treat state as read-only 

In other words, you should **treat any JavaScript object that you put into state as read-only.**

This example holds an object in state to represent the current pointer position. The red dot is supposed to move when you touch or move the cursor over the preview area. But the dot stays in the initial position:

The problem is with this bit of code.

```js
onPointerMove={e => }
```

This code modifies the object assigned to `position` from [the previous render.](/learn/state-as-a-snapshot#rendering-takes-a-snapshot-in-time) But without using the state setting function, React has no idea that object has changed. So React does not do anything in response. It's like trying to change the order after you've already eaten the meal. While mutating state can work in some cases, we don't recommend it. You should treat the state value you have access to in a render as read-only.

To actually [trigger a re-render](/learn/state-as-a-snapshot#setting-state-triggers-renders) in this case, **create a *new* object and pass it to the state setting function:**

```js
onPointerMove={e => {
  setPosition();
}}
```

With `setPosition`, you're telling React:

* Replace `position` with this new object
* And render this component again

Notice how the red dot now follows your pointer when you touch or hover over the preview area:

## Copying objects with the spread syntax 

In the previous example, the `position` object is always created fresh from the current cursor position. But often, you will want to include *existing* data as a part of the new object you're creating. For example, you may want to update *only one* field in a form, but keep the previous values for all other fields.

These input fields don't work because the `onChange` handlers mutate the state:

For example, this line mutates the state from a past render:

```js
person.firstName = e.target.value;
```

The reliable way to get the behavior you're looking for is to create a new object and pass it to `setPerson`. But here, you want to also **copy the existing data into it** because only one of the fields has changed:

```js
setPerson();
```

You can use the `...` [object spread](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Spread_syntax#spread_in_object_literals) syntax so that you don't need to copy every property separately.

```js
setPerson();
```

Now the form works!

Notice how you didn't declare a separate state variable for each input field. For large forms, keeping all data grouped in an object is very convenient--as long as you update it correctly!

Note that the `...` spread syntax is "shallow"--it only copies things one level deep. This makes it fast, but it also means that if you want to update a nested property, you'll have to use it more than once.

Here, `e.target.name` refers to the `name` property given to the `` DOM element.

## Updating a nested object 

Consider a nested object structure like this:

```js
const [person, setPerson] = useState({
  name: 'Niki de Saint Phalle',
  artwork: 
});
```

If you wanted to update `person.artwork.city`, it's clear how to do it with mutation:

```js
person.artwork.city = 'New Delhi';
```

But in React, you treat state as immutable! In order to change `city`, you would first need to produce the new `artwork` object (pre-populated with data from the previous one), and then produce the new `person` object which points at the new `artwork`:

```js
const nextArtwork = ;
const nextPerson = ;
setPerson(nextPerson);
```

Or, written as a single function call:

```js
setPerson({
  ...person, // Copy other fields
  artwork: 
});
```

This gets a bit wordy, but it works fine for many cases:

### Write concise update logic with Immer 

If your state is deeply nested, you might want to consider [flattening it.](/learn/choosing-the-state-structure#avoid-deeply-nested-state) But, if you don't want to change your state structure, you might prefer a shortcut to nested spreads. [Immer](https://github.com/immerjs/use-immer) is a popular library that lets you write using the convenient but mutating syntax and takes care of producing the copies for you. With Immer, the code you write looks like you are "breaking the rules" and mutating an object:

```js
updatePerson(draft => );
```

But unlike a regular mutation, it doesn't overwrite the past state!

To try Immer:

1. Run `npm install use-immer` to add Immer as a dependency
2. Then replace `import  from 'react'` with `import  from 'use-immer'`

Here is the above example converted to Immer:

Notice how much more concise the event handlers have become. You can mix and match `useState` and `useImmer` in a single component as much as you like. Immer is a great way to keep the update handlers concise, especially if there's nesting in your state, and copying objects leads to repetitive code.

The problem with `handlePlusClick` was that it mutated the `player` object. As a result, React did not know that there's a reason to re-render, and did not update the score on the screen. This is why, when you edited the first name, the state got updated, triggering a re-render which _also_ updated the score on the screen.

The problem with `handleLastNameChange` was that it did not copy the existing `...player` fields into the new object. This is why the score got lost after you edited the last name.

#### Find and fix the mutation 

There is a draggable box on a static background. You can change the box's color using the select input.

But there is a bug. If you move the box first, and then change its color, the background (which isn't supposed to move!) will "jump" to the box position. But this should not happen: the `Background`'s `position` prop is set to `initialPosition`, which is ``. Why is the background moving after the color change?

Find the bug and fix it.

    </>
  );
}
```

```js src/Box.js
import  from 'react';

export default function Box() {
  const [
    lastCoordinates,
    setLastCoordinates
  ] = useState(null);

  function handlePointerDown(e) {
    e.target.setPointerCapture(e.pointerId);
    setLastCoordinates();
  }

  function handlePointerMove(e) {
    if (lastCoordinates) {
      setLastCoordinates();
      const dx = e.clientX - lastCoordinates.x;
      const dy = e.clientY - lastCoordinates.y;
      onMove(dx, dy);
    }
  }

  function handlePointerUp(e) 

  return (
    
  );
}
```

```js src/Background.js
export default function Background() ;
```

```css
body 
select 
```

    </>
  );
}
```

```js src/Box.js
import  from 'react';

export default function Box() {
  const [
    lastCoordinates,
    setLastCoordinates
  ] = useState(null);

  function handlePointerDown(e) {
    e.target.setPointerCapture(e.pointerId);
    setLastCoordinates();
  }

  function handlePointerMove(e) {
    if (lastCoordinates) {
      setLastCoordinates();
      const dx = e.clientX - lastCoordinates.x;
      const dy = e.clientY - lastCoordinates.y;
      onMove(dx, dy);
    }
  }

  function handlePointerUp(e) 

  return (
    
  );
}
```

```js src/Background.js
export default function Background() ;
```

```css
body 
select 
```

#### Update an object with Immer 

This is the same buggy example as in the previous challenge. This time, fix the mutation by using Immer. For your convenience, `useImmer` is already imported, so you need to change the `shape` state variable to use it.

    </>
  );
}
```

```js src/Box.js
import  from 'react';

export default function Box() {
  const [
    lastCoordinates,
    setLastCoordinates
  ] = useState(null);

  function handlePointerDown(e) {
    e.target.setPointerCapture(e.pointerId);
    setLastCoordinates();
  }

  function handlePointerMove(e) {
    if (lastCoordinates) {
      setLastCoordinates();
      const dx = e.clientX - lastCoordinates.x;
      const dy = e.clientY - lastCoordinates.y;
      onMove(dx, dy);
    }
  }

  function handlePointerUp(e) 

  return (
    
  );
}
```

```js src/Background.js
export default function Background() ;
```

```css
body 
select 
```

```json package.json
{
  "dependencies": ,
  "scripts": 
}
```

    </>
  );
}
```

```js src/Box.js
import  from 'react';

export default function Box() {
  const [
    lastCoordinates,
    setLastCoordinates
  ] = useState(null);

  function handlePointerDown(e) {
    e.target.setPointerCapture(e.pointerId);
    setLastCoordinates();
  }

  function handlePointerMove(e) {
    if (lastCoordinates) {
      setLastCoordinates();
      const dx = e.clientX - lastCoordinates.x;
      const dy = e.clientY - lastCoordinates.y;
      onMove(dx, dy);
    }
  }

  function handlePointerUp(e) 

  return (
    
  );
}
```

```js src/Background.js
export default function Background() ;
```

```css
body 
select 
```

```json package.json
{
  "dependencies": ,
  "scripts": 
}
```

