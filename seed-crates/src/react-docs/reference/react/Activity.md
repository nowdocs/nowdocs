---
title: 
```

```

When an Activity boundary is , React will visually hide  using the `display: "none"` CSS property. It will also destroy their Effects, cleaning up any active subscriptions.

While hidden, children still re-render in response to new props, albeit at a lower priority than the rest of the content.

When the boundary becomes  again, React will reveal the children with their previous state restored, and re-create their Effects.

In this way, Activity can be thought of as a mechanism for rendering "background activity". Rather than completely discarding content that's likely to become visible again, you can use Activity to maintain and restore that content's UI and internal state, while ensuring that your hidden content has no unwanted side effects.

[See more examples below.](#usage)

#### Props 

* `children`: The UI you intend to show and hide.
* `mode`: A string value of either `'visible'` or `'hidden'`. If omitted, defaults to `'visible'`.

#### Caveats 

- If an Activity is rendered inside of a [ViewTransition](/reference/react/ViewTransition), and it becomes visible as a result of an update caused by [startTransition](/reference/react/startTransition), it will activate the ViewTransition's `enter` animation. If it becomes hidden, it will activate its `exit` animation.
- A *hidden* Activity that just renders text will not render anything rather than rendering hidden text, because there’s no corresponding DOM element to apply visibility changes to. For example, `` will not produce any output in the DOM for `const ComponentThatJustReturnsText = () => "Hello, World!"`. `` will render visible text.

---

## Usage 

### Restoring the state of hidden components 

In React, when you want to conditionally show or hide a component, you typically mount or unmount it based on that condition:

```jsx
{isShowingSidebar && (
  
```

This makes it possible to hide and then later restore components in the state they were previously in.

The following example has a sidebar with an expandable section. You can press "Overview" to reveal the three subitems below it. The main app area also has a button that hides and shows the sidebar.

Try expanding the Overview section, and then toggling the sidebar closed then open:

The Overview section always starts out collapsed. Because we unmount the sidebar when `isShowingSidebar` flips to `false`, all its internal state is lost.

This is a perfect use case for Activity. We can preserve the internal state of our sidebar, even when visually hiding it.

Let's replace the conditional rendering of our sidebar with an Activity boundary:

```jsx 
// Before
>
          Toggle sidebar
        
        Main content
      
    </>
  );
}
```

```js src/Sidebar.js
import  from 'react';

export default function Sidebar() >
        Overview
        
          &#9650;
        
      

      
    
  );
}
```

```css
body 
#root 
nav 
main 
p 
h1 
.indicator 
.indicator.down 
```

Our sidebar's internal state is now restored, without any changes to its implementation.

---

### Restoring the DOM of hidden components 

Since Activity boundaries hide their children using `display: none`, their children's DOM is also preserved when hidden. This makes them great for maintaining ephemeral state in parts of the UI that the user is likely to interact with again.

In this example, the Contact tab has a `` where the user can enter a message. If you enter some text, change to the Home tab, then change back to the Contact tab, the draft message is lost:

      

      

      
```

```js src/TabButton.js
export default function TabButton() {
  if (isActive) {
    return 
  }

  return (
    
      
    
  );
}
```

```js src/Home.js
export default function Home() 
```

```js src/Contact.js
export default function Contact() 
```

```css
body 
button 
b 
.pending 
```

Again, the Activity boundary let us preserve the Contact tab's internal state without changing its implementation.

---

### Pre-rendering content that's likely to become visible 

So far, we've seen how Activity can hide some content that the user has interacted with, without discarding that content's ephemeral state.

But Activity boundaries can also be used to _prepare_ content that the user has yet to see for the first time:

```jsx [[1, 1, "\\"hidden\\""]]

```

When an Activity boundary is  during its initial render, its children won't be visible on the page — but they will _still be rendered_, albeit at a lower priority than the visible content, and without mounting their Effects.

This _pre-rendering_ allows the children to load any code or data they need ahead of time, so that later, when the Activity boundary becomes visible, the children can appear faster with reduced loading times.

Let's look at an example.

In this demo, the Posts tab loads some data. If you press it, you'll see a Suspense fallback displayed while the data is being fetched:

      

      

      
    </>
  );
}
```

```js src/TabButton.js hidden
export default function TabButton() {
  if (isActive) {
    return 
  }

  return (
    
      
    
  );
}
```

```js src/Home.js
export default function Home() 
```

```js src/Posts.js
import  from 'react';
import  from './data.js';

export default function Posts() {
  const posts = use(fetchData('/posts'));

  return (
    
      {posts.map(post =>
        
          
        
      )}
    
  );
}
```

```js src/data.js hidden
// Note: the way you would do data fetching depends on
// the framework that you use together with Suspense.
// Normally, the caching logic would be inside a framework.

let cache = new Map();

export function fetchData(url) {
  if (!cache.has(url)) 
  return cache.get(url);
}

async function getData(url) {
  if (url.startsWith('/posts'))  else 
}

async function getPosts() {
  // Add a fake delay to make waiting noticeable.
  await new Promise(resolve => );
  let posts = [];
  for (let i = 0; i < 10; i++) {
    posts.push();
  }
  return posts;
}
```

```css
body 
button 
b 
.pending 
video 
```

This is because `App` doesn't mount `Posts` until its tab is active.

If we update `App` to use an Activity boundary to show and hide the active tab, `Posts` will be pre-rendered when the app first loads, allowing it to fetch its data before it becomes visible.

Try clicking the Posts tab now:

      

      

      
        
      
    </>
  );
}
```

```js src/TabButton.js hidden
export default function TabButton() {
  if (isActive) {
    return 
  }

  return (
    
      
    
  );
}
```

```js src/Home.js
export default function Home() 
```

```js src/Posts.js
import  from 'react';
import  from './data.js';

