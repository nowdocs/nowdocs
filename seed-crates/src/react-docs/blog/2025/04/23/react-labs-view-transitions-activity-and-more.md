---
title: "React Labs: View Transitions, Activity, and more"
author: Ricky Hanlon
date: 2025/04/23
description: In React Labs posts, we write about projects in active research and development. In this post, we're sharing two new experimental features that are ready to try today, and updates on other areas we're working on now.
---

April 23, 2025 by [Ricky Hanlon](https://twitter.com/rickhanlonii)

---

Today, we're excited to release documentation for two new experimental features that are ready for testing:

- [View Transitions](#view-transitions)
- [Activity](#activity)

We're also sharing updates on new features currently in development:
- [React Performance Tracks](#react-performance-tracks)
- [Compiler IDE Extension](#compiler-ide-extension)
- [Automatic Effect Dependencies](#automatic-effect-dependencies)
- [Fragment Refs](#fragment-refs)
- [Concurrent Stores](#concurrent-stores)

---

# New Experimental Features 

View Transitions and Activity are now ready for testing in `react@experimental`. These features have been tested in production and are stable, but the final API may still change as we incorporate feedback.

You can try them by upgrading React packages to the most recent experimental version:

- `react@experimental`
- `react-dom@experimental`

Read on to learn how to use these features in your app, or check out the newly published docs:

- [`
```

This new component lets you declaratively define "what" to animate when an animation is activated.

You can define "when" to animate by using one of these three triggers for a View Transition:

```js
// "when" to animate.

// Transitions
startTransition(() => setState(...));

// Deferred Values
const deferred = useDeferredValue(value);

// Suspense

```

By default, these animations use the [default CSS animations for View Transitions](https://developer.mozilla.org/en-US/docs/Web/API/View_Transition_API/Using#customizing_your_animations) applied (typically a smooth cross-fade). You can use [view transition pseudo-selectors](https://developer.mozilla.org/en-US/docs/Web/API/View_Transition_API/Using#the_view_transition_pseudo-element_tree) to define "how" the animation runs. For example, you can use `*` to change the default animation for all transitions:

```
// "how" to animate.
::view-transition-old(*) 
::view-transition-new(*) 
```

When the DOM updates due to an animation trigger&mdash;like `startTransition`, `useDeferredValue`, or a `Suspense` fallback switching to content&mdash;React will use [declarative heuristics](/reference/react/ViewTransition#viewtransition) to automatically determine which `
        
      
    
  );
}

```

```js src/Home.js
import  from "./Videos";
import Layout from "./Layout";
import  from "./data";
import  from "react";
import  from "./Icons";

function SearchInput() >
      
        Search
      
      
        
          
  );
}

```

```js src/Icons.js
export function ChevronLeft() 

export function PauseIcon() 

export function PlayIcon() 
export function Heart() {
  return (
    <>
      
        
      

      
        
      
    </>
  );
}

export function IconSearch(props) 
```

```js src/Layout.js
import  from "./router";

export default function Page() {
  const isPending = useIsNavPending();
  return (
    
      
        
          
          
        
      

      
        
      
    
  );
}
```

```js src/LikeButton.js
import  from 'react';
import  from './Icons';

// A hack since we don't actually have a backend.
// Unlike local state, this survives videos being filtered.
const likedVideos = new Set();

export default function LikeButton() {
  const [isLiked, setIsLiked] = useState(() => likedVideos.has(video.id));
  const [animate, setAnimate] = useState(false);
  return (
     {
        const nextIsLiked = !isLiked;
        if (nextIsLiked)  else 
        setAnimate(true);
        setIsLiked(nextIsLiked);
      }}>
      

        
          
          
        
      
      
  );
}
```

```css src/styles.css
@font-face 

@font-face 

@font-face 

@font-face 

* 

html 

body 

#root 

h1 

h2 

h3 

h4 

h5 

h6 

code 

ul 

.sr-only 

.absolute 

.overflow-visible 

.visible 

.fit 

/* Layout */
.page 

.top-hero 

.bottom 

.top-nav 

.content 

.loader 

@keyframes loading-spinner {
  0%,
  100% 
  12.5% 
  25% 
  37.5% 
  50% 
  62.5% 
  75% 
  87.5% 
}

/* LikeButton */
.like-button 

.like-button:focus 

.like-button:active 

.like-button:hover 

.like-button.liked 

/* Icons */
@keyframes circle {
  0% 

  50% 

  to 
}

.circle 

.circle.liked.animate 

.heart 

.heart.liked 

.heart.liked.animate 

.control-icon 

.chevron-left 

/* Video */
.thumbnail 

.thumbnail.blue 

.thumbnail.red 

.thumbnail.green 

.thumbnail.purple 

.thumbnail.yellow 

.thumbnail.gray 

.video 

.video .link 

.video .info 

.video .info:hover 

.video-title 

.video-description 

/* Details */
.details .thumbnail 

.video-details-title 

.video-details-speaker 

.back 

.back:hover 

.info-title 

.info-description 

.controls 

.fallback 

.fallback.title 

.fallback.description 

@keyframes shimmer {
  0% 

  100% 
}

.search 
.search-input 

.search-icon 

.search-input input 

.search-input input:hover, .search-input input:active 

/* Home */
.video-list 

.video-list .videos 
```

```js src/index.js hidden
import React,  from 'react';
import  from 'react-dom/client';
import './styles.css';

import App from './App';
import  from './router';

const root = createRoot(document.getElementById('root'));
root.render(
  
  
);
```

```json package.json hidden
{
  "dependencies": ,
  "scripts": 
}
```

### Animating navigations 

Our app includes a Suspense-enabled router, with [page transitions already marked as Transitions](/reference/react/useTransition#building-a-suspense-enabled-router), which means navigations are performed with `startTransition`:

```js
function navigate(url) {
  startTransition(() => );
}
```

`startTransition` is a View Transition trigger, so we can add `
```

When the `url` changes, the `
  );
}
```

```js src/Details.js hidden
import  from "./data";
import  from "./Videos";
import  from "./router";
import Layout from "./Layout";
import  from "react";
import  from "./Icons";

function VideoInfo() {
  const details = use(fetchVideoDetails(id));
  return (
    <>
      
      
    </>
  );
}

function VideoInfoFallback() 

export default function Details() {
  const  = useRouter();
  const videoId = url.split("/").pop();
  const video = use(fetchVideo(videoId));

  return (
    
        
      
    
  );
}

```

```js src/Home.js hidden
import  from "./Videos";
import Layout from "./Layout";
import  from "./data";
import  from "react";
import  from "./Icons";

function SearchInput() >
      
        Search
      
      
        
          
  );
}

```

```js src/Icons.js hidden
export function ChevronLeft() 

export function PauseIcon() 

export function PlayIcon() 
export function Heart() {
  return (
    <>
      
        
      

      
        
      
    </>
  );
}

export function IconSearch(props) 
```

```js src/Layout.js
import  from 'react'; import  from "./router";

export default function Page() {
  const isPending = useIsNavPending();

  return (
    
      
        
          
          
        
      
      
      
      
    
  );
}
```

```js src/LikeButton.js hidden
import  from 'react';
import  from './Icons';

// A hack since we don't actually have a backend.
// Unlike local state, this survives videos being filtered.
const likedVideos = new Set();

export default function LikeButton() {
  const [isLiked, setIsLiked] = useState(() => likedVideos.has(video.id));
  const [animate, setAnimate] = useState(false);
  return (
     {
        const nextIsLiked = !isLiked;
        if (nextIsLiked)  else 
        setAnimate(true);
        setIsLiked(nextIsLiked);
      }}>
      

        
          
          
        
      
      
  );
}

const RouterContext = createContext({ url: "/", params:  });

export function useRouter() 

export function useIsNavPending() 
```

```css src/styles.css hidden
@font-face 

@font-face 

@font-face 

@font-face 

* 

html 

body 

#root 

h1 

h2 

h3 

h4 

h5 

h6 

code 

ul 

.sr-only 

.absolute 

.overflow-visible 

.visible 

.fit 

/* Layout */
.page 

.top-hero 

.bottom 

.top-nav 

.content 

.loader 

@keyframes loading-spinner {
  0%,
  100% 
  12.5% 
  25% 
  37.5% 
  50% 
  62.5% 
  75% 
  87.5% 
}

/* LikeButton */
.like-button 

.like-button:focus 

.like-button:active 

.like-button:hover 

.like-button.liked 

/* Icons */
@keyframes circle {
  0% 

  50% 

  to 
}

.circle 

.circle.liked.animate 

.heart 

.heart.liked 

.heart.liked.animate 

.control-icon 

.chevron-left 

/* Video */
.thumbnail 

.thumbnail.blue 

.thumbnail.red 

.thumbnail.green 

.thumbnail.purple 

.thumbnail.yellow 

.thumbnail.gray 

.video 

.video .link 

.video .info 

.video .info:hover 

.video-title 

.video-description 

/* Details */
.details .thumbnail 

.video-details-title 

.video-details-speaker 

.back 

.back:hover 

.info-title 

.info-description 

.controls 

.fallback 

.fallback.title 

.fallback.description 

@keyframes shimmer {
  0% 

  100% 
}

.search 
.search-input 

.search-icon 

.search-input input 

.search-input input:hover, .search-input input:active 

/* Home */
.video-list 

.video-list .videos 
```

```js src/index.js hidden
import React,  from 'react';
import  from 'react-dom/client';
import './styles.css';

import App from './App';
import  from './router';

const root = createRoot(document.getElementById('root'));
root.render(
  
  
);
```

```json package.json hidden
{
  "dependencies": ,
  "scripts": 
}
```

Since our router already updates the route using `startTransition`, this one line change to add `
```

In practice, navigations should be done via "enter" and "exit" props, or by using Transition Types.

### Customizing animations 

By default, `
```

And define `slow-fade` in CSS using [view transition classes](/reference/react/ViewTransition#view-transition-class):

```css
::view-transition-old(.slow-fade) 

::view-transition-new(.slow-fade) 
```

Now, the cross fade is slower:

  );
}
```

```js src/Details.js hidden
import  from "./data";
import  from "./Videos";
import  from "./router";
import Layout from "./Layout";
import  from "react";
import  from "./Icons";

function VideoInfo() {
  const details = use(fetchVideoDetails(id));
  return (
    <>
      
      
    </>
  );
}

function VideoInfoFallback() 

export default function Details() {
  const  = useRouter();
  const videoId = url.split("/").pop();
  const video = use(fetchVideo(videoId));

  return (
    
        
      
    
  );
}

```

```js src/Home.js hidden
import  from "./Videos";
import Layout from "./Layout";
import  from "./data";
import  from "react";
import  from "./Icons";

function SearchInput() >
      
        Search
      
      
        
          
  );
}

```

```js src/Icons.js hidden
export function ChevronLeft() 

export function PauseIcon() 

export function PlayIcon() 
export function Heart() {
  return (
    <>
      
        
      

      
        
      
    </>
  );
}

export function IconSearch(props) 
```

```js src/Layout.js hidden
import  from 'react'; import  from "./router";

export default function Page() {
  const isPending = useIsNavPending();

  return (
    
      
        
          
          
        
      
      
      
      
    
  );
}
```

```js src/LikeButton.js hidden
import  from 'react';
import  from './Icons';

// A hack since we don't actually have a backend.
// Unlike local state, this survives videos being filtered.
const likedVideos = new Set();

export default function LikeButton() {
  const [isLiked, setIsLiked] = useState(() => likedVideos.has(video.id));
  const [animate, setAnimate] = useState(false);
  return (
     {
        const nextIsLiked = !isLiked;
        if (nextIsLiked)  else 
        setAnimate(true);
        setIsLiked(nextIsLiked);
      }}>
      

        
          
          
        
      
      
  );
}
```

```css src/styles.css hidden
@font-face 

@font-face 

@font-face 

@font-face 

* 

html 

body 

#root 

h1 

h2 

h3 

h4 

h5 

h6 

code 

ul 

.sr-only 

.absolute 

.overflow-visible 

.visible 

.fit 

/* Layout */
.page 

.top-hero 

.bottom 

.top-nav 

.content 

.loader 

@keyframes loading-spinner {
  0%,
  100% 
  12.5% 
  25% 
  37.5% 
  50% 
  62.5% 
  75% 
  87.5% 
}

/* LikeButton */
.like-button 

.like-button:focus 

.like-button:active 

.like-button:hover 

.like-button.liked 

/* Icons */
@keyframes circle {
  0% 

  50% 

  to 
}

.circle 

.circle.liked.animate 

.heart 

.heart.liked 

.heart.liked.animate 

.control-icon 

.chevron-left 

/* Video */
.thumbnail 

.thumbnail.blue 

.thumbnail.red 

.thumbnail.green 

.thumbnail.purple 

.thumbnail.yellow 

.thumbnail.gray 

.video 

.video .link 

.video .info 

.video .info:hover 

.video-title 

.video-description 

/* Details */
.details .thumbnail 

.video-details-title 

.video-details-speaker 

.back 

.back:hover 

.info-title 

.info-description 

.controls 

.fallback 

.fallback.title 

.fallback.description 

@keyframes shimmer {
  0% 

  100% 
}

.search 
.search-input 

.search-icon 

.search-input input 

.search-input input:hover, .search-input input:active 

/* Home */
.video-list 

.video-list .videos 
```

```css src/animations.css
/* Define .slow-fade using view transition classes */
::view-transition-old(.slow-fade) 

::view-transition-new(.slow-fade) 
```

```js src/index.js hidden
import React,  from 'react';
import  from 'react-dom/client';
import './styles.css';
import './animations.css';

import App from './App';
import  from './router';

const root = createRoot(document.getElementById('root'));
root.render(
  
  
);
```

```json package.json hidden
{
  "dependencies": ,
  "scripts": 
}
```

See [Styling View Transitions](/reference/react/ViewTransition#styling-view-transitions) for a full guide on styling `
```

Now the video thumbnail animates between the two pages:

  );
}
```

```js src/Details.js hidden
import  from "./data";
import  from "./Videos";
import  from "./router";
import Layout from "./Layout";
import  from "react";
import  from "./Icons";

function VideoInfo() {
  const details = use(fetchVideoDetails(id));
  return (
    <>
      
      
    </>
  );
}

function VideoInfoFallback() 

export default function Details() {
  const  = useRouter();
  const videoId = url.split("/").pop();
  const video = use(fetchVideo(videoId));

  return (
    
        
      
    
  );
}

```

```js src/Home.js hidden
import  from "./Videos";
import Layout from "./Layout";
import  from "./data";
import  from "react";
import  from "./Icons";

function SearchInput() >
      
        Search
      
      
        
          
  );
}

```

```js src/Icons.js hidden
export function ChevronLeft() 

export function PauseIcon() 

export function PlayIcon() 
export function Heart() {
  return (
    <>
      
        
      

      
        
      
    </>
  );
}

export function IconSearch(props) 
```

```js src/Layout.js hidden
import  from 'react'; import  from "./router";

export default function Page() {
  const isPending = useIsNavPending();

  return (
    
      
        
          
          
        
      
      
      
      
    
  );
}
```

```js src/LikeButton.js hidden
import  from 'react';
import  from './Icons';

// A hack since we don't actually have a backend.
// Unlike local state, this survives videos being filtered.
const likedVideos = new Set();

export default function LikeButton() {
  const [isLiked, setIsLiked] = useState(() => likedVideos.has(video.id));
  const [animate, setAnimate] = useState(false);
  return (
     {
        const nextIsLiked = !isLiked;
        if (nextIsLiked)  else 
        setAnimate(true);
        setIsLiked(nextIsLiked);
      }}>
      
  );
}

export function VideoControls() {
  const [isPlaying, setIsPlaying] = useState(false);

  return (
    
        startTransition(() => )
      }
    >
      {isPlaying ? 

        
          
          
        
      
      
  );
}
```

```css src/styles.css hidden
@font-face 

@font-face 

@font-face 

@font-face 

* 

html 

body 

#root 

h1 

h2 

h3 

h4 

h5 

h6 

code 

ul 

.sr-only 

.absolute 

.overflow-visible 

.visible 

.fit 

/* Layout */
.page 

.top-hero 

.bottom 

.top-nav 

.content 

.loader 

@keyframes loading-spinner {
  0%,
  100% 
  12.5% 
  25% 
  37.5% 
  50% 
  62.5% 
  75% 
  87.5% 
}

/* LikeButton */
.like-button 

.like-button:focus 

.like-button:active 

.like-button:hover 

.like-button.liked 

/* Icons */
@keyframes circle {
  0% 

  50% 

  to 
}

.circle 

.circle.liked.animate 

.heart 

.heart.liked 

.heart.liked.animate 

.control-icon 

.chevron-left 

/* Video */
.thumbnail 

.thumbnail.blue 

.thumbnail.red 

.thumbnail.green 

.thumbnail.purple 

.thumbnail.yellow 

.thumbnail.gray 

.video 

.video .link 

.video .info 

.video .info:hover 

.video-title 

.video-description 

/* Details */
.details .thumbnail 

.video-details-title 

.video-details-speaker 

.back 

.back:hover 

.info-title 

.info-description 

.controls 

.fallback 

.fallback.title 

.fallback.description 

@keyframes shimmer {
  0% 

  100% 
}

.search 
.search-input 

.search-icon 

.search-input input 

.search-input input:hover, .search-input input:active 

/* Home */
.video-list 

.video-list .videos 
```

```css src/animations.css
/* No additional animations needed */

/* Previously defined animations below */

::view-transition-old(.slow-fade) 

::view-transition-new(.slow-fade) 
```

```js src/index.js hidden
import React,  from 'react';
import  from 'react-dom/client';
import './styles.css';
import './animations.css';

import App from './App';
import  from './router';

const root = createRoot(document.getElementById('root'));
root.render(
  
  
);
```

```json package.json hidden
{
  "dependencies": ,
  "scripts": 
}
```

By default, React automatically generates a unique `name` for each element activated for a transition (see [How does `
```

Here we pass a `share` prop to define how to animate based on the transition type. When the share transition activates from `nav-forward`, the view transition class `slide-forward` is applied. When it's from `nav-back`, the `slide-back` animation is activated. Let's define these animations in CSS:

```css
::view-transition-old(.slide-forward) 

::view-transition-new(.slide-forward) 

::view-transition-old(.slide-back) 

::view-transition-new(.slide-back) 
```

Now we can animate the header along with thumbnail based on navigation type:

  );
}
```

```js src/Details.js hidden
import  from "./data";
import  from "./Videos";
import  from "./router";
import Layout from "./Layout";
import  from "react";
import  from "./Icons";

function VideoInfo() {
  const details = use(fetchVideoDetails(id));
  return (
    <>
      
      
    </>
  );
}

function VideoInfoFallback() 

export default function Details() {
  const  = useRouter();
  const videoId = url.split("/").pop();
  const video = use(fetchVideo(videoId));

  return (
    
        
      
    
  );
}

```

```js src/Home.js hidden
import  from "./Videos";
import Layout from "./Layout";
import  from "./data";
import  from "react";
import  from "./Icons";

function SearchInput() >
      
        Search
      
      
        
          
  );
}

