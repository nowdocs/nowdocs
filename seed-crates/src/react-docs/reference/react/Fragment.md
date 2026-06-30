---
title: 

```js
<>
  

` in most cases.

#### Props 

- **optional** `key`: Fragments declared with the explicit ``.

* React does not [reset state](/learn/preserving-and-resetting-state) when you go from rendering `<>`.

---

### 

  );
}
```

Usually you won't need this unless you need to [pass a `key` to your `Fragment`.](#rendering-a-list-of-fragments)

---

### Assigning multiple elements to a variable 

Like any other element, you can assign Fragment elements to variables, pass them as props, and so on:

```js
function CloseDialog() 
```

---

### Grouping elements with text 

You can use `Fragment` to group text together with components:

```js
function DateRangePicker() 
```

You can inspect the DOM to verify that there are no wrapper elements around the Fragment children:

  );
}

function PostTitle() {
  return 
}

function PostBody() {
  return (
    
      
    
  );
}
```

---

### 
  );
}

export default function App() {
  const [clicks, setClicks] = useState(0);

  return (
    <>
      Total clicks: 
      
    </>
  );
}
```

```json package.json hidden
{
  "dependencies": 
}
```

The `addEventListener` call applies the listener to every first-level DOM child of the Fragment. When children are dynamically added or removed, the `FragmentInstance` automatically adds or removes the listener.

  

```

`Wrapper` is a React component, so the `FragmentInstance` looks through it to find DOM nodes. The targeted children are `A`, `B`, and `D`. `C` is not targeted because it is nested inside the DOM element `B`.

Methods like `addEventListener`, `observeUsing`, and `getClientRects` operate on these first-level DOM children. `focus` and `focusLast` are different—they search *all* nested children depth-first to find focusable elements.

---

### 
    </>
  );
}

// Even though the inputs are deeply nested,
// focus() searches depth-first to find them.
export default function App() 
```

```css
.buttons 

label 
```

```json package.json hidden
{
  "dependencies": 
}
```

Calling `focus()` focuses the `street` input—even though it is nested inside a `` and ``. `focus()` searches depth-first through all nested children, not just direct children of the Fragment. `focusLast()` does the same in reverse, and `blur()` removes focus if the currently focused element is within the Fragment.

---

### 
      
    </>
  );
}

const items = [];
for (let i = 1; i <= 25; i++) 

export default function App() 
```

```css
.buttons 

.container 

h3 

p 
```

```json package.json hidden
{
  "dependencies": 
}
```

---

### 
  );
}

export default function App() 
```

```css
.page 

.page.visible 

.filler 

.card 
```

```js src/Card.js hidden
export default function Card() {
  return ;
}
```

```json package.json hidden
{
  "dependencies": 
}
```

---

### 
      
      
      Scroll up
    
  );
}
```

```js src/ObservedGroup.js
import  from 'react';

const callbackMap = new WeakMap();
const observerCache = new Map();

function getOptionsKey(options) {
  const root = options?.root ?? null;
  const rootMargin = options?.rootMargin ?? '0px';
  const threshold = options?.threshold ?? 0;
  return `$|$`;
}

function getSharedObserver(
  fragmentInstance,
  onIntersection,
  options,
) {
  // Register this callback for the
  // fragment instance.
  const existing =
    callbackMap.get(fragmentInstance);
  callbackMap.set(
    fragmentInstance,
    existing
      ? [...existing, onIntersection]
      : [onIntersection],
  );

  const key = getOptionsKey(options);
  if (observerCache.has(key)) 

  const observer = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        // Look up which FragmentInstances own
        // this element.
        const fragmentInstances =
          entry.target.reactFragments;
        if (fragmentInstances) {
          for (const inst of fragmentInstances) 
        }
      }
    },
    options,
  );

  observerCache.set(key, observer);
  return observer;
}

export default function ObservedGroup() {
  const fragmentRef = useRef(null);

  useLayoutEffect(() => {
    const fragmentInstance = fragmentRef.current;
    const observer = getSharedObserver(
      fragmentInstance,
      onIntersection,
      options,
    );
    fragmentInstance.observeUsing(observer);
    return () => ;
  }, [onIntersection, options]);

  return (
    
  );
}
```

```css
.page 

.filler 

.card 

.card.green 

.card.blue 
```

```js src/Card.js hidden
export default function Card() {
  return ;
}
```

```json package.json hidden
{
  "dependencies": 
}
```

Multiple `ObservedGroup` components with the same options reuse a single `IntersectionObserver`. When either section scrolls into view, the shared observer fires and uses `reactFragments` to route the entry to the correct callback.
