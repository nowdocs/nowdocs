---
title: set-state-in-effect
---

## Rule Details 

Setting state immediately inside an effect forces React to restart the entire render cycle. When you update state in an effect, React must re-render your component, apply changes to the DOM, and then run effects again. This creates an extra render pass that could have been avoided by transforming data directly during render or deriving state from props. Transform data at the top level of your component instead. This code will naturally re-run when props or state change without triggering additional render cycles.

Synchronous `setState` calls in effects trigger immediate re-renders before the browser can paint, causing performance issues and visual jank. React has to render twice: once to apply the state update, then again after effects run. This double rendering is wasteful when the same result could be achieved with a single render.

In many cases, you may also not need an effect at all. Please see [You Might Not Need an Effect](/learn/you-might-not-need-an-effect) for more information.

## Common Violations 

This rule catches several patterns where synchronous setState is used unnecessarily:

- Setting loading state synchronously
- Deriving state from props in effects
- Transforming data in effects instead of render

### Invalid 

Examples of incorrect code for this rule:

```js
// ❌ Synchronous setState in effect
function Component() {
  const [items, setItems] = useState([]);

  useEffect(() => , [data]);
}

// ❌ Setting loading state synchronously
function Component() {
  const [loading, setLoading] = useState(false);

  useEffect(() => , []);
}

// ❌ Transforming data in effect
function Component() {
  const [processed, setProcessed] = useState([]);

  useEffect(() => , [rawData]);
}

// ❌ Deriving state from props
function Component() {
  const [selected, setSelected] = useState(null);

  useEffect(() => , [selectedId, items]);
}
```

### Valid 

Examples of correct code for this rule:

```js
// ✅ setState in an effect is fine if the value comes from a ref
function Tooltip() {
  const ref = useRef(null);
  const [tooltipHeight, setTooltipHeight] = useState(0);

  useLayoutEffect(() => {
    const  = ref.current.getBoundingClientRect();
    setTooltipHeight(height);
  }, []);
}

// ✅ Calculate during render
function Component() {
  const selected = items.find(i => i.id === selectedId);
  return ;
}
```

**When something can be calculated from the existing props or state, don't put it in state.** Instead, calculate it during rendering. This makes your code faster, simpler, and less error-prone. Learn more in [You Might Not Need an Effect](/learn/you-might-not-need-an-effect).