```

```js src/Icons.js hidden
export function ChevronLeft() 

export function PauseIcon() 

export function PlayIcon() 
export function Heart() {
  return (
    <>
      
        
      

      
        
      
    </>
  );
}

export function IconSearch(props) 
```

```js src/Layout.js active
import  from 'react'; import  from "./router";

export default function Page() {
  const isPending = useIsNavPending();
  return (
    
      
        
          
          
          
        
      
      
      
      
    
  );
}
```

```js src/LikeButton.js hidden
import  from 'react';
import  from './Icons';

// A hack since we don't actually have a backend.
// Unlike local state, this survives videos being filtered.
const likedVideos = new Set();

export default function LikeButton() {
  const [isLiked, setIsLiked] = useState(() => likedVideos.has(video.id));
  const [animate, setAnimate] = useState(false);
  return (
     {
        const nextIsLiked = !isLiked;
        if (nextIsLiked)  else 
        setAnimate(true);
        setIsLiked(nextIsLiked);
      }}>
      
  );
}

export function VideoControls() {
  const [isPlaying, setIsPlaying] = useState(false);

  return (
    
        startTransition(() => )
      }
    >
      {isPlaying ? 

        
          
          
        
      
      
  );
}

const RouterContext = createContext({ url: "/", params:  });

