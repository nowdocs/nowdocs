---
title: prefetchDNS
---

---

## Reference 

### `prefetchDNS(href)` 

To look up a host, call the `prefetchDNS` function from `react-dom`.

```js
import  from 'react-dom';

function AppRoot() 

```

[See more examples below.](#usage)

The prefetchDNS function provides the browser with a hint that it should look up the IP address of a given server. If the browser chooses to do so, this can speed up the loading of resources from that server.

#### Parameters 

* `href`: a string. The URL of the server you want to connect to.

#### Returns 

`prefetchDNS` returns nothing.

#### Caveats 

* Multiple calls to `prefetchDNS` with the same server have the same effect as a single call.
* In the browser, you can call `prefetchDNS` in any situation: while rendering a component, in an Effect, in an event handler, and so on.
* In server-side rendering or when rendering Server Components, `prefetchDNS` only has an effect if you call it while rendering a component or in an async context originating from rendering a component. Any other calls will be ignored.
* If you know the specific resources you'll need, you can call [other functions](/reference/react-dom/#resource-preloading-apis) instead that will start loading the resources right away.
* There is no benefit to prefetching the same server the webpage itself is hosted from because it's already been looked up by the time the hint would be given.
* Compared with [`preconnect`](/reference/react-dom/preconnect), `prefetchDNS` may be better if you are speculatively connecting to a large number of domains, in which case the overhead of preconnections might outweigh the benefit.

---

## Usage 

### Prefetching DNS when rendering 

Call `prefetchDNS` when rendering a component if you know that its children will load external resources from that host.

```js
import  from 'react-dom';

function AppRoot() 
```

### Prefetching DNS in an event handler 

Call `prefetchDNS` in an event handler before transitioning to a page or state where external resources will be needed. This gets the process started earlier than if you call it during the rendering of the new page or state.

```js
import  from 'react-dom';

function CallToAction() {
  const onClick = () => 
  return (
    Start Wizard
  );
}
```
