---
title: 

`
```

```

[See more examples below.](#usage)

` then React adds a suffix to the name to make each unique but conceptually they're part of the same one. React doesn't apply these eagerly but only at the time that boundary should participate in an animation.

React automatically calls `startViewTransition` itself behind the scenes so you should never do that yourself. In fact, if you have something else on the page running a ViewTransition React will interrupt it. So it's recommended that you use React itself to coordinate these. If you had other ways to trigger ViewTransitions in the past, we recommend that you migrate to the built-in way.

If there are other React ViewTransitions already running then React will wait for them to finish before starting the next one. However, importantly if there are multiple updates happening while the first one is running, those will all be batched into one. If you start A->B. Then in the meantime you get an update to go to C and then D. When the first A->B animation finishes the next one will animate from B->D.

The `getSnapshotBeforeUpdate` lifecycle will be called before `startViewTransition` and some `view-transition-name` will update at the same time.

Then React calls `startViewTransition`. Inside the `updateCallback`, React will:

- Apply its mutations to the DOM and invoke `useInsertionEffect`.
- Wait for fonts to load.
- Call `componentDidMount`, `componentDidUpdate`, `useLayoutEffect` and refs.
- Wait for any pending Navigation to finish.
- Then React will measure any changes to the layout to see which boundaries will need to animate.

After the ready Promise of the `startViewTransition` is resolved, React will then revert the `view-transition-name`. Then React will invoke the `onEnter`, `onExit`, `onUpdate` and `onShare` callbacks to allow for manual programmatic control over the animations. This will be after the built-in default ones have already been computed.

If a `flushSync` happens to get in the middle of this sequence, then React will skip the Transition since it relies on being able to complete synchronously.

After the finished Promise of the `startViewTransition` is resolved, React will then invoke `useEffect`. This prevents those from interfering with the performance of the animation. However, this is not a guarantee because if another `setState` happens while the animation is running it'll still have to invoke the `useEffect` earlier to preserve the sequential guarantees.

#### Props 

- **optional** `name`: A string or object. The name of the View Transition used for shared element transitions. If not provided, React will use a unique name for each View Transition to prevent unexpected animations.
- [View Transition Class](#view-transition-class) props.
- [View Transition Event](#view-transition-event) props.

#### Caveats 

- Only use `name` for [shared element transitions](#animating-a-shared-element). For all other animations, React automatically generates a unique name to prevent unexpected animations.
- By default, `setState` updates immediately and does not activate `

### View Transition Class 

`
```

See [Animating with JavaScript](#animating-with-javascript) for more examples.

---

## Styling View Transitions 

To customize the animation for a `
  );
}

function Parent() {
  const [show, setShow] = useState();
  if (show) {
    return 
        
          
          
        
      
    
  );
}
```

```js
import  from 'react';
import  from './Video';
import videos from './data';

function Item() 

export default function Component() {
  const [showItem, setShowItem] = useState(false);
  return (
    <>
       {
          startTransition(() => );
        }}>
        
      

      
```

This constraint prevents subtle bugs where too much or too little animates.

---

### Animating enter/exit with Activity 

If you want to animate a component in and out while preserving its state, or pre-rendering content for an animation, you can use [`

```

In this example, `Counter` has a counter with internal state. Try incrementing the counter, hiding it, then showing it again. The counter's value is preserved while the sidebar animates in and out:

      
    
  );
}
function Toggle() {
  return (
     {
        startTransition(() => );
      }}>
      
    
  )
}
function Counter() {
  const [count, setCount] = useState(0);
  return (
    
      Counter
      Count: 
       setCount(count + 1)}>
        Increment
      
    
  );
}

```

```css
.layout 
.counter 
.counter h2 
.counter p 
.toggle 
.toggle:hover 
.counter button 
```

```json package.json hidden
{
  "dependencies": 
}
```

Without `
```

When one tree unmounts and another mounts, if there's a pair where the same name exists in the unmounting tree and the mounting tree, they trigger the "share" animation on both. It animates from the unmounting side to the mounting side.

Unlike an exit/enter animation this can be deeply inside the deleted/mounted tree. If a `
  );
}

export function Video() 
```

```js src/data.js hidden
export default [
  ,
];
```

```css
#root 
button 
button:hover 
.thumbnail 
.thumbnail.blue 
.thumbnail.red 
.thumbnail.fullscreen 
.video 
.video .link 
.video .info 
.video .info:hover 
.video-title 
.video-description 
.fullscreenLayout 
.close-button 
@keyframes progress-animation {
  from 
  to 
}
```

```json package.json hidden
{
  "dependencies": 
}
```

---

### Animating reorder of items in a list 

```js
items.map((item) => 
  );
}
```

        
          
          
        
      
    
  );
}
```

```js
import  from 'react';
import  from './Video';
import videos from './data';