export function useRouter() 

export function useIsNavPending() 

```

```css src/styles.css hidden
@font-face 

@font-face 

@font-face 

@font-face 

* 

html 

body 

#root 

h1 

h2 

h3 

h4 

h5 

h6 

code 

ul 

.sr-only 

.absolute 

.overflow-visible 

.visible 

.fit 

/* Layout */
.page 

.top-hero 

.bottom 

.top-nav 

.content 

.loader 

@keyframes loading-spinner {
  0%,
  100% 
  12.5% 
  25% 
  37.5% 
  50% 
  62.5% 
  75% 
  87.5% 
}

/* LikeButton */
.like-button 

.like-button:focus 

.like-button:active 

.like-button:hover 

.like-button.liked 

/* Icons */
@keyframes circle {
  0% 

  50% 

  to 
}

.circle 

.circle.liked.animate 

.heart 

.heart.liked 

.heart.liked.animate 

.control-icon 

.chevron-left 

/* Video */
.thumbnail 

.thumbnail.blue 

.thumbnail.red 

.thumbnail.green 

.thumbnail.purple 

.thumbnail.yellow 

.thumbnail.gray 

.video 

.video .link 

.video .info 

.video .info:hover 

.video-title 

.video-description 

/* Details */
.details .thumbnail 

.video-details-title 

.video-details-speaker 

.back 

.back:hover 

.info-title 

.info-description 

.controls 

.fallback 

.fallback.title 

.fallback.description 

@keyframes shimmer {
  0% 

  100% 
}

.search 
.search-input 

.search-icon 

.search-input input 

.search-input input:hover, .search-input input:active 

/* Home */
.video-list 

.video-list .videos 
```

```css src/animations.css
/* Animations for view transition classed added by transition type */
::view-transition-old(.slide-forward) 

::view-transition-new(.slide-forward) 

::view-transition-old(.slide-back) 

::view-transition-new(.slide-back) 

/* New keyframes to support our animations above. */
@keyframes fade-in {
    from 
}

@keyframes fade-out {
    to 
}

@keyframes slide-to-right {
    to 
}

@keyframes slide-from-right {
    from 
    to 
}

@keyframes slide-to-left {
    to 
}

@keyframes slide-from-left {
    from 
    to 
}

/* Previously defined animations. */

/* Default .slow-fade. */
::view-transition-old(.slow-fade) 

::view-transition-new(.slow-fade) 
```

```js src/index.js hidden
import React,  from 'react';
import  from 'react-dom/client';
import './styles.css';
import './animations.css';

import App from './App';
import  from './router';

const root = createRoot(document.getElementById('root'));
root.render(
  
  
);
```

```json package.json hidden
{
  "dependencies": ,
  "scripts": 
}
```

### Animating Suspense Boundaries 

Suspense will also activate View Transitions.

To animate the fallback to content, we can wrap `Suspense` with `

```

By adding this, the fallback will cross-fade into the content. Click a video and see the video info animate in:

  );
}
```

```js src/Details.js active
import  from "react"; import  from "./data"; import  from "./Videos"; import  from "./router"; import Layout from "./Layout"; import  from "./Icons";

function VideoDetails() 

function VideoInfoFallback() 

export default function Details() {
  const  = useRouter();
  const videoId = url.split("/").pop();
  const video = use(fetchVideo(videoId));

  return (
    
        
  );
}

function VideoInfo() {
  const details = use(fetchVideoDetails(id));
  return (
    
      
      
    
  );
}
```

```js src/Home.js hidden
import  from "./Videos";
import Layout from "./Layout";
import  from "./data";
import  from "react";
import  from "./Icons";

function SearchInput() >
      
        Search
      
      
        
          
  );
}

```

