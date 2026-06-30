---
title: useLayoutEffect
---

      
      
      
      
    
  );
}
```

```js src/ButtonWithTooltip.js
import  from 'react';
import Tooltip from './Tooltip.js';

export default function ButtonWithTooltip() {
  const [targetRect, setTargetRect] = useState(null);
  const buttonRef = useRef(null);
  return (
    <>
       {
          const rect = buttonRef.current.getBoundingClientRect();
          setTargetRect();
        }}
        onPointerLeave={() => }
      />
      
    </>
  );
}
```

```js src/Tooltip.js active
import  from 'react';
import  from 'react-dom';
import TooltipContainer from './TooltipContainer.js';

export default function Tooltip() {
  const ref = useRef(null);
  const [tooltipHeight, setTooltipHeight] = useState(0);

  useLayoutEffect(() => {
    const  = ref.current.getBoundingClientRect();
    setTooltipHeight(height);
    console.log('Measured tooltip height: ' + height);
  }, []);

  let tooltipX = 0;
  let tooltipY = 0;
  if (targetRect !== null) {
    tooltipX = targetRect.left;
    tooltipY = targetRect.top - tooltipHeight;
    if (tooltipY < 0) 
  }

  return createPortal(
    ,
    document.body
  );
}
```

```js src/TooltipContainer.js
export default function TooltipContainer() {
  return (
    
      
        
      
    
  );
}
```

```css
.tooltip 
```

Notice that even though the `Tooltip` component has to render in two passes (first, with `tooltipHeight` initialized to `0` and then with the real measured height), you only see the final result. This is why you need `useLayoutEffect` instead of [`useEffect`](/reference/react/useEffect) for this example. Let's look at the difference in detail below.

      
      
      
      
    
  );
}
```

```js src/ButtonWithTooltip.js
import  from 'react';
import Tooltip from './Tooltip.js';

export default function ButtonWithTooltip() {
  const [targetRect, setTargetRect] = useState(null);
  const buttonRef = useRef(null);
  return (
    <>
       {
          const rect = buttonRef.current.getBoundingClientRect();
          setTargetRect();
        }}
        onPointerLeave={() => }
      />
      
    </>
  );
}
```

```js src/Tooltip.js active
import  from 'react';
import  from 'react-dom';
import TooltipContainer from './TooltipContainer.js';

export default function Tooltip() {
  const ref = useRef(null);
  const [tooltipHeight, setTooltipHeight] = useState(0);

  useLayoutEffect(() => {
    const  = ref.current.getBoundingClientRect();
    setTooltipHeight(height);
  }, []);

  let tooltipX = 0;
  let tooltipY = 0;
  if (targetRect !== null) {
    tooltipX = targetRect.left;
    tooltipY = targetRect.top - tooltipHeight;
    if (tooltipY < 0) 
  }

  return createPortal(
    ,
    document.body
  );
}
```

```js src/TooltipContainer.js
export default function TooltipContainer() {
  return (
    
      
        
      
    
  );
}
```

```css
.tooltip 
```

      
      
      
      
    
  );
}
```

```js src/ButtonWithTooltip.js
import  from 'react';
import Tooltip from './Tooltip.js';

export default function ButtonWithTooltip() {
  const [targetRect, setTargetRect] = useState(null);
  const buttonRef = useRef(null);
  return (
    <>
       {
          const rect = buttonRef.current.getBoundingClientRect();
          setTargetRect();
        }}
        onPointerLeave={() => }
      />
      
    </>
  );
}
```

```js src/Tooltip.js active
import  from 'react';
import  from 'react-dom';
import TooltipContainer from './TooltipContainer.js';

export default function Tooltip() {
  const ref = useRef(null);
  const [tooltipHeight, setTooltipHeight] = useState(0);

  useEffect(() => {
    const  = ref.current.getBoundingClientRect();
    setTooltipHeight(height);
  }, []);

  let tooltipX = 0;
  let tooltipY = 0;
  if (targetRect !== null) {
    tooltipX = targetRect.left;
    tooltipY = targetRect.top - tooltipHeight;
    if (tooltipY < 0) 
  }

  return createPortal(
    ,
    document.body
  );
}
```

```js src/TooltipContainer.js
export default function TooltipContainer() {
  return (
    
      
        
      
    
  );
}
```

```css
.tooltip 
```

To make the bug easier to reproduce, this version adds an artificial delay during rendering. React will let the browser paint the screen before it processes the state update inside `useEffect`. As a result, the tooltip flickers:

      
      
      
      
    
  );
}
```

