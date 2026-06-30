---
title: "React DOM Components"
---

---

## Common components 

All of the built-in browser components support some props and events.

* [Common components (e.g. ``)](/reference/react-dom/components/common)

This includes React-specific props like `ref` and `dangerouslySetInnerHTML`.

---

## Form components 

These built-in browser components accept user input:

* [``](/reference/react-dom/components/input)
* [``](/reference/react-dom/components/select)
* [``](/reference/react-dom/components/textarea)

They are special in React because passing the `value` prop to them makes them *[controlled.](/reference/react-dom/components/input#controlling-an-input-with-a-state-variable)*

---

## Resource and Metadata Components 

These built-in browser components let you load external resources or annotate the document with metadata:

* [``](/reference/react-dom/components/link)
* [``](/reference/react-dom/components/meta)
* [``](/reference/react-dom/components/script)
* [``](/reference/react-dom/components/style)
* [``](/reference/react-dom/components/title)

They are special in React because React can render them into the document head, suspend while resources are loading, and enact other behaviors that are described on the reference page for each specific component.

---

## All HTML components 

React supports all built-in browser HTML components. This includes:

* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/aside)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/audio)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/b)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/base)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/bdi)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/bdo)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/blockquote)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/body)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/br)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/button)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/canvas)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/caption)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/cite)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/code)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/col)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/colgroup)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/data)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/datalist)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/dd)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/del)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/details)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/dfn)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/dialog)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/div)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/dl)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/dt)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/em)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/embed)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/fieldset)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/figcaption)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/figure)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/footer)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/form)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/h1)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/head)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/header)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/hgroup)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/hr)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/html)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/i)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/iframe)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/img)
* [``](/reference/react-dom/components/input)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/ins)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/kbd)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/label)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/legend)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/li)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/link)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/main)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/map)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/mark)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/menu)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/meta)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/meter)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/nav)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/noscript)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/object)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/ol)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/optgroup)
* [``](/reference/react-dom/components/option)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/output)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/p)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/picture)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/pre)
* [``](/reference/react-dom/components/progress)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/q)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/rp)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/rt)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/ruby)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/s)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/samp)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/script)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/section)
* [``](/reference/react-dom/components/select)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/slot)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/small)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/source)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/span)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/strong)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/style)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/sub)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/summary)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/sup)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/table)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/tbody)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/td)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/template)
* [``](/reference/react-dom/components/textarea)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/tfoot)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/th)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/thead)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/time)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/title)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/tr)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/track)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/u)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/ul)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/var)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/video)
* [``](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/wbr)

---

### Custom HTML elements 

If you render a tag with a dash, like ``, React will assume you want to render a [custom HTML element.](https://developer.mozilla.org/en-US/docs/Web/Web_Components/Using_custom_elements)

If you render a built-in browser HTML element with an [`is`](https://developer.mozilla.org/en-US/docs/Web/HTML/Global_attributes/is) attribute, it will also be treated as a custom element.

#### Setting values on custom elements 

Custom elements have two methods of passing data into them:

1) Attributes: Which are displayed in markup and can only be set to string values
2) Properties: Which are not displayed in markup and can be set to arbitrary JavaScript values

By default, React will pass values bound in JSX as attributes:

```jsx

```

Non-string JavaScript values passed to custom elements will be serialized by default:

```jsx
// Will be passed as `"1,2,3"` as the output of `[1,2,3].toString()`

```

React will, however, recognize an custom element's property as one that it may pass arbitrary values to if the property name shows up on the class during construction:

#### Listening for events on custom elements 

A common pattern when using custom elements is that they may dispatch [`CustomEvent`s](https://developer.mozilla.org/en-US/docs/Web/API/CustomEvent) rather than accept a function to call when an event occur. You can listen for these events using an `on` prefix when binding to the event via JSX.

---

## All SVG components 

React supports all built-in browser SVG components. This includes:

* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/a)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/animate)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/animateMotion)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/animateTransform)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/circle)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/clipPath)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/defs)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/desc)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/discard)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/ellipse)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/feBlend)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/feColorMatrix)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/feComponentTransfer)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/feComposite)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/feConvolveMatrix)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/feDiffuseLighting)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/feDisplacementMap)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/feDistantLight)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/feDropShadow)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/feFlood)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/feFuncA)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/feFuncB)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/feFuncG)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/feFuncR)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/feGaussianBlur)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/feImage)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/feMerge)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/feMergeNode)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/feMorphology)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/feOffset)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/fePointLight)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/feSpecularLighting)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/feSpotLight)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/feTile)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/feTurbulence)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/filter)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/foreignObject)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/g)
* ``
* ``
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/image)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/line)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/linearGradient)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/marker)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/mask)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/metadata)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/mpath)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/path)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/pattern)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/polygon)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/polyline)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/radialGradient)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/rect)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/script)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/set)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/stop)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/style)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/svg)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/switch)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/symbol)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/text)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/textPath)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/title)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/tspan)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/use)
* [``](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/view)