```js src/Icons.js hidden
export function ChevronLeft() 

export function PauseIcon() 

export function PlayIcon() 
export function Heart() {
  return (
    <>
      
        
      

      
        
      
    </>
  );
}

export function IconSearch(props) 
```

```js src/Layout.js hidden
import  from 'react';
import  from "./router";

export default function Page() {
  const isPending = useIsNavPending();
  return (
    
      
        
          
          
          
        
      
      
      
      
    
  );
}
```

```js src/LikeButton.js hidden
import  from 'react';
import  from './Icons';

// A hack since we don't actually have a backend.
// Unlike local state, this survives videos being filtered.
const likedVideos = new Set();

export default function LikeButton() {
  const [isLiked, setIsLiked] = useState(() => likedVideos.has(video.id));
  const [animate, setAnimate] = useState(false);
  return (
     {
        const nextIsLiked = !isLiked;
        if (nextIsLiked)  else 
        setAnimate(true);
        setIsLiked(nextIsLiked);
      }}>
      
  );
}

export function VideoControls() {
  const [isPlaying, setIsPlaying] = useState(false);

  return (
    
        startTransition(() => )
      }
    >
      {isPlaying ? 

        
          
          
        
      
      
  );
}

const RouterContext = createContext({ url: "/", params:  });

export function useRouter() 

export function useIsNavPending() 

```

```css src/styles.css hidden
@font-face 

@font-face 

@font-face 

@font-face 

* 

html 

body 

#root 

h1 

h2 

h3 

h4 

h5 

h6 

code 

ul 

.sr-only 

.absolute 

.overflow-visible 

.visible 

.fit 

/* Layout */
.page 

.top-hero 

.bottom 

.top-nav 

.content 

.loader 

@keyframes loading-spinner {
  0%,
  100% 
  12.5% 
  25% 
  37.5% 
  50% 
  62.5% 
  75% 
  87.5% 
}

/* LikeButton */
.like-button 

.like-button:focus 

.like-button:active 

.like-button:hover 

.like-button.liked 

/* Icons */
@keyframes circle {
  0% 

  50% 

  to 
}

.circle 

.circle.liked.animate 

.heart 

.heart.liked 

.heart.liked.animate 

.control-icon 

.chevron-left 

/* Video */
.thumbnail 

.thumbnail.blue 

.thumbnail.red 

.thumbnail.green 

.thumbnail.purple 

.thumbnail.yellow 

.thumbnail.gray 

.video 

.video .link 

.video .info 

.video .info:hover 

.video-title 

.video-description 

/* Details */
.details .thumbnail 

.video-details-title 

.video-details-speaker 

.back 

.back:hover 

.info-title 

.info-description 

.controls 

.fallback 

.fallback.title 

.fallback.description 

@keyframes shimmer {
  0% 

  100% 
}

.search 
.search-input 

.search-icon 

.search-input input 

.search-input input:hover, .search-input input:active 

/* Home */
.video-list 

.video-list .videos 
```

```css src/animations.css
/* Slide the fallback down */
::view-transition-old(.slide-down) 

/* Slide the content up */
::view-transition-new(.slide-up) 

/* Define the new keyframes */
@keyframes slide-up {
    from 
    to 
}

@keyframes slide-down {
    from 
    to 
}

/* Previously defined animations below */

/* Animations for view transition classed added by transition type */
::view-transition-old(.slide-forward) 

::view-transition-new(.slide-forward) 

::view-transition-old(.slide-back) 

::view-transition-new(.slide-back) 

/* Keyframes to support our animations above. */
@keyframes fade-in {
    from 
}

@keyframes fade-out {
    to 
}

@keyframes slide-to-right {
    to 
}

@keyframes slide-from-right {
    from 
    to 
}

@keyframes slide-to-left {
    to 
}

@keyframes slide-from-left {
    from 
    to 
}

/* Default .slow-fade. */
::view-transition-old(.slow-fade) 

::view-transition-new(.slow-fade) 
```

```js src/index.js hidden
import React,  from 'react';
import  from 'react-dom/client';
import './styles.css';
import './animations.css';

import App from './App';
import  from './router';

const root = createRoot(document.getElementById('root'));
root.render(
  
  
);
```

```json package.json hidden
{
  "dependencies": ,
  "scripts": 
}
```

We can also provide custom animations using an `exit` on the fallback, and `enter` on the content:

```js 

  }
>
  

```

Here's how we'll define `slide-down` and `slide-up` with CSS:

```css 
::view-transition-old(.slide-down) 

::view-transition-new(.slide-up) 
```

Now, the Suspense content replaces the fallback with a sliding animation:

  );
}
```

```js src/Details.js active
import  from "react"; import  from "./data"; import  from "./Videos"; import  from "./router"; import Layout from "./Layout"; import  from "./Icons";

function VideoDetails() 
    >
      
      
    
  );
}

function VideoInfoFallback() 

export default function Details() {
  const  = useRouter();
  const videoId = url.split("/").pop();
  const video = use(fetchVideo(videoId));

  return (
    
        
  );
}

function VideoInfo() {
  const details = use(fetchVideoDetails(id));
  return (
    <>
      
      
    </>
  );
}
```

```js src/Home.js hidden
import  from "./Videos";
import Layout from "./Layout";
import  from "./data";
import  from "react";
import  from "./Icons";

function SearchInput() >
      
        Search
      
      
        
          
  );
}

```

```js src/Icons.js hidden
export function ChevronLeft() 

export function PauseIcon() 

export function PlayIcon() 
export function Heart() {
  return (
    <>
      
        
      

      
        
      
    </>
  );
}

export function IconSearch(props) 
```

```js src/Layout.js hidden
import  from 'react';
import  from "./router";

export default function Page() {
  const isPending = useIsNavPending();
  return (
    
      
        
          
          
          
        
      
      
      
      
    
  );
}
```

```js src/LikeButton.js hidden
import  from 'react';
import  from './Icons';

// A hack since we don't actually have a backend.
// Unlike local state, this survives videos being filtered.
const likedVideos = new Set();

export default function LikeButton() {
  const [isLiked, setIsLiked] = useState(() => likedVideos.has(video.id));
  const [animate, setAnimate] = useState(false);
  return (
     {
        const nextIsLiked = !isLiked;
        if (nextIsLiked)  else 
        setAnimate(true);
        setIsLiked(nextIsLiked);
      }}>
      
  );
}

export function VideoControls() {
  const [isPlaying, setIsPlaying] = useState(false);

  return (
    
        startTransition(() => )
      }
    >
      {isPlaying ? 

        
          
          
        
      
      
  );
}

const RouterContext = createContext({ url: "/", params:  });

export function useRouter() 

export function useIsNavPending() 

```

```css src/styles.css hidden
@font-face 

@font-face 

@font-face 

@font-face 

* 

html 

body 

#root 

h1 

h2 

h3 

h4 

h5 

h6 

code 

ul 

.sr-only 

.absolute 

.overflow-visible 

.visible 

.fit 

/* Layout */
.page 

.top-hero 

.bottom 

.top-nav 

.content 

.loader 

@keyframes loading-spinner {
  0%,
  100% 
  12.5% 
  25% 
  37.5% 
  50% 
  62.5% 
  75% 
  87.5% 
}

/* LikeButton */
.like-button 

.like-button:focus 

.like-button:active 

.like-button:hover 

.like-button.liked 

/* Icons */
@keyframes circle {
  0% 

  50% 

  to 
}

.circle 

.circle.liked.animate 

.heart 

.heart.liked 

.heart.liked.animate 

.control-icon 

.chevron-left 

/* Video */
.thumbnail 

.thumbnail.blue 

.thumbnail.red 

.thumbnail.green 

.thumbnail.purple 

.thumbnail.yellow 

.thumbnail.gray 

.video 

.video .link 

.video .info 

.video .info:hover 

.video-title 

.video-description 

/* Details */
.details .thumbnail 

.video-details-title 

.video-details-speaker 

.back 

.back:hover 

.info-title 

.info-description 

.controls 

.fallback 

.fallback.title 

.fallback.description 

