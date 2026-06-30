---
title: Updating Arrays in State
---

## Updating arrays without mutation 

In JavaScript, arrays are just another kind of object. [Like with objects](/learn/updating-objects-in-state), **you should treat arrays in React state as read-only.** This means that you shouldn't reassign items inside an array like `arr[0] = 'bird'`, and you also shouldn't use methods that mutate the array, such as `push()` and `pop()`.

Instead, every time you want to update an array, you'll want to pass a *new* array to your state setting function. To do that, you can create a new array from the original array in your state by calling its non-mutating methods like `filter()` and `map()`. Then you can set your state to the resulting new array.

Here is a reference table of common array operations. When dealing with arrays inside React state, you will need to avoid the methods in the left column, and instead prefer the methods in the right column:

|           | avoid (mutates the array)           | prefer (returns a new array)                                        |
| --------- | ----------------------------------- | ------------------------------------------------------------------- |
| adding    | `push`, `unshift`                   | `concat`, `[...arr]` spread syntax ([example](#adding-to-an-array)) |
| removing  | `pop`, `shift`, `splice`            | `filter`, `slice` ([example](#removing-from-an-array))              |
| replacing | `splice`, `arr[i] = ...` assignment | `map` ([example](#replacing-items-in-an-array))                     |
| sorting   | `reverse`, `sort`                   | copy the array first ([example](#making-other-changes-to-an-array)) |

Alternatively, you can [use Immer](#write-concise-update-logic-with-immer) which lets you use methods from both columns.

### Adding to an array 

`push()` will mutate an array, which you don't want:

Instead, create a *new* array which contains the existing items *and* a new item at the end. There are multiple ways to do this, but the easiest one is to use the `...` [array spread](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Spread_syntax#spread_in_array_literals) syntax:

```js
setArtists( // Replace the state
  [ // with a new array
    ...artists, // that contains all the old items
     // and one new item at the end
  ]
);
```

Now it works correctly:

The array spread syntax also lets you prepend an item by placing it *before* the original `...artists`:

```js
setArtists([
  ,
  ...artists // Put old items at the end
]);
```

In this way, spread can do the job of both `push()` by adding to the end of an array and `unshift()` by adding to the beginning of an array. Try it in the sandbox above!

### Removing from an array 

The easiest way to remove an item from an array is to *filter it out*. In other words, you will produce a new array that will not contain that item. To do this, use the `filter` method, for example:

Click the "Delete" button a few times, and look at its click handler.

```js
setArtists(
  artists.filter(a => a.id !== artist.id)
);
```

Here, `artists.filter(a => a.id !== artist.id)` means "create an array that consists of those `artists` whose IDs are different from `artist.id`". In other words, each artist's "Delete" button will filter _that_ artist out of the array, and then request a re-render with the resulting array. Note that `filter` does not modify the original array.

### Transforming an array 

If you want to change some or all items of the array, you can use `map()` to create a **new** array. The function you will pass to `map` can decide what to do with each item, based on its data or its index (or both).

In this example, an array holds coordinates of two circles and a square. When you press the button, it moves only the circles down by 50 pixels. It does this by producing a new array of data using `map()`:

### Replacing items in an array 

It is particularly common to want to replace one or more items in an array. Assignments like `arr[0] = 'bird'` are mutating the original array, so instead you'll want to use `map` for this as well.

To replace an item, create a new array with `map`. Inside your `map` call, you will receive the item index as the second argument. Use it to decide whether to return the original item (the first argument) or something else:

### Inserting into an array 

Sometimes, you may want to insert an item at a particular position that's neither at the beginning nor at the end. To do this, you can use the `...` array spread syntax together with the `slice()` method. The `slice()` method lets you cut a "slice" of the array. To insert an item, you will create an array that spreads the slice _before_ the insertion point, then the new item, and then the rest of the original array.

In this example, the Insert button always inserts at the index `1`:

### Making other changes to an array 

There are some things you can't do with the spread syntax and non-mutating methods like `map()` and `filter()` alone. For example, you may want to reverse or sort an array. The JavaScript `reverse()` and `sort()` methods are mutating the original array, so you can't use them directly.

**However, you can copy the array first, and then make changes to it.**

For example:

Here, you use the `[...list]` spread syntax to create a copy of the original array first. Now that you have a copy, you can use mutating methods like `nextList.reverse()` or `nextList.sort()`, or even assign individual items with `nextList[0] = "something"`.

However, **even if you copy an array, you can't mutate existing items _inside_ of it directly.** This is because copying is shallow--the new array will contain the same items as the original one. So if you modify an object inside the copied array, you are mutating the existing state. For example, code like this is a problem.

```js
const nextList = [...list];
nextList[0].seen = true; // Problem: mutates list[0]
setList(nextList);
```

Although `nextList` and `list` are two different arrays, **`nextList[0]` and `list[0]` point to the same object.** So by changing `nextList[0].seen`, you are also changing `list[0].seen`. This is a state mutation, which you should avoid! You can solve this issue in a similar way to [updating nested JavaScript objects](/learn/updating-objects-in-state#updating-a-nested-object)--by copying individual items you want to change instead of mutating them. Here's how.

## Updating objects inside arrays 

Objects are not _really_ located "inside" arrays. They might appear to be "inside" in code, but each object in an array is a separate value, to which the array "points". This is why you need to be careful when changing nested fields like `list[0]`. Another person's artwork list may point to the same element of the array!

**When updating nested state, you need to create copies from the point where you want to update, and all the way up to the top level.** Let's see how this works.

In this example, two separate artwork lists have the same initial state. They are supposed to be isolated, but because of a mutation, their state is accidentally shared, and checking a box in one list affects the other list:

The problem is in code like this:

```js
const myNextList = [...myList];
const artwork = myNextList.find(a => a.id === artworkId);
artwork.seen = nextSeen; // Problem: mutates an existing item
setMyList(myNextList);
```

Although the `myNextList` array itself is new, the *items themselves* are the same as in the original `myList` array. So changing `artwork.seen` changes the *original* artwork item. That artwork item is also in `yourList`, which causes the bug. Bugs like this can be difficult to think about, but thankfully they disappear if you avoid mutating state.

**You can use `map` to substitute an old item with its updated version without mutation.**

```js
setMyList(myList.map(artwork => {
  if (artwork.id === artworkId) {
    // Create a *new* object with changes
    return ;
  } else 
}));
```

Here, `...` is the object spread syntax used to [create a copy of an object.](/learn/updating-objects-in-state#copying-objects-with-the-spread-syntax)

With this approach, none of the existing state items are being mutated, and the bug is fixed:

In general, **you should only mutate objects that you have just created.** If you were inserting a *new* artwork, you could mutate it, but if you're dealing with something that's already in state, you need to make a copy.

### Write concise update logic with Immer 

Updating nested arrays without mutation can get a little bit repetitive. [Just as with objects](/learn/updating-objects-in-state#write-concise-update-logic-with-immer):

- Generally, you shouldn't need to update state more than a couple of levels deep. If your state objects are very deep, you might want to [restructure them differently](/learn/choosing-the-state-structure#avoid-deeply-nested-state) so that they are flat.
- If you don't want to change your state structure, you might prefer to use [Immer](https://github.com/immerjs/use-immer), which lets you write using the convenient but mutating syntax and takes care of producing the copies for you.

Here is the Art Bucket List example rewritten with Immer:

Note how with Immer, **mutation like `artwork.seen = nextSeen` is now okay:**

```js
updateMyTodos(draft => );
```

This is because you're not mutating the _original_ state, but you're mutating a special `draft` object provided by Immer. Similarly, you can apply mutating methods like `push()` and `pop()` to the content of the `draft`.

Behind the scenes, Immer always constructs the next state from scratch according to the changes that you've done to the `draft`. This keeps your event handlers very concise without ever mutating state.

#### Remove an item from the shopping cart 

This shopping cart has a working "+" button, but the "–" button doesn't do anything. You need to add an event handler to it so that pressing it decreases the `count` of the corresponding product. If you press "–" when the count is 1, the product should automatically get removed from the cart. Make sure it never shows 0.

#### Fix the mutations using non-mutative methods 

In this example, all of the event handlers in `App.js` use mutation. As a result, editing and deleting todos doesn't work. Rewrite `handleAddTodo`, `handleChangeTodo`, and `handleDeleteTodo` to use the non-mutative methods:

#### Fix the mutations using Immer 

This is the same example as in the previous challenge. This time, fix the mutations by using Immer. For your convenience, `useImmer` is already imported, so you need to change the `todos` state variable to use it.

You can also mix and match the mutative and non-mutative approaches with Immer.

For example, in this version `handleAddTodo` is implemented by mutating the Immer `draft`, while `handleChangeTodo` and `handleDeleteTodo` use the non-mutative `map` and `filter` methods:

With Immer, you can pick the style that feels the most natural for each separate case.

