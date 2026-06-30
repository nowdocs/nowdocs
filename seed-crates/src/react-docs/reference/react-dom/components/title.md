---
title: ""
---

---

## Usage 

### Set the document title 

Render the `` component from any component with text as its children. React will put a `` DOM node in the document ``.

  );
}
```

### Use variables in the title 

The children of the `` component must be a single string of text. (Or a single number or a single object with a `toString` method.) It might not be obvious, but using JSX curly braces like this:

```js
Results page  // 🔴 Problem: This is not a single string
```

... actually causes the `` component to get a two-element array as its children (the string `"Results page"` and the value of `pageNumber`). This will cause an error. Instead, use string interpolation to pass `` a single string:

```js

```