export default function Component() {
  const [orderedVideos, setOrderedVideos] = useState(videos);
  const reorder = () => {
    startTransition(() => {
      setOrderedVideos((prev) => );
    });
  };
  return (
    <>
      🎲
      
        {orderedVideos.map((video, i) => )}
      
    </>
  );
}
```

```js src/data.js hidden
export default [
  ,
  ,
  ,
  ,
];
```

```css
#root 
button 
button:hover 
.thumbnail 
.thumbnail.blue 
.thumbnail.red 
.thumbnail.green 
.thumbnail.purple 
.video 
.video .link 
.video .info 
.video .info:hover 
.video-title 
.video-description 
```

```json package.json hidden
{
  "dependencies": 
}
```

However, this wouldn't animate each individual item:

```js
function Component() 
```

Instead, any parent `
        
          
          
        
      
    
  );
}
```

```js
import  from 'react';
import  from './Video';
import videos from './data';

export default function Component() {
  const [orderedVideos, setOrderedVideos] = useState(videos);
  const reorder = () => {
    startTransition(() => {
      setOrderedVideos((prev) => );
    });
  };
  return (
    <>
      🎲
      
    </>
  );
}
```

```js src/data.js hidden
export default [
  ,
  ,
  ,
  ,
];
```

```css
#root 
button 
button:hover 
.thumbnail 
.thumbnail.blue 
.thumbnail.red 
.thumbnail.green 
.thumbnail.purple 
.video 
.video .link 
.video .info 
.video .info:hover 
.video-title 
.video-description 
```

```json package.json hidden
{
  "dependencies": 
}
```

This means you might want to avoid wrapper elements in lists where you want to allow the Component to control its own reorder animation:

```
items.map(item => 

---

### Animating from Suspense content 

Like any Transition, React waits for data and new CSS (``) before running the animation. In addition to this, ViewTransitions also wait up to 500ms for new fonts to load before starting the animation to avoid them flickering in later. For the same reason, an image wrapped in ViewTransition will wait for the image to load.

If it's inside a new Suspense boundary instance, then the fallback is shown first. After the Suspense boundary fully loads, it triggers the `

```

In this scenario when the content goes from A to B, it'll be treated as an "update" and apply that class if appropriate. Both A and B will get the same view-transition-name and therefore they're acting as a cross-fade by default.

        
          
          
        
      
    
  );
}

export function VideoPlaceholder() {
  const video = ;
  return (
    
      
        
        
          
          
        
      
    
  );
}
```

```js
import  from 'react';
import  from './Video';
import  from './data';

function LazyVideo() 
    </>
  );
}
```

```js src/data.js hidden
import  from 'react';

let cache = null;

function fetchVideo() {
  if (!cache) {
    cache = new Promise((resolve) => {
      setTimeout(() => {
        resolve();
      }, 1000);
    });
  }
  return cache;
}

export function useLazyVideoData() 
```

```css
#root 
button 
button:hover 
.thumbnail 
.thumbnail.blue 
.loading 
@keyframes shimmer {
  0% 
  100% 
}
.video 
.video .link 
.video .info 
.video .info:hover 
.video-title 
.video-title.loading 
.video-description 
.video-description.loading 
```

```json package.json hidden
{
  "dependencies": 
}
```

**Enter/Exit:**

```
}>
  

```

In this scenario, these are two separate ViewTransition instances each with their own `view-transition-name`. This will be treated as an "exit" of the `
  

```

This will only animate if the theme changes and not if only the children update. The children can still opt-in again with their own `
```

And define slow-fade in CSS using view transition classes:

```css
::view-transition-old(.slow-fade) 

::view-transition-new(.slow-fade) 
```

        
          
          
        
      
    
  );
}
```

```js
import  from 'react';
import  from './Video';
import videos from './data';