```js src/ButtonWithTooltip.js
import  from 'react';
import Tooltip from './Tooltip.js';

export default function ButtonWithTooltip() {
  const [targetRect, setTargetRect] = useState(null);
  const buttonRef = useRef(null);
  return (
    <>
       {
          const rect = buttonRef.current.getBoundingClientRect();
          setTargetRect();
        }}
        onPointerLeave={() => }
      />
      
    </>
  );
}
```

```js {expectedErrors: } src/Tooltip.js active
import  from 'react';
import  from 'react-dom';
import TooltipContainer from './TooltipContainer.js';

export default function Tooltip() {
  const ref = useRef(null);
  const [tooltipHeight, setTooltipHeight] = useState(0);

  // This artificially slows down rendering
  let now = performance.now();
  while (performance.now() - now < 100) 

  useEffect(() => {
    const  = ref.current.getBoundingClientRect();
    setTooltipHeight(height);
  }, []);

  let tooltipX = 0;
  let tooltipY = 0;
  if (targetRect !== null) {
    tooltipX = targetRect.left;
    tooltipY = targetRect.top - tooltipHeight;
    if (tooltipY < 0) 
  }

  return createPortal(
    ,
    document.body
  );
}
```

```js src/TooltipContainer.js
export default function TooltipContainer() {
  return (
    
      
        
      
    
  );
}
```

```css
.tooltip 
```

Edit this example to `useLayoutEffect` and observe that it blocks the paint even if rendering is slowed down.

---

## Troubleshooting 

### I'm getting an error: "`useLayoutEffect` does nothing on the server" 

The purpose of `useLayoutEffect` is to let your component [use layout information for rendering:](#measuring-layout-before-the-browser-repaints-the-screen)

1. Render the initial content.
2. Measure the layout *before the browser repaints the screen.*
3. Render the final content using the layout information you've read.

When you or your framework uses [server rendering](/reference/react-dom/server), your React app renders to HTML on the server for the initial render. This lets you show the initial HTML before the JavaScript code loads.

The problem is that on the server, there is no layout information.

In the [earlier example](#measuring-layout-before-the-browser-repaints-the-screen), the `useLayoutEffect` call in the `Tooltip` component lets it position itself correctly (either above or below content) depending on the content height. If you tried to render `Tooltip` as a part of the initial server HTML, this would be impossible to determine. On the server, there is no layout yet! So, even if you rendered it on the server, its position would "jump" on the client after the JavaScript loads and runs.

Usually, components that rely on layout information don't need to render on the server anyway. For example, it probably doesn't make sense to show a `Tooltip` during the initial render. It is triggered by a client interaction.

However, if you're running into this problem, you have a few different options:

- Replace `useLayoutEffect` with [`useEffect`.](/reference/react/useEffect) This tells React that it's okay to display the initial render result without blocking the paint (because the original HTML will become visible before your Effect runs).

- Alternatively, [mark your component as client-only.](/reference/react/Suspense#providing-a-fallback-for-server-errors-and-client-only-content) This tells React to replace its content up to the closest [``](/reference/react/Suspense) boundary with a loading fallback (for example, a spinner or a glimmer) during server rendering.

- Alternatively, you can render a component with `useLayoutEffect` only after hydration. Keep a boolean `isMounted` state that's initialized to `false`, and set it to `true` inside a `useEffect` call. Your rendering logic can then be like `return isMounted ?  : `. On the server and during the hydration, the user will see `FallbackContent` which should not call `useLayoutEffect`. Then React will replace it with `RealContent` which runs on the client only and can include `useLayoutEffect` calls.

- If you synchronize your component with an external data store and rely on `useLayoutEffect` for different reasons than measuring layout, consider [`useSyncExternalStore`](/reference/react/useSyncExternalStore) instead which [supports server rendering.](/reference/react/useSyncExternalStore#adding-support-for-server-rendering)