export default function Posts() {
  const posts = use(fetchData('/posts'));

  return (
    
      {posts.map(post =>
        
          
        
      )}
    
  );
}
```

```js src/data.js hidden
// Note: the way you would do data fetching depends on
// the framework that you use together with Suspense.
// Normally, the caching logic would be inside a framework.

let cache = new Map();

export function fetchData(url) {
  if (!cache.has(url)) 
  return cache.get(url);
}

async function getData(url) {
  if (url.startsWith('/posts'))  else 
}

async function getPosts() {
  // Add a fake delay to make waiting noticeable.
  await new Promise(resolve => );
  let posts = [];
  for (let i = 0; i < 10; i++) {
    posts.push();
  }
  return posts;
}
```

```css
body 
button 
b 
.pending 
video 
```

`Posts` was able to prepare itself for a faster render, thanks to the hidden Activity boundary.

---

Pre-rendering components with hidden Activity boundaries is a powerful way to reduce loading times for parts of the UI that the user is likely to interact with next.

---

### Speeding up interactions during page load 

React includes an under-the-hood performance optimization called Selective Hydration. It works by hydrating your app's initial HTML _in chunks_, enabling some components to become interactive even if other components on the page haven't loaded their code or data yet.

Suspense boundaries participate in Selective Hydration, because they naturally divide your component tree into units that are independent from one another:

```jsx
function Page() 
```

Here, `MessageComposer` can be fully hydrated during the initial render of the page, even before `Chats` is mounted and starts to fetch its data.

So by breaking up your component tree into discrete units, Suspense allows React to hydrate your app's server-rendered HTML in chunks, enabling parts of your app to become interactive as fast as possible.

But what about pages that don't use Suspense?

Take this tabs example:

```jsx
function Page() {
  const [activeTab, setActiveTab] = useState('home');

  return (
    <>
      
      

      
```

...but it would also change the UI, since the `Placeholder` fallback would be displayed on the initial render.

Instead, we can use Activity. Since Activity boundaries show and hide their children, they already naturally divide the component tree into independent units. And just like Suspense, this feature allows them to participate in Selective Hydration.

Let's update our example to use Activity boundaries around the active tab:

```jsx 
function Page() 
```

Now our initial server-rendered HTML looks the same as it did in the original version, but thanks to Activity, React can hydrate the tab buttons first, before it even mounts `Home` or `Video`.

---

Thus, in addition to hiding and showing content, Activity boundaries help improve your app's performance during hydration by letting React know which parts of your page can become interactive in isolation.

And even if your page doesn't ever hide part of its content, you can still add always-visible Activity boundaries to improve hydration performance:

```jsx
function Page() 
```

---

## Troubleshooting 

### My hidden components have unwanted side effects 

An Activity boundary hides its content by setting `display: none` on its children and cleaning up any of their Effects. So, most well-behaved React components that properly clean up their side effects will already be robust to being hidden by Activity.

But there _are_ some situations where a hidden component behaves differently than an unmounted one. Most notably, since a hidden component's DOM is not destroyed, any side effects from that DOM will persist, even after the component is hidden.

As an example, consider a `` tag. Typically it doesn't require any cleanup, because even if you're playing a video, unmounting the tag stops the video and audio from playing in the browser. Try playing the video and then pressing Home in this demo:

      

      

      
```

```js src/TabButton.js hidden
export default function TabButton() {
  if (isActive) {
    return 
  }

  return (
    
      
    
  );
}
```

```js src/Home.js
export default function Home() 
```

```js src/Video.js
export default function Video() 
```

```css
body 
button 
b 
.pending 
video 
```

Whoops! The video and audio continue to play even after it's been hidden, because the tab's `` element is still in the DOM.

To fix this, we can add an Effect with a cleanup function that pauses the video:

```jsx 
export default function VideoTab() {
  const ref = useRef();

  useLayoutEffect(() => {
    const videoRef = ref.current;

    return () => 
  }, []);

  return (
    

  );
}
```

We call `useLayoutEffect` instead of `useEffect` because conceptually the clean-up code is tied to the component's UI being visually hidden. If we used a regular effect, the code could be delayed by (say) a re-suspending Suspense boundary or a View Transition.

Let's see the new behavior. Try playing the video, switching to the Home tab, then back to the Video tab:

      

      

      
      
    </>
  );
}
```

```js src/TabButton.js hidden
export default function TabButton() {
  if (isActive) {
    return 
  }

  return (
    
      
    
  );
}
```

```js src/Home.js
export default function Home() 
```

```js src/Video.js
import  from 'react';