function Item() 

export default function Component() {
  const [showItem, setShowItem] = useState(false);
  return (
    <>
       {
          startTransition(() => );
        }}>
        
      

      {showItem ? 

In addition to setting the `default`, you can also provide configurations for `enter`, `exit`, `update`, and `share` animations.

        
          
          
        
      
    
  );
}
```

```js
import  from 'react';
import  from './Video';
import videos from './data';

function Item() 

export default function Component() {
  const [showItem, setShowItem] = useState(false);
  return (
    <>
       {
          startTransition(() => );
        }}>
        
      

      {showItem ? 

---

### Customizing animations with types 

You can use the [`addTransitionType`](/reference/react/addTransitionType) API to add a class name to the child elements when a specific transition type is activated for a specific activation trigger. This allows you to customize the animation for each type of transition.

For example, to customize the animation for all forward and backward navigations:

```js
;

// in your router:
startTransition(() => );
```

When the ViewTransition activates a "navigation-back" animation, React will add the class name "slide-right". When the ViewTransition activates a "navigation-forward" animation, React will add the class name "slide-left".

In the future, routers and other libraries may add support for standard view-transition types and styles.

        
          
          
        
      
    
  );
}
```

```js
import  from 'react';
import  from './Video';
import videos from './data';

function Item() 

export default function Component() {
  const [showItem, setShowItem] = useState(false);
  return (
    <>
      
         {
            startTransition(() => {
              if (showItem)  else 
              setShowItem((prev) => !prev);
            });
          }}>
          ⬅️
        
         {
            startTransition(() => {
              if (showItem)  else 
              setShowItem((prev) => !prev);
            });
          }}>
          ➡️
        
      
      {showItem ? 

---

### Animating with JavaScript 

While [View Transition Classes](#view-transition-class) let you define animations with CSS, sometimes you need imperative control over the animation. The `onEnter`, `onExit`, `onUpdate`, and `onShare` callbacks give you direct access to the view transition pseudo-elements so you can animate them using the [Web Animations API](https://developer.mozilla.org/en-US/docs/Web/API/Web_Animations_API).

Each callback receives an `instance` with `.old` and `.new` properties representing the view transition pseudo-elements. You can call `.animate()` on them just like you would on a DOM element:

```js

```

This allows you to combine CSS-driven animations and JavaScript-driven animations.

In the following example, the default cross-fade is handled by CSS, and the slide animations are driven by JavaScript in the `onEnter` and `onExit` animations:

        
          
          
        
      
    
  );
}
```

```js
import  from 'react';
import  from './Video';
import videos from './data';
import  from './animations';

function Item() 

export default function Component() {
  const [showItem, setShowItem] = useState(false);
  return (
    <>
       {
          startTransition(() => );
        }}>
        
      

      {showItem ? 

---

### Animating transition types with JavaScript 

You can use `types` passed to `ViewTransition` events to conditionally apply different animations based on how the Transition was triggered.

```js 
 

        
          
          
        
      
    
  );
}
```

```js
import  from 'react';
import  from './Video';
import videos from './data';
import  from './animations';

function Item() 

export default function Component() {
  const [showItem, setShowItem] = useState(false);
  const [isFast, setIsFast] = useState(false);
  return (
    <>
      
        Fast:  } value=>
      
       {
          startTransition(() => {
            if (isFast) 
            setShowItem((prev) => !prev);
          });
        }}>
        
      

      {showItem ? 

---

### Building View Transition enabled routers 

React waits for any pending Navigation to finish to ensure that scroll restoration happens within the animation. If the Navigation is blocked on React, your router must unblock in `useLayoutEffect` since `useEffect` would lead to a deadlock.

If a `startTransition` is started from the legacy popstate event, such as during a "back"-navigation then it must finish synchronously to ensure scroll and form restoration works correctly. This is in conflict with running a View Transition animation. Therefore, React will skip animations from popstate and animations won't run for the back button. You can fix this by upgrading your router to use the Navigation API.

---

## Troubleshooting 

### My `
    
  );
}
```

To fix, ensure that the `
  );
}
```

### I'm getting an error "There are two `;
}

function ItemList() {
  return (
    <>
      

function ItemList() {
  return (
    <>
      
    </>
  );
}
```