@keyframes shimmer {
  0% 

  100% 
}

.search 
.search-input 

.search-icon 

.search-input input 

.search-input input:hover, .search-input input:active 

/* Home */
.video-list 

.video-list .videos 
```

```css src/animations.css
/* Slide the fallback down */
::view-transition-old(.slide-down) 

/* Slide the content up */
::view-transition-new(.slide-up) 

/* Define the new keyframes */
@keyframes slide-up {
    from 
    to 
}

@keyframes slide-down {
    from 
    to 
}

/* Previously defined animations below */

/* Animations for view transition classed added by transition type */
::view-transition-old(.slide-forward) 

::view-transition-new(.slide-forward) 

::view-transition-old(.slide-back) 

::view-transition-new(.slide-back) 

/* Keyframes to support our animations above. */
@keyframes fade-in {
    from 
}

@keyframes fade-out {
    to 
}

@keyframes slide-to-right {
    to 
}

@keyframes slide-from-right {
    from 
    to 
}

@keyframes slide-to-left {
    to 
}

@keyframes slide-from-left {
    from 
    to 
}

/* Default .slow-fade. */
::view-transition-old(.slow-fade) 

::view-transition-new(.slow-fade) 
```

```js src/index.js hidden
import React,  from 'react';
import  from 'react-dom/client';
import './styles.css';
import './animations.css';

import App from './App';
import  from './router';

const root = createRoot(document.getElementById('root'));
root.render(
  
  
);
```

```json package.json hidden
{
  "dependencies": ,
  "scripts": 
}
```

### Animating Lists 

You can also use `
  ))}

```

To activate the ViewTransition, we can use `useDeferredValue`:

```js 
const [searchText, setSearchText] = useState('');
const deferredSearchText = useDeferredValue(searchText);
const filteredVideos = filterVideos(videos, deferredSearchText);
```

Now the items animate as you type in the search bar:

  );
}
```

```js src/Details.js hidden
import  from "react";
import  from "./data";
import  from "./Videos";
import  from "./router";
import Layout from "./Layout";
import  from "./Icons";

function VideoDetails() 
    >
      
      
    
  );
}

function VideoInfoFallback() 

export default function Details() {
  const  = useRouter();
  const videoId = url.split("/").pop();
  const video = use(fetchVideo(videoId));

  return (
    
        
  );
}

function VideoInfo() {
  const details = use(fetchVideoDetails(id));
  return (
    <>
      
      
    </>
  );
}
```

```js src/Home.js
import  from "react";import  from "./Videos";import Layout from "./Layout";import  from "./data";import  from "./Icons";

function SearchList() {
  // Activate with useDeferredValue ("when")
  const deferredSearchText = useDeferredValue(searchText);
  const filteredVideos = filterVideos(videos, deferredSearchText);
  return (
    
      
        
      
      
    
  );
}

export default function Home() 

function SearchInput() >
      
        Search
      
      
        
          
          
        
      
      
      
      
    
  );
}
```

```js src/LikeButton.js hidden
import  from 'react';
import  from './Icons';

// A hack since we don't actually have a backend.
// Unlike local state, this survives videos being filtered.
const likedVideos = new Set();

export default function LikeButton() {
  const [isLiked, setIsLiked] = useState(() => likedVideos.has(video.id));
  const [animate, setAnimate] = useState(false);
  return (
     {
        const nextIsLiked = !isLiked;
        if (nextIsLiked)  else 
        setAnimate(true);
        setIsLiked(nextIsLiked);
      }}>
      
  );
}

export function VideoControls() {
  const [isPlaying, setIsPlaying] = useState(false);

  return (
    
        startTransition(() => )
      }
    >
      {isPlaying ? 

        
          
          
        
      
      
  );
}

const RouterContext = createContext({ url: "/", params:  });

export function useRouter() 

export function useIsNavPending() 

```

```css src/styles.css hidden
@font-face 

@font-face 

@font-face 

@font-face 

* 

html 

body 

#root 

h1 

h2 

h3 

h4 

h5 

h6 

code 

ul 

.sr-only 

.absolute 

.overflow-visible 

.visible 

.fit 

/* Layout */
.page 

.top-hero 

.bottom 

.top-nav 

.content 

.loader 

@keyframes loading-spinner {
  0%,
  100% 
  12.5% 
  25% 
  37.5% 
  50% 
  62.5% 
  75% 
  87.5% 
}

/* LikeButton */
.like-button 

.like-button:focus 

.like-button:active 

.like-button:hover 

.like-button.liked 

/* Icons */
@keyframes circle {
  0% 

  50% 

  to 
}

.circle 

.circle.liked.animate 

.heart 

.heart.liked 

.heart.liked.animate 

.control-icon 

.chevron-left 

/* Video */
.thumbnail 

.thumbnail.blue 

.thumbnail.red 

.thumbnail.green 

.thumbnail.purple 

.thumbnail.yellow 

.thumbnail.gray 

.video 

.video .link 

.video .info 

.video .info:hover 

.video-title 

.video-description 

/* Details */
.details .thumbnail 

.video-details-title 

.video-details-speaker 

.back 

.back:hover 

.info-title 

.info-description 

.controls 

.fallback 

.fallback.title 

.fallback.description 

@keyframes shimmer {
  0% 

  100% 
}

.search 
.search-input 

.search-icon 

.search-input input 

.search-input input:hover, .search-input input:active 

/* Home */
.video-list 

.video-list .videos 
```

```css src/animations.css
/* No additional animations needed */

/* Previously defined animations below */

/* Slide animation for Suspense */
::view-transition-old(.slide-down) 

::view-transition-new(.slide-up) 

/* Animations for view transition classed added by transition type */
::view-transition-old(.slide-forward) 

::view-transition-new(.slide-forward) 

::view-transition-old(.slide-back) 

::view-transition-new(.slide-back) 

/* Keyframes to support our animations above. */
@keyframes slide-up {
    from 
    to 
}

@keyframes slide-down {
    from 
    to 
}

@keyframes fade-in {
    from 
}

@keyframes fade-out {
    to 
}

@keyframes slide-to-right {
    to 
}

@keyframes slide-from-right {
    from 
    to 
}

@keyframes slide-to-left {
    to 
}

@keyframes slide-from-left {
    from 
    to 
}

/* Default .slow-fade. */
::view-transition-old(.slow-fade) 

::view-transition-new(.slow-fade) 
```

```js src/index.js hidden
import React,  from 'react';
import  from 'react-dom/client';
import './styles.css';
import './animations.css';

import App from './App';
import  from './router';

const root = createRoot(document.getElementById('root'));
root.render(
  
  
);
```

```json package.json hidden
{
  "dependencies": ,
  "scripts": 
}
```

### Final result 

By adding a few `
  );
}
```

```js src/Details.js
import  from "react"; import  from "./data"; import  from "./Videos"; import  from "./router"; import Layout from "./Layout"; import  from "./Icons";

function VideoDetails() 
    >
      
      
    
  );
}

function VideoInfoFallback() 

export default function Details() {
  const  = useRouter();
  const videoId = url.split("/").pop();
  const video = use(fetchVideo(videoId));

  return (
    
        
  );
}

function VideoInfo() {
  const details = use(fetchVideoDetails(id));
  return (
    <>
      
      
    </>
  );
}
```

```js src/Home.js
import  from "react";import  from "./Videos";import Layout from "./Layout";import  from "./data";import  from "./Icons";

function SearchList() {
  // Activate with useDeferredValue ("when")
  const deferredSearchText = useDeferredValue(searchText);
  const filteredVideos = filterVideos(videos, deferredSearchText);
  return (
    
      
        
      
      
    
  );
}

export default function Home() 

function SearchInput() >
      
        Search
      
      
        
          
          
        
      
      
      
      
    
  );
}
```