export default function Video() {
  const ref = useRef();

  useLayoutEffect(() => {
    const videoRef = ref.current

    return () => ;
  }, [])

  return (
    

  );
}
```

```css
body 
button 
b 
.pending 
video 
```

It works great! Our cleanup function ensures that the video stops playing if it's ever hidden by an Activity boundary, and even better, because the `` tag is never destroyed, the timecode is preserved, and the video itself doesn't need to be initialized or downloaded again when the user switches back to keep watching it.

This is a great example of using Activity to preserve ephemeral DOM state for parts of the UI that become hidden, but the user is likely to interact with again soon.

---

Our example illustrates that for certain tags like ``, unmounting and hiding have different behavior. If a component renders DOM that has a side effect, and you want to prevent that side effect when an Activity boundary hides it, add an Effect with a return function to clean it up.

The most common cases of this will be from the following tags:

  - ``
  - ``
  - ``

Typically, though, most of your React components should already be robust to being hidden by an Activity boundary. And conceptually, you should think of "hidden" Activities as being unmounted.

To eagerly discover other Effects that don't have proper cleanup, which is important not only for Activity boundaries but for many other behaviors in React, we recommend using [``](/reference/react/StrictMode).

---

### My hidden components have Effects that aren't running 

When an `` is "hidden", all its children's Effects are cleaned up. Conceptually, the children are unmounted, but React saves their state for later. This is a feature of Activity because it means subscriptions won't be active for hidden parts of the UI, reducing the amount of work needed for hidden content.

If you're relying on an Effect mounting to clean up a component's side effects, refactor the Effect to do the work in the returned cleanup function instead.

To eagerly find problematic Effects, we recommend adding [``](/reference/react/StrictMode) which will eagerly perform Activity unmounts and mounts to catch any unexpected side-effects.
