---
title: Sharing State Between Components
---

## Lifting state up by example 

In this example, a parent `Accordion` component renders two separate `Panel`s:

* `Accordion`
  - `Panel`
  - `Panel`

Each `Panel` component has a boolean `isActive` state that determines whether its content is visible.

Press the Show button for both panels:

      
    </>
  );
}
```

```css
h3, p 
.panel 
```

Notice how pressing one panel's button does not affect the other panel--they are independent.

**But now let's say you want to change it so that only one panel is expanded at any given time.** With that design, expanding the second panel should collapse the first one. How would you do that?

To coordinate these two panels, you need to "lift their state up" to a parent component in three steps:

1. **Remove** state from the child components.
2. **Pass** hardcoded data from the common parent.
3. **Add** state to the common parent and pass it down together with the event handlers.

This will allow the `Accordion` component to coordinate both `Panel`s and only expand one at a time.

### Step 1: Remove state from the child components 

You will give control of the `Panel`'s `isActive` to its parent component. This means that the parent component will pass `isActive` to `Panel` as a prop instead. Start by **removing this line** from the `Panel` component:

```js
const [isActive, setIsActive] = useState(false);
```

And instead, add `isActive` to the `Panel`'s list of props:

```js
function Panel() {
```

Now the `Panel`'s parent component can *control* `isActive` by [passing it down as a prop.](/learn/passing-props-to-a-component) Conversely, the `Panel` component now has *no control* over the value of `isActive`--it's now up to the parent component!

### Step 2: Pass hardcoded data from the common parent 

To lift state up, you must locate the closest common parent component of *both* of the child components that you want to coordinate:

* `Accordion` *(closest common parent)*
  - `Panel`
  - `Panel`

In this example, it's the `Accordion` component. Since it's above both panels and can control their props, it will become the "source of truth" for which panel is currently active. Make the `Accordion` component pass a hardcoded value of `isActive` (for example, `true`) to both panels:

      
    </>
  );
}

function Panel() {
  return (
    
      
      {isActive ? (
        
      ) : (
         setIsActive(true)}>
          Show
        
      )}
    
  );
}
```

```css
h3, p 
.panel 
```

Try editing the hardcoded `isActive` values in the `Accordion` component and see the result on the screen.

### Step 3: Add state to the common parent 

Lifting state up often changes the nature of what you're storing as state.

In this case, only one panel should be active at a time. This means that the `Accordion` common parent component needs to keep track of *which* panel is the active one. Instead of a `boolean` value, it could use a number as the index of the active `Panel` for the state variable:

```js
const [activeIndex, setActiveIndex] = useState(0);
```

When the `activeIndex` is `0`, the first panel is active, and when it's `1`, it's the second one.

Clicking the "Show" button in either `Panel` needs to change the active index in `Accordion`. A `Panel` can't set the `activeIndex` state directly because it's defined inside the `Accordion`. The `Accordion` component needs to *explicitly allow* the `Panel` component to change its state by [passing an event handler down as a prop](/learn/responding-to-events#passing-event-handlers-as-props):

```js
<>
  
  
</>
```

The `` inside the `Panel` will now use the `onShow` prop as its click event handler:

      
    </>
  );
}

function Panel() {
  return (
    
      
      {isActive ? (
        
      ) : (
        
          Show
        
      )}
    
  );
}
```

```css
h3, p 
.panel 
```

This completes lifting state up! Moving state into the common parent component allowed you to coordinate the two panels. Using the active index instead of two "is shown" flags ensured that only one panel is active at a given time. And passing down the event handler to the child allowed the child to change the parent's state.

## A single source of truth for each state 

In a React application, many components will have their own state. Some state may "live" close to the leaf components (components at the bottom of the tree) like inputs. Other state may "live" closer to the top of the app. For example, even client-side routing libraries are usually implemented by storing the current route in the React state, and passing it down by props!

**For each unique piece of state, you will choose the component that "owns" it.** This principle is also known as having a ["single source of truth".](https://en.wikipedia.org/wiki/Single_source_of_truth) It doesn't mean that all state lives in one place--but that for _each_ piece of state, there is a _specific_ component that holds that piece of information. Instead of duplicating shared state between components, *lift it up* to their common shared parent, and *pass it down* to the children that need it.

Your app will change as you work on it. It is common that you will move state down or back up while you're still figuring out where each piece of the state "lives". This is all part of the process!

To see what this feels like in practice with a few more components, read [Thinking in React.](/learn/thinking-in-react)

#### Filtering a list 

In this example, the `SearchBar` has its own `query` state that controls the text input. Its parent `FilterableList` component displays a `List` of items, but it doesn't take the search query into account.

Use the `filterItems(foods, query)` function to filter the list according to the search query. To test your changes, verify that typing "s" into the input filters down the list to "Sushi", "Shish kebab", and "Dim sum".

Note that `filterItems` is already implemented and imported so you don't need to write it yourself!