```js src/LikeButton.js hidden
import  from 'react';
import  from './Icons';

// A hack since we don't actually have a backend.
// Unlike local state, this survives videos being filtered.
const likedVideos = new Set();

export default function LikeButton() {
  const [isLiked, setIsLiked] = useState(() => likedVideos.has(video.id));
  const [animate, setAnimate] = useState(false);
  return (
     {
        const nextIsLiked = !isLiked;
        if (nextIsLiked)  else 
        setAnimate(true);
        setIsLiked(nextIsLiked);
      }}>
      
  );
}

export function VideoControls() {
  const [isPlaying, setIsPlaying] = useState(false);

  return (
    
        startTransition(() => )
      }
    >
      {isPlaying ? 

        
          
          
        
      
      
  );
}

const RouterContext = createContext({ url: "/", params:  });

export function useRouter() 

export function useIsNavPending() 

```

```css src/styles.css hidden
@font-face 

@font-face 

@font-face 

@font-face 

* 

html 

body 

#root 

h1 

h2 

h3 

h4 

h5 

h6 

code 

ul 

.sr-only 

.absolute 

.overflow-visible 

.visible 

.fit 

/* Layout */
.page 

.top-hero 

.bottom 

.top-nav 

.content 

.loader 

@keyframes loading-spinner {
  0%,
  100% 
  12.5% 
  25% 
  37.5% 
  50% 
  62.5% 
  75% 
  87.5% 
}

/* LikeButton */
.like-button 

.like-button:focus 

.like-button:active 

.like-button:hover 

.like-button.liked 

/* Icons */
@keyframes circle {
  0% 

  50% 

  to 
}

.circle 

.circle.liked.animate 

.heart 

.heart.liked 

.heart.liked.animate 

.control-icon 

.chevron-left 

/* Video */
.thumbnail 

.thumbnail.blue 

.thumbnail.red 

.thumbnail.green 

.thumbnail.purple 

.thumbnail.yellow 

.thumbnail.gray 

.video 

.video .link 

.video .info 

.video .info:hover 

.video-title 

.video-description 

/* Details */
.details .thumbnail 

.video-details-title 

.video-details-speaker 

.back 

.back:hover 

.info-title 

.info-description 

.controls 

.fallback 

.fallback.title 

.fallback.description 

@keyframes shimmer {
  0% 

  100% 
}

.search 
.search-input 

.search-icon 

.search-input input 

.search-input input:hover, .search-input input:active 

/* Home */
.video-list 

.video-list .videos 
```

```css src/animations.css
/* Slide animations for Suspense the fallback down */
::view-transition-old(.slide-down) 

::view-transition-new(.slide-up) 

/* Animations for view transition classed added by transition type */
::view-transition-old(.slide-forward) 

::view-transition-new(.slide-forward) 

::view-transition-old(.slide-back) 

::view-transition-new(.slide-back) 

/* Keyframes to support our animations above. */
@keyframes slide-up {
    from 
    to 
}

@keyframes slide-down {
    from 
    to 
}

@keyframes fade-in {
    from 
}

@keyframes fade-out {
    to 
}

@keyframes slide-to-right {
    to 
}

@keyframes slide-from-right {
    from 
    to 
}

@keyframes slide-to-left {
    to 
}

@keyframes slide-from-left {
    from 
    to 
}
```

```js src/index.js hidden
import React,  from 'react';
import  from 'react-dom/client';
import './styles.css';
import './animations.css';

import App from './App';
import  from './router';

const root = createRoot(document.getElementById('root'));
root.render(
  
  
);
```

```json package.json hidden
{
  "dependencies": ,
  "scripts": 
}
```

If you're curious to know more about how they work, check out [How Does `

