---
title: Rendering Lists
---

## Rendering data from arrays 

Say that you have a list of content.

```js

  Creola Katherine Johnson: mathematician
  Mario José Molina-Pasquel Henríquez: chemist
  Mohammad Abdus Salam: physicist
  Percy Lavon Julian: chemist
  Subrahmanyan Chandrasekhar: astrophysicist

```

The only difference among those list items is their contents, their data. You will often need to show several instances of the same component using different data when building interfaces: from lists of comments to galleries of profile images. In these situations, you can store that data in JavaScript objects and arrays and use methods like [`map()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/map) and [`filter()`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Array/filter) to render lists of components from them.

Here’s a short example of how to generate a list of items from an array:

1. **Move** the data into an array:

```js
const people = [
  'Creola Katherine Johnson: mathematician',
  'Mario José Molina-Pasquel Henríquez: chemist',
  'Mohammad Abdus Salam: physicist',
  'Percy Lavon Julian: chemist',
  'Subrahmanyan Chandrasekhar: astrophysicist'
];
```

2. **Map** the `people` members into a new array of JSX nodes, `listItems`:

```js
const listItems = people.map(person => );
```

3. **Return** `listItems` from your component wrapped in a ``:

```js
return ;
```

Here is the result:

Notice the sandbox above displays a console error:

You'll learn how to fix this error later on this page. Before we get to that, let's add some structure to your data.

## Filtering arrays of items 

This data can be structured even more.

```js
const people = [, , , , ];
```

Let's say you want a way to only show people whose profession is `'chemist'`. You can use JavaScript's `filter()` method to return just those people. This method takes an array of items, passes them through a “test” (a function that returns `true` or `false`), and returns a new array of only those items that passed the test (returned `true`).

You only want the items where `profession` is `'chemist'`. The "test" function for this looks like `(person) => person.profession === 'chemist'`. Here's how to put it together:

1. **Create** a new array of just “chemist” people, `chemists`, by calling `filter()` on the `people` filtering by `person.profession === 'chemist'`:

```js
const chemists = people.filter(person =>
  person.profession === 'chemist'
);
```

2. Now **map** over `chemists`:

```js 
const listItems = chemists.map(person =>
  
     
     
       :
       
       known for 
     
  
);
```

3. Lastly, **return** the `listItems` from your component:

```js
return ;
```

## Keeping list items in order with `key` 

Notice that all the sandboxes above show an error in the console:

You need to give each array item a `key` -- a string or a number that uniquely identifies it among other items in that array:

```js
...
```

Keys tell React which array item each component corresponds to, so that it can match them up later. This becomes important if your array items can move (e.g. due to sorting), get inserted, or get deleted. A well-chosen `key` helps React infer what exactly has happened, and make the correct updates to the DOM tree.

Rather than generating keys on the fly, you should include them in your data:

);
```

Fragments disappear from the DOM, so this will produce a flat list of ``, ``, ``, ``, and so on.

### Where to get your `key` 

Different sources of data provide different sources of keys:

* **Data from a database:** If your data is coming from a database, you can use the database keys/IDs, which are unique by nature.
* **Locally generated data:** If your data is generated and persisted locally (e.g. notes in a note-taking app), use an incrementing counter, [`crypto.randomUUID()`](https://developer.mozilla.org/en-US/docs/Web/API/Crypto/randomUUID) or a package like [`uuid`](https://www.npmjs.com/package/uuid) when creating items.

### Rules of keys 

* **Keys must be unique among siblings.** However, it’s okay to use the same keys for JSX nodes in _different_ arrays.
* **Keys must not change** or that defeats their purpose! Don't generate them while rendering.

### Why does React need keys? 

Imagine that files on your desktop didn't have names. Instead, you'd refer to them by their order -- the first file, the second file, and so on. You could get used to it, but once you delete a file, it would get confusing. The second file would become the first file, the third file would be the second file, and so on.

File names in a folder and JSX keys in an array serve a similar purpose. They let us uniquely identify an item between its siblings. A well-chosen key provides more information than the position within the array. Even if the _position_ changes due to reordering, the `key` lets React identify the item throughout its lifetime.

In this solution, the `map` calls are placed directly inline into the parent `` elements, but you could introduce variables for them if you find that more readable.

There is still a bit duplication between the rendered lists. You can go further and extract the repetitive parts into a `

A very attentive reader might notice that with two `filter` calls, we check each person's profession twice. Checking a property is very fast, so in this example it's fine. If your logic was more expensive than that, you could replace the `filter` calls with a loop that manually constructs the arrays and checks each person once.

In fact, if `people` never change, you could move this code out of your component. From React's perspective, all that matters is that you give it an array of JSX nodes in the end. It doesn't care how you produce that array:

#### Nested lists in one component 

Make a list of recipes from this array! For each recipe in the array, display its name as an `` and list its ingredients in a ``.

Each of the `recipes` already includes an `id` field, so that's what the outer loop uses for its `key`. There is no ID you could use to loop over ingredients. However, it's reasonable to assume that the same ingredient won't be listed twice within the same recipe, so its name can serve as a `key`. Alternatively, you could change the data structure to add IDs, or use index as a `key` (with the caveat that you can't safely reorder ingredients).

#### Extracting a list item component 

This `RecipeList` component contains two nested `map` calls. To simplify it, extract a `Recipe` component from it which will accept `id`, `name`, and `ingredients` props. Where do you place the outer `key` and why?

Here, `

#### List with a separator 

This example renders a famous haiku by Tachibana Hokushi, with each line wrapped in a `` tag. Your job is to insert an `` separator between each paragraph. Your resulting structure should look like this:

```js

  I write, erase, rewrite
  
  Erase again, and then
  
  A poppy blooms.

```

A haiku only contains three lines, but your solution should work with any number of lines. Note that `` elements only appear *between* the `` elements, not in the beginning or the end!

(This is a rare case where index as a key is acceptable because a poem's lines will never reorder.)

Using the original line index as a `key` doesn't work anymore because each separator and paragraph are now in the same array. However, you can give each of them a distinct key using a suffix, e.g. `key=`.

Alternatively, you could render a collection of Fragments which contain `` and `...`. However, the `<>...</>` shorthand syntax doesn't support passing keys, so you'd have to write `
      )}
    
  );
}
```

```css
body 
p 
hr 
```

Remember, Fragments (often written as `<> </>`) let you group JSX nodes without adding extra ``s!