In [past](/blog/2022/06/15/react-labs-what-we-have-been-working-on-june-2022#offscreen) [updates](/blog/2024/02/15/react-labs-what-we-have-been-working-on-february-2024#offscreen-renamed-to-activity), we shared that we were researching an API to allow components to be visually hidden and deprioritized, preserving UI state with reduced performance costs relative to unmounting or hiding with CSS.

We're now ready to share the API and how it works, so you can start testing it in experimental React versions.

`
```

When an Activity is  it's rendered as normal. When an Activity is  it is unmounted, but will save its state and continue to render at a lower priority than anything visible on screen.

You can use `Activity` to save state for parts of the UI the user isn't using, or pre-render parts that a user is likely to use next.

Let's look at some examples improving the View Transition examples above.

### Restoring state with Activity 

When a user navigates away from a page, it's common to stop rendering the old page:

```js 
function App() {
  const  = useRouter();

  return (
    <>
      {url === '/' && 
      {url !== '/' && 
      
```

```js src/Details.js hidden
import  from "react";
import  from "./data";
import  from "./Videos";
import  from "./router";
import Layout from "./Layout";
import  from "./Icons";

function VideoDetails() 
    >
      
      
    
  );
}

function VideoInfoFallback() 

export default function Details() {
  const  = useRouter();
  const videoId = url.split("/").pop();
  const video = use(fetchVideo(videoId));

  return (
    
        
  );
}

function VideoInfo() {
  const details = use(fetchVideoDetails(id));
  return (
    <>
      
      
    </>
  );
}
```

```js src/Home.js hidden
import  from "react";import  from "./Videos";import Layout from "./Layout";import  from "./data";import  from "./Icons";

function SearchList() {
  // Activate with useDeferredValue ("when")
  const deferredSearchText = useDeferredValue(searchText);
  const filteredVideos = filterVideos(videos, deferredSearchText);
  return (
    
      
      
        
      
    
  );
}

export default function Home() 

function SearchInput() >
      
        Search
      
      
        
          
          
        
      
      
      
      
    
  );
}
```

```js src/LikeButton.js hidden
import  from 'react';
import  from './Icons';

// A hack since we don't actually have a backend.
// Unlike local state, this survives videos being filtered.
const likedVideos = new Set();

export default function LikeButton() {
  const [isLiked, setIsLiked] = useState(() => likedVideos.has(video.id));
  const [animate, setAnimate] = useState(false);
  return (
     {
        const nextIsLiked = !isLiked;
        if (nextIsLiked)  else 
        setAnimate(true);
        setIsLiked(nextIsLiked);
      }}>
      
  );
}

export function VideoControls() {
  const [isPlaying, setIsPlaying] = useState(false);

  return (
    
        startTransition(() => )
      }
    >
      {isPlaying ? 

        
          
          
        
      
      
  );
}

const RouterContext = createContext({ url: "/", params:  });

export function useRouter() 

export function useIsNavPending() 

```

```css src/styles.css hidden
@font-face 

@font-face 

@font-face 

@font-face 

* 

html 

body 

#root 

h1 

h2 

h3 

h4 

h5 

h6 

code 

ul 

.sr-only 

.absolute 

.overflow-visible 

.visible 

.fit 

/* Layout */
.page 

.top-hero 

.bottom 

.top-nav 

.content 

.loader 

@keyframes loading-spinner {
  0%,
  100% 
  12.5% 
  25% 
  37.5% 
  50% 
  62.5% 
  75% 
  87.5% 
}

/* LikeButton */
.like-button 

.like-button:focus 

.like-button:active 

.like-button:hover 

.like-button.liked 

/* Icons */
@keyframes circle {
  0% 

  50% 

  to 
}

.circle 

.circle.liked.animate 

.heart 

.heart.liked 

.heart.liked.animate 

.control-icon 

.chevron-left 

/* Video */
.thumbnail 

.thumbnail.blue 

.thumbnail.red 

.thumbnail.green 

.thumbnail.purple 

.thumbnail.yellow 

.thumbnail.gray 

.video 

.video .link 

.video .info 

.video .info:hover 

.video-title 

.video-description 

/* Details */
.details .thumbnail 

.video-details-title 

.video-details-speaker 

.back 

.back:hover 

.info-title 

.info-description 

.controls 

.fallback 

.fallback.title 

.fallback.description 

@keyframes shimmer {
  0% 

  100% 
}

.search 
.search-input 

.search-icon 

.search-input input 

.search-input input:hover, .search-input input:active 

/* Home */
.video-list 

.video-list .videos 
```

```css src/animations.css
/* No additional animations needed */

/* Previously defined animations below */

/* Slide animations for Suspense the fallback down */
::view-transition-old(.slide-down) 

::view-transition-new(.slide-up) 

/* Animations for view transition classed added by transition type */
::view-transition-old(.slide-forward) 

::view-transition-new(.slide-forward) 

::view-transition-old(.slide-back) 

::view-transition-new(.slide-back) 

/* Keyframes to support our animations above. */
@keyframes slide-up {
    from 
    to 
}

@keyframes slide-down {
    from 
    to 
}

@keyframes fade-in {
    from 
}

@keyframes fade-out {
    to 
}

@keyframes slide-to-right {
    to 
}

@keyframes slide-from-right {
    from 
    to 
}

@keyframes slide-to-left {
    to 
}

@keyframes slide-from-left {
    from 
    to 
}

/* Default .slow-fade. */
::view-transition-old(.slow-fade) 

::view-transition-new(.slow-fade) 
```

```js src/index.js hidden
import React,  from 'react';
import  from 'react-dom/client';
import './styles.css';
import './animations.css';

import App from './App';
import  from './router';

const root = createRoot(document.getElementById('root'));
root.render(
  
  
);
```

```json package.json hidden
{
  "dependencies": ,
  "scripts": 
}
```

### Pre-rendering with Activity 

Sometimes, you may want to prepare the next part of the UI a user is likely to use ahead of time, so it's ready by the time they are ready to use it. This is especially useful if the next route needs to suspend on data it needs to render, because you can help ensure the data is already fetched before the user navigates.

For example, our app currently needs to suspend to load the data for each video when you select one. We can improve this by rendering all of the pages in a hidden `
  
  

      ))}
      
    
  );
}
```

```js src/Details.js
import  from "react"; import  from "./data"; import  from "./Videos"; import  from "./router"; import Layout from "./Layout"; import  from "./Icons";

function VideoDetails() 
    >
      
      
    
  );
}

function VideoInfoFallback() 

export default function Details() {
  const  = useRouter();
  const video = use(fetchVideo(id));

  return (
    
        
  );
}

function VideoInfo() {
  const details = use(fetchVideoDetails(id));
  return (
    <>
      
      
    </>
  );
}
```

```js src/Home.js hidden
import  from "react";import  from "./Videos";import Layout from "./Layout";import  from "./data";import  from "./Icons";

function SearchList() {
  // Activate with useDeferredValue ("when")
  const deferredSearchText = useDeferredValue(searchText);
  const filteredVideos = filterVideos(videos, deferredSearchText);
  return (
    
      
      
        
      
    
  );
}

export default function Home() 

function SearchInput() >
      
        Search
      
      
        
          
          
        
      
      
      
      
    
  );
}
```

```js src/LikeButton.js hidden
import  from 'react';
import  from './Icons';

// A hack since we don't actually have a backend.
// Unlike local state, this survives videos being filtered.
const likedVideos = new Set();

export default function LikeButton() {
  const [isLiked, setIsLiked] = useState(() => likedVideos.has(video.id));
  const [animate, setAnimate] = useState(false);
  return (
     {
        const nextIsLiked = !isLiked;
        if (nextIsLiked)  else 
        setAnimate(true);
        setIsLiked(nextIsLiked);
      }}>
      
  );
}

export function VideoControls() {
  const [isPlaying, setIsPlaying] = useState(false);

  return (
    
        startTransition(() => )
      }
    >
      {isPlaying ? 

        
          
          
        
      
      
  );
}

const RouterContext = createContext({ url: "/", params:  });

export function useRouter() 

export function useIsNavPending() 

```

```css src/styles.css hidden
@font-face 

@font-face 

@font-face 

@font-face 

* 

html 

body 

#root 

h1 

h2 

h3 

h4 

h5 

h6 

code 

ul 

.sr-only 

.absolute 

.overflow-visible 

.visible 

.fit 

/* Layout */
.page 

.top-hero 

.bottom 

.top-nav 

.content 

.loader 

@keyframes loading-spinner {
  0%,
  100% 
  12.5% 
  25% 
  37.5% 
  50% 
  62.5% 
  75% 
  87.5% 
}

/* LikeButton */
.like-button 

.like-button:focus 

.like-button:active 

.like-button:hover 

.like-button.liked 

/* Icons */
@keyframes circle {
  0% 

  50% 

  to 
}

.circle 

.circle.liked.animate 

.heart 

.heart.liked 

.heart.liked.animate 

.control-icon 

.chevron-left 

/* Video */
.thumbnail 

.thumbnail.blue 

.thumbnail.red 

.thumbnail.green 

.thumbnail.purple 

.thumbnail.yellow 

.thumbnail.gray 

.video 

.video .link 

.video .info 

.video .info:hover 

.video-title 

.video-description 

/* Details */
.details .thumbnail 

.video-details-title 

.video-details-speaker 

.back 

.back:hover 

.info-title 

.info-description 

.controls 

.fallback 

.fallback.title 

.fallback.description 

@keyframes shimmer {
  0% 

  100% 
}

.search 
.search-input 

.search-icon 

.search-input input 

.search-input input:hover, .search-input input:active 

/* Home */
.video-list 

.video-list .videos 
```

```css src/animations.css
/* No additional animations needed */

/* Previously defined animations below */

/* Slide animations for Suspense the fallback down */
::view-transition-old(.slide-down) 

::view-transition-new(.slide-up) 

/* Animations for view transition classed added by transition type */
::view-transition-old(.slide-forward) 

::view-transition-new(.slide-forward) 

::view-transition-old(.slide-back) 

::view-transition-new(.slide-back) 

/* Keyframes to support our animations above. */
@keyframes slide-up {
    from 
    to 
}

@keyframes slide-down {
    from 
    to 
}

@keyframes fade-in {
    from 
}

@keyframes fade-out {
    to 
}

@keyframes slide-to-right {
    to 
}

@keyframes slide-from-right {
    from 
    to 
}

@keyframes slide-to-left {
    to 
}

@keyframes slide-from-left {
    from 
    to 
}

/* Default .slow-fade. */
::view-transition-old(.slow-fade) 

::view-transition-new(.slow-fade) 
```

```js src/index.js hidden
import React,  from 'react';
import  from 'react-dom/client';
import './styles.css';
import './animations.css';

import App from './App';
import  from './router';

const root = createRoot(document.getElementById('root'));
root.render(
  
  
);
```

```json package.json hidden
{
  "dependencies": ,
  "scripts": 
}
```

### Server-Side Rendering with Activity 

When using Activity on a page that uses server-side rendering (SSR), there are additional optimizations.

If part of the page is rendered with `mode="hidden"`, then it will not be included in the SSR response. Instead, React will schedule a client render for the content inside Activity while the rest of the page hydrates, prioritizing the visible content on screen.

For parts of the UI rendered with `mode="visible"`, React will de-prioritize hydration of content within Activity, similar to how Suspense content is hydrated at a lower priority. If the user interacts with the page, we'll prioritize hydration within the boundary if needed.

These are advanced use cases, but they show the additional benefits considered with Activity.

### Future modes for Activity 

In the future, we may add more modes to Activity.

For example, a common use case is rendering a modal, where the previous "inactive" page is visible behind the "active" modal view. The "hidden" mode does not work for this use case because it's not visible and not included in SSR.

Instead, we're considering a new mode that would keep the content visible&mdash;and included in SSR&mdash;but keep it unmounted and de-prioritize updates. This mode may also need to "pause" DOM updates, since it can be distracting to see backgrounded content updating while a modal is open.

Another mode we're considering for Activity is the ability to automatically destroy state for hidden Activities if there is too much memory being used. Since the component is already unmounted, it may be preferable to destroy state for the least recently used hidden parts of the app rather than consume too many resources.

These are areas we're still exploring, and we'll share more as we make progress. For more information on what Activity includes today, [check out the docs](/reference/react/Activity).

---

# Features in development 

We're also developing features to help solve the common problems below.

As we iterate on possible solutions, you may see some potential APIs we're testing being shared based on the PRs we are landing. Please keep in mind that as we try different ideas, we often change or remove different solutions after trying them out.

When the solutions we're working on are shared too early, it can create churn and confusion in the community. To balance being transparent and limiting confusion, we're sharing the problems we're currently developing solutions for, without sharing a particular solution we have in mind.

As these features progress, we'll announce them on the blog with docs included so you can try them out.

## React Performance Tracks 

We're working on a new set of custom tracks to performance profilers using browser APIs that [allow adding custom tracks](https://developer.chrome.com/docs/devtools/performance/extension) to provide more information about the performance of your React app.

This feature is still in progress, so we're not ready to publish docs to fully release it as an experimental feature yet. You can get a sneak preview when using an experimental version of React, which will automatically add the performance tracks to profiles:

  
      
      
  
  
      
      
  

There are a few known issues we plan to address such as performance, and the scheduler track not always "connecting" work across Suspended trees, so it's not quite ready to try. We're also still collecting feedback from early adopters to improve the design and usability of the tracks.

Once we solve those issues, we'll publish experimental docs and share that it's ready to try.

---

## Automatic Effect Dependencies 

When we released hooks, we had three motivations:

- **Sharing code between components**: hooks replaced patterns like render props and higher-order components to allow you to reuse stateful logic without changing your component hierarchy.
- **Think in terms of function, not lifecycles**: hooks let you split one component into smaller functions based on what pieces are related (such as setting up a subscription or fetching data), rather than forcing a split based on lifecycle methods.
- **Support ahead-of-time compilation**: hooks were designed to support ahead-of-time compilation with less pitfalls causing unintentional de-optimizations caused by lifecycle methods, and limitations of classes.

Since their release, hooks have been successful at *sharing code between components*. Hooks are now the favored way to share logic between components, and there are less use cases for render props and higher order components. Hooks have also been successful at supporting features like Fast Refresh that were not possible with class components.

### Effects can be hard 

Unfortunately, some hooks are still hard to think in terms of function instead of lifecycles. Effects specifically are still hard to understand and are the most common pain point we hear from developers. Last year, we spent a significant amount of time researching how Effects were used, and how those use cases could be simplified and easier to understand.

We found that often, the confusion is from using an Effect when you don't need to. The [You Might Not Need an Effect](/learn/you-might-not-need-an-effect) guide covers many cases for when Effects are not the right solution. However, even when an Effect is the right fit for a problem, Effects can still be harder to understand than class component lifecycles.

We believe one of the reasons for confusion is that developers to think of Effects from the _component's_ perspective (like a lifecycle), instead of the _Effects_ point of view (what the Effect does).

Let's look at an example [from the docs](/learn/lifecycle-of-reactive-effects#thinking-from-the-effects-perspective):

```js
useEffect(() => {
  // Your Effect connected to the room specified with roomId...
  const connection = createConnection(serverUrl, roomId);
  connection.connect();
  return () => ;
}, [roomId]);
```

Many users would read this code as "on mount, connect to the roomId. whenever `roomId` changes, disconnect to the old room and re-create the connection". However, this is thinking from the component's lifecycle perspective, which means you will need to think of every component lifecycle state to write the Effect correctly. This can be difficult, so it's understandable that Effects seem harder than class lifecycles when using the component perspective.

### Effects without dependencies 

Instead, it's better to think from the Effect's perspective. The Effect doesn't know about the component lifecycles. It only describes how to start synchronization and how to stop it. When users think of Effects in this way, their Effects tend to be easier to write, and more resilient to being started and stopped as many times as is needed.

We spent some time researching why Effects are thought of from the component perspective, and we think one of the reasons is the dependency array. Since you have to write it, it's right there and in your face reminding you of what you're "reacting" to and baiting you into the mental model of 'do this when these values change'.

When we released hooks, we knew we could make them easier to use with ahead-of-time compilation. With the React Compiler, you're now able to avoid writing `useCallback` and `useMemo` yourself in most cases. For Effects, the compiler can insert the dependencies for you:

```js
useEffect(() => {
  const connection = createConnection(serverUrl, roomId);
  connection.connect();
  return () => ;
}); // compiler inserted dependencies.
```

With this code, the React Compiler can infer the dependencies for you and insert them automatically so you don't need to see or write them. With features like [the IDE extension](#compiler-ide-extension) and [`useEffectEvent`](/reference/react/useEffectEvent), we can provide a CodeLens to show you what the Compiler inserted for times you need to debug, or to optimize by removing a dependency. This helps reinforce the correct mental model for writing Effects, which can run at any time to synchronize your component or hook's state with something else.

Our hope is that automatically inserting dependencies is not only easier to write, but that it also makes them easier to understand by forcing you to think in terms of what the Effect does, and not in component lifecycles.

---

## Compiler IDE Extension 

Later in 2025 [we shared](/blog/2025/10/07/react-compiler-1) the first stable release of React Compiler, and we're continuing to invest in shipping more improvements.

We've also begun exploring ways to use the React Compiler to provide information that can improve understanding and debugging your code. One idea we've started exploring is a new experimental LSP-based React IDE extension powered by React Compiler, similar to the extension used in [Lauren Tan's React Conf talk](https://conf2024.react.dev/talks/5).

Our idea is that we can use the compiler's static analysis to provide more information, suggestions, and optimization opportunities directly in your IDE. For example, we can provide diagnostics for code breaking the Rules of React, hovers to show if components and hooks were optimized by the compiler, or a CodeLens to see [automatically inserted Effect dependencies](#automatic-effect-dependencies).

The IDE extension is still an early exploration, but we'll share our progress in future updates.

---

## Fragment Refs 

Many DOM APIs like those for event management, positioning, and focus are difficult to compose when writing with React. This often leads developers to reach for Effects, managing multiple Refs, by using APIs like `findDOMNode` (removed in React 19).

We are exploring adding refs to Fragments that would point to a group of DOM elements, rather than just a single element. Our hope is that this will simplify managing multiple children and make it easier to write composable React code when calling DOM APIs.

Fragment refs are still being researched. We'll share more when we're closer to having the final API finished.

---

## Gesture Animations 

We're also researching ways to enhance View Transitions to support gesture animations such as swiping to open a menu, or scroll through a photo carousel.

Gestures present new challenges for a few reasons:

- **Gestures are continuous**: as you swipe the animation is tied to your finger placement time, rather than triggering and running to completion.
- **Gestures don't complete**: when you release your finger gesture animations can run to completion, or revert to their original state (like when you only partially open a menu) depending on how far you go.
- **Gestures invert old and new**: while you're animating, you want the page you are animating from to stay "alive" and interactive. This inverts the browser View Transition model where the "old" state is a snapshot and the "new" state is the live DOM.

We believe we’ve found an approach that works well and may introduce a new API for triggering gesture transitions. For now, we're focused on shipping ``, and will revisit gestures afterward.

---

## Concurrent Stores 

When we released React 18 with concurrent rendering, we also released `useSyncExternalStore` so external store libraries that did not use React state or context could [support concurrent rendering](https://github.com/reactwg/react-18/discussions/70) by forcing a synchronous render when the store is updated.

Using `useSyncExternalStore` comes at a cost though, since it forces a bail out from concurrent features like transitions, and forces existing content to show Suspense fallbacks.

Now that React 19 has shipped, we're revisiting this problem space to create a primitive to fully support concurrent external stores with the `use` API:

```js
const value = use(store);
```

Our goal is to allow external state to be read during render without tearing, and to work seamlessly with all of the concurrent features React offers.

This research is still early. We'll share more, and what the new APIs will look like, when we're further along.

---

_Thanks to [Aurora Scharff](https://bsky.app/profile/aurorascharff.no), [Dan Abramov](https://bsky.app/profile/danabra.mov), [Eli White](https://twitter.com/Eli_White), [Lauren Tan](https://bsky.app/profile/no.lol), [Luna Wei](https://github.com/lunaleaps), [Matt Carroll](https://twitter.com/mattcarrollcode), [Jack Pope](https://jackpope.me), [Jason Bonta](https://threads.net/someextent), [Jordan Brown](https://github.com/jbrown215), [Jordan Eldredge](https://bsky.app/profile/capt.dev), [Mofei Zhang](https://threads.net/z_mofei), [Sebastien Lorber](https://bsky.app/profile/sebastienlorber.com), [Sebastian Markbåge](https://bsky.app/profile/sebmarkbage.calyptus.eu), and [Tim Yung](https://github.com/yungsters) for reviewing this post._
